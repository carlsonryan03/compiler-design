use std::collections::HashSet;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
struct Production {
    lhs: String,
    rhs: Vec<String>,
}

#[derive(Debug)]
struct CFG {
    productions: Vec<Production>,
    non_terminals: HashSet<String>,
    symbols: HashSet<String>,
    start_symbol: String,
}

fn is_nonterminal(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_uppercase())
}

fn parse_grammar(input: &str) -> CFG {
    let mut productions = Vec::new();
    let mut non_terminals = HashSet::new();
    let mut symbols = HashSet::new();
    let mut start_symbol = String::new();

    let mut current_lhs: Option<String> = None;

    for line in input.lines() {
        let tokens: Vec<&str> = line.split_whitespace().collect();

        if tokens.len() == 0 {
            continue;
        }

        let mut i = 0;

        // New LHS rule
        if tokens.len() >= 2 && tokens[1] == "->" {
            let lhs = tokens[0].to_string();
            if !is_nonterminal(&lhs) {
                eprintln!("{} should be a nonterminal but isnt!", lhs);
                std::process::exit(1);
            }
            non_terminals.insert(lhs.clone());
            current_lhs = Some(lhs.clone());

            // if this is the first nt we have found its the start
            if start_symbol.is_empty() {
                start_symbol = lhs.clone();
            }

            i = 2;
        } else if tokens[0] == "|" {
            i = 1;
        }
        else {
            eprintln!("Malformed grammar");
            std::process::exit(1);
        }

        let mut rhs = Vec::new();

        // everything else should be a rhs production
        while i < tokens.len() {
            let token = tokens[i];

            if token == "|" {
                // finish current production
                productions.push(Production {lhs: current_lhs.clone().expect("asdlfknaksdvjn"), rhs: rhs.clone()});
                rhs.clear();
            } else {
                if token != "lambda" {
                    if is_nonterminal(token) {
                        non_terminals.insert(token.to_string());
                    }
                    symbols.insert(token.to_string());
                }
                // lambda is special and is not a terminal!!! it shouldnt be added to the sets but does need to be counted in productions
                rhs.push(token.to_string());
            }

            i += 1;
        }

        if !rhs.is_empty() {
            productions.push(Production {lhs: current_lhs.clone().expect("lafknsdlkavlsvn"), rhs});
        }
    }

    CFG {productions, non_terminals, symbols, start_symbol}
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cfg <grammar_file>");
        std::process::exit(1);
    }

    let path = &args[1];
    let input = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading '{}': {}", path, e);
        std::process::exit(1);
    });

    let grammar = parse_grammar(&input);

    println!("Grammar Non-Terminals");
    for nt in &grammar.non_terminals {
        print!("{}, ", nt);
    }
    println!("\n");

    println!("Grammar Symbols");
    for t in &grammar.symbols {
        print!("{}, ", t);
    }
    println!("\n");

    println!("Grammar Rules");
    for (i, p) in grammar.productions.iter().enumerate() {
        println!(
            "({}) {} -> {}",
            i + 1,
            p.lhs,
            p.rhs.join(" ")
        );
    }

    println!("\nGrammar Start Symbol or Goal: {}", grammar.start_symbol);
}