use marcos::ParseAttributes;

#[derive(ParseAttributes)]
#[attr_path(construct)]
struct BuilderAttrs {
    derive: Vec<syn::Ident>,          // #[construct(derive(Debug, Clone))]
    rename: Option<String>,
}

fn main() {
    // empty attrs: Vec is empty
    let attrs: Vec<syn::Attribute> = Vec::new();
    let parsed = BuilderAttrs::parse_attributes(&attrs).unwrap();
    assert!(parsed.derive.is_empty());
    assert!(parsed.rename.is_none());

    // with actual attrs
    let input: syn::DeriveInput = syn::parse_quote! {
        #[construct(derive(Debug, Clone), rename = "Builder")]
        struct S;
    };
    let parsed = BuilderAttrs::parse_attributes(&input.attrs).unwrap();
    assert_eq!(parsed.derive.len(), 2);
    assert_eq!(parsed.derive[0].to_string(), "Debug");
    assert_eq!(parsed.derive[1].to_string(), "Clone");
    assert_eq!(parsed.rename.as_deref(), Some("Builder"));
}
