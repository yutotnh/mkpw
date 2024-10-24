# Password Maker

This is a password generation library for Rust.

## Example

## Generate a password with the default settings

The default settings are as follows:

```rust
use password_maker::PasswordMaker;

fn main() {
    let mut password_maker = PasswordMaker::default();
    let password = password_maker.generate().unwrap();
    println!("{}", password); // => 8m8s]@IV[d=2\f_(
}
```

### Specify the length of the password

You can specify the length of the password as follows:

```rust
use password_maker::PasswordMaker;

fn main() {
    let mut password_maker = PasswordMaker {
        length: 20,
        ..Default::default()
    };
    let password = password_maker.generate().unwrap();
    println!("{}", password); // => /(DBnw!pv4@"(ku|)/rx
}
```

### Specify symbols

You can change the symbols as follows:

```rust
use password_maker::PasswordMaker;

fn main() {
    let mut password_maker = PasswordMaker::default();
    password_maker.symbol.candidates = vec!["@".to_string(), "^".to_string()];
    let password = password_maker.generate().unwrap();
    println!("{}", password); // => dHt^fO5fzgR@X4EC
}
```

### Specify the minimum number of occurrences

You can specify the minimum number of times a character appears as follows:

```rust
use password_maker::PasswordMaker;

fn main() {
    let mut password_maker = PasswordMaker::default();
    password_maker.symbol.minimum_count = 10;
    let password = password_maker.generate().unwrap();
    println!("{}", password); // => ZS^('}):?l}$<$|2
}
```

### Generate a password with only lowercases, excluding symbols, uppercases, and numbers

You can exclude symbols as follows:

```rust
use password_maker::PasswordMaker;

fn main() {
    let mut password_maker = PasswordMaker::default();
    password_maker.uppercase.candidates = vec![];
    password_maker.uppercase.minimum_count = 0; // If candidates are empty, min must be 0, otherwise it will result in an error
    password_maker.number.candidates = vec![];
    password_maker.number.minimum_count = 0;
    password_maker.symbol.candidates = vec![];
    password_maker.symbol.minimum_count = 0;
    let password = password_maker.generate().unwrap();
    println!("{}", password); // => tfpwxjzudvaibnwg
}
```

### Add emojis and other characters as candidates

You can add emojis and other characters as candidates as follows:

```rust
use password_maker::{Classifier, PasswordMaker};

fn main() {
    let mut password_maker = PasswordMaker {
        others: vec![Classifier {
            candidates: [
                '😀', '😁', '😂', '🤣', '😃', '😄', '😅', '😆', '😉', '😊', '😋', '😎', '😍', '😘',
                '😗', '一', '二', '三', '四', '五', '六', '七', '八', '九', '十',
            ]
            .iter()
            .map(|&c| c.to_string())
            .collect(),
            minimum_count: 1,
        }],
        ..Default::default()
    };
    let password = password_maker.generate().unwrap();
    println!("{}", password); // => ?🤣-五😍7=0*😘r}Nta>
}
```

## License

Licensed under both the Apache License, Version 2.0 and the MIT License.

You may select, at your option, one of the above-listed licenses.

See the [LICENSE-APACHE](./LICENSE-APACHE) and [LICENSE-MIT](./LICENSE-MIT) files for the full text of the Apache License, Version 2.0 and the MIT License, respectively.
