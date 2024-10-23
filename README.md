# mkpw

Highly customizable password generation tool. 🔑

This tool uses the password generation library [password-maker](./password-maker). For more details, please refer to [password-maker/README.md](./password-maker/README.md).

## Installation

### From source

```console
cargo install --path .
```

### From crates.io

```console
cargo install mkpw
```

## Example

### Generate a password with the default settings

The default settings are as follows:

```console
$ mkpw
8m8s]@IV[d=2\f_(
```

### Specify the length of the password

You can specify the length of the password as follows:

```console
# Generate a password with a length of 20
$ mkpw --length 20
/(DBnw!pv4@"(ku|)/rx
```

### Specify symbols

You can change the symbols included in the password:

```console
# Change the symbols included in the password to @ or ^
$ mkpw --symbol-candidates @^
0@mg71C12TZNQuIj
```

### Specify the minimum count of occurrences

You can specify the minimum count of times a character appears as follows:

```console
# Generate a password with a minimum of 2 lowercases
$ mkpw --lowercase-minimum-count 2
6E?t(f/&$muBK,HJ
```

### Specify other characters in the password

You can specify other characters to include in the password:

```console
# Generate a password with at least 2 of the following characters: 😺😸😹😻
$ mkpw --other-candidates 😺😸😹😻 --other-minimum-count 2
(6😸3aOx(8s7'T91😺
```

### Specify the number of passwords to generate

You can specify the number of passwords to generate:

```console
# Generate 5 passwords
$ mkpw --count 5
_{!sBZYjUO%8uAa!
J5_N@f{M%Akn5)+=
Bx6Wa.f`J|nE{Cx^
zoWby9vgd31h6F,?
Ps<-1lWE*,IaK8Ab
```

### Specify copying the password to the clipboard

You can copy the generated password to the clipboard:

```console
# Copy the generated password to the clipboard
$ mkpw --clipboard
```

### Load completion script

You can load the completion script for the `mkpw` command:

```console
# Load the completion script for the mkpw command
$ source <(mkpw --completion bash)
```

## License

Licensed under both the Apache License, Version 2.0 and the MIT License.

You may select, at your option, one of the above-listed licenses.

See the [LICENSE-APACHE](./LICENSE-APACHE) and [LICENSE-MIT](./LICENSE-MIT) files for the full text of the Apache License, Version 2.0 and the MIT License, respectively.
