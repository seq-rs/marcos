# marcos

Derive macro for parsing proc-macro attributes into typed structs.

Built on [syn](https://github.com/dtolnay/syn) — no hidden parsing magic, just
straightforward code generation from your struct definition.

## Attribute Parsing

Add `marcos` to your proc-macro crate:

```toml
[dependencies]
marcos = "0.1"
syn = { version = "2", features = ["full"] }
```

### Basic usage

Derive `ParseAttributes` on a struct and annotate it with `#[attr_path(name)]`
to declare which attribute path it parses. Fields become meta keys automatically.

```rust
use marcos::ParseAttributes;

#[derive(ParseAttributes)]
#[attr_path(widget)]
struct WidgetAttrs {
    rename: Option<String>,
    skip: Option<bool>,
}
```

This parses attributes like:

```rust
#[widget(rename = "Button", skip)]
```

Call it from your proc-macro with `WidgetAttrs::parse_attributes(&input.attrs)`.

### Type-driven parsing

The field type determines how values are extracted:

| Field type | Attribute syntax | Behavior |
|---|---|---|
| `Option<bool>` / `bool` | `#[widget(skip)]` | Presence flag |
| `Option<String>` / `String` | `#[widget(rename = "Foo")]` | String literal |
| Integer types (`u32`, `i64`, etc.) | `#[widget(count = 42)]` | Integer literal |
| `Ident` | `#[widget(via = from_str)]` | `syn::parse::Parse` |
| `Option<T>` / `T` | `#[widget(key = value)]` | Fallback: `syn::parse::Parse` |

### Required vs optional

`Option<T>` fields are optional — `None` when absent. Non-`Option` fields are
required and produce a compile error if missing. Bare `bool` is an exception —
it defaults to `false` when the flag is absent.

```rust
#[derive(ParseAttributes)]
#[attr_path(thing)]
struct Attrs {
    name: String,            // required
    label: Option<String>,   // optional
    skip: bool,              // defaults to false
}
```

### Overriding meta keys

By default, the field name is the meta key. Use `#[meta(key)]` to override:

```rust
#[derive(ParseAttributes)]
#[attr_path(widget)]
struct WidgetAttrs {
    #[meta(alias)]
    rename: Option<String>,  // parses #[widget(alias = "...")]
}
```

### Nested attributes

Use `#[meta(outer(inner))]` for nested attribute syntax, up to 3 levels deep:

```rust
#[derive(ParseAttributes)]
#[attr_path(config)]
struct ConfigAttrs {
    #[meta(nested(value))]
    deep: Option<String>,    // parses #[config(nested(value = "..."))]
}
```

### Custom parsers

For types that aren't covered by the built-in parsing, use `#[parse(with = func)]`.
The function receives `&syn::meta::ParseNestedMeta` and returns `syn::Result<T>`:

```rust
fn parse_mode(meta: &syn::meta::ParseNestedMeta) -> syn::Result<u32> {
    let value = meta.value()?;
    let lit: syn::LitInt = value.parse()?;
    lit.base10_parse()
}

#[derive(ParseAttributes)]
#[attr_path(thing)]
struct Attrs {
    #[parse(with = parse_mode)]
    mode: Option<u32>,       // parses #[thing(mode = 42)]
}
```

### Intersection mode

When your macro needs attributes from multiple paths, use `#[intersection]` to
combine several `ParseAttributes` types:

```rust
#[derive(ParseAttributes)]
#[attr_path(widget)]
struct WidgetAttrs {
    rename: Option<String>,
}

#[derive(ParseAttributes)]
#[attr_path(serde)]
struct SerdeAttrs {
    skip: Option<bool>,
}

#[derive(ParseAttributes)]
#[intersection]
struct AllAttrs {
    widget: WidgetAttrs,
    serde: SerdeAttrs,
}
```

Each sub-struct filters the full attribute slice by its own `#[attr_path]`, so
`AllAttrs::parse_attributes(&input.attrs)` handles both `#[widget(...)]` and
`#[serde(...)]` attributes.

### Error handling

The generated `parse_attributes` returns `syn::Result<Self>` and will error on:

- Missing required (non-`Option`) fields
- Duplicate attribute keys
- Unknown attribute keys
- Malformed values

## Crate structure

| Crate | Purpose |
|---|---|
| `marcos` | Facade — re-exports everything |
| `marcos_core` | `ParseAttributes` trait + `ErrCtx` error collector |
| `marcos_derive` | `#[derive(ParseAttributes)]` proc macro |

## License

MIT
