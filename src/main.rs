use crate::base64_encoder_decoder::Base64DecodeError::InvalidBase64Character;
use crate::densifyer::{densify, revert_to_original_string, DensifyerError};
use std::env;

mod base64_encoder_decoder;
mod bit_representer;
mod densifyer;

fn main() {
    //okay, let's pretend this is not a useless program
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: {} (-d | -e) <message>", args[0]);
        std::process::exit(1);
    }
    let flag = &args[1];
    let input = &args[2];

    if input.is_empty() {
        eprintln!("input message cannot be empty");
        std::process::exit(1);
    }

    match flag.as_str() {
        "-d" => {
            match revert_to_original_string(input) {
                Ok(s) => println!("{}", s),
                Err(e) => {
                    match e {
                        DensifyerError::Base64DecodingError(InvalidBase64Character(c)) => {
                            eprintln!("base64 decoding error, found invalid symbol: {}", c)
                        }
                        // unlikely!
                        DensifyerError::BitRepresentationDecodingError(_) => {
                            eprintln!("service has some inner troubles🚬")
                        }
                    }
                }
            }
        }
        "-e" => {
            //  validate input string
            if !input.chars().all(|c| c.is_ascii_digit() || c == ',') {
                eprintln!("input must be comma-separated numbers (e.g., '1,2,3')");
                std::process::exit(1);
            }

            // (another validation step)
            let validation_result: Result<Vec<u16>, _> =
                input.split(',').map(|s| s.parse::<u16>()).collect();

            match validation_result {
                Ok(_) => {
                    let compressed = densify(input);
                    println!("{}", compressed);
                }
                Err(parse_err) => {
                    eprintln!("invalid number format: {}", parse_err);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("invalid flag: {}, use -d or -e", flag);
            std::process::exit(1);
        }
    }
}
