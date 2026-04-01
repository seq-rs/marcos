use marcos::ParseAttributes;

#[derive(ParseAttributes)]
#[attr_path(config)]
struct NestedAttrs {
    #[meta(nested(value))]
    deep: Option<String>,
    #[meta(a(b(c)))]
    triple: Option<bool>,
}

fn main() {}
