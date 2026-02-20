mod dfa;
mod alphabetencoding;

use std::env;
use std::io::{self, BufRead, Write};
use std::process;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::fs::File;
use std::path::Path;
use dfa::{DFA, StateRow};
use std::fs;

#[derive(Debug)]
pub struct TokenRecognizer {
    pub dfa: DFA,
    pub token_id: String,
    pub token_value: Option<String>,
}

fn read_lines<P>(input_path: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(&input_path).map_err(|e| {
        eprintln!("ERROR: could not open file '{}': {}", input_path.as_ref().display(), e);
        e
    })?;
    Ok(io::BufReader::new(file).lines())
}


// 
fn get_dfa_from_file(input_path: &str, alphabet: HashMap<u8, usize>) -> DFA {
    // println!("Input path for dfa: {}", input_path);
    // get lines for the tt
    let mut is_first_line = true;
    let mut num_transitions = u32::MAX;

    let mut tt: Vec<StateRow> = Vec::new();

    if let Ok(lines) = read_lines(input_path) {
        // println!("We can read the file");
        for line in lines.map_while(Result::ok) {
            let delimited_line: Vec<&str> = line.split_whitespace().collect();

            let mut current_row = StateRow {
                accepting : false,
                state_id : u32::MAX,
                transitions : Vec::new(),
            };

            // println!("{}", line);

            // on first line, check number of transitions we should have
            if is_first_line {
                num_transitions = delimited_line.len() as u32 - 2;
                is_first_line = false;
            }
            else if num_transitions != (delimited_line.len() as u32 - 2) {
                eprintln!("ERROR (malformed tt): Based on first line of tt, expected {} transitions, but got {} transitions.", num_transitions, delimited_line.len() - 2);
                process::exit(1);
            }

            // first symbol needs to be a +/-
            if delimited_line[0] != "+" && delimited_line[0] != "-" {
                eprintln!("ERROR: first symbol of line must be +/-, read {}", delimited_line[0]);
                process::exit(1);
            }
            else if delimited_line[0] == "+" {
                current_row.accepting = true;
            }

            // second symbol needs to be an int
            current_row.state_id = delimited_line[1].parse::<u32>().unwrap_or_else(|_| {
                eprintln!("ERROR: read second symbol of transition table as {} and expected a nonnegative integer!", delimited_line[1]);
                process::exit(1);
            });

            // everything else is a transition holding some int or E
            for i in 2..=(delimited_line.len() as u32 - 1)  {
                if delimited_line[i as usize] == "E" {
                    current_row.transitions.push(None);
                }
                else {
                    let transition = delimited_line[i as usize].parse::<u32>().unwrap_or_else(|_| {
                        eprintln!("ERROR (invalid transition): read {} and expected E or a nonnegative integer!", delimited_line[i as usize]);
                        process::exit(1);
                    });
                    current_row.transitions.push(Some(transition));
                }
            }
            tt.push(current_row.clone());
        }
    }
    // println!("tt size: {}", tt.len());
    DFA::new(tt, alphabet).expect("Failed to create DFA")
}

// expect line 1, whitespace delimited and alphabet encoded alphabet
// subsequent lines: path to tt, token id, optional token value
fn parse_scanner_file(input_path: &str) -> HashMap<usize, TokenRecognizer> {
    // println!("I'm beign called");

    let mut scanner = HashMap::new();

    if let Ok(mut lines) = read_lines(input_path) {
        // get the alphabet, which is on the first line
        let enc_alphabet = match lines.next() {
            Some(Ok(line)) => line,
            _ => {
                eprintln!("ERROR: alphabet is empty.");
                process::exit(1);
            }
        };

        let enc_alphabet_clean: String = enc_alphabet.chars()
            .filter(|c| !c.is_whitespace())
            .collect();

        // println!("Encoded alphabet: {:?}", enc_alphabet_clean); //DEBUG
        let dec_alphabet: Vec<u8> = alphabetencoding::decode(&enc_alphabet_clean).bytes().collect();
        // println!("Decoded alphabet: {:?}", dec_alphabet); //DEBUG

        let mut alphabet: HashMap<u8, usize> = HashMap::new();

        // now we iterate through every character and add it to a hashmap for alphabet
        for (i, b) in dec_alphabet.iter().enumerate() {
            alphabet.insert(*b, i);
        }

        // then parse all subsequent lines
        let mut token_recognizer_index = 0;
        for line in lines.map_while(Result::ok) {
            let delimited_line: Vec<String> = line.split_whitespace().map(String::from).collect();

            // just an edge case but we should skip the line if its a newline
            if delimited_line.len() == 0 {
                // line is empty. This is fine
                continue;
            }

            // we need to check if there are not 2 or 3 args
            if delimited_line.len() != 2 && delimited_line.len() != 3 {
                eprintln!("ERROR: malformed scanner defn file: read {} but expected <tt_path> <token_id> OPTIONAL<token_value>", line);
            }

            // create the token recognizer from the transition table file

            let mut token_val = None;
            if delimited_line.len() == 3 {
                token_val = Some(delimited_line[2].clone());
            }

            let token_rec = TokenRecognizer {
                dfa : get_dfa_from_file(&delimited_line[0], alphabet.clone()),
                token_id : delimited_line[1].clone(),
                token_value : token_val,
            };

            // println!("Token recognizer {}: {:?}", token_recognizer_index, token_rec);
            // add dfa to a hashmap or something which also contains the token_id and value or None
            scanner.insert(token_recognizer_index, token_rec);

            token_recognizer_index += 1;
        }
    }
    scanner
}

/*
TODO: i think to_tokenize might be mandatorily separated by newlines. So tokens cant span lines. 
this means that i need to split it up by lines too and do iterations by line
when theres no more lines and no more content we are done

that way keeping track of line number is easy. Column number will be tracked by an index anyway. we dont
even need to splice the string, we can just keep an index. In fact, that is definiely the way to do this
*/

// goes through a string and attempts to scan the largest possible section into a token
fn tokenize(to_be_tokenized: &str, scanner: &mut HashMap<usize, TokenRecognizer>) -> String {
    let mut tokens = String::new();
    
    // Tokenize until there's nothing left
    let mut current_line = 1;
    let mut current_column = 1;
    let mut token_start_index = 0;

    while token_start_index < to_be_tokenized.len() {
        // try the entire remaining string on every token recognizer in the scanner in ascending order of scanner id
        let mut longest_token_length = 0;
        let mut longest_token = String::new();
        let mut longest_token_index: Option<usize> = None;

        for i in 0..scanner.len() {
            // println!("TR {} is trying to scan {}", scanner[&i].token_id, &to_be_tokenized[token_start_index..to_be_tokenized.len()]);
            if let Some(token_rec) = scanner.get_mut(&i) {
                token_rec.dfa.simulate(&to_be_tokenized[token_start_index..]);
            }
            
            // if the dfa accepted, see if its the longest weve seen so far
            if scanner[&i].dfa.self_is_accepting() {
                // println!("TR {} accepted {}", scanner[&i].token_id, scanner[&i].dfa.get_longest_accepting_match());
                let longest_seq = scanner[&i].dfa.get_longest_accepting_match();

                if longest_seq.len() > longest_token_length {
                    longest_token_length = longest_seq.len();
                    longest_token = longest_seq;
                    longest_token_index = Some(i);
                }
            }
        }

        // No token was matched at all
        let idx = match longest_token_index {
            Some(i) => i,
            None => {
                eprintln!("ERROR: No token was found at line {} col {}", current_line, current_column);
                process::exit(1);
            }
        };

        // we found the longest token from the remaining portion
        // the value is the token if no value specified
        let token_rec = &scanner[&idx];
        // println!("The longest token was found by {}", token_rec.token_id);

        // TODO:
        /*
        figure out why its pushing so many things argjkhakdghalksdgjh
        println!("TR {} accepted {}" is showing that token's longest accepted match is nothing. That's a problem
        its also saying the longest token was found by a because of that
        TODO
        newlines are being ignored! We need to not do that!
        */

        let token_val = match &token_rec.token_value {
            Some(v) => v.clone(),
            None => {
                let value = alphabetencoding::encode(&longest_token.clone());   
                value
            }
        };

        tokens.push_str(&format!("{} {} {} {}\n", token_rec.token_id, token_val, current_line, current_column));
        // println!("Just found a token: {} {} {} {}", token_rec.token_id, token_val, current_line, current_column);
        // println!("Tokens:\n{}", tokens);

        for c in longest_token.chars() {
            if c == '\n' {
                current_line += 1;
                current_column = 1;
            }
            else {
                current_column += 1;
            }
        }

        // finally, update the index of the string to be past the token we read
        token_start_index += longest_token_length;
    }
    tokens
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        eprintln!("ERROR: Argument number\nUsage: cargo run -- <scanner_definition_file> <file_to_be_tokenized> <output_file>");
        process::exit(1);
    }
    let scanner_path = &args[1];
    let input_path = &args[2];
    let output_path = &args[3];

    // call the parser for the scanner file
    let mut scanner = parse_scanner_file(scanner_path);

    // read in input to be tokenized
    let to_be_tokenized = match fs::read_to_string(input_path) {
        Ok(contents) => contents,
        Err(_) => {
            eprintln!("ERROR: could not read from {}", input_path);
            process::exit(1);
        }
    };
    // if to_be_tokenized.is_empty() {
    //     eprintln!("ERROR: empty program.src! Nothing to tokenize.");
    //     process::exit(1);
    // }

    let tokenized_data = tokenize(&to_be_tokenized, &mut scanner);

    // DEBUG
    // println!("Tokenize:\n{}", to_be_tokenized);
    // println!("Scanner token recognizers: {}", scanner.len());
    // println!("Tokenized data: {}", tokenized_data);

    // jank order but I want to check if we can access the output file before doing all this extra work
    let mut output_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_path)
    {
        Ok(file) => file, // successfully opened the file
        Err(_) => {
            eprintln!("ERROR: Could not write/create/truncate/open output file. No permissions.");
            process::exit(1);
        }
    };

    if let Err(e) = output_file.write_all(tokenized_data.as_bytes()) {
        eprintln!("ERROR: Failed to write tokenized data to output file: {}", e);
        process::exit(1);
    }

    // // Debug
    // for row in &tt {
    //     println!("{:?}", row); // need debug flag for this
    // }
    
    // Create DFA object

}