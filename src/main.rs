mod encoding;
use arboard::Clipboard;
use clap::{CommandFactory, Parser};
use clap_complete::aot::{generate, Generator, Shell};
use encoding::encode;
use password_maker::PasswordMaker;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::{ffi::OsString, io, process::ExitCode};
use unicode_segmentation::UnicodeSegmentation;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Specify the length (number of characters) of the password
    #[arg(long, default_value_t = 16)]
    length: u32,

    /// Specify the number of passwords to output
    #[arg(long, default_value_t = 1)]
    count: u32,

    /// Candidates for uppercases to include in the password
    ///
    /// If an empty string is specified, no uppercases will be included in the password.
    #[arg(long, default_value = "ABCDEFGHIJKLMNOPQRSTUVWXYZ")]
    uppercase_candidates: OsString,

    /// The number of uppercases to always include in the password
    ///
    /// Generates a password that includes at least this number of uppercases.
    #[arg(long, default_value_t = 1)]
    uppercase_minimum_count: u32,

    /// Candidates for lowercases to include in the password
    ///
    /// If an empty string is specified, no lowercases will be included in the password.
    #[arg(long, default_value = "abcdefghijklmnopqrstuvwxyz")]
    lowercase_candidates: OsString,

    /// The number of lowercases to always include in the password
    ///
    /// Generates a password that includes at least this number of lowercases.
    #[arg(long, default_value_t = 1)]
    lowercase_minimum_count: u32,

    /// Candidates for numbers to include in the password
    ///
    /// If an empty string is specified, no numbers will be included in the password.
    #[arg(long, default_value = "0123456789")]
    number_candidates: OsString,

    /// The number of numbers to always include in the password
    ///
    /// Generates a password that includes at least this number of numbers.
    #[arg(long, default_value_t = 1)]
    number_minimum_count: u32,

    /// Candidates for symbols to include in the password
    ///
    /// If an empty string is specified, no symbols will be included in the password.
    #[arg(long, default_value = "!\"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~")]
    symbol_candidates: OsString,

    /// The number of symbols to always include in the password
    ///
    /// Generates a password that includes at least this number of symbols.
    #[arg(long, default_value_t = 1)]
    symbol_minimum_count: u32,

    /// Candidates for other characters to include in the password
    ///
    /// By specifying this option multiple times, you can specify multiple other characters.
    /// For example, by specifying "--other-candidates 😀👨‍👩‍👦😂 --other-candidates あいう", you can register each.
    /// By registering each, you can specify the occurrence count of each character candidate with "--other_minimum_count".
    #[arg(long)]
    other_candidates: Option<Vec<OsString>>,

    /// The minimum occurrence count of other characters to include in the password
    ///
    /// Generates a password that includes at least this number of other characters.
    /// Can be specified multiple times and corresponds to the order specified with "--other-candidates".
    /// For example, by specifying "--other-candidates 😀👨‍👩‍👦😂 --other-candidates あいう --other-minimum-count 1 --other-minimum-count 2",
    /// "😀👨‍👩‍👦😂" will appear at least once, and "あいう" will appear at least twice. If omitted, it is the same as specifying 0.
    #[arg(long)]
    other_minimum_count: Option<Vec<u32>>,

    /// Separate with null characters
    ///
    /// If this option is not specified, passwords are separated by newline characters.
    #[arg(long)]
    null: bool,

    /// Copy the password to the clipboard
    ///
    /// If not specified, the password is output to standard output.
    /// If multiple passwords are generated with "--count", all passwords are copied to the clipboard at once.
    #[arg(long)]
    clipboard: bool,

    /// Specify the encoding
    ///
    /// Specify the encoding for each candidate string (--*-candidates).
    #[arg(long, default_value = "utf-8")]
    encoding: String,

    /// Print the completion script
    ///
    /// If this option is specified, the password is not output. Also, even if '--clipboard' is specified, the completion script is output to standard output.
    #[arg(long, value_name = "SHELL")]
    completion: Option<Shell>,
}

impl Default for Cli {
    fn default() -> Self {
        Cli {
            length: 16,
            count: 1,
            uppercase_candidates: OsString::from("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
            uppercase_minimum_count: 1,
            lowercase_candidates: OsString::from("abcdefghijklmnopqrstuvwxyz"),
            lowercase_minimum_count: 1,
            number_candidates: OsString::from("0123456789"),
            number_minimum_count: 1,
            symbol_candidates: OsString::from("!\"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~"),
            symbol_minimum_count: 1,
            other_candidates: None,
            other_minimum_count: None,
            null: false,
            clipboard: false,
            encoding: String::from("utf-8"),
            completion: None,
        }
    }
}

/// Output the completion script
///
/// # Arguments
///
/// * `gen` - Generator to create the completion script
fn print_completions<G: Generator>(gen: G) {
    let mut cmd = Cli::command();
    generate(gen, &mut cmd, env!("CARGO_PKG_NAME"), &mut io::stdout());
}

/// Write text to the clipboard
///
/// # Arguments
///
/// * `text` - Text to write to the clipboard
///
/// # Returns
///
/// Returns an error message if an error occurs
fn write_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())?;
    Ok(())
}

/// Set character types for the password generator
///
/// # Arguments
///
/// * `maker` - Password generator
/// * `args` - Command line arguments
///
/// # Returns
///
/// Returns an error message if an error occurs
fn set_classifiers(maker: &mut PasswordMaker, args: &Cli) -> Result<(), String> {
    fn set_candidates_and_minimum_count(
        candidates: &OsString,
        encoding: &String,
        minimum_count: u32,
    ) -> Result<(Vec<String>, u32), String> {
        let decoded = encoding::decode(candidates, encoding)?
            .graphemes(true)
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let min_count = if decoded.is_empty() { 0 } else { minimum_count };
        Ok((decoded, min_count))
    }

    (maker.uppercase.candidates, maker.uppercase.minimum_count) = set_candidates_and_minimum_count(
        &args.uppercase_candidates,
        &args.encoding,
        args.uppercase_minimum_count,
    )?;

    (maker.lowercase.candidates, maker.lowercase.minimum_count) = set_candidates_and_minimum_count(
        &args.lowercase_candidates,
        &args.encoding,
        args.lowercase_minimum_count,
    )?;

    (maker.number.candidates, maker.number.minimum_count) = set_candidates_and_minimum_count(
        &args.number_candidates,
        &args.encoding,
        args.number_minimum_count,
    )?;

    (maker.symbol.candidates, maker.symbol.minimum_count) = set_candidates_and_minimum_count(
        &args.symbol_candidates,
        &args.encoding,
        args.symbol_minimum_count,
    )?;

    let mut other_candidates = args
        .other_candidates
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|s| encoding::decode(s, &args.encoding))
        .collect::<Result<Vec<String>, String>>()?;
    let mut other_minimum_count = args.other_minimum_count.clone().unwrap_or_default();

    // Adjust the number of candidates and minimum counts
    while other_candidates.len() < other_minimum_count.len() {
        other_candidates.push(String::new());
    }
    while other_minimum_count.len() < other_candidates.len() {
        other_minimum_count.push(0);
    }

    maker.others = other_candidates
        .into_iter()
        .zip(other_minimum_count)
        .map(|(candidates, minimum_count)| {
            let candidates = candidates.graphemes(true).map(|s| s.to_string()).collect();
            password_maker::Classifier {
                candidates,
                minimum_count,
            }
        })
        .collect();

    Ok(())
}

/// Generate passwords
///
/// # Arguments
///
/// * `args` - Command line arguments
///
/// # Returns
///
/// List of passwords
///
/// # Errors
///
/// Returns an error if password generation fails
fn generate_passwords(args: &Cli) -> Result<Vec<String>, String> {
    let mut passwords: Vec<String> = Vec::new();
    let mut maker = PasswordMaker {
        length: args.length,
        ..PasswordMaker::default()
    };

    set_classifiers(&mut maker, args)?;

    for _ in 0..args.count {
        let password = maker.generate()?;
        passwords.push(password);
    }

    Ok(passwords)
}

/// Format passwords
///
/// If null_separator is true, separate with null characters; otherwise, separate with newline characters (\n)
///
/// # Arguments
///
/// * `passwords` - List of passwords
/// * `null_separator` - Whether to separate with null characters
///
/// # Returns
///
/// Formatted passwords
fn format_passwords(passwords: Vec<String>, null_separator: bool) -> String {
    let separater = match null_separator {
        true => "\0",
        false => "\n",
    };

    passwords.join(separater) + separater
}

/// Output passwords
///
/// Copy to clipboard if specified, otherwise output to standard output
///
/// # Arguments
///
/// * `text` - Text to output
/// * `args` - Command line arguments
///
/// # Returns
///
/// Returns an error message if an error occurs
fn output_passwords(text: &str, args: &Cli) -> Result<(), String> {
    if args.clipboard {
        write_to_clipboard(text)?;
    } else {
        let encoded_string = encode(text, &args.encoding)?;

        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle
            .write_all(encoded_string.as_bytes())
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Generate passwords
///
/// # Arguments
///
/// * `args` - Command line arguments
///
/// # Returns
///
/// Returns an error message if an error occurs
fn password(args: Cli) -> Result<(), String> {
    let passwords = generate_passwords(&args)?;
    let output_string = format_passwords(passwords, args.null);
    output_passwords(&output_string, &args)
}

fn main() -> ExitCode {
    let args = Cli::parse();

    if let Some(shell) = args.completion {
        print_completions(shell);
        return ExitCode::SUCCESS;
    }

    match password(args) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use std::{os::unix::ffi::OsStringExt, vec};

    use super::*;

    #[test]
    fn default_password_generation() {
        let args = Cli::default();

        let passwords = generate_passwords(&args).unwrap();
        assert_eq!(passwords.len(), 1);
        // If candidates are added, one character may not be 1 byte, but by default, one character is 1 byte, so check the length with len()
        assert_eq!(passwords[0].len(), 16);
    }

    #[test]
    fn multiple_password_generation() {
        let args = Cli {
            count: 5,
            ..Default::default()
        };

        let passwords = generate_passwords(&args).unwrap();
        assert_eq!(passwords.len(), 5);

        // Check that there are no duplicate passwords when generating multiple passwords
        let unique_passwords: std::collections::HashSet<_> = passwords.iter().collect();
        assert_eq!(passwords.len(), unique_passwords.len());
    }

    #[test]
    fn password_with_other_characters() {
        // Generate a password that includes special characters such as surrogate pairs
        // There may be more special characters, but since we are also testing zero-width joiners, this is sufficient.
        let args = Cli {
            other_candidates: Some(vec![
                // Surrogate pair
                OsString::from("😀🚀🐱"),
                // Variation Selectors
                OsString::from("花󠄁龍󠄀舟󠄁👍🏿"),
                // Combining character
                OsString::from("áパぎ"),
                // Zero-width joiner
                OsString::from("🏳️‍🌈❤️‍🔥👨‍👩‍👦"),
                // Emoji flag sequence
                OsString::from("🇯🇵🇺🇸🇲🇦🇨🇦"),
            ]),
            other_minimum_count: Some(vec![1, 2, 3, 4, 2]),
            ..Default::default()
        };

        let passwords = generate_passwords(&args).unwrap();
        println!("{}", passwords[0]);

        assert_eq!(passwords.len(), 1);
        assert_eq!(passwords[0].graphemes(true).count(), 16);

        // Test if the string contains the characters
        let count_rocket = passwords[0].matches("🚀").count();
        let count_cat = passwords[0].matches("🐱").count();
        let count_smile = passwords[0].matches("😀").count();
        assert!(1 <= count_rocket + count_cat + count_smile);

        let count_hana = passwords[0].matches("花󠄁").count();
        let count_ryu = passwords[0].matches("龍󠄀").count();
        let count_fune = passwords[0].matches("舟󠄁").count();
        let count_ok = passwords[0].matches("👍🏿").count();
        assert!(2 <= count_hana + count_ryu + count_fune + count_ok);

        let count_a = passwords[0].matches("á").count();
        let count_pa = passwords[0].matches("パ").count();
        let count_ki = passwords[0].matches("ぎ").count();
        assert!(3 <= count_a + count_pa + count_ki);

        let count_rainbow = passwords[0].matches("🏳️‍🌈").count();
        let count_fire = passwords[0].matches("❤️‍🔥").count();
        let count_family = passwords[0].matches("👨‍👩‍👦").count();
        assert!(4 <= count_rainbow + count_fire + count_family);

        let count_jp = passwords[0].matches("🇯🇵").count();
        let count_us = passwords[0].matches("🇺🇸").count();
        let count_ma = passwords[0].matches("🇲🇦").count();
        let count_ca = passwords[0].matches("🇨🇦").count();
        assert!(2 <= count_jp + count_us + count_ma + count_ca);
    }

    #[test]
    fn generate_passwords_err() {
        let args = Cli {
            length: 0,
            ..Default::default()
        };

        let result = generate_passwords(&args);
        assert!(result.is_err());
    }

    #[test]
    fn format_passwords_with_null_separator() {
        let passwords = vec!["password1".to_string(), "password2".to_string()];
        let formatted = format_passwords(passwords, true);
        assert_eq!(formatted, "password1\0password2\0");
    }

    #[test]
    fn format_passwords_with_newline_separator() {
        let passwords = vec!["password1".to_string(), "password2".to_string()];
        let formatted = format_passwords(passwords, false);
        assert_eq!(formatted, "password1\npassword2\n");
    }

    #[test]
    fn set_classifiers_utf8() {
        let mut maker = PasswordMaker::default();
        let args = Cli {
            uppercase_candidates: OsString::from("ABC"),
            uppercase_minimum_count: 2,
            lowercase_candidates: OsString::from("abc"),
            lowercase_minimum_count: 3,
            number_candidates: OsString::from("123"),
            number_minimum_count: 4,
            symbol_candidates: OsString::from("!@#"),
            symbol_minimum_count: 5,
            other_candidates: Some(vec![OsString::from("😀👨‍👩‍👦😂"), OsString::from("あいう")]),
            other_minimum_count: Some(vec![6, 7]),
            ..Default::default()
        };

        set_classifiers(&mut maker, &args).unwrap();

        assert_eq!(maker.uppercase.candidates, vec!["A", "B", "C"]);
        assert_eq!(maker.uppercase.minimum_count, 2);
        assert_eq!(maker.lowercase.candidates, vec!["a", "b", "c"]);
        assert_eq!(maker.lowercase.minimum_count, 3);
        assert_eq!(maker.number.candidates, vec!["1", "2", "3"]);
        assert_eq!(maker.number.minimum_count, 4);
        assert_eq!(maker.symbol.candidates, vec!["!", "@", "#"]);
        assert_eq!(maker.symbol.minimum_count, 5);
        assert_eq!(maker.others.len(), 2);
        assert_eq!(maker.others[0].candidates, vec!["😀", "👨‍👩‍👦", "😂"]);
        assert_eq!(maker.others[0].minimum_count, 6);
        assert_eq!(maker.others[1].candidates, vec!["あ", "い", "う"]);
        assert_eq!(maker.others[1].minimum_count, 7);
    }

    #[test]
    fn set_classifiers_shift_jis() {
        let mut maker = PasswordMaker::default();
        let args = Cli {
            // Shift_JIS for "あいうえお"
            uppercase_candidates: OsString::from_vec(vec![
                0x82, 0xA0, 0x82, 0xA2, 0x82, 0xA4, 0x82, 0xA6, 0x82, 0xA8,
            ]),

            // Shift_JIS for "アイウエオ"
            lowercase_candidates: OsString::from_vec(vec![
                0x83, 0x41, 0x83, 0x43, 0x83, 0x45, 0x83, 0x47, 0x83, 0x49,
            ]),

            // Shift_JIS for "ｱｲｳｴｵ"
            number_candidates: OsString::from_vec(vec![0xB1, 0xB2, 0xB3, 0xB4, 0xB5]),

            // Shift_JIS for "ｧｨｩｪｫ"
            symbol_candidates: OsString::from_vec(vec![0xA7, 0xA8, 0xA9, 0xAA, 0xAB]),

            // Shift_JIS for "ａｉｕｅｏ" and "安以宇衣於"
            other_candidates: Some(vec![
                OsString::from_vec(vec![
                    0x82, 0x81, 0x82, 0x89, 0x82, 0x95, 0x82, 0x85, 0x82, 0x8f,
                ]),
                OsString::from_vec(vec![
                    0x88, 0xC0, 0x88, 0xC8, 0x89, 0x46, 0x88, 0xDF, 0x89, 0x97,
                ]),
            ]),
            encoding: "shift_jis".to_string(),
            ..Default::default()
        };

        set_classifiers(&mut maker, &args).unwrap();

        assert_eq!(
            maker.uppercase.candidates,
            vec!["あ", "い", "う", "え", "お"]
        );

        assert_eq!(
            maker.lowercase.candidates,
            vec!["ア", "イ", "ウ", "エ", "オ"]
        );

        assert_eq!(maker.number.candidates, vec!["ｱ", "ｲ", "ｳ", "ｴ", "ｵ"]);

        assert_eq!(maker.symbol.candidates, vec!["ｧ", "ｨ", "ｩ", "ｪ", "ｫ"]);

        assert_eq!(maker.others.len(), 2);
        assert_eq!(
            maker.others[0].candidates,
            vec!["ａ", "ｉ", "ｕ", "ｅ", "ｏ"]
        );
        assert_eq!(
            maker.others[1].candidates,
            vec!["安", "以", "宇", "衣", "於"]
        );
    }

    #[test]
    fn set_classifiers_empty() {
        // When all candidates are empty, and everything else is default
        {
            let mut maker = PasswordMaker::default();
            let args = Cli {
                uppercase_candidates: OsString::from(""),
                lowercase_candidates: OsString::from(""),
                number_candidates: OsString::from(""),
                symbol_candidates: OsString::from(""),
                ..Default::default()
            };

            set_classifiers(&mut maker, &args).unwrap();

            assert!(maker.uppercase.candidates.is_empty());
            assert_eq!(maker.uppercase.minimum_count, 0);
            assert!(maker.lowercase.candidates.is_empty());
            assert_eq!(maker.lowercase.minimum_count, 0);
            assert!(maker.number.candidates.is_empty());
            assert_eq!(maker.number.minimum_count, 0);
            assert!(maker.symbol.candidates.is_empty());
            assert_eq!(maker.symbol.minimum_count, 0);
            assert!(maker.others.is_empty());
        }

        // When all candidates and minimum counts are empty, and everything else is default
        {
            let mut maker = PasswordMaker::default();

            let args = Cli {
                uppercase_candidates: OsString::from(""),
                uppercase_minimum_count: 1,
                lowercase_candidates: OsString::from(""),
                lowercase_minimum_count: 2,
                number_candidates: OsString::from(""),
                number_minimum_count: 3,
                symbol_candidates: OsString::from(""),
                symbol_minimum_count: 4,
                ..Default::default()
            };

            set_classifiers(&mut maker, &args).unwrap();

            assert!(maker.uppercase.candidates.is_empty());
            assert_eq!(maker.uppercase.minimum_count, 0);
            assert!(maker.lowercase.candidates.is_empty());
            assert_eq!(maker.lowercase.minimum_count, 0);
            assert!(maker.number.candidates.is_empty());
            assert_eq!(maker.number.minimum_count, 0);
            assert!(maker.symbol.candidates.is_empty());
            assert_eq!(maker.symbol.minimum_count, 0);
            assert!(maker.others.is_empty());
        }
    }

    #[test]
    fn set_classifiers_err() {
        let mut maker = PasswordMaker::default();
        let args = Cli {
            uppercase_candidates: OsString::from("ABC"),
            encoding: "invalid".to_string(),
            ..Default::default()
        };

        let result = set_classifiers(&mut maker, &args);
        assert_eq!(result, Err("Unsupported encoding: invalid".to_string()));
    }

    #[test]
    fn set_classifiers_omit_other_minimum_count() {
        // When there are no other_candidates
        {
            let mut maker = PasswordMaker::default();
            let args = Cli {
                other_candidates: None,
                ..Default::default()
            };

            set_classifiers(&mut maker, &args).unwrap();

            assert!(maker.others.is_empty());
        }

        // When there is one other_candidate and no other_minimum_count
        {
            let mut maker = PasswordMaker::default();
            let args = Cli {
                other_candidates: Some(vec![OsString::from("😀👨‍👩‍👦😂")]),
                ..Default::default()
            };

            set_classifiers(&mut maker, &args).unwrap();

            assert_eq!(maker.others.len(), 1);
            assert_eq!(maker.others[0].candidates, vec!["😀", "👨‍👩‍👦", "😂"]);
            assert_eq!(maker.others[0].minimum_count, 0);
        }

        // When there are two other_candidates and no other_minimum_count
        {
            let mut maker = PasswordMaker::default();
            let args = Cli {
                other_candidates: Some(vec![OsString::from("😀👨‍👩‍👦😂"), OsString::from("あいう")]),
                ..Default::default()
            };

            set_classifiers(&mut maker, &args).unwrap();

            assert_eq!(maker.others.len(), 2);
            assert_eq!(maker.others[0].candidates, vec!["😀", "👨‍👩‍👦", "😂"]);
            assert_eq!(maker.others[0].minimum_count, 0);

            assert_eq!(maker.others[1].candidates, vec!["あ", "い", "う"]);
            assert_eq!(maker.others[1].minimum_count, 0);
        }

        // When there are two other_candidates and one other_minimum_count
        {
            let mut maker = PasswordMaker::default();
            let args = Cli {
                other_candidates: Some(vec![OsString::from("😀👨‍👩‍👦😂"), OsString::from("あいう")]),
                other_minimum_count: Some(vec![6]),
                ..Default::default()
            };

            set_classifiers(&mut maker, &args).unwrap();

            assert_eq!(maker.others.len(), 2);
            assert_eq!(maker.others[0].candidates, vec!["😀", "👨‍👩‍👦", "😂"]);
            assert_eq!(maker.others[0].minimum_count, 6);

            assert_eq!(maker.others[1].candidates, vec!["あ", "い", "う"]);
            assert_eq!(maker.others[1].minimum_count, 0);
        }

        // When there are two other_candidates and two other_minimum_counts
        {
            let mut maker = PasswordMaker::default();
            let args = Cli {
                other_candidates: Some(vec![OsString::from("😀👨‍👩‍👦😂"), OsString::from("あいう")]),
                other_minimum_count: Some(vec![1, 2]),
                ..Default::default()
            };

            set_classifiers(&mut maker, &args).unwrap();

            assert_eq!(maker.others.len(), 2);
            assert_eq!(maker.others[0].candidates, vec!["😀", "👨‍👩‍👦", "😂"]);
            assert_eq!(maker.others[0].minimum_count, 1);

            assert_eq!(maker.others[1].candidates, vec!["あ", "い", "う"]);
            assert_eq!(maker.others[1].minimum_count, 2);
        }

        // When there are two other_candidates and three other_minimum_counts
        {
            let mut maker = PasswordMaker::default();
            let args = Cli {
                other_candidates: Some(vec![OsString::from("😀👨‍👩‍👦😂"), OsString::from("あいう")]),
                other_minimum_count: Some(vec![1, 2, 3]),
                ..Default::default()
            };

            set_classifiers(&mut maker, &args).unwrap();

            assert_eq!(maker.others.len(), 3);
            assert_eq!(maker.others[0].candidates, vec!["😀", "👨‍👩‍👦", "😂"]);
            assert_eq!(maker.others[0].minimum_count, 1);

            assert_eq!(maker.others[1].candidates, vec!["あ", "い", "う"]);
            assert_eq!(maker.others[1].minimum_count, 2);

            assert_eq!(maker.others[2].candidates, Vec::<String>::new());
            assert_eq!(maker.others[2].minimum_count, 3);
        }
    }

    #[test]
    fn output_passwords_to_clipboard() {
        // When testing in an environment where DISPLAY is not set,
        // arboard::Clipboard::new() will fail, so test in an environment where DISPLAY is set.
        // Also, since it copies to the clipboard, running this will change the clipboard contents.
        // This is a major feature, so do not exclude it with ignore.

        let args = Cli {
            clipboard: true,
            ..Default::default()
        };

        let text = "password1\npassword2\0password3";
        output_passwords(text, &args).unwrap();

        let mut clipboard = Clipboard::new().unwrap();
        let clipboard_text = clipboard.get_text().unwrap();
        assert_eq!(clipboard_text, text);
    }

    #[test]
    fn output_passwords_to_stdout() {
        // It's easier to test with assert_cmd than to capture standard output.

        // In the default case
        // Since the content is random, just check the length.
        {
            let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
            let assert = cmd.assert();
            let output = assert.get_output();

            // By default, the password is 16 characters long with a single newline code at the back.
            // Since one character is one byte, check the length with len().
            assert_eq!(output.stdout.len(), 17);
        }

        // When the encoding is euc-jp
        // Check that either "あい" is included.
        {
            let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

            let binding = cmd
                .args(vec![
                    // By setting length to 5, it will always contain four 1-byte characters and one 2-byte character.
                    OsString::from("--length"),
                    OsString::from("5"),
                    OsString::from("--encoding"),
                    OsString::from("euc-jp"),
                    OsString::from("--other-candidates"),
                    // In EUC-JP, "あ" is 0xA4 0xA2, and "い" is 0xA4 0xA4.
                    OsString::from_vec(vec![0xA4, 0xA2, 0xA4, 0xA4]),
                    OsString::from("--other-minimum-count"),
                    OsString::from("1"),
                ])
                .assert();

            let output = binding.get_output();
            // Check if "あ(0xA4 0xA2)" or "い(0xA4 0xA4)" appears in the output.
            assert!(output
                .stdout
                .windows(2)
                .any(|w| w == b"\xA4\xA2" || w == b"\xA4\xA4"));
            // Since only one character is 2 bytes, check the length with len().
            assert_eq!(output.stdout.len(), 7);
        }
    }

    #[test]
    fn print_completions() {
        // It's easier to test with assert_cmd than to capture standard output.

        {
            let mut cmd = Command::cargo_bin("mkpw").unwrap();
            let assert = cmd.args(["--completion", "bash"]).assert();

            let output = assert.get_output();

            // If the specification of the clap completion script changes, the test may fail.
            // Check only the part that starts with _mkpw, which is unlikely to change.
            assert!(output.stdout.starts_with(b"_mkpw"));
        }

        // Even if "--clipboard" is specified, the completion script is output to standard output.
        {
            let mut cmd = Command::cargo_bin("mkpw").unwrap();

            let assert = cmd.args(["--completion", "zsh", "--clipboard"]).assert();

            let output = assert.get_output();

            // If the specification of the clap completion script changes, the test may fail.
            // Check only the part that starts with #compdef mkpw, which is unlikely to change.
            assert!(output.stdout.starts_with(b"#compdef mkpw"));
        }
    }
}
