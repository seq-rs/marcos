use proc_macro2::Span;
use syn::{DeriveInput, Fields, Ident, Type, PathArguments, GenericArgument};

/// Top-level mode for the struct.
pub(crate) enum AttrMode {
    /// `#[attr_path(ident)]` — single attribute path, fields are meta keys.
    Path(Ident),
    /// `#[intersection]` — fields are `ParseAttributes` types aggregated by path.
    Intersection,
}

/// Parsed representation of a field in `#[attr_path]` mode.
pub(crate) struct MetaFieldDef {
    pub ident: Ident,
    pub optional: bool,
    /// The inner type (unwrapped from Option if optional).
    pub inner_ty: Type,
    /// Meta key path, e.g. `rename` or `nested, parsing` for `#[meta(nested(parsing))]`.
    pub meta_keys: Vec<Ident>,
    /// Custom parser function path from `#[parse(with = ...)]`.
    pub custom_parser: Option<syn::Path>,
}

/// Parsed representation of a field in `#[intersection]` mode.
pub(crate) struct IntersectionFieldDef {
    pub ident: Ident,
    pub ty: Type,
}

pub(crate) enum ParsedInput {
    Path {
        ident: Ident,
        path: Ident,
        fields: Vec<MetaFieldDef>,
    },
    Intersection {
        ident: Ident,
        fields: Vec<IntersectionFieldDef>,
    },
}

pub(crate) fn parse_input(input: &DeriveInput) -> syn::Result<ParsedInput> {
    let mode = parse_mode(input)?;
    let struct_ident = input.ident.clone();

    let named_fields = match &input.data {
        syn::Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => return Err(syn::Error::new_spanned(&input.ident, "ParseAttributes only supports structs with named fields")),
        },
        _ => return Err(syn::Error::new_spanned(&input.ident, "ParseAttributes only supports structs")),
    };

    match mode {
        AttrMode::Path(path) => {
            let mut fields = Vec::new();
            for field in named_fields {
                fields.push(parse_meta_field(field)?);
            }
            Ok(ParsedInput::Path { ident: struct_ident, path, fields })
        }
        AttrMode::Intersection => {
            let mut fields = Vec::new();
            for field in named_fields {
                fields.push(parse_intersection_field(field)?);
            }
            Ok(ParsedInput::Intersection { ident: struct_ident, fields })
        }
    }
}

fn parse_mode(input: &DeriveInput) -> syn::Result<AttrMode> {
    let mut mode: Option<AttrMode> = None;

    for attr in &input.attrs {
        if attr.path().is_ident("attr_path") {
            if mode.is_some() {
                return Err(syn::Error::new_spanned(attr, "cannot specify both #[attr_path] and #[intersection]"));
            }
            let ident: Ident = attr.parse_args()?;
            mode = Some(AttrMode::Path(ident));
        } else if attr.path().is_ident("intersection") {
            if mode.is_some() {
                return Err(syn::Error::new_spanned(attr, "cannot specify both #[attr_path] and #[intersection]"));
            }
            mode = Some(AttrMode::Intersection);
        }
    }

    mode.ok_or_else(|| syn::Error::new(Span::call_site(), "expected #[attr_path(name)] or #[intersection]"))
}

fn parse_meta_field(field: &syn::Field) -> syn::Result<MetaFieldDef> {
    let ident = field.ident.clone().unwrap();
    let ty = field.ty.clone();
    let (optional, inner_ty) = unwrap_option(&ty);

    let mut meta_keys: Option<Vec<Ident>> = None;
    let mut custom_parser: Option<syn::Path> = None;

    for attr in &field.attrs {
        if attr.path().is_ident("meta") {
            let mut keys = Vec::new();
            parse_nested_meta_keys(attr, &mut keys)?;
            meta_keys = Some(keys);
        } else if attr.path().is_ident("parse") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("with") {
                    let value = meta.value()?;
                    let path: syn::Path = value.parse()?;
                    custom_parser = Some(path);
                    Ok(())
                } else {
                    Err(meta.error("expected `with`"))
                }
            })?;
        }
    }

    // default: field name is the meta key
    let meta_keys = meta_keys.unwrap_or_else(|| vec![ident.clone()]);

    Ok(MetaFieldDef { ident, optional, inner_ty, meta_keys, custom_parser })
}

fn parse_nested_meta_keys(attr: &syn::Attribute, keys: &mut Vec<Ident>) -> syn::Result<()> {
    // #[meta(rename)] -> keys = [rename]
    // #[meta(nested(parsing))] -> keys = [nested, parsing]
    attr.parse_nested_meta(|meta| {
        let ident = meta.path.get_ident()
            .ok_or_else(|| meta.error("expected identifier"))?
            .clone();
        keys.push(ident);

        // check for further nesting via parenthesized content
        if meta.input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in meta.input);
            let nested_ident: Ident = content.parse()?;
            keys.push(nested_ident);
            // support one more level of nesting
            if content.peek(syn::token::Paren) {
                let inner;
                syn::parenthesized!(inner in content);
                let deep_ident: Ident = inner.parse()?;
                keys.push(deep_ident);
            }
        }

        Ok(())
    })
}

fn parse_intersection_field(field: &syn::Field) -> syn::Result<IntersectionFieldDef> {
    let ident = field.ident.clone().unwrap();
    let ty = field.ty.clone();
    Ok(IntersectionFieldDef { ident, ty })
}

/// If the type is `Option<T>`, returns `(true, T)`. Otherwise `(false, original)`.
#[allow(clippy::collapsible_if)]
fn unwrap_option(ty: &Type) -> (bool, Type) {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner)) = args.args.first() {
                        return (true, inner.clone());
                    }
                }
            }
        }
    }
    (false, ty.clone())
}
