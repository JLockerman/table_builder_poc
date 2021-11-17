
use proc_macro2::TokenStream as TokenStream2;

use quote::quote;

use syn::{parse::{Parse, ParseStream}, punctuated::Punctuated, spanned::Spanned};

pub struct Table {
    name: syn::Ident,
    fields: Punctuated<Field, syn::Token![,]>,
    insert: Option<syn::Block>,
    requires: Option<syn::ExprArray>,
}

struct Field {
    name: syn::Ident,
    ty: syn::TypePath,
}

impl Parse for Table {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let content;
        let _ = syn::parenthesized!(content in input);
        let fields = Punctuated::parse_terminated(&content)?;

        let mut insert: Option<syn::Block> = None;
        let mut requires: Option<syn::ExprArray> = None;
        super::parse_marked(input, [
            ("insert", &mut |input| {
                if insert.is_some() {
                    panic!("duplicate `insert`")
                }
                insert = Some(input.parse()?);
                Ok(())
            }),
            ("requires", &mut |input| {
                if requires.is_some() {
                    panic!("duplicate `requires`")
                }
                requires = Some(input.parse()?);
                Ok(())
            })
        ])?;

        Ok(Self {
            name,
            fields,
            insert,
            requires,
        })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let _: syn::Token![:] = input.parse()?;
        let ty = input.parse()?;
        Ok(Self {
            name,
            ty,
        })
    }
}


//
//
//
//

pub fn expand(agg: Table) -> TokenStream2 {
    use std::fmt::Write;

    let Table{ name, fields, insert, requires } = agg;
    let struct_fields = fields.iter().map(|Field { name, ty }| quote!{
        #name: #ty,
    });

    let table_fields = sql_fields(&fields);

    let mut create_table = format!("\
            CREATE TABLE {name} (\n\
                {fields}\n\
            );\n\
            ",
        name=name,
        fields=table_fields
    );

    let mod_name = super::table_mod(&name);
    let field_types = fields.iter().map(|Field {name, ty}| {
        let optional_name = super::optional_name(name);
        let optional_ty = option_type(ty);
        quote! {
            pub type #name = #ty;
            pub type #optional_name = #optional_ty;
        }
    });

    let field_ty = fields.iter().map(|Field {ty, ..}| ty);
    let field_name = fields.iter().map(|Field {name, ..}| name);
    let field = fields.iter().map(|Field {name, ..}| name);


    let table_insert_function = insert.as_ref().map(|body| {
        let table_insert = syn::Ident::new(
            &format!("__table_builder_insert_{}", name),
            // TODO should this be def_site?
            body.span(),
        );
        let _ = write!(&mut create_table, "\
            INSERT INTO {table} SELECT * FROM \"{insert_fn}\"();\n\
            DROP FUNCTION \"{insert_fn}\";\n",
            table = name,
            insert_fn = table_insert,
        );
        let return_ty = fields.iter().map(|Field {name, ty}| quote! {
            pgx::name!(#name,#ty)
        });
        quote!{
            // hack b/c this doesn't understand the schema markers
            #[pg_extern(schema="public")]
            pub fn #table_insert() -> impl Iterator<Item = (#(#return_ty),*)> #body
        }
    });

    let create_table_name = format!("__CREATE_TABLE_{}", name);

    quote! {
        struct #name {
            #(#struct_fields)*
        }

        unsafe impl framework::PgTable for #name {}

        // inherent associated types are unstable, so fake it with a mod
        #[allow(non_snake_case)]
        #[allow(non_camel_case_types)]
        mod #mod_name {
            #(#field_types)*
        }

        #[allow(non_snake_case)]
        #table_insert_function

        impl #name {
            pub fn to_values_vec(self) -> Vec<(pgx::PgOid, Option<pgx::pg_sys::Datum>)> {
                use pgx::IntoDatum;
                let Self{ #(#field_name),* } = self;
                vec![
                    #((
                        pgx::PgOid::from(<#field_ty as pgx::IntoDatum>::type_oid()),
                        #field.into_datum()
                    ),)*
                ]
            }
        }

        pgx::extension_sql! {
            #create_table,
            name = #create_table_name,
        }
    }
}

fn sql_fields(fields: &Punctuated<Field, syn::Token![,]>) -> String {
    use std::fmt::Write as _;

    let mut sql = String::new();
    let mut is_first = true;
    for Field { name, ty } in fields {
        if !is_first {
            sql.push_str(",\n");
        }
        is_first = false;
        let _ = write!(&mut sql, "    {} {}", name, sql_field_ty(ty));
    }
    sql
}

// TODO a better way to do this would be `<#ty as PgTyped>::TY` but that would
//      require both SQL type info in a trait, and dynamic `extension_sql!`
//      strings, neither of which exists yet. Instead, we're using a
//      special-cased hack for demo purposes.
fn sql_field_ty(ty: &syn::TypePath) -> String {
    let (ty_string, nullable) = sql_type(ty);
    let mut ty_string = ty_string.to_string();
    if !nullable {
        ty_string.push_str(" NOT NULL")
    }
    ty_string
}


fn sql_type(ty: &syn::TypePath) -> (&'static str, bool) {
    let end = ty.path.segments.last().unwrap();
    match &*end.ident.to_string() {
        "i16"    => ("smallint", false),
        "i32"    => ("integer", false),
        "i64"    => ("bigint", false),
        "f32"    => ("real", false),
        "f64"    => ("double precision", false),
        "String" => ("text", false),
        "bool"   => ("boolean", false),
        "Option" => {
            let ty = option_contents(end);

            let (inner_type , _)= sql_type(ty);
            (inner_type, true)
        },
        _ => unimplemented!(),
    }
}

fn option_contents(end: &syn::PathSegment) -> &syn::TypePath {
    let args = match &end.arguments {
        syn::PathArguments::AngleBracketed(args) => &args.args,
        _ => unimplemented!(),
    };
    if args.len() != 1 {
        unimplemented!()
    }
    let arg = args.first().unwrap();
    let ty = match arg {
        syn::GenericArgument::Type(syn::Type::Path(ty)) => ty,
        _ => unimplemented!()
    };
    ty
}

fn option_type(ty: &syn::TypePath) -> TokenStream2 {
    let end = ty.path.segments.last().unwrap();
    if end.ident == "Option" {
        return quote! { #ty };
    }

    quote! { Option<#ty> }
}