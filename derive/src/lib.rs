mod errctx;
mod attributes;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(ParseAttributes, attributes(attr_path, intersection, meta, parse))]
pub fn derive_parse_attributes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match attributes::derive_parse_attributes(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
