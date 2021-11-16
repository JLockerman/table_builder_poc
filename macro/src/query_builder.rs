
use proc_macro2::TokenStream as TokenStream2;

use quote::quote;

use syn::{parse::{Parse, ParseStream}, punctuated::Punctuated};

pub struct Query {
    spi_client: syn::Ident,
    table: syn::Ident,
    fields: Punctuated<syn::Ident, syn::Token![,]>,
    where_clause: Option<syn::LitStr>,
}

impl Parse for Query {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let spi_client = input.parse()?;

        validate_marker(input, "from")?;
        let table = input.parse()?;

        validate_marker(input, "select")?;
        let content;
        let _ = syn::parenthesized!(content in input);
        let fields = Punctuated::parse_terminated(&content)?;

        let mut where_clause = None;
        if input.peek(syn::Token![where]) {
            let _: syn::Token![where] = input.parse()?;
            let _: syn::Token![:] = input.parse()?;
            where_clause = Some(input.parse()?);
        }

        Ok(Self {
            spi_client,
            table,
            fields,
            where_clause,
        })
    }
}

fn validate_marker(input: ParseStream, expected: &str) -> syn::Result<()> {
    let field: syn::Ident = input.parse()?;
    if field != expected {
        return Err(syn::Error::new(
            field.span(),
            format!("expected `{}` found `{}`", expected, field)
        ))
    }
    let _: syn::Token![:] = input.parse()?;
    return Ok(())
}
impl Query {
    pub(crate) fn expand(&self) -> TokenStream2 {
        use std::fmt::Write as _;

        let Query { spi_client, table, fields, where_clause } = self;
        let mod_name = super::table_mod(&table);

        let mut query_string = "SELECT ".to_string();
        let mut first = true;
        for field in fields {
            if !first {
                query_string.push_str(", ")
            }
            first = false;
            // cast each column to the SQL type we expect so that any errors in
            // DDL will just cause SQL errors not corruption. It might be nicer
            // to do this with a compile-time concatenation
            let _ = write!(&mut query_string, "{field}::{{{field}}}", field=field);
        }
        let _ = write!(&mut query_string, " FROM {}", table);
        if let Some(where_clause) = where_clause {
            let _ = write!(&mut query_string, " WHERE {}", where_clause.value());
        }

        let column_types = fields.iter().map(|field| quote!{
            #field = <#mod_name::#field as framework::PgTyped>::SQL_TYPE
        });


        let mut field_idx = 0usize;
        let field_reads = fields.iter().map(|field| {
            field_idx += 1;
            let optional_name = super::optional_name(field);
            quote! {
                let #field: #mod_name::#optional_name = __tuple.by_ordinal(#field_idx).unwrap().value();
                let #field: #mod_name::#field = <_ as framework::UnwrapTo<_>>::unwrap_to(#field);
            }
        });

        let field_names = fields.iter();

        quote! {
            #spi_client.select(&format!(#query_string, #(#column_types,)*), None, None).map(|__tuple| {
                #(#field_reads)*
                (#(#field_names),*)
            })
        }
    }
}