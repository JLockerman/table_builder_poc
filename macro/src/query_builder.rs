
use proc_macro2::TokenStream as TokenStream2;

use quote::quote;

use syn::{parse::{Parse, ParseStream}, punctuated::Punctuated};

pub enum Query {
    Select(Select),
    Insert(Insert),
}

impl Parse for Query {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let spi_client = input.parse()?;
        let first_marker: syn::Ident = input.parse()?;
        if first_marker == "from" {
            let _: syn::Token![:] = input.parse()?;
            let s = Select::parse_after_first_marker(input, spi_client)?;
            Ok(Self::Select(s))
        } else if first_marker == "insert" {
            let i = Insert::parse_after_insert(input, spi_client)?;
            Ok(Self::Insert(i))
        } else {
            Err(syn::Error::new(
                first_marker.span(),
                format!(
                    "expected one of `from` or `insert` found `{}`",
                    first_marker,
                )
            ))
        }
    }
}

impl Query {
    pub(crate) fn expand(&self) -> TokenStream2 {
        match self {
            Query::Select(s) => s.expand(),
            Query::Insert(i) => i.expand(),
        }
    }
}

pub struct Select {
    spi_client: syn::Ident,
    table: syn::Ident,
    fields: Punctuated<syn::Ident, syn::Token![,]>,
    where_clause: Option<syn::LitStr>,
}

impl Parse for Select {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let spi_client = input.parse()?;

        super::validate_marker(input, "from")?;

        Self::parse_after_first_marker(input, spi_client)
    }
}

impl Select {
    pub(crate) fn parse_after_first_marker(input: ParseStream, spi_client: syn::Ident)
    -> syn::Result<Self> {
        let table = input.parse()?;

        super::validate_marker(input, "select")?;
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

    pub(crate) fn expand(&self) -> TokenStream2 {
        use std::fmt::Write as _;

        let Select { spi_client, table, fields, where_clause } = self;
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

pub struct Insert {
    spi_client: syn::Ident,
    table: syn::Ident,
    values: Values,
}

enum Values {
    Single(syn::Expr),
    Multiple(syn::Expr),
}

impl Insert {
    fn parse_after_insert(input: ParseStream, spi_client: syn::Ident)
    -> syn::Result<Self> {
        super::validate_marker(input, "into")?;
        let table = input.parse()?;
        let val_marker: syn::Ident = input.parse()?;
        let _: syn::Token![:] = input.parse()?;

        let values =
            if val_marker == "value" {
                Values::Single(input.parse()?)
            } else if val_marker == "values" {
                Values::Multiple(input.parse()?)
            } else {
                return Err(syn::Error::new(
                    val_marker.span(),
                    format!(
                        "expected one of `value` or `values` found `{}`",
                        val_marker,
                    )
                ))
            };

        Ok(Self {
            spi_client,
            table,
            values,
        })
    }

    fn expand(&self) -> TokenStream2 {
        let Insert { spi_client, table, values } = self;
        let insert_string = format!("INSERT INTO {} VALUES ($1, $2)", table);
        match values {
            Values::Single(val) => {
                quote! {
                    {
                        use pgx::IntoDatum;
                        let value: #table = #val;
                        let args = value.to_values_vec();
                        #spi_client.update(#insert_string, None, Some(args))
                    }
                }
            },
            Values::Multiple(vals) => {
                // TODO we should cache the planned query, but the APIs aren't exposed
                quote! {
                    {
                        use pgx::IntoDatum;
                        let vals = #vals;
                        for value in vals {
                            let value: #table = value;
                            let args = value.to_values_vec();
                            #spi_client.update(#insert_string, None, Some(args));
                        }
                    }
                }
            },
        }
    }
}