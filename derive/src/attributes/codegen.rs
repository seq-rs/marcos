use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use super::parse::{MetaFieldDef, IntersectionFieldDef, ParsedInput};

pub(crate) fn generate(input: ParsedInput) -> TokenStream {
    match input {
        ParsedInput::Path { ident, path, fields } => gen_path_impl(ident, path, fields),
        ParsedInput::Intersection { ident, fields } => gen_intersection_impl(ident, fields),
    }
}

fn gen_path_impl(struct_ident: Ident, path: Ident, fields: Vec<MetaFieldDef>) -> TokenStream {
    let field_decls = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! { let mut #ident = None; }
    });

    let match_arms = fields.iter().map(gen_meta_match_arm);

    let field_finalizers = fields.iter().map(|f| {
        let ident = &f.ident;
        let ident_str = ident.to_string();
        let path_str_for_err = path.to_string();
        if f.optional {
            quote! { #ident }
        } else {
            quote! {
                #ident: #ident.ok_or_else(|| {
                    ::marcos_core::missing_required(#path_str_for_err, #ident_str)
                })?
            }
        }
    });

    let path_str = path.to_string();

    quote! {
        impl ::marcos_core::ParseAttributes for #struct_ident {
            fn parse_attributes(attrs: &[::syn::Attribute]) -> ::syn::Result<Self> {
                #(#field_decls)*

                for attr in attrs {
                    if attr.path().is_ident(#path_str) {
                        if matches!(attr.meta, ::syn::Meta::Path(_)) {
                            continue;
                        }
                        attr.parse_nested_meta(|meta| {
                            #(#match_arms)*
                            Err(meta.error("unknown attribute"))
                        })?;
                    }
                }

                Ok(Self {
                    #(#field_finalizers),*
                })
            }
        }
    }
}

fn gen_meta_match_arm(field: &MetaFieldDef) -> TokenStream {
    let field_ident = &field.ident;
    let keys = &field.meta_keys;

    // the outermost key is what we match in parse_nested_meta
    let first_key = &keys[0];
    let first_key_str = first_key.to_string();

    if let Some(ref parser) = field.custom_parser {
        let dup_guard = gen_duplicate_guard(field_ident, &first_key_str);
        // #[parse(with = func)] — fn(&ParseNestedMeta) -> syn::Result<T>
        return quote! {
            if meta.path.is_ident(#first_key_str) {
                #dup_guard
                #field_ident = Some(#parser(&meta)?);
                return Ok(());
            }
        };
    }

    let inner_ty = &field.inner_ty;

    if keys.len() == 1 {
        // simple: #[path(key)] or #[path(key = "value")]
        gen_value_parse(field_ident, &first_key_str, inner_ty)
    } else {
        // nested: #[path(a(b))] or #[path(a(b = "value"))]
        gen_nested_value_parse(field_ident, keys, inner_ty)
    }
}

fn gen_duplicate_guard(field_ident: &Ident, key_str: &str) -> TokenStream {
    let err_msg = format!("duplicate attribute `{key_str}`");
    quote! {
        if #field_ident.is_some() {
            return Err(meta.error(#err_msg));
        }
    }
}

fn gen_value_parse(field_ident: &Ident, key_str: &str, inner_ty: &syn::Type) -> TokenStream {
    let dup_guard = gen_duplicate_guard(field_ident, key_str);
    if is_bool_type(inner_ty) {
        quote! {
            if meta.path.is_ident(#key_str) {
                #dup_guard
                #field_ident = Some(true);
                return Ok(());
            }
        }
    } else if is_string_type(inner_ty) {
        quote! {
            if meta.path.is_ident(#key_str) {
                #dup_guard
                let value = meta.value()?;
                let lit: ::syn::LitStr = value.parse()?;
                #field_ident = Some(lit.value());
                return Ok(());
            }
        }
    } else {
        quote! {
            if meta.path.is_ident(#key_str) {
                #dup_guard
                let value = meta.value()?;
                let parsed: #inner_ty = value.parse()?;
                #field_ident = Some(parsed);
                return Ok(());
            }
        }
    }
}

fn gen_nested_value_parse(field_ident: &Ident, keys: &[Ident], inner_ty: &syn::Type) -> TokenStream {
    let first_key_str = keys[0].to_string();
    let second_key_str = keys[1].to_string();

    if keys.len() == 2 {
        let value_parse = if is_bool_type(inner_ty) {
            quote! { #field_ident = Some(true); }
        } else if is_string_type(inner_ty) {
            quote! {
                let value = meta.value()?;
                let lit: ::syn::LitStr = value.parse()?;
                #field_ident = Some(lit.value());
            }
        } else {
            quote! {
                let value = meta.value()?;
                let parsed: #inner_ty = value.parse()?;
                #field_ident = Some(parsed);
            }
        };

        quote! {
            if meta.path.is_ident(#first_key_str) {
                meta.parse_nested_meta(|meta| {
                    if meta.path.is_ident(#second_key_str) {
                        #value_parse
                        return Ok(());
                    }
                    Err(meta.error("unexpected attribute"))
                })?;
                return Ok(());
            }
        }
    } else if keys.len() == 3 {
        let third_key_str = keys[2].to_string();
        let value_parse = if is_bool_type(inner_ty) {
            quote! { #field_ident = Some(true); }
        } else if is_string_type(inner_ty) {
            quote! {
                let value = meta.value()?;
                let lit: ::syn::LitStr = value.parse()?;
                #field_ident = Some(lit.value());
            }
        } else {
            quote! {
                let value = meta.value()?;
                let parsed: #inner_ty = value.parse()?;
                #field_ident = Some(parsed);
            }
        };

        quote! {
            if meta.path.is_ident(#first_key_str) {
                meta.parse_nested_meta(|meta| {
                    if meta.path.is_ident(#second_key_str) {
                        meta.parse_nested_meta(|meta| {
                            if meta.path.is_ident(#third_key_str) {
                                #value_parse
                                return Ok(());
                            }
                            Err(meta.error("unexpected attribute"))
                        })?;
                        return Ok(());
                    }
                    Err(meta.error("unexpected attribute"))
                })?;
                return Ok(());
            }
        }
    } else {
        quote! { compile_error!("nesting deeper than 3 levels is not supported"); }
    }
}

fn gen_intersection_impl(struct_ident: Ident, fields: Vec<IntersectionFieldDef>) -> TokenStream {
    // each sub-type's parse_attributes filters by its own #[path], so just pass all attrs through
    let field_parses = fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote! {
            let #ident = <#ty as ::marcos_core::ParseAttributes>::parse_attributes(attrs)?;
        }
    });

    let field_idents = fields.iter().map(|f| &f.ident);

    quote! {
        impl ::marcos_core::ParseAttributes for #struct_ident {
            fn parse_attributes(attrs: &[::syn::Attribute]) -> ::syn::Result<Self> {
                #(#field_parses)*

                Ok(Self {
                    #(#field_idents),*
                })
            }
        }
    }
}

fn is_bool_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(p) = ty {
        p.path.is_ident("bool")
    } else {
        false
    }
}

fn is_string_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(p) = ty {
        p.path.is_ident("String")
    } else {
        false
    }
}
