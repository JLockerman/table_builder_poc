
use proc_macro::TokenStream;


use syn::{parse::{Parse, ParseStream}, parse_macro_input};

#[proc_macro]
pub fn table(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as table_builder::Table);
    let expanded = table_builder::expand(input);
    // if cfg!(feature = "print-generated") {
        println!("{}", expanded.to_string());
    // }
    expanded.into()
}

#[proc_macro]
pub fn query(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as query_builder::Query);
    let expanded = input.expand();
    // if cfg!(feature = "print-generated") {
        println!("{}", expanded.to_string());
    // }
    expanded.into()
}

mod table_builder;
mod query_builder;


// utilities shared across both modules

fn table_mod(name: &proc_macro2::Ident) -> proc_macro2::Ident {
    syn::Ident::new(&format!("_{}_table_mod", name), name.span())
}

fn optional_name(name: &syn::Ident) -> syn::Ident {
    syn::Ident::new(&format!("_optional_{}", name), name.span())
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

fn parse_marked<const N: usize>(
    input: ParseStream,
    mut parsers: [(&str, &mut dyn FnMut(ParseStream) -> syn::Result<()>); N],
) -> syn::Result<()> {
    use std::fmt::Write as _;

    let marker_match_error =
        |marker: &syn::Ident, parsers: &[(&str, _); N]| {
            let mut msg = format!("found `{}` expected one of", marker);
            for (i, mark) in parsers.iter().map(|&(s, _)| s).enumerate() {
                if i + 1 == N {
                    let _ = write!(&mut msg, ", or `{}`", mark);
                } else {
                    let _ = write!(&mut msg, ", `{}`", mark);
                }
            }
            Err(syn::Error::new(marker.span(),msg,))
        };

    'parse_marked: while !input.is_empty() {
        let marker: syn::Ident = input.parse()?;
        let _: syn::Token![:] = input.parse()?;
        for (mark, parser) in &mut parsers {
            if marker == mark {
                parser(input)?;
                continue 'parse_marked
            }
        }
        return marker_match_error(&marker, &parsers)
    }
    Ok(())
}