use marcos::ParseAttributes;

#[derive(ParseAttributes)]
#[attr_path(my_attr)]
struct BasicAttrs {
    rename: Option<String>,
    skip: Option<bool>,
}

fn main() {
    let attrs: Vec<syn::Attribute> = Vec::new();
    let parsed = BasicAttrs::parse_attributes(&attrs).unwrap();
    assert!(parsed.rename.is_none());
    assert!(parsed.skip.is_none());
}
