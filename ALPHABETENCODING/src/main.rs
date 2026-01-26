use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::env;
use std::process;

fn encode(input_path: &str) {
    println!("you have reached the encode function");
    // read file
    // create path
    let path = Path::new(input_path);
    let display = path.display();

    // Open the path in read-only mode
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    // Read the file contents into a string
    let mut token_to_encode = String::new();
    match file.read_to_string(&mut token_to_encode) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => print!("{} contains:\n{}", display, token_to_encode),
    }

    // parse the string
    let bytes = token_to_encode.as_bytes();
    let mut index = 0;
    let mut output = String::new();

    while index < bytes.len() {
        let b = bytes[index];

        // Decide if this byte must be encoded

        if !b.is_ascii_graphic()
            || b.is_ascii_whitespace()
            || b == b':' || b == b'\\' || b == b'x'
            || !((b'!'..=b'9').contains(&b)
            || (b';'..=b'[').contains(&b)
            || (b']'..=b'w').contains(&b)
            || (b'y'..=b'~').contains(&b)) {
            // encode in hex
            output.push_str(&format!("x{:02x}", b));
        } else {
            // keep literal
            output.push(b as char);
        }

        index += 1;
    }
    println!("{}", output)
}

fn decode(token_to_decode: &str, output_path: &str) {
    // Open output file in write mode
    let mut output_file = match File::create(output_path) {
        Ok(f) => f,
        Err(_) => process::exit(1), // assignment: no output, exit non-zero
    };

    let bytes = token_to_decode.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'x' {
            // Check if there are at least 2 more characters
            if index + 2 >= bytes.len() {
                println!("ERROR: Malformed token. Not enough characters after 'x'");
                process::exit(1);
            }

            // Read the two hex digits
            let hex_str = &token_to_decode[index + 1..index + 3]; // exclusive on end

            // Convert hex to u8
            let byte_value = match u8::from_str_radix(hex_str, 16) {
                Ok(b) => b,
                Err(_) => process::exit(1),
            };

            // Write the byte to the file
            if let Err(_) = output_file.write_all(&[byte_value]) {
                process::exit(1);
            }

            index += 3; // skip past xHH
        } 
        // Not in hex, treat as literal character
        else {
            let c = bytes[index];

            // Only printable and NOT ':', '\', or 'x'
            if (c.is_ascii_graphic() || c == b' ') && c != b':' && c != b'\\' && c != b'x' {
                if let Err(_) = output_file.write_all(&[c]) {
                    println!("ERROR: Could not open file");
                    process::exit(1);
                }
            } else {
                // invalid literal character
                println!("ERROR: invalid literal character!");
                process::exit(1);
            }

            index += 1;
        }
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("ERROR: Argument number\nUsage: ./ALPHABETENCODING ENCODE <input_file> OR ./ALPHABETENCODING DECODE <single_token> <input_file>");
        process::exit(1);
    }

    let mode = args[1].as_str();

    match mode {
        "ENCODE" => {
            // ENCODE <input_file>
            if args.len() != 3 {
                println!("ERROR: Argument number\nUsage: ./ALPHABETENCODING ENCODE <input_file>");
                process::exit(1);
            }
            let input_path = &args[2];
            
            encode(input_path);
        }

        "DECODE" => {
            // DECODE <single_token> <output_file>
            if args.len() != 4 {
                println!("ERROR: Argument number\n./ALPHABETENCODING DECODE <single_token> <input_file>");
                process::exit(1);
            }
            let encoded_token = &args[2];
            let output_path = &args[3];
            
            decode(encoded_token, output_path);
        }

        _ => {
            // invalid mode
            println!("ERORR: Invalid mode\nUsage: ./ALPHABETENCODING ENCODE <input_file> OR ./ALPHABETENCODING DECODE <single_token> <input_file>");
            process::exit(1);
        }
    }

    // `file` goes out of scope, and the "hello.txt" file gets closed
}