
use proc_macro::TokenStream;


use syn::parse_macro_input;

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

fn table_mod(name: &proc_macro2::Ident) -> proc_macro2::Ident {
    syn::Ident::new(&format!("_{}_table_mod", name), name.span())
}

fn optional_name(name: &syn::Ident) -> syn::Ident {
    syn::Ident::new(&format!("_optional_{}", name), name.span())
}