
use proc_macro2::TokenStream as TokenStream2;

use quote::quote;

use syn::{parse::{Parse, ParseStream}, punctuated::Punctuated};

pub struct Query {
    spi_client: syn::Ident,
    table: syn::Ident,
    fields: Punctuated<syn::Ident, syn::Token![,]>,
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
        Ok(Self {
            spi_client,
            table,
            fields,
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

        let Query { spi_client, table, fields } = self;
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

        let column_types = fields.iter().map(|field| quote!{
            #field = <#mod_name::#field as framework::PgTyped>::SQL_TYPE
        });


        let mut field_idx = 0usize;
        let field_reads = fields.iter().map(|field| {
            field_idx += 1;
            quote! {
                let #field: Option<#mod_name::#field> = __tuple.by_ordinal(#field_idx).unwrap().value();
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