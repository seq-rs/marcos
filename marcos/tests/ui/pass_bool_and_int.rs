use marcos::ParseAttributes;

#[derive(ParseAttributes)]
#[attr_path(thing)]
struct TypedAttrs {
    skip: bool,              // bare bool: defaults to false
    verbose: Option<bool>,   // optional bool: None when absent
    count: Option<u32>,      // integer via LitInt
    offset: Option<i64>,
    name: Option<syn::Ident>, // fallback to syn::parse::Parse
}

fn main() {
    let attrs: Vec<syn::Attribute> = Vec::new();
    let parsed = TypedAttrs::parse_attributes(&attrs).unwrap();
    assert!(!parsed.skip);
    assert!(parsed.verbose.is_none());
    assert!(parsed.count.is_none());
}
