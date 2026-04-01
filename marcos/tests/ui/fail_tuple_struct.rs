use marcos::ParseAttributes;

#[derive(ParseAttributes)]
#[attr_path(thing)]
struct TupleStruct(Option<String>);

fn main() {}
