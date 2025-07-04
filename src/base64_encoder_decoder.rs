pub(crate) fn base2_to_base64(s: &str) -> String {
    // bit_representer guarantees valid input string
    // so we can assume it has a proper length (if not empty), and unwrap() won't panic
    if s.len() == 0 {
        return "".to_owned();
    }
    let mut start_idx = 0;
    let mut end_idx = 6;
    let mut slices = Vec::new();
    loop {
        slices.push(&s[start_idx..end_idx]);
        if end_idx >= s.len() {
            break;
        }
        start_idx += 6;
        end_idx += 6;
    }
    slices
        .into_iter()
        .map(|str_radix_2| (u8::from_str_radix(str_radix_2, 2).unwrap() + 32) as char)
        .collect()
}

pub(crate) fn base64_to_base2(s: &str) -> Result<String, Base64DecodeError> {
    let mut base2 = Vec::with_capacity(s.len());
    for ch in s.chars() {
        let v = ch as u8 - 32;
        if v >= 64 {
            return Err(Base64DecodeError::InvalidBase64Character(ch));
        }
        base2.push(format!("{:06b}", v));
    }
    Ok(base2.into_iter().collect::<String>())
}

#[derive(Debug)]
pub enum Base64DecodeError {
    InvalidBase64Character(char),
}

#[cfg(test)]
mod base64_conversion_tests {
    use super::*;

    #[test]
    fn test_base2_to_base64_simple() {
        let input = "000000";
        let result = base2_to_base64(input);
        assert_eq!(result, " ");

        let input = "111111";
        let result = base2_to_base64(input);
        assert_eq!(result, "_");
    }

    #[test]
    fn test_base2_to_base64_multiple_chunks() {
        let input = "000000111111";
        let result = base2_to_base64(input);
        assert_eq!(result, " _");

        let input = "000001000010000011";
        let result = base2_to_base64(input);
        assert_eq!(result, "!\"#");
    }

    #[test]
    fn test_base64_to_base2_simple() {
        let input = " ";
        let result = base64_to_base2(input).unwrap();
        assert_eq!(result, "000000");

        let input = "_";
        let result = base64_to_base2(input).unwrap();
        assert_eq!(result, "111111");
    }

    #[test]
    fn test_roundtrip_conversion() {
        let test_cases = vec![
            "000000",
            "111111",
            "101010",
            "000000111111101010",
            "001100110011001100110011",
        ];

        for input in test_cases {
            let base64 = base2_to_base64(input);
            let back_to_base2 = base64_to_base2(&base64).unwrap();
            assert_eq!(
                back_to_base2, input,
                "Roundtrip failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_base64_to_base2_error_handling() {
        let invalid_inputs = vec![
            "`", // ASCII 96
            "~", // ASCII 126
            "€", // non-ASCII at all!
        ];

        for input in invalid_inputs {
            let result = base64_to_base2(input);
            assert!(result.is_err(), "expected error for input: {}", input);
        }
    }

    #[test]
    fn test_base64_to_base2_empty() {
        let result = base64_to_base2("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }
}
