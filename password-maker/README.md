# Password Maker

This is a password generation library for Rust.

## Usage

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
    password_maker.symbols.candidates = vec!["@".to_string(), "^".to_string()];
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
    password_maker.symbols.minimum_count = 10;
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
    password_maker.uppercases.candidates = vec![];
    password_maker.uppercases.minimum_count = 0; // If candidates are empty, min must be 0, otherwise it will result in an error
    password_maker.numbers.candidates = vec![];
    password_maker.numbers.minimum_count = 0;
    password_maker.symbols.candidates = vec![];
    password_maker.symbols.minimum_count = 0;
    let password = password_maker.generate().unwrap();
    println!("{}", password); // => tfpwxjzudvaibnwg
}
```

### Add emojis and other characters as candidates

You can add emojis and other characters as candidates as follows:

```rust
use password_maker::PasswordMaker;

fn main() {
    let mut password_maker = PasswordMaker::default();
    password_maker.other_characters.candidates = vec![
        '😀', '😁', '😂', '🤣', '😃', '😄', '😅', '😆', '😉', '😊', '😋', '😎', '😍', '😘', '😗',
        '一', '二', '三', '四', '五', '六', '七', '八', '九', '十',
    ];
    let password = password_maker.generate().unwrap();
    println!("{}", password); // => >Za7q\I~N😁`=九^*[
}
```

## License

The password-maker project is licensed under both the Apache License, Version 2.0 and the MIT License.

You may select, at your option, one of the above-listed licenses.

See the [LICENSE-APACHE](../LICENSE-APACHE) and [LICENSE-MIT](../LICENSE-MIT) files for the full text of the Apache License, Version 2.0 and the MIT License, respectively.
