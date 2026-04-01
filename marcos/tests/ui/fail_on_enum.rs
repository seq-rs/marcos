use marcos::ParseAttributes;

#[derive(ParseAttributes)]
#[attr_path(thing)]
enum NotAStruct {
    A,
    B,
}

fn main() {}
