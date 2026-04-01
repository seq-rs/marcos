use marcos::ParseAttributes;

#[derive(ParseAttributes)]
#[attr_path(thing)]
struct WithRequired {
    name: String,
    label: Option<String>,
}

fn main() {}
