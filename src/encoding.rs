use encoding_rs::Encoding;

/// Converts a string with the specified encoding to a String type (UTF-8)
///
/// # Arguments
///
/// * `text` - The string to be converted
/// * `encoding` - The encoding
///
/// # Returns
///
/// The converted string
///
/// # Errors
///
/// If the encoding is not supported
///
/// # Examples
///
/// ```
/// let candidates = Vec::<u8>::from(vec![0x82, 0xA0, 0x82, 0xA2, 0x82, 0xA4, 0x82, 0xA6, 0x82, 0xA8]);
/// let encoding = "shift_jis".to_string();
/// let result = password_maker::encoding::decode(&candidates, &encoding);
/// assert_eq!(result, Ok("あいうえお".to_string()));
/// ```
pub fn decode(text: &[u8], encoding: &String) -> Result<String, String> {
    let encoding = Encoding::for_label_no_replacement(encoding.as_bytes())
        .ok_or(format!("Unsupported encoding: {}", encoding))?;

    Ok(encoding.decode(text).0.into_owned())
}

/// Converts a UTF-8 string to a string with the specified encoding
///
/// # Arguments
///
/// * `text` - The string to be converted
/// * `encoding` - The encoding
///
/// # Returns
///
/// The converted string
///
/// # Errors
///
/// If the encoding is not supported
///
/// # Examples
///
/// ```
/// let text = "あいうえお";
/// let encoding = "shift_jis";
/// let result = password_maker::encoding::encode(text, encoding);
/// assert_eq!(result, Ok(Vec::<u8>::from(vec![0x82, 0xA0, 0x82, 0xA2, 0x82, 0xA4, 0x82, 0xA6, 0x82, 0xA8])));
/// ```
pub fn encode(text: &str, encoding: &str) -> Result<Vec<u8>, String> {
    let encoding = Encoding::for_label_no_replacement(encoding.as_bytes())
        .ok_or(format!("Unsupported encoding: {}", encoding))?;

    Ok(encoding.encode(text).0.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_from_utf8() {
        let candidates = Vec::<u8>::from("あいうえお");
        let encoding = "utf-8".to_string();
        let result = decode(&candidates, &encoding);
        assert_eq!(result, Ok("あいうえお".to_string()));
    }

    #[test]
    fn decode_from_shift_jis() {
        // Shift_JIS encoding of "あいうえお"
        let candidates = vec![0x82, 0xA0, 0x82, 0xA2, 0x82, 0xA4, 0x82, 0xA6, 0x82, 0xA8];
        let encoding = "shift_jis".to_string();
        let result = decode(&candidates, &encoding);
        assert_eq!(result, Ok("あいうえお".to_string()));
    }

    #[test]
    fn decode_from_invalid_encoding() {
        let candidates = Vec::<u8>::from("abc");
        let encoding = "invalid".to_string();
        let result = decode(&candidates, &encoding);
        assert_eq!(result, Err("Unsupported encoding: invalid".to_string()));
    }

    #[test]
    fn decode_from_different_encoding() {
        // "あいうえお"のShift_JIS
        let candidates = vec![0x82, 0xA0, 0x82, 0xA2, 0x82, 0xA4, 0x82, 0xA6, 0x82, 0xA8];
        let encoding = "utf-8".to_string();
        let result = decode(&candidates, &encoding);
        // Since the encoding is different, the original Shift_JIS byte sequence is returned as is.
        // It is returned as Ok, but there may be cases where the character conversion does not work well.
        // Since the user specified the encoding, we will leave this behavior as it is for now.
        assert_eq!(
            result,
            Ok(String::from_utf8_lossy(b"\x82\xA0\x82\xA2\x82\xA4\x82\xA6\x82\xA8").to_string())
        );
    }

    #[test]
    fn encode_to_utf8() {
        let text = "あいうえお";
        let encoding = "utf-8";
        let result = encode(text, encoding);
        assert_eq!(result, Ok(Vec::<u8>::from("あいうえお")));
    }

    #[test]
    fn encode_to_shift_jis() {
        let text = "あいうえお";
        let encoding = "shift_jis";
        let result = encode(text, encoding);
        // "あいうえお"のShift_JIS
        assert_eq!(
            result,
            Ok(vec![
                0x82, 0xA0, 0x82, 0xA2, 0x82, 0xA4, 0x82, 0xA6, 0x82, 0xA8
            ])
        );
    }

    #[test]
    fn encode_to_invalid_encoding() {
        let text = "abc";
        let encoding = "invalid";
        let result = encode(text, encoding);
        assert_eq!(result, Err("Unsupported encoding: invalid".to_string()));
    }
}
