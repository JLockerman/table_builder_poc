
use proc_macro2::TokenStream as TokenStream2;

use quote::quote;

use syn::{parse::{Parse, ParseStream}, punctuated::Punctuated};

pub struct Table {
    name: syn::Ident,
    fields: Punctuated<Field, syn::Token![,]>,
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
        Ok(Self {
            name,
            fields,
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
    let Table{ name, fields } = agg;
    let struct_fields = fields.iter().map(|Field { name, ty }| quote!{
        #name: #ty,
    });

    let table_fields = sql_fields(&fields);

    let create_table = format!("\
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