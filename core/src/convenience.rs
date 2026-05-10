//! Convenience value types for attribute parsing.
//!
//! These types fill common gaps in syn's vocabulary when writing
//! `#[derive(ParseAttributes)]` field types. They split into two groups:
//!
//! ## Value-position types
//!
//! Implement `syn::parse::Parse` and can appear directly as a field type:
//!
//! - [`StrOrIdent`] — accepts either `key = "literal"` or `key = literal`
//! - [`Callable`] — accepts a path or a closure expression
//! - [`PathOrLit`] — accepts a path or any literal
//! - [`Spanned<T>`] — wraps any `T: Parse` and remembers its source span
//!
//! ## Meta-position types
//!
//! Distinguish *bare word* (`key`) from *value form* (`key = value` or
//! `key(...)`). They cannot be implemented as standalone `Parse` impls
//! because the "is there an `=`?" decision happens upstream of value
//! parsing, in [`syn::meta::ParseNestedMeta`]:
//!
//! - [`WordOr<T>`] — `key` (bare) or `key = value`
//! - [`WordOrList<T>`] — `key` (bare) or `key(a, b, c)`
//! - [`Override<T>`] — alias for the inherit-default-or-replace pattern
//!
//! Until the derive macro is updated to special-case these types, use them
//! through a `#[parse(with = parse_word_or)]` helper. Each has a
//! corresponding free function (`parse_word_or`, `parse_word_or_list`,
//! `parse_override`) suitable for that role.

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    ExprClosure, Ident, Lit, LitStr, Path, Token,
    meta::ParseNestedMeta,
    parse::{Parse, ParseStream},
};

// =====================================================================
// StrOrIdent
// =====================================================================

/// Accept either a string literal or a bare identifier.
///
/// Useful for attribute values where users shouldn't have to care which form
/// they wrote — e.g. `#[my_attr(builder = "FooBuilder")]` and
/// `#[my_attr(builder = FooBuilder)]` should both work.
///
/// # Examples
///
/// ```ignore
/// use marcos::ParseAttributes;
/// use marcos::convenience::StrOrIdent;
///
/// #[derive(ParseAttributes)]
/// #[attr_path(my_attr)]
/// struct Attrs {
///     builder: Option<StrOrIdent>,  // accepts both forms
/// }
/// ```
#[derive(Debug, Clone)]
pub enum StrOrIdent {
    /// `key = "value"` — the literal form.
    Str(LitStr),
    /// `key = value` — the bare-ident form.
    Ident(Ident),
}

impl StrOrIdent {
    /// Get the value as a `String`, regardless of which form it was.
    pub fn as_string(&self) -> String {
        match self {
            StrOrIdent::Str(s) => s.value(),
            StrOrIdent::Ident(i) => i.to_string(),
        }
    }

    /// Get the value as an `Ident`, synthesizing one from the literal if
    /// needed. The returned ident inherits the original source span.
    pub fn as_ident(&self) -> Ident {
        match self {
            StrOrIdent::Str(s) => Ident::new(&s.value(), s.span()),
            StrOrIdent::Ident(i) => i.clone(),
        }
    }

    /// Source span of the underlying token, for diagnostics.
    pub fn span(&self) -> Span {
        match self {
            StrOrIdent::Str(s) => s.span(),
            StrOrIdent::Ident(i) => i.span(),
        }
    }
}

impl Parse for StrOrIdent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitStr) {
            input.parse().map(StrOrIdent::Str)
        } else if lookahead.peek(Ident) {
            input.parse().map(StrOrIdent::Ident)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for StrOrIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            StrOrIdent::Str(s) => s.to_tokens(tokens),
            StrOrIdent::Ident(i) => i.to_tokens(tokens),
        }
    }
}

// =====================================================================
// Callable
// =====================================================================

/// Accept either a path or a closure expression.
///
/// Useful for attributes that take a callable target, e.g. `validator =
/// my_module::check` or `validator = |x| x > 0`.
///
/// # Examples
///
/// ```ignore
/// use marcos::convenience::Callable;
///
/// #[derive(ParseAttributes)]
/// #[attr_path(my_attr)]
/// struct Attrs {
///     validator: Option<Callable>,
/// }
/// ```
#[derive(Debug, Clone)]
pub enum Callable {
    /// A path expression (`my_module::check`, `Foo::method`, `func`).
    Path(Path),
    /// A closure expression (`|x| x > 0`, `move || something()`).
    Closure(ExprClosure),
}

impl Parse for Callable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Closures start with `|`, `move`, or `async` — none of which can
        // appear in a path. Lookahead lets us pick cleanly.
        if input.peek(Token![|])
            || input.peek(Token![||])
            || input.peek(Token![move])
            || input.peek(Token![async])
        {
            input.parse().map(Callable::Closure)
        } else {
            input.parse().map(Callable::Path)
        }
    }
}

impl ToTokens for Callable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Callable::Path(p) => p.to_tokens(tokens),
            Callable::Closure(c) => c.to_tokens(tokens),
        }
    }
}

// =====================================================================
// PathOrLit
// =====================================================================

/// Accept either a literal value or a path.
///
/// Useful for attribute values that may be a constant (`default = 42`,
/// `default = "hello"`) or a named constant/expression (`default =
/// MY_CONST`, `default = my_module::DEFAULT`).
#[derive(Debug, Clone)]
pub enum PathOrLit {
    /// A literal value (`42`, `"hello"`, `true`, `'a'`, `1.5`, …).
    Lit(Lit),
    /// A path (`MY_CONST`, `module::ITEM`, `Foo::Variant`).
    Path(Path),
}

impl Parse for PathOrLit {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Lit) {
            input.parse().map(PathOrLit::Lit)
        } else {
            input.parse().map(PathOrLit::Path)
        }
    }
}

impl ToTokens for PathOrLit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            PathOrLit::Lit(l) => l.to_tokens(tokens),
            PathOrLit::Path(p) => p.to_tokens(tokens),
        }
    }
}

// =====================================================================
// Spanned<T>
// =====================================================================

/// Wrap any `T: Parse` and remember the span it was parsed from.
///
/// Useful when you want to emit a diagnostic spanned at an attribute value
/// later in the codegen pipeline — without `Spanned<T>`, the value's source
/// location is lost as soon as parsing is done.
///
/// # Examples
///
/// ```ignore
/// use marcos::convenience::Spanned;
///
/// #[derive(ParseAttributes)]
/// #[attr_path(my_attr)]
/// struct Attrs {
///     name: Spanned<String>,
/// }
///
/// // Later in codegen:
/// fn check(attrs: &Attrs) -> syn::Result<()> {
///     if attrs.name.value.is_empty() {
///         return Err(syn::Error::new(attrs.name.span, "name cannot be empty"));
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    /// The parsed value.
    pub value: T,
    /// The span the value was parsed from.
    pub span: Span,
}

impl<T: Parse> Parse for Spanned<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let value = input.parse()?;
        Ok(Spanned { value, span })
    }
}

impl<T: ToTokens> ToTokens for Spanned<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.value.to_tokens(tokens);
    }
}

// =====================================================================
// WordOr<T>
// =====================================================================

/// A bare word or a `key = value` pair.
///
/// Common attribute pattern: `#[serde(default)]` (the bare word form) versus
/// `#[serde(default = "function")]` (the value form). `WordOr<T>` carries
/// the distinction — `Word` for the bare form, `Value(T)` for the value
/// form.
///
/// # Standalone `Parse`
///
/// `WordOr<T>` does **not** implement `Parse` directly. The bare-word vs
/// value distinction is made in attribute meta-position (around the `=`),
/// which is upstream of any value parser. Use it through the
/// [`parse_word_or`] helper inside a `#[parse(with = ...)]` field:
///
/// ```ignore
/// use marcos::convenience::{WordOr, parse_word_or};
/// use syn::LitStr;
///
/// fn parse_default(meta: &syn::meta::ParseNestedMeta) -> syn::Result<WordOr<LitStr>> {
///     parse_word_or(meta)
/// }
///
/// #[derive(ParseAttributes)]
/// #[attr_path(my_attr)]
/// struct Attrs {
///     #[parse(with = parse_default)]
///     default: Option<WordOr<LitStr>>,
/// }
/// ```
///
/// To express *absence* of the attribute key entirely, wrap in `Option`.
#[derive(Debug, Clone)]
pub enum WordOr<T> {
    /// `key` (bare word, no `=`).
    Word,
    /// `key = value` — the parsed `T`.
    Value(T),
}

/// Parse a [`WordOr<T>`] from an attribute meta entry.
///
/// Checks for `=` after the key; if absent, returns [`WordOr::Word`].
/// Otherwise parses the value as `T`.
pub fn parse_word_or<T: Parse>(meta: &ParseNestedMeta) -> syn::Result<WordOr<T>> {
    if meta.input.peek(Token![=]) {
        let value = meta.value()?;
        Ok(WordOr::Value(value.parse()?))
    } else {
        Ok(WordOr::Word)
    }
}

// =====================================================================
// WordOrList<T>
// =====================================================================

/// A bare word or a `key(a, b, c)` parenthesized list.
///
/// Common attribute pattern: `#[derive(Debug)]` (bare-form is meaningless
/// here, but consider `#[my_attr(strict)]` vs `#[my_attr(strict(a, b, c))]`).
///
/// # Standalone `Parse`
///
/// Like [`WordOr`], cannot be a standalone `Parse` impl — the bare-vs-list
/// decision happens at meta-position. Use through [`parse_word_or_list`]
/// inside a `#[parse(with = ...)]` field.
#[derive(Debug, Clone)]
pub enum WordOrList<T> {
    /// `key` (bare).
    Word,
    /// `key(a, b, c)` — the parsed list.
    List(Vec<T>),
}

/// Parse a [`WordOrList<T>`] from an attribute meta entry.
///
/// Checks for `(` after the key; if absent, returns [`WordOrList::Word`].
/// Otherwise parses a comma-separated list of `T` inside the parens.
pub fn parse_word_or_list<T: Parse>(meta: &ParseNestedMeta) -> syn::Result<WordOrList<T>> {
    if meta.input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in meta.input);
        let items: syn::punctuated::Punctuated<T, Token![,]> =
            content.parse_terminated(T::parse, Token![,])?;
        Ok(WordOrList::List(items.into_iter().collect()))
    } else {
        Ok(WordOrList::Word)
    }
}

// =====================================================================
// Override<T>
// =====================================================================

/// "Inherit a default" or "replace with this value" — the third common
/// shape after [`WordOr`] and [`WordOrList`].
///
/// Conceptually equivalent to `WordOr<T>` with renamed variants for
/// clarity at use sites. `Inherit` reads better than `Word` when the
/// semantic is "fall back to default behavior."
///
/// # Examples
///
/// ```ignore
/// use marcos::convenience::{Override, parse_override};
///
/// fn parse_my_default(meta: &syn::meta::ParseNestedMeta) -> syn::Result<Override<syn::Expr>> {
///     parse_override(meta)
/// }
///
/// #[derive(ParseAttributes)]
/// #[attr_path(my_attr)]
/// struct Attrs {
///     #[parse(with = parse_my_default)]
///     default: Option<Override<syn::Expr>>,
/// }
/// ```
#[derive(Debug, Clone)]
pub enum Override<T> {
    /// Use the default behavior — typically synthesized later as
    /// `T::default()` or a domain-specific fallback.
    Inherit,
    /// Replace with this explicit value.
    Explicit(T),
}

/// Parse an [`Override<T>`] from an attribute meta entry.
///
/// Checks for `=` after the key; if absent, returns [`Override::Inherit`].
/// Otherwise parses the value as `T` and wraps as [`Override::Explicit`].
pub fn parse_override<T: Parse>(meta: &ParseNestedMeta) -> syn::Result<Override<T>> {
    if meta.input.peek(Token![=]) {
        let value = meta.value()?;
        Ok(Override::Explicit(value.parse()?))
    } else {
        Ok(Override::Inherit)
    }
}

