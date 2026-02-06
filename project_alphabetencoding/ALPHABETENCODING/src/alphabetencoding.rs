use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::env;
use std::process;
use std::fs::OpenOptions;

fn encode(input_path: &str) {
    // read file
    // create path
    let _path = Path::new(input_path);
    // let _display = path.display();

    // Open the path in read-only mode
    let mut file = match File::open(&_path) {
        Ok(file) => file,
        Err(_) => process::exit(1),
    };

    // Read the file contents into a string
    let mut token_to_encode = String::new();
    if let Err(_why) = file.read_to_string(&mut token_to_encode) {
        // eprintln!("ERROR: couldn't read {}: {}", _display, _why);
        process::exit(1); // exit with non-zero status
    }

    if token_to_encode.is_empty() {
        process::exit(1);
    }

    // parse the string
    let bytes = token_to_encode.as_bytes();
    let mut index = 0;
    let mut output = String::new();

    while index < bytes.len() {
        let b = bytes[index];

        // ascii graphics are unprintable, we dont accept whitespace, we dont accept unsafe characters
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
    println!("OUTPUT :{}:", output);
}

fn decode(encoded_token: &str, output_path: &str) {
    // Try to create output file, exit 1 on failure
    let mut output_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_path)
    {
        Ok(f) => f,
        Err(_) => process::exit(1),
    };

    let bytes = encoded_token.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'x' {
            if index + 2 >= bytes.len() {
                process::exit(1); // malformed token
            }

            let hex_str = &encoded_token[index + 1..index + 3];
            let byte_val = match u8::from_str_radix(hex_str, 16) {
                Ok(b) => b,
                Err(_) => process::exit(1),
            };

            if let Err(_) = output_file.write_all(&[byte_val]) {
                process::exit(1);
            }

            index += 3;
        } else {
            let c = bytes[index];
            if (c.is_ascii_graphic() || c == b' ') && c != b':' && c != b'\\' && c != b'x' {
                if let Err(_) = output_file.write_all(&[c]) {
                    process::exit(1);
                }
            } else {
                process::exit(1); // invalid literal
            }
            index += 1;
        }
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // println!("ERROR: Argument number\nUsage: ./ALPHABETENCODING ENCODE <input_file> OR ./ALPHABETENCODING DECODE <single_token> <input_file>");
        process::exit(1);
    }

    let mode = args[1].as_str();

    match mode {
        "ENCODE"  | "encode" => {
            // ENCODE <input_file>
            if args.len() != 3 {
                // println!("ERROR: Argument number\nUsage: ./ALPHABETENCODING ENCODE <input_file>");
                process::exit(1);
            }
            let input_path = &args[2];
            
            encode(input_path);
        }

        "DECODE" | "decode" => {
            // DECODE <single_token> <output_file>
            if args.len() != 4 {
                // println!("ERROR: Argument number\n./ALPHABETENCODING DECODE <single_token> <input_file>");
                process::exit(1);
            }
            let encoded_token = &args[2];
            let output_path = &args[3];
            
            decode(encoded_token, output_path);
        }

        _ => {
            // invalid mode
            // println!("ERORR: Invalid mode\nUsage: ./ALPHABETENCODING ENCODE <input_file> OR ./ALPHABETENCODING DECODE <single_token> <input_file>");
            process::exit(1);
        }
    }
}