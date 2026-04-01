#[doc(hidden)]
pub use marcos_core;

pub use marcos_core::ErrCtx;

/// Re-export the `ParseAttributes` trait from `marcos_core`.
///
/// Implement this trait via `#[derive(ParseAttributes)]` to convert
/// `&[syn::Attribute]` into typed structs. See [`ParseAttributes`] for full
/// documentation and examples.
pub use marcos_core::ParseAttributes;

/// Derive macro for [`ParseAttributes`].
///
/// # Struct-level attributes
///
/// Exactly one of these is required:
///
/// - **`#[attr_path(name)]`** — parse attributes under a single path.
///   Fields become meta keys: `#[name(field_name = "value")]`.
///
/// - **`#[intersection]`** — aggregate multiple attribute paths.
///   Each field must be a type that implements `ParseAttributes`.
///   The full attribute slice is passed to each field's `parse_attributes`,
///   so each sub-struct filters by its own `#[attr_path]`.
///
/// # Field-level attributes (in `#[attr_path]` mode)
///
/// - **`#[meta(key)]`** — override the meta key (default: field name).
/// - **`#[meta(outer(inner))]`** — parse nested attributes like `#[name(outer(inner = "value"))]`.
///   Supports up to 3 levels of nesting.
/// - **`#[parse(with = func)]`** — custom parser function.
///   Signature: `fn(&syn::meta::ParseNestedMeta) -> syn::Result<T>`.
///
/// # Examples
///
/// Basic usage:
///
/// ```ignore
/// use marcos::ParseAttributes;
///
/// #[derive(ParseAttributes)]
/// #[attr_path(widget)]
/// struct WidgetAttrs {
///     rename: Option<String>,
///     skip: Option<bool>,
/// }
/// ```
///
/// Intersection of multiple paths:
///
/// ```ignore
/// use marcos::ParseAttributes;
///
/// #[derive(ParseAttributes)]
/// #[intersection]
/// struct AllAttrs {
///     widget: WidgetAttrs,
///     ext: ExternalAttrs,
/// }
/// ```
pub use marcos_derive::ParseAttributes;
