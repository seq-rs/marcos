use marcos::ParseAttributes;

/// Bare #[my_attr] (no parens) should be accepted without error.
#[derive(ParseAttributes)]
#[attr_path(my_attr)]
struct BarePathAttrs {
    rename: Option<String>,
}

fn main() {}
