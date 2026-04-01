use marcos::ParseAttributes;

#[derive(ParseAttributes)]
#[attr_path(marcos)]
struct MarcosAttrs {
    rename: Option<String>,
}

#[derive(ParseAttributes)]
#[attr_path(external)]
struct ExternalAttrs {
    default: Option<bool>,
}

#[derive(ParseAttributes)]
#[intersection]
struct AllAttrs {
    marcos: MarcosAttrs,
    ext: ExternalAttrs,
}

fn main() {}
