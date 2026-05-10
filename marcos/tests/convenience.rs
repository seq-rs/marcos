//! Smoke tests for the convenience attribute-value types.

use marcos::convenience::{
    Callable, Override, PathOrLit, Spanned, StrOrIdent, WordOr, WordOrList, parse_override,
    parse_word_or, parse_word_or_list,
};
use syn::{Attribute, LitStr, parse_quote};

fn parse_first_meta_attr<F>(attr: &Attribute, mut handler: F) -> syn::Result<()>
where
    F: FnMut(&syn::meta::ParseNestedMeta) -> syn::Result<()>,
{
    attr.parse_nested_meta(|meta| handler(&meta))
}

#[test]
fn str_or_ident_accepts_string() {
    let parsed: StrOrIdent = syn::parse_str("\"hello\"").unwrap();
    assert!(matches!(parsed, StrOrIdent::Str(_)));
    assert_eq!(parsed.as_string(), "hello");
}

#[test]
fn str_or_ident_accepts_ident() {
    let parsed: StrOrIdent = syn::parse_str("hello").unwrap();
    assert!(matches!(parsed, StrOrIdent::Ident(_)));
    assert_eq!(parsed.as_string(), "hello");
}

#[test]
fn callable_accepts_path() {
    let parsed: Callable = syn::parse_str("my_module::func").unwrap();
    assert!(matches!(parsed, Callable::Path(_)));
}

#[test]
fn callable_accepts_closure() {
    let parsed: Callable = syn::parse_str("|x| x > 0").unwrap();
    assert!(matches!(parsed, Callable::Closure(_)));
}

#[test]
fn path_or_lit_distinguishes() {
    let lit: PathOrLit = syn::parse_str("42").unwrap();
    assert!(matches!(lit, PathOrLit::Lit(_)));
    let path: PathOrLit = syn::parse_str("MY_CONST").unwrap();
    assert!(matches!(path, PathOrLit::Path(_)));
}

#[test]
fn spanned_remembers_value() {
    let s: Spanned<LitStr> = syn::parse_str("\"hello\"").unwrap();
    assert_eq!(s.value.value(), "hello");
}

#[test]
fn word_or_recognizes_bare_word() {
    let attr: Attribute = parse_quote! { #[my(default)] };
    let mut got: Option<WordOr<LitStr>> = None;
    parse_first_meta_attr(&attr, |meta| {
        if meta.path.is_ident("default") {
            got = Some(parse_word_or(meta)?);
        }
        Ok(())
    })
    .unwrap();
    assert!(matches!(got, Some(WordOr::Word)));
}

#[test]
fn word_or_recognizes_value_form() {
    let attr: Attribute = parse_quote! { #[my(default = "hello")] };
    let mut got: Option<WordOr<LitStr>> = None;
    parse_first_meta_attr(&attr, |meta| {
        if meta.path.is_ident("default") {
            got = Some(parse_word_or(meta)?);
        }
        Ok(())
    })
    .unwrap();
    let WordOr::Value(s) = got.unwrap() else {
        panic!("expected Value");
    };
    assert_eq!(s.value(), "hello");
}

#[test]
fn word_or_list_recognizes_paren_list() {
    let attr: Attribute = parse_quote! { #[my(strict(a, b, c))] };
    let mut got: Option<WordOrList<syn::Ident>> = None;
    parse_first_meta_attr(&attr, |meta| {
        if meta.path.is_ident("strict") {
            got = Some(parse_word_or_list(meta)?);
        }
        Ok(())
    })
    .unwrap();
    let WordOrList::List(items) = got.unwrap() else {
        panic!("expected List");
    };
    assert_eq!(items.len(), 3);
}

#[test]
fn override_recognizes_inherit_and_explicit() {
    let attr_inherit: Attribute = parse_quote! { #[my(default)] };
    let mut got_inherit: Option<Override<LitStr>> = None;
    parse_first_meta_attr(&attr_inherit, |meta| {
        if meta.path.is_ident("default") {
            got_inherit = Some(parse_override(meta)?);
        }
        Ok(())
    })
    .unwrap();
    assert!(matches!(got_inherit, Some(Override::Inherit)));

    let attr_explicit: Attribute = parse_quote! { #[my(default = "x")] };
    let mut got_explicit: Option<Override<LitStr>> = None;
    parse_first_meta_attr(&attr_explicit, |meta| {
        if meta.path.is_ident("default") {
            got_explicit = Some(parse_override(meta)?);
        }
        Ok(())
    })
    .unwrap();
    assert!(matches!(got_explicit, Some(Override::Explicit(_))));
}
