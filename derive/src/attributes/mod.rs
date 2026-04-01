mod parse;
mod codegen;

use syn::DeriveInput;
use proc_macro2::TokenStream;

pub(crate) fn derive_parse_attributes(input: DeriveInput) -> syn::Result<TokenStream> {
    let parsed = parse::parse_input(&input)?;
    Ok(codegen::generate(parsed))
}
