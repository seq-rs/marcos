use marcos::ParseAttributes;

#[derive(ParseAttributes)]
#[attr_path(thing)]
#[intersection]
struct BothModes {
    name: Option<String>,
}

fn main() {}
