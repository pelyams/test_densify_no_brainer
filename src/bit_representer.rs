use std::collections::HashMap;

const CONTROL_BIT_VALUE_U16: u16 = 0b10000000_00000000;
const CONTROL_BIT_COUNT_U8: u8 = 0b11000000;
const CONTROL_BIT_COUNT_U16: u16 = 0b11100000_00000000;

pub(crate) fn encode_to_bitwise_string(s: &str) -> String {
    let mut hm = HashMap::new();
    // (ideally, i should check if condition of input numbers count doesn't exceed 1000 is fulfilled)
    s.split_terminator(',').for_each(|n| {
        hm.entry(n.trim()).and_modify(|e| *e += 1).or_insert(1);
    });
    let mut output = hm
        .into_iter()
        .map(|(k, v)| {
            if v == 1 {
                value_to_string(k)
            } else {
                let mut entry = count_to_string(v);
                entry.push_str(&value_to_string(k));
                entry
            }
        })
        .collect::<String>();
    // pad string length to be product of 6, aligning with base64
    while output.len() % 6 != 0 {
        output.push('0');
    }
    output
}

fn count_to_string(count: usize) -> String {
    if count <= 31 {
        format!("{:08b}", count as u8 | CONTROL_BIT_COUNT_U8)
    } else {
        format!("{:016b}", count as u16 | CONTROL_BIT_COUNT_U16)
    }
}

fn value_to_string(value: &str) -> String {
    // function should rather return errors in following cases, but who cares
    let int_value = value
        .parse::<u16>()
        .expect("could not convert value to u16");
    assert!(int_value <= 300, "input values must be le 300");
    // actually, i am pretty fine with 0, but this comes from problem conditions
    assert!(int_value > 0, "input values must be greater than 0");

    if int_value <= 127 {
        format!("{:08b}", int_value as u8)
    } else {
        format!("{:016b}", int_value | CONTROL_BIT_VALUE_U16)
    }
}

pub(crate) fn decode_to_vec(s: &str) -> Result<Vec<u16>, BitsDecodeError> {
    if s.len() < 8 {
        return Ok(vec![]);
    }

    // remove padding if there's any:
    let mut s_end_idx = s.len();
    while s_end_idx % 8 != 0 {
        s_end_idx -= 1;
    }

    let mut start_idx = 0;
    let mut output = Vec::<u16>::new();

    while start_idx < s_end_idx {
        let mut count = 1;
        if s[start_idx..].starts_with("11") {
            let (_count, offset) = extract_count(&s[start_idx..])?;
            count = _count;
            start_idx += offset;
        }
        let (number, offset) = extract_value(&s[start_idx..])?;
        for _ in 0..count {
            output.push(number);
        }
        start_idx += offset;
    }
    Ok(output)
}

fn extract_count(s: &str) -> Result<(u16, usize), BitsDecodeError> {
    match s.chars().nth(2).unwrap() {
        '0' => Ok((
            (u8::from_str_radix(&s[..8], 2)? ^ CONTROL_BIT_COUNT_U8) as u16,
            8,
        )),
        '1' => {
            if s.len() < 16 {
                return Err(BitsDecodeError::UnexpectedEndOfInput);
            }
            Ok((
                u16::from_str_radix(&s[..16], 2)? ^ CONTROL_BIT_COUNT_U16,
                16,
            ))
        }
        x => Err(BitsDecodeError::UnexpectedChar(x)),
    }
}

fn extract_value(s: &str) -> Result<(u16, usize), BitsDecodeError> {
    if s.len() < 8 {
        return Err(BitsDecodeError::UnexpectedEndOfInput);
    }
    match s.chars().nth(0).unwrap() {
        '0' => Ok((u8::from_str_radix(&s[..8], 2)? as u16, 8)),
        '1' => {
            if s.len() < 16 {
                return Err(BitsDecodeError::UnexpectedEndOfInput);
            }
            Ok((
                (u16::from_str_radix(&s[..16], 2)? ^ CONTROL_BIT_VALUE_U16),
                16,
            ))
        }
        x => Err(BitsDecodeError::UnexpectedChar(x)),
    }
}

#[derive(Debug)]
pub(crate) enum BitsDecodeError {
    ParseError(std::num::ParseIntError),
    UnexpectedEndOfInput,
    UnexpectedChar(char),
}

impl From<std::num::ParseIntError> for BitsDecodeError {
    fn from(err: std::num::ParseIntError) -> BitsDecodeError {
        BitsDecodeError::ParseError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        let input = "";
        let encoded = encode_to_bitwise_string(input);
        let decoded = decode_to_vec(&encoded).unwrap();
        assert_eq!(decoded, vec![]);
    }

    #[test]
    fn test_single_value() {
        let input = "42";
        let encoded = encode_to_bitwise_string(input);
        let decoded = decode_to_vec(&encoded).unwrap();
        assert_eq!(decoded, vec![42]);
    }

    #[test]
    fn test_multiple_unique_values() {
        let input = "1,2,3,4,5";
        let encoded = encode_to_bitwise_string(input);
        let decoded = decode_to_vec(&encoded).unwrap();
        let mut expected = vec![1, 2, 3, 4, 5];
        let mut result = decoded;
        expected.sort();
        result.sort();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_repeated_values() {
        let input = "7,7,7";
        let encoded = encode_to_bitwise_string(input);
        let decoded = decode_to_vec(&encoded).unwrap();
        let mut result = decoded;
        result.sort();
        assert_eq!(result, vec![7, 7, 7]);
    }

    #[test]
    fn test_mixed_single_and_repeated() {
        let input = "1,2,2,3,3,3,4";
        let encoded = encode_to_bitwise_string(input);
        let decoded = decode_to_vec(&encoded).unwrap();
        let mut result = decoded;
        result.sort();
        assert_eq!(result, vec![1, 2, 2, 3, 3, 3, 4]);
    }

    #[test]
    fn test_boundary_values_8_bit() {
        // values at 7-bit boundaries (127, 128)
        let input = "127,128";
        let encoded = encode_to_bitwise_string(input);
        let decoded = decode_to_vec(&encoded).unwrap();
        let mut result = decoded;
        result.sort();
        assert_eq!(result, vec![127, 128]);
    }

    #[test]
    fn test_count_boundary_cases() {
        // 5 bit boundary (aka 31 vs 32)
        let repeated_31 = "5,".repeat(31);
        let repeated_32 = "5,".repeat(32);

        let input_31 = &repeated_31[..repeated_31.len() - 1];
        let input_32 = &repeated_32[..repeated_32.len() - 1];

        let encoded_31 = encode_to_bitwise_string(input_31);
        let encoded_32 = encode_to_bitwise_string(input_32);

        let decoded_31 = decode_to_vec(&encoded_31).unwrap();
        let decoded_32 = decode_to_vec(&encoded_32).unwrap();

        assert_eq!(decoded_31.len(), 31);
        assert_eq!(decoded_32.len(), 32);
        assert!(decoded_31.iter().all(|&x| x == 5));
        assert!(decoded_32.iter().all(|&x| x == 5));
    }

    #[test]
    fn test_large_counts() {
        // (with 16 bit encoding)
        let repeated_100 = "9,".repeat(100);
        let input = &repeated_100[..repeated_100.len() - 1];

        let encoded = encode_to_bitwise_string(input);
        let decoded = decode_to_vec(&encoded).unwrap();

        assert_eq!(decoded.len(), 100);
        assert!(decoded.iter().all(|&x| x == 9));
    }

    #[test]
    fn test_roundtrip() {
        let test_cases = vec![
            "1,2,3,4,5",
            "100,100,200,300,300,300",
            "42",
            "1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1",
        ];

        for input in test_cases {
            let encoded = encode_to_bitwise_string(input);
            let decoded = decode_to_vec(&encoded).unwrap();

            let mut original: Vec<u16> = input.split(',').map(|s| s.parse().unwrap()).collect();
            original.sort();

            let mut result = decoded;
            result.sort();

            assert_eq!(result, original, "roundtrip failed for input: {}", input);
        }
    }

    #[test]
    fn test_decode_less_than_8_bits() {
        // should return empty vec
        assert!(decode_to_vec("1234567").is_ok());
        assert!(decode_to_vec("1234567").unwrap().is_empty());
    }

    #[test]
    fn test_decode_error_cases() {
        // some invalid binary strings
        assert!(decode_to_vec("abcdefgh").is_err());
        assert!(decode_to_vec("1111111G").is_err());
    }

    #[test]
    fn test_encode_output_length() {
        let test_inputs = vec!["1", "1,2", "1,2,3", "1,2,3,4", "1,2,3,4,5"];

        for input in test_inputs {
            let encoded = encode_to_bitwise_string(input);
            assert_eq!(
                encoded.len() % 6,
                0,
                "length is not multiple of 6 for input: {}",
                input
            );
        }
    }
}
