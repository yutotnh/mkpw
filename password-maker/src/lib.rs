use indexmap::IndexSet;
use rand::prelude::*;

#[cfg(test)]
// Use a fixed seed random number generator during tests to ensure reproducibility
use rand_chacha::ChaCha20Rng;

#[derive(Debug, Clone)]
/// Settings for characters used in the password
pub struct Classifier {
    /// Candidate characters
    pub candidates: Vec<String>,
    /// Minimum number of characters to include
    pub minimum_count: u32,
}

#[derive(Debug, Clone)]
/// Password generator
///
/// You can specify the following for the generated password:
/// - Length
/// - Whether to include similar characters
/// - Whether to include whitespace
/// - Candidates for uppercase, lowercase, numbers, symbols, and other characters
/// - Minimum number of characters for each type
pub struct PasswordMaker {
    /// Length of the password
    pub length: u32,
    /// Exclude similar characters ('i', 'l', '1', 'o', '0', 'O') from the password
    pub exclude_similar: bool,
    /// Include whitespace in the candidate characters for the password
    pub include_whitespace_in_candidate: bool,
    /// Settings for lowercases
    pub lowercase: Classifier,
    /// Settings for uppercases
    pub uppercase: Classifier,
    /// Settings for numbers
    pub number: Classifier,
    /// Settings for symbols
    pub symbol: Classifier,
    /// Settings for other characters
    pub others: Vec<Classifier>,
}

impl PasswordMaker {
    /// Generate a password
    ///
    /// Generates a password according to the settings of the password generator.
    /// Returns an error if there is an issue with the settings.
    ///
    /// Issues include:
    /// - No candidates for a character type, but the minimum number of characters is set to 1 or more
    /// - The total minimum number of characters for all types exceeds the password length
    ///
    /// # Returns
    ///
    /// * Ok: Password
    /// * Err: Error message
    ///
    /// # Examples
    ///
    /// ```
    /// use password_maker::PasswordMaker;
    ///
    /// let mut password_maker = PasswordMaker::default();
    /// let password = password_maker.generate().unwrap();
    /// println!("{}", password);
    /// ```
    ///
    pub fn generate(&mut self) -> Result<String, String> {
        // Return an error if validation fails
        self.validate()?;

        let candidates = self.candidates();

        // 候補文字列が空の場合はエラーを返す
        if candidates.is_empty() {
            return Err(
                "No candidates for the password. Please set the candidates for the password."
                    .to_string(),
            );
        }

        let mut rng = Self::create_rng();

        // 上書き処理があるので、String ではなく Vec<String> を使う
        let mut password: Vec<String> = (0..self.length)
            .map(|_| candidates.choose(&mut rng).unwrap().to_string())
            .collect();

        // Ensure the minimum number of characters is met
        // To maintain randomness, overwrite random positions with characters that meet the minimum count
        self.overwrite_to_meet_minimum_count(&mut password);

        Ok(password.concat())
    }

    /// Return a list of candidate characters for the password according to the settings of the password generator
    ///
    /// # Returns
    ///
    /// * List of candidate characters for the password
    ///
    /// # Examples
    ///
    /// ```
    /// use password_maker::PasswordMaker;
    ///
    /// let password_maker = PasswordMaker::default();
    /// let candidates = password_maker.candidates();
    /// println!("{:?}", candidates);
    /// ```
    pub fn candidates(&self) -> Vec<String> {
        let mut candidates = Vec::new();
        candidates.extend(self.lowercase.candidates.clone());
        candidates.extend(self.uppercase.candidates.clone());
        candidates.extend(self.number.candidates.clone());
        candidates.extend(self.symbol.candidates.clone());
        for classifier in &self.others {
            candidates.extend(classifier.candidates.clone());
        }

        if self.include_whitespace_in_candidate {
            candidates.push(" ".to_string());
        }

        if self.exclude_similar {
            candidates.retain(|c| !matches!(c.as_str(), "i" | "l" | "1" | "o" | "0" | "O"));
        }

        candidates
    }

    /// Create a random number generator
    ///
    /// During unit tests, return a fixed seed random number generator to ensure reproducibility
    ///
    /// Outside of unit tests, return a random number generator with a different seed for each thread
    ///
    /// # Returns
    ///
    /// * Random number generator
    fn create_rng() -> Box<dyn RngCore> {
        #[cfg(test)]
        {
            // Use a fixed seed during unit tests to ensure reproducibility
            // StdRng may change with version upgrades, so use ChaCha20Rng during tests to ensure future reproducibility
            Box::new(ChaCha20Rng::seed_from_u64(0))
        }
        #[cfg(not(test))]
        {
            // Use random numbers outside of unit tests
            Box::new(rand::thread_rng())
        }
    }

    /// Check if the minimum number of characters for each parameter is not violated
    ///
    /// Checks:
    /// - No candidates for a character type, but the minimum number of characters is set to 1 or more
    /// - The total minimum number of characters for all types exceeds the password length
    fn validate(&self) -> Result<(), String> {
        let classifier = [
            // Capitalize the first letter for error messages
            (&self.uppercase, "Uppercases"),
            (&self.lowercase, "Lowercases"),
            (&self.number, "Numbers"),
            (&self.symbol, "Symbols"),
        ];

        for (index, classify) in self.others.iter().enumerate() {
            if classify.candidates.is_empty() && 0 < classify.minimum_count {
                return Err(format!(
                    "Other characters at index {} is empty, but the minimum number of characters is set to {}. Please set the minimum number of characters to 0.",
                    index, classify.minimum_count
                ));
            }
        }

        for (classify, name) in classifier.iter() {
            if classify.candidates.is_empty() && 0 < classify.minimum_count {
                return Err(format!(
                    "{} is empty, but the minimum number of characters is set to {}. Please set the minimum number of characters to 0.",
                    name, classify.minimum_count
                ));
            }
        }

        let total_min = self.lowercase.minimum_count
            + self.uppercase.minimum_count
            + self.number.minimum_count
            + self.symbol.minimum_count
            + self.others.iter().map(|c| c.minimum_count).sum::<u32>();

        if self.length < total_min {
            return Err(format!("The total minimum number of characters is greater than the password length. The total minimum number of characters is {}, but the password length is {}", total_min, self.length));
        }

        Ok(())
    }

    /// Update the password string to meet the minimum number of characters for each type
    ///
    /// To maintain randomness, overwrite random positions with characters that meet the minimum count
    ///
    /// # Arguments
    ///
    /// * `password` - Password
    fn overwrite_to_meet_minimum_count(&self, password: &mut [String]) {
        // Number of characters to overwrite
        let overwrite_count = std::cmp::min(
            self.length,
            self.lowercase.minimum_count
                + self.uppercase.minimum_count
                + self.number.minimum_count
                + self.symbol.minimum_count
                + self.others.iter().map(|c| c.minimum_count).sum::<u32>(),
        );

        // Randomly select characters to overwrite
        let mut overwrite_chars =
            self.unique_random_numbers(overwrite_count as usize, 0..password.len() as u32);

        // Update each character type in order (the order can be changed without affecting functionality)
        let mut classifier = vec![&self.uppercase, &self.lowercase, &self.number, &self.symbol];
        for classify in &self.others {
            classifier.push(classify);
        }

        for classify in classifier.iter() {
            self.replace_characters(
                password,
                classify,
                overwrite_chars
                    .drain(0..classify.minimum_count as usize)
                    .map(|x| x as usize)
                    .collect(),
            );
        }
    }

    /// Overwrite characters in the password string
    ///
    /// For example, if the password is "abcde" and overwrite_indexes is \[3, 1, 4\], it becomes "aXcXXe"
    /// (X is a character randomly chosen from the classifier candidates)
    ///
    /// # Arguments
    ///
    /// * `password` - Password
    /// * `classifier` - Character type to replace
    /// * `overwrite_indexes` - Indexes of characters to replace
    ///
    /// # Panics
    ///
    /// * If the index of an element in overwrite_indexes is greater than the number of characters in the password
    fn replace_characters(
        &self,
        password: &mut [String],
        classifier: &Classifier,
        overwrite_indexes: Vec<usize>,
    ) {
        let mut rng = Self::create_rng();
        for index in overwrite_indexes {
            // ここはユーザーの入力ミスなどで index が password.len() 以上になることはなく、
            // なった場合はプログラムのバグなので panic しても問題ない
            if password.len() <= index {
                panic!(
                    "Index out of range: index {} is greater than or equal to password length {}",
                    index,
                    password.len()
                );
            }

            let overwrite_char = classifier.candidates.choose(&mut rng).unwrap().clone();
            password[index] = overwrite_char;
        }
    }

    /// Generate unique random numbers
    /// The generated values are between 0 and max (exclusive)
    ///
    /// # Arguments
    ///
    /// * count: Number of random numbers to generate
    /// * max: Maximum value of the generated random numbers
    fn unique_random_numbers(&self, count: usize, range: std::ops::Range<u32>) -> Vec<u32> {
        let mut rng = Self::create_rng();
        let mut numbers = IndexSet::new();

        while numbers.len() < count {
            let num = rng.gen_range(range.clone());
            numbers.insert(num);
        }

        numbers.into_iter().collect()
    }
}

impl Default for PasswordMaker {
    /// Create a password generator with default settings
    ///
    /// The default settings are as follows:
    /// - length: 16
    /// - exclude_similar: false
    /// - include_whitespace_in_candidate: false
    /// - lowercase_letters
    ///   - candidates: a-z
    ///   - min: 1
    /// - uppercase_letters
    ///   - candidates: A-Z
    ///   - min: 1
    /// - numbers:
    ///   - candidates: 0-9
    ///   - min: 1
    /// - symbols:
    ///   - candidates: ! " # $ % & ' ( ) * + , - . / : ; < = > ? @ \[ \ \] ^ _ \` { | } ~
    ///   - min: 1
    /// - other_characters:
    ///   - candidates: None
    ///   - min: 0
    fn default() -> Self {
        PasswordMaker {
            length: 16,
            exclude_similar: false,
            // Whitespace is less commonly used in passwords compared to other symbols,
            // and leading or trailing whitespace can cause input errors, so it is disabled by default.
            include_whitespace_in_candidate: false,
            lowercase: Classifier {
                candidates: ('a'..='z').map(|c| c.to_string()).collect(),
                minimum_count: 1,
            },
            uppercase: Classifier {
                candidates: ('A'..='Z').map(|c| c.to_string()).collect(),
                minimum_count: 1,
            },
            number: Classifier {
                candidates: (0..=9).map(|c| c.to_string()).collect(),
                minimum_count: 1,
            },
            // Symbols are sorted in ascending order of ASCII values
            symbol: Classifier {
                candidates: "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"
                    .chars()
                    .map(|c| c.to_string())
                    .collect(),
                minimum_count: 1,
            },
            others: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test if a password that meets the conditions can be generated
    // If the number of characters is small, it may not be possible to generate a password that meets the conditions,
    // so set a large number of characters (1000) for tests other than length tests
    const PASSWORD_LENGTH: u32 = 1000;

    #[test]
    fn default() {
        let mut password_maker = PasswordMaker::default();
        let password = password_maker.generate().unwrap();
        assert_eq!(password.chars().count(), 16);
    }

    #[test]
    /// Test if a password with a length of 0 can be generated
    /// By default, the minimum number of characters is set to 1, so an error occurs
    /// Set the minimum number of characters to 0 for the test
    fn empty() {
        let mut password_maker = PasswordMaker {
            length: 0,
            ..PasswordMaker::default()
        };

        // By default, the minimum number of characters for uppercase, lowercase, numbers, and symbols is set to 1, so an error occurs
        // Therefore, set the minimum number of characters to 0 for the test
        password_maker.uppercase.minimum_count = 0;
        password_maker.lowercase.minimum_count = 0;
        password_maker.number.minimum_count = 0;
        password_maker.symbol.minimum_count = 0;

        let password = password_maker.generate().unwrap();
        assert_eq!(password.chars().count(), 0);
    }

    #[test]
    fn length() {
        // Set the password length to 8 characters
        let mut password_maker = PasswordMaker {
            length: 8,
            ..PasswordMaker::default()
        };
        let password = password_maker.generate().unwrap();
        assert_eq!(password.chars().count(), 8);

        // Set the password length to 32 characters
        password_maker.length = 32;
        let password = password_maker.generate().unwrap();
        assert_eq!(password.chars().count(), 32);
    }

    #[test]
    fn uppercases() {
        // By default, include uppercases
        let mut password_maker = PasswordMaker {
            length: PASSWORD_LENGTH,
            ..PasswordMaker::default()
        };
        let password = password_maker.generate().unwrap();
        assert!(password.chars().any(|c| c.is_ascii_uppercase()));

        // Do not include uppercases
        password_maker.uppercase = Classifier {
            candidates: vec![],
            minimum_count: 0,
        };
        let password = password_maker.generate().unwrap();
        assert!(password.chars().all(|c| !c.is_ascii_uppercase()));

        // Specify the types of uppercases
        password_maker.uppercase = Classifier {
            candidates: vec![
                'A'.to_string(),
                'M'.to_string(),
                'N'.to_string(),
                'Z'.to_string(),
            ],
            minimum_count: 1,
        };
        let password = password_maker.generate().unwrap();
        // Check if the types of uppercases are only those specified
        // Specify the first, middle, and last letters of the alphabet
        assert!(password.contains('A'));
        assert!(password.contains('M'));
        assert!(password.contains('N'));
        assert!(password.contains('Z'));
        // Check if other uppercases are not included
        assert!(password.chars().all(|c| !matches!(
            c,
            'B' | 'C'
                | 'D'
                | 'E'
                | 'F'
                | 'G'
                | 'H'
                | 'I'
                | 'J'
                | 'K'
                | 'L'
                | 'O'
                | 'P'
                | 'Q'
                | 'R'
                | 'S'
                | 'T'
                | 'U'
                | 'V'
                | 'W'
                | 'X'
                | 'Y'
        )));

        // Ensure the minimum number of characters is met
        // No candidates for uppercases, but the minimum number of characters is set to 1
        password_maker.length = 4;
        password_maker.uppercase.minimum_count = 1;
        let password = password_maker.generate().unwrap();
        assert!(password.chars().any(|c| c.is_ascii_uppercase()));
    }
    #[test]
    fn lowercases() {
        // Include lowercases by default
        let mut password_maker = PasswordMaker {
            length: PASSWORD_LENGTH,
            ..PasswordMaker::default()
        };
        let password = password_maker.generate().unwrap();
        assert!(password.chars().any(|c| c.is_ascii_lowercase()));

        // Do not include lowercases
        password_maker.lowercase = Classifier {
            candidates: vec![],
            minimum_count: 0,
        };
        let password = password_maker.generate().unwrap();
        assert!(password.chars().all(|c| !c.is_ascii_lowercase()));

        // Specify the types of lowercases
        password_maker.lowercase = Classifier {
            candidates: ['a', 'm', 'n', 'z']
                .iter()
                .map(|&c| c.to_string())
                .collect(),
            minimum_count: 1,
        };
        let password = password_maker.generate().unwrap();
        // Check if the types of lowercases are only those specified
        // Specify the first, middle, and last letters of the alphabet
        assert!(password.contains('a'));
        assert!(password.contains('m'));
        assert!(password.contains('n'));
        assert!(password.contains('z'));
        // Check if other lowercases are not included
        assert!(password.chars().all(|c| !matches!(
            c,
            'b' | 'c'
                | 'd'
                | 'e'
                | 'f'
                | 'g'
                | 'h'
                | 'i'
                | 'j'
                | 'k'
                | 'l'
                | 'o'
                | 'p'
                | 'q'
                | 'r'
                | 's'
                | 't'
                | 'u'
                | 'v'
                | 'w'
                | 'x'
                | 'y'
        )));
    }

    #[test]
    fn numbers() {
        // Include numbers by default
        let mut password_maker = PasswordMaker {
            length: PASSWORD_LENGTH,
            ..PasswordMaker::default()
        };
        let password = password_maker.generate().unwrap();
        assert!(password.chars().any(|c| c.is_ascii_digit()));

        // Do not include numbers
        password_maker.number = Classifier {
            candidates: vec![],
            minimum_count: 0,
        };
        let password = password_maker.generate().unwrap();
        assert!(password.chars().all(|c| !c.is_ascii_digit()));

        // Specify the types of numbers
        password_maker.number = Classifier {
            candidates: ['0', '5', '9'].iter().map(|&c| c.to_string()).collect(),
            minimum_count: 1,
        };
        let password = password_maker.generate().unwrap();
        // Check if the types of numbers are only those specified
        assert!(password.contains('0'));
        assert!(password.contains('5'));
        assert!(password.contains('9'));
        // Check if other numbers are not included
        assert!(password
            .chars()
            .all(|c| !matches!(c, '1' | '2' | '3' | '4' | '6' | '7' | '8')));

        // Ensure the minimum number of characters is met

        // Error case
        // No candidates for numbers, but the minimum number of characters is set to 1
        password_maker.number = Classifier {
            candidates: vec![],
            minimum_count: 1,
        };
        let password = password_maker.generate();
        assert!(password.is_err());
    }

    #[test]
    fn symbols() {
        // Include symbols by default
        let mut password_maker = PasswordMaker {
            length: PASSWORD_LENGTH,
            ..PasswordMaker::default()
        };
        let password = password_maker.generate().unwrap();
        assert!(password.chars().any(|c| c.is_ascii_punctuation()));

        // Do not include symbols
        password_maker.symbol = Classifier {
            candidates: vec![],
            minimum_count: 0,
        };
        let password = password_maker.generate().unwrap();
        assert!(password.chars().all(|c| !c.is_ascii_punctuation()));

        // Specify the types of symbols
        password_maker.symbol = Classifier {
            candidates: ['!', '@', '~'].iter().map(|&c| c.to_string()).collect(),
            minimum_count: 1,
        };
        let password = password_maker.generate().unwrap();
        // Check if the types of symbols are only those specified
        assert!(password.contains('!'));
        assert!(password.contains('@'));
        assert!(password.contains('~'));
        // Check if other symbols are not included
        assert!(password.chars().all(|c| !matches!(
            c,
            '"' | '#'
                | '$'
                | '%'
                | '&'
                | '\''
                | '('
                | ')'
                | '*'
                | '+'
                | ','
                | '-'
                | '.'
                | '/'
                | ':'
                | ';'
                | '<'
                | '='
                | '>'
                | '?'
                | '['
                | '\\'
                | ']'
                | '^'
                | '_'
                | '`'
                | '{'
                | '|'
                | '}'
        )));
    }

    #[test]
    fn similar() {
        // Do not include similar characters
        let mut password_maker = PasswordMaker {
            length: PASSWORD_LENGTH,
            exclude_similar: true,
            ..PasswordMaker::default()
        };
        let password = password_maker.generate().unwrap();
        assert!(password
            .chars()
            .all(|c| !matches!(c, 'i' | 'l' | '1' | 'o' | '0' | 'O')));

        // Include similar characters
        password_maker.exclude_similar = false;
        let password = password_maker.generate().unwrap();
        assert!(password
            .chars()
            .any(|c| matches!(c, 'i' | 'l' | '1' | 'o' | '0' | 'O')));

        // Include similar characters by default
        let mut password_maker = PasswordMaker::default();
        let password = password_maker.generate().unwrap();
        assert!(password
            .chars()
            .any(|c| matches!(c, 'i' | 'l' | '1' | 'o' | '0' | 'O')));
    }

    #[test]
    fn whitespace() {
        // Do not include whitespace
        let mut password_maker = PasswordMaker {
            length: PASSWORD_LENGTH,
            include_whitespace_in_candidate: false,
            ..PasswordMaker::default()
        };
        let password = password_maker.generate().unwrap();
        assert!(!password.contains(' '));

        // Include whitespace
        password_maker.include_whitespace_in_candidate = true;
        let password = password_maker.generate().unwrap();
        assert!(password.contains(' '));
    }

    #[test]
    fn other_chars() {
        // Do not include other characters
        // For testing other characters, include only numbers, excluding alphabets and symbols
        let mut password_maker = PasswordMaker {
            length: PASSWORD_LENGTH,
            uppercase: Classifier {
                candidates: vec![],
                minimum_count: 0,
            },
            lowercase: Classifier {
                candidates: vec![],
                minimum_count: 0,
            },
            symbol: Classifier {
                candidates: vec![],
                minimum_count: 0,
            },
            ..PasswordMaker::default()
        };
        let password = password_maker.generate().unwrap();
        assert!(password.chars().all(|c| c.is_ascii_digit()));

        // Include other characters
        // Include Variable-width characters (characters that are treated as one character in char type, such as ⌨️, are not included)
        password_maker.others = vec![Classifier {
            candidates: ['あ', '🍣', '！'].iter().map(|&c| c.to_string()).collect(),
            minimum_count: 1,
        }];
        let password = password_maker.generate().unwrap();
        assert!(password.contains('あ'));
        assert!(password.contains('🍣'));
        assert!(password.contains('！'));
        assert!(password.chars().any(|c| c.is_ascii_digit()));
    }

    #[test]
    fn candidates() {
        // Include uppercase, lowercase, numbers, and symbols by default
        let password_maker = PasswordMaker {
            length: PASSWORD_LENGTH,
            ..PasswordMaker::default()
        };
        let candidates = password_maker.candidates();
        assert!(candidates
            .iter()
            .any(|c| c.chars().all(|ch| ch.is_ascii_uppercase())));
        assert!(candidates
            .iter()
            .any(|c| c.chars().all(|ch| ch.is_ascii_lowercase())));
        assert!(candidates
            .iter()
            .any(|c| c.chars().all(|ch| ch.is_ascii_digit())));
        assert!(candidates
            .iter()
            .any(|c| c.chars().all(|ch| ch.is_ascii_punctuation())));

        // Do not include something
        // This time, do not include uppercases
        let password_maker = PasswordMaker {
            length: PASSWORD_LENGTH,
            uppercase: Classifier {
                candidates: vec![],
                minimum_count: 0,
            },
            ..PasswordMaker::default()
        };
        let candidates = password_maker.candidates();
        assert!(!candidates
            .iter()
            .any(|c| c.chars().all(|ch| ch.is_ascii_uppercase())));

        // Include characters other than uppercase, lowercase, numbers, and symbols
        let password_maker = PasswordMaker {
            others: vec![Classifier {
                candidates: ['あ', '🍣', '！'].iter().map(|&c| c.to_string()).collect(),
                minimum_count: 1,
            }],
            ..PasswordMaker::default()
        };
        let candidates = password_maker.candidates();
        assert!(candidates
            .iter()
            .any(|c| c.chars().all(|ch| ch.is_ascii_uppercase())));
        assert!(candidates
            .iter()
            .any(|c| c.chars().all(|ch| ch.is_ascii_lowercase())));
        assert!(candidates
            .iter()
            .any(|c| c.chars().all(|ch| ch.is_ascii_digit())));
        assert!(candidates
            .iter()
            .any(|c| c.chars().all(|ch| ch.is_ascii_punctuation())));
        assert!(candidates.contains(&"あ".to_string()));
        assert!(candidates.contains(&"🍣".to_string()));
        assert!(candidates.contains(&"！".to_string()));
    }

    #[test]
    fn validate_uppercase_letter() {
        // Normal case
        {
            // There are candidates for uppercases, and the minimum number of characters is set to 1
            {
                let password_maker = PasswordMaker {
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_ok());
            }

            // There are no candidates for uppercases, but the minimum number of characters is set to 0
            {
                let password_maker = PasswordMaker {
                    uppercase: Classifier {
                        candidates: vec![],
                        minimum_count: 0,
                    },
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_ok());
            }
        }

        // Error case
        {
            // There are no candidates for uppercases, but the minimum number of characters is set to 1
            {
                let password_maker = PasswordMaker {
                    uppercase: Classifier {
                        candidates: vec![],
                        minimum_count: 1,
                    },
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_err());
            }

            // There are no candidates for uppercases, but the minimum number of characters is set to 2
            {
                let password_maker = PasswordMaker {
                    uppercase: Classifier {
                        candidates: vec![],
                        minimum_count: 2,
                    },
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_err());
            }
        }
    }

    #[test]
    fn validate_lowercase_letter() {
        // Normal case
        {
            // There are candidates for lowercases, and the minimum number of characters is set to 1
            {
                let password_maker = PasswordMaker {
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_ok());
            }

            // There are no candidates for lowercases, but the minimum number of characters is set to 0
            {
                let password_maker = PasswordMaker {
                    lowercase: Classifier {
                        candidates: vec![],
                        minimum_count: 0,
                    },
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_ok());
            }
        }

        // Error case
        {
            // There are no candidates for lowercases, but the minimum number of characters is set to 1
            {
                let password_maker = PasswordMaker {
                    lowercase: Classifier {
                        candidates: vec![],
                        minimum_count: 1,
                    },
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_err());
            }

            // There are no candidates for lowercases, but the minimum number of characters is set to 2
            {
                let password_maker = PasswordMaker {
                    lowercase: Classifier {
                        candidates: vec![],
                        minimum_count: 2,
                    },
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_err());
            }
        }
    }

    #[test]
    fn validate_number() {
        // Normal case
        {
            // There are candidates for numbers, and the minimum number of characters is set to 1
            {
                let password_maker = PasswordMaker {
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_ok());
            }

            // There are no candidates for numbers, but the minimum number of characters is set to 0
            {
                let password_maker = PasswordMaker {
                    number: Classifier {
                        candidates: vec![],
                        minimum_count: 0,
                    },
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_ok());
            }
        }

        // Error case
        {
            // There are no candidates for numbers, but the minimum number of characters is set to 1
            {
                let password_maker = PasswordMaker {
                    number: Classifier {
                        candidates: vec![],
                        minimum_count: 1,
                    },
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_err());
            }

            // There are no candidates for numbers, but the minimum number of characters is set to 2
            {
                let password_maker = PasswordMaker {
                    number: Classifier {
                        candidates: vec![],
                        minimum_count: 2,
                    },
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_err());
            }
        }
    }

    #[test]
    fn validate_symbol() {
        // Normal case
        {
            // There are candidates for symbols, and the minimum number of characters is set to 1
            {
                let password_maker = PasswordMaker {
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_ok());
            }

            // There are no candidates for symbols, but the minimum number of characters is set to 0
            {
                let password_maker = PasswordMaker {
                    symbol: Classifier {
                        candidates: vec![],
                        minimum_count: 0,
                    },
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_ok());
            }
        }

        // Error case
        {
            // There are no candidates for symbols, but the minimum number of characters is set to 1
            {
                let password_maker = PasswordMaker {
                    symbol: Classifier {
                        candidates: vec![],
                        minimum_count: 1,
                    },
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_err());
            }

            // There are no candidates for symbols, but the minimum number of characters is set to 2
            {
                let password_maker = PasswordMaker {
                    symbol: Classifier {
                        candidates: vec![],
                        minimum_count: 2,
                    },
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_err());
            }
        }
    }

    #[test]
    fn validate_other_characters() {
        // Normal case
        {
            // There are candidates for other characters, and the minimum number of characters is set to 1
            {
                let password_maker = PasswordMaker {
                    others: vec![Classifier {
                        candidates: ['あ', '🍣', '！'].iter().map(|&c| c.to_string()).collect(),
                        minimum_count: 1,
                    }],
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_ok());
            }

            // There are no candidates for other characters, but the minimum number of characters is set to 0
            {
                let password_maker = PasswordMaker {
                    others: vec![Classifier {
                        candidates: vec![],
                        minimum_count: 0,
                    }],
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_ok());
            }
        }

        // Error case
        {
            // There are no candidates for other characters, but the minimum number of characters is set to 1
            {
                let password_maker = PasswordMaker {
                    others: vec![Classifier {
                        candidates: vec![],
                        minimum_count: 1,
                    }],
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_err());
            }

            // There are no candidates for other characters, but the minimum number of characters is set to 2
            {
                let password_maker = PasswordMaker {
                    others: vec![Classifier {
                        candidates: vec![],
                        minimum_count: 2,
                    }],
                    ..PasswordMaker::default()
                };
                let result = password_maker.validate();
                assert!(result.is_err());
            }
        }
    }

    #[test]
    fn validate_total() {
        // Test the total minimum number of characters for each type
        {
            let mut password_maker = PasswordMaker {
                others: vec![Classifier {
                    candidates: ['あ', '🍣', '！'].iter().map(|&c| c.to_string()).collect(),
                    minimum_count: 1,
                }],
                ..PasswordMaker::default()
            };

            // The total minimum number of characters is less than the password length
            {
                password_maker.length = password_maker.uppercase.minimum_count
                    + password_maker.lowercase.minimum_count
                    + password_maker.number.minimum_count
                    + password_maker.symbol.minimum_count
                    + password_maker
                        .others
                        .iter()
                        .map(|c| c.minimum_count)
                        .sum::<u32>()
                    - 1;
                let result = password_maker.validate();
                assert!(result.is_err());
            }

            // The total minimum number of characters is equal to the password length
            {
                password_maker.length = password_maker.uppercase.minimum_count
                    + password_maker.lowercase.minimum_count
                    + password_maker.number.minimum_count
                    + password_maker.symbol.minimum_count
                    + password_maker
                        .others
                        .iter()
                        .map(|c| c.minimum_count)
                        .sum::<u32>();
                let result = password_maker.validate();
                assert!(result.is_ok());
            }

            // The total minimum number of characters is greater than the password length
            {
                password_maker.length = password_maker.uppercase.minimum_count
                    + password_maker.lowercase.minimum_count
                    + password_maker.number.minimum_count
                    + password_maker.symbol.minimum_count
                    + password_maker
                        .others
                        .iter()
                        .map(|c| c.minimum_count)
                        .sum::<u32>()
                    + 1;
                let result = password_maker.validate();
                assert!(result.is_ok());
            }
        }
    }

    #[test]
    fn overwrite_to_meet_minimum_count() {
        // Confirm that it is overwritten by making everything blank

        // By default, uppercase, lowercase, numbers, symbols, and the minimum number of characters are set to 1,
        // and there are no candidates for other characters, so check if each type of character is included
        {
            let mut password = vec![" ".to_string(); 5];

            let password_maker = PasswordMaker::default();

            password_maker.overwrite_to_meet_minimum_count(&mut password);

            assert!(password
                .iter()
                .any(|s| s.chars().any(|c| c.is_ascii_uppercase())));
            assert!(password
                .iter()
                .any(|s| s.chars().any(|c| c.is_ascii_lowercase())));
            assert!(password
                .iter()
                .any(|s| s.chars().any(|c| c.is_ascii_digit())));
            assert!(password
                .iter()
                .any(|s| s.chars().any(|c| c.is_ascii_punctuation())));
            assert!(!password
                .iter()
                .any(|s| s.chars().any(|c| c.eq(&'あ') || c.eq(&'🍣') || c.eq(&'！'))));
        }

        // If the minimum number of characters is 0, that type of character is not included
        // Test by setting 0 and 1 alternately
        // Since the test for other variable-width strings is done above, make the string blank before overwriting
        {
            let mut password = vec![" ".to_string(); 5];

            let mut password_maker = PasswordMaker {
                others: vec![Classifier {
                    candidates: ['あ', '🍣', '！'].iter().map(|&c| c.to_string()).collect(),
                    minimum_count: 1,
                }],
                ..PasswordMaker::default()
            };

            password_maker.uppercase.minimum_count = 0;
            password_maker.lowercase.minimum_count = 1;
            password_maker.number.minimum_count = 0;
            password_maker.symbol.minimum_count = 1;
            for classifier in &mut password_maker.others {
                classifier.minimum_count = 0;
            }
            password_maker.overwrite_to_meet_minimum_count(&mut password);

            assert!(!password
                .iter()
                .any(|s| s.chars().any(|c| c.is_ascii_uppercase())));
            assert!(password
                .iter()
                .any(|s| s.chars().any(|c| c.is_ascii_lowercase())));
            assert!(!password
                .iter()
                .any(|s| s.chars().any(|c| c.is_ascii_digit())));
            assert!(password
                .iter()
                .any(|s| s.chars().any(|c| c.is_ascii_punctuation())));
            assert!(!password
                .iter()
                .any(|s| s.chars().any(|c| c.eq(&'あ') || c.eq(&'🍣') || c.eq(&'！'))));

            let mut password = vec![" ".to_string(); 5];
            password_maker.uppercase.minimum_count = 1;
            password_maker.lowercase.minimum_count = 0;
            password_maker.number.minimum_count = 1;
            password_maker.symbol.minimum_count = 0;
            for classifier in &mut password_maker.others {
                classifier.minimum_count = 1;
            }
            password_maker.overwrite_to_meet_minimum_count(&mut password);

            assert!(password
                .iter()
                .any(|s| s.chars().any(|c| c.is_ascii_uppercase())));
            assert!(!password
                .iter()
                .any(|s| s.chars().any(|c| c.is_ascii_lowercase())));
            assert!(password
                .iter()
                .any(|s| s.chars().any(|c| c.is_ascii_digit())));
            assert!(!password
                .iter()
                .any(|s| s.chars().any(|c| c.is_ascii_punctuation())));
            assert!(password
                .iter()
                .any(|s| s.chars().any(|c| c.eq(&'あ') || c.eq(&'🍣') || c.eq(&'！'))));
        }
    }

    #[test]
    fn replace_characters() {
        // Confirm that it is overwritten by making all characters not in the candidates
        // Include characters that consist of 1 to 4 bytes to check if character boundaries are properly recognized
        let mut password = vec![
            "μ".to_string(),
            "日".to_string(),
            "🦀".to_string(),
            "1".to_string(),
            "°".to_string(),
        ];
        let old_password_length = password.len();

        let password_maker = PasswordMaker {
            others: vec![Classifier {
                candidates: ['あ', '🍣', '！'].iter().map(|&c| c.to_string()).collect(),
                minimum_count: 1, // 引数で上書き数を指定するため、値はなんでもよい
            }],
            ..PasswordMaker::default()
        };
        for classifier in &password_maker.others {
            password_maker.replace_characters(&mut password, classifier, vec![0, 4, 2]);
        }

        // The number of characters does not change
        assert_eq!(
            password.iter().map(|s| s.chars().count()).sum::<usize>(),
            old_password_length
        );

        let mut iter = password.iter();
        // The 0th character becomes one of あ, 🍣, ！
        assert!(matches!(iter.next(), Some(s) if s == "あ" || s == "🍣" || s == "！"));

        // The 1st character is the same as the 1st character of the original string
        assert_eq!(iter.next().map(String::as_str), Some("日"));

        // The 2nd character becomes one of あ, 🍣, ！
        assert!(matches!(iter.next(), Some(s) if s == "あ" || s == "🍣" || s == "！"));

        // The 3rd character is the same as the 3rd character of the original string
        assert_eq!(iter.next().map(String::as_str), Some("1"));

        // The 4th character becomes one of あ, 🍣, ！
        assert!(matches!(iter.next(), Some(s) if s == "あ" || s == "🍣" || s == "！"));
    }

    #[test]
    #[should_panic]
    fn replace_characters_panic() {
        // Test for panic when the index is out of range
        let mut password = vec![
            "μ".to_string(),
            "日".to_string(),
            "🦀".to_string(),
            "1".to_string(),
            "°".to_string(),
        ];

        let password_maker = PasswordMaker {
            others: vec![Classifier {
                candidates: ['あ', '🍣', '！'].iter().map(|&c| c.to_string()).collect(),
                minimum_count: 1, // 引数で上書き数を指定するため、値はなんでもよい
            }],
            ..PasswordMaker::default()
        };
        password_maker.replace_characters(&mut password, &password_maker.others[0], vec![5]);
    }

    #[test]
    fn unique_random_numbers() {
        let password_maker = PasswordMaker::default();

        // Generate 0 random numbers
        {
            let numbers = password_maker.unique_random_numbers(0, 0..100);
            assert_eq!(numbers.len(), 0);
        }

        // Generate 1 random number
        {
            let numbers = password_maker.unique_random_numbers(1, 0..100);
            assert_eq!(numbers.len(), 1);
            // Check if the value is within the range
            assert!(numbers[0] < 100);
        }

        // Generate 10 random numbers
        {
            let numbers = password_maker.unique_random_numbers(10, 0..100);
            assert_eq!(numbers.len(), 10);
            // Check for duplicates
            assert_eq!(
                numbers
                    .iter()
                    .collect::<std::collections::HashSet<_>>()
                    .len(),
                10
            );
            // Check if all values are within the range
            assert!(numbers.iter().all(|&x| x < 100));
        }
    }
}
