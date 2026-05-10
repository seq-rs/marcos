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
| `Vec<T>` | `#[widget(derive(Debug, Clone))]` | Comma-separated list |
| [`StrOrIdent`][cnv] | `#[widget(name = "Foo")]` *or* `#[widget(name = Foo)]` | Either form, see [Convenience types](#convenience-types) |
| [`Callable`][cnv] | `#[widget(check = path::fn)]` *or* `#[widget(check = \|x\| x > 0)]` | Path or closure |
| [`PathOrLit`][cnv] | `#[widget(default = 42)]` *or* `#[widget(default = MY_CONST)]` | Literal or path |
| [`Spanned<T>`][cnv] | any `T` | Wraps `T` and remembers source span for diagnostics |
| `Option<T>` / `T` | `#[widget(key = value)]` | Fallback: `syn::parse::Parse` |

[cnv]: #convenience-types

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

### Convenience types

`marcos::convenience` ships value types covering common attribute shapes that
are awkward to express with primitives. Two groups by where they fit:

**Drop-in field types** — implement `syn::parse::Parse`, use directly:

| Type | Accepts | Use when |
|---|---|---|
| `StrOrIdent` | `key = "literal"` or `key = literal` | You don't want to force users to quote a name. |
| `Callable` | `key = path::to::fn` or `key = \|x\| ...` | Validators, transformers, anything callable. |
| `PathOrLit` | `key = 42` / `"x"` / `true` or `key = MY_CONST` | Defaults that may be a literal or a named constant. |
| `Spanned<T>` | any `T: Parse` | You'll emit a diagnostic later spanned at this value. |

```rust
use marcos::ParseAttributes;
use marcos::convenience::{StrOrIdent, Callable, Spanned};

#[derive(ParseAttributes)]
#[attr_path(widget)]
struct WidgetAttrs {
    builder: Option<StrOrIdent>,         // #[widget(builder = "Foo")] or builder = Foo
    validator: Option<Callable>,         // path or closure
    name: Option<Spanned<String>>,       // keeps the LitStr span for later errors
}
```

**Word-or-value patterns** — distinguish bare `key` from `key = value` /
`key(...)`. Used through a one-line custom parser via `#[parse(with = ...)]`:

| Type | Bare form | Value form | Helper |
|---|---|---|---|
| `WordOr<T>` | `#[ser(default)]` | `#[ser(default = "fn")]` | `parse_word_or` |
| `WordOrList<T>` | `#[ser(strict)]` | `#[ser(strict(a, b, c))]` | `parse_word_or_list` |
| `Override<T>` | `#[ser(default)]` | `#[ser(default = expr)]` | `parse_override` |

```rust
use marcos::convenience::{Override, parse_override};
use syn::Expr;

fn parse_default(meta: &syn::meta::ParseNestedMeta) -> syn::Result<Override<Expr>> {
    parse_override(meta)
}

#[derive(ParseAttributes)]
#[attr_path(ser)]
struct Attrs {
    #[parse(with = parse_default)]
    default: Option<Override<Expr>>,
}
```

`Override<T>::Inherit` carries the "use the default" semantic at the API
surface; codegen decides what "default" means (call `T::default()`, look up
a registered fallback, etc.).

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
