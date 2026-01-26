use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::env;
use std::process;

fn encode(token_to_encode: &str) {
    println!("you have reached the encode function");
}

fn decode(token_to_decode: &str) {
    println!("you have reached the decode function");
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
            let mut s = String::new();
            match file.read_to_string(&mut s) {
                Err(why) => panic!("couldn't read {}: {}", display, why),
                Ok(_) => print!("{} contains:\n{}", display, s),
            }
            
            encode(&s);
        }

        "DECODE" => {
            // DECODE <single_token> <output_file>
            if args.len() != 4 {
                println!("ERROR: Argument number\n./ALPHABETENCODING DECODE <single_token> <input_file>");
                process::exit(1);
            }
            let encoded_token = &args[2];
            let output_path = &args[3];
            
            decode(encoded_token);

            // write to file 
        }

        _ => {
            // invalid mode
            println!("ERORR: Invalid mode\nUsage: ./ALPHABETENCODING ENCODE <input_file> OR ./ALPHABETENCODING DECODE <single_token> <input_file>");
            process::exit(1);
        }
    }

    // `file` goes out of scope, and the "hello.txt" file gets closed
}