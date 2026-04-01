mod errctx;

pub use errctx::ErrCtx;

/// Trait for converting `&[syn::Attribute]` into typed structs.
///
/// This is the core trait behind `#[derive(ParseAttributes)]`. It provides a single
/// method that takes a slice of syn attributes and produces a populated struct,
/// validating required fields and parsing values by type.
///
/// # Type-driven parsing
///
/// Field types determine how attribute values are parsed:
///
/// | Field type | Attribute syntax | Behavior |
/// |---|---|---|
/// | `Option<bool>` / `bool` | `#[name(flag)]` | Presence = `true`, absence = `None`/`false` |
/// | `Option<String>` / `String` | `#[name(key = "value")]` | Parses string literal |
/// | Integer types (`u32`, `i64`, etc.) | `#[name(key = 42)]` | Parses integer literal |
/// | `Option<Ident>` / `Ident` | `#[name(key = foo)]` | Delegates to `syn::parse::Parse` |
/// | `Option<T>` / `T` | `#[name(key = value)]` | Fallback: `syn::parse::Parse` |
///
/// # Required vs optional
///
/// - `Option<T>` fields are optional — `None` if the attribute key is absent.
/// - Bare `T` fields are required — a `syn::Error` is returned if missing.
/// - **Exception:** bare `bool` defaults to `false` when the flag is absent.
///
/// # Example
///
/// ```ignore
/// use marcos::ParseAttributes;
///
/// #[derive(ParseAttributes)]
/// #[attr_path(widget)]
/// struct WidgetAttrs {
///     rename: Option<String>,   // #[widget(rename = "NewName")]
///     skip: Option<bool>,       // #[widget(skip)]
///     label: String,            // required: #[widget(label = "...")]
/// }
/// ```
///
/// # Intersection mode
///
/// For structs that aggregate multiple `ParseAttributes` types. Each field's
/// type handles its own attribute path — the full `&[Attribute]` slice is
/// passed to each sub-struct's `parse_attributes`.
///
/// ```ignore
/// #[derive(ParseAttributes)]
/// #[intersection]
/// struct AllAttrs {
///     widget: WidgetAttrs,
///     ext: ExternalAttrs,
/// }
/// ```
///
/// # Custom parsers
///
/// Use `#[parse(with = path)]` for fields that need custom parsing logic.
/// The function receives `&syn::meta::ParseNestedMeta` and returns `syn::Result<T>`:
///
/// ```ignore
/// fn parse_mode(meta: &syn::meta::ParseNestedMeta) -> syn::Result<u32> {
///     let value = meta.value()?;
///     let lit: syn::LitInt = value.parse()?;
///     lit.base10_parse()
/// }
///
/// #[derive(ParseAttributes)]
/// #[attr_path(thing)]
/// struct Attrs {
///     #[parse(with = parse_mode)]
///     mode: Option<u32>,
/// }
/// ```
pub trait ParseAttributes: Sized {
    /// Parse a slice of `syn::Attribute` into `Self`.
    ///
    /// Returns `Err` if a required field is missing, a value fails to parse,
    /// or a duplicate/unknown attribute key is encountered.
    fn parse_attributes(attrs: &[syn::Attribute]) -> syn::Result<Self>;
}

/// Helper for generated code — creates a "missing required field" error.
#[doc(hidden)]
pub fn missing_required(path: &str, field: &str) -> syn::Error {
    syn::Error::new(
        proc_macro2::Span::call_site(),
        format!("missing required attribute `#[{path}({field})]`"),
    )
}
