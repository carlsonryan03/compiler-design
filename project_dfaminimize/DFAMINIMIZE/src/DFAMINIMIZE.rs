mod dfa;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process;
use crate::dfa::{DFA, StateRow};
use std::fs::OpenOptions;

fn read_lines<P>(input_path: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
        let file = File::open(input_path)?;
        Ok(io::BufReader::new(file).lines())
        // where to throw errors
}

// 
fn get_tt_from_file(input_path: &str) -> Vec<StateRow> {
    // get lines for the tt
    let mut is_first_line = true;
    let mut num_transitions = u32::MAX;

    let mut tt: Vec<StateRow> = Vec::new();

    if let Ok(lines) = read_lines(input_path) {
        for line in lines.map_while(Result::ok) {
            let delimited_line: Vec<&str> = line.split_whitespace().collect();

            let mut current_row = StateRow {
                accepting : false,
                state_id : u32::MAX,
                transitions : Vec::new(),
            };

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
    tt
}

fn main() {
    // need to read from files
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("ERROR: Argument number\nUsage: cargo run -- <input_file> <output_file>");
        process::exit(1);
    }
    let input_path = &args[1];
    let output_path = &args[2];

    // jank order but I want to check ifwe can access the output file before doing all this extra work
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

    // parse input into a transition table
    let tt = get_tt_from_file(input_path);

    // // Debug
    // for row in &tt {
    //     println!("{:?}", row); // need debug flag for this
    // }
    
    // Create DFA object
    let mut dfa = DFA::new(tt).expect("Failed to create DFA");

    // println!("Initial DFA:\n{}", dfa.print()); // DEBUG
    
    // call minimize
    dfa.minimize();

    println!("Final DFA:\n{}", dfa.print()); // DEBUG

    if let Err(_) = output_file.write_all(dfa.print().as_bytes()) {
        eprintln!("ERROR: Could not write dfa to output file {}\n. Will print dfa to stderr: {}", output_path, dfa.print());
        process::exit(1);
    }
    
    // TODO: later, write it to file specified in clargs

    // Debug: test on inputs
}
