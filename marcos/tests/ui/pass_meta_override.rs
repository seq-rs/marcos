use marcos::ParseAttributes;

#[derive(ParseAttributes)]
#[attr_path(widget)]
struct MetaOverride {
    #[meta(alias)]
    rename: Option<String>,
}

fn main() {}
