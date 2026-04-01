use marcos::ParseAttributes;

fn parse_mode(meta: &syn::meta::ParseNestedMeta) -> syn::Result<u32> {
    let value = meta.value()?;
    let lit: syn::LitInt = value.parse()?;
    lit.base10_parse()
}

#[derive(ParseAttributes)]
#[attr_path(thing)]
struct CustomAttrs {
    #[parse(with = parse_mode)]
    mode: Option<u32>,
}

fn main() {}
