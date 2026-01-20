# Serde YAML Neo

[<img alt="github" src="https://img.shields.io/badge/github-firstdorsal/serde--yaml--neo-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/firstdorsal/serde-yaml-neo)
[<img alt="crates.io" src="https://img.shields.io/crates/v/serde_yaml_neo.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/serde_yaml_neo)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-serde__yaml__neo-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/serde_yaml_neo)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/firstdorsal/serde-yaml-neo/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/firstdorsal/serde-yaml-neo/actions?query=branch%3Amain)

Rust library for using the [Serde] serialization framework with data in [YAML]
file format. This library only follows the [YAML specification 1.1.](https://yaml.org/spec/1.1/).

[Serde]: https://github.com/serde-rs/serde
[YAML]: https://yaml.org/

## Dependency

```toml
[dependencies]
serde = "1.0"
serde_yaml_neo = "0.10"
```

Release notes are available under [GitHub releases].

[GitHub releases]: https://github.com/firstdorsal/serde-yaml-neo/releases

## Using Serde YAML

[API documentation is available in rustdoc form][docs.rs] but the general idea
is:

[docs.rs]: https://docs.rs/serde_yaml_neo

```rust
use std::collections::BTreeMap;

fn main() -> Result<(), serde_yaml_neo::Error> {
    // You have some type.
    let mut map = BTreeMap::new();
    map.insert("x".to_string(), 1.0);
    map.insert("y".to_string(), 2.0);

    // Serialize it to a YAML string.
    let yaml = serde_yaml_neo::to_string(&map)?;
    assert_eq!(yaml, "x: 1.0\ny: 2.0\n");

    // Deserialize it back to a Rust type.
    let deserialized_map: BTreeMap<String, f64> = serde_yaml_neo::from_str(&yaml)?;
    assert_eq!(map, deserialized_map);
    Ok(())
}
```

It can also be used with Serde's derive macros to handle structs and enums
defined in your program.

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_yaml_neo = "0.10"
```

Structs serialize in the obvious way:

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Point {
    x: f64,
    y: f64,
}

fn main() -> Result<(), serde_yaml_neo::Error> {
    let point = Point { x: 1.0, y: 2.0 };

    let yaml = serde_yaml_neo::to_string(&point)?;
    assert_eq!(yaml, "x: 1.0\ny: 2.0\n");

    let deserialized_point: Point = serde_yaml_neo::from_str(&yaml)?;
    assert_eq!(point, deserialized_point);
    Ok(())
}
```

Enums serialize using YAML's `!tag` syntax to identify the variant name.

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum Enum {
    Unit,
    Newtype(usize),
    Tuple(usize, usize, usize),
    Struct { x: f64, y: f64 },
}

fn main() -> Result<(), serde_yaml_neo::Error> {
    let yaml = "
        - !Newtype 1
        - !Tuple [0, 0, 0]
        - !Struct {x: 1.0, y: 2.0}
    ";
    let values: Vec<Enum> = serde_yaml_neo::from_str(yaml).unwrap();
    assert_eq!(values[0], Enum::Newtype(1));
    assert_eq!(values[1], Enum::Tuple(0, 0, 0));
    assert_eq!(values[2], Enum::Struct { x: 1.0, y: 2.0 });

    // The last two in YAML's block style instead:
    let yaml = "
        - !Tuple
          - 0
          - 0
          - 0
        - !Struct
          x: 1.0
          y: 2.0
    ";
    let values: Vec<Enum> = serde_yaml_neo::from_str(yaml).unwrap();
    assert_eq!(values[0], Enum::Tuple(0, 0, 0));
    assert_eq!(values[1], Enum::Struct { x: 1.0, y: 2.0 });

    // Variants with no data can be written using !Tag or just the string name.
    let yaml = "
        - Unit  # serialization produces this one
        - !Unit
    ";
    let values: Vec<Enum> = serde_yaml_neo::from_str(yaml).unwrap();
    assert_eq!(values[0], Enum::Unit);
    assert_eq!(values[1], Enum::Unit);

    Ok(())
}
```

## Configurable Indentation

You can customize the indentation level (2-9 spaces) when serializing:

```rust
use std::collections::BTreeMap;

fn main() -> Result<(), serde_yaml_neo::Error> {
    let mut data = BTreeMap::new();
    data.insert("outer", BTreeMap::from([("inner", 1)]));

    // Default 2-space indent
    let yaml = serde_yaml_neo::to_string(&data)?;

    // Custom 4-space indent
    let yaml = serde_yaml_neo::to_string_with_indent(&data, 4)?;

    Ok(())
}
```

## License

Licensed <a href="LICENSE-MIT">MIT license</a>.
