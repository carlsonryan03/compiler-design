

use std::collections::HashMap;
use std::collections::HashSet;
use std::process;

#[derive(Debug, Clone)] // debug lets you print NfaStateRow with :? for debugging purposes. clone lets you deep copy
pub struct NfaStateRow {
    pub accepting: bool,
    pub state_id: u32,
    pub transitions: Vec<HashSet<u32>>,
}

#[derive(Debug, Clone)]
pub struct NFA {
    pub states: Vec<NfaStateRow>,
    pub len_alphabet: usize,
    pub matching: bool,
    pub will_not_match: bool,
    pub current_states: HashSet<u32>,
    pub longest_accepting_match: String,
    pub current_read_sequence: String,
    pub alphabet: HashMap<u8, usize>,
    pub start_state_id: u32,
}

// impl block for Nfa
impl NFA {
    // Create Nfa from rows
    pub fn new(states: Vec<NfaStateRow>, alphabet: HashMap<u8, usize>, start: u32) -> Result<Self, String> {
        if states.is_empty() {
            return Err("NFA must contain at least one state".to_string());
        }

        let len_alphabet = states[0].transitions.len();
        let matching = true;
        let will_not_match = false;
        let current_states = HashSet::new();
        let longest_accepting_match = String::new();
        let current_read_sequence = String::new();
        let start_state_id = start;

        Ok(NFA {
            states,
            len_alphabet,
            matching,
            will_not_match,
            current_states,
            longest_accepting_match,
            current_read_sequence,
            alphabet,
            start_state_id
        })
    }

    // Get state from ID: Reference
    pub fn get_state(&self, id: u32) -> Option<&NfaStateRow> {
        self.states.get(id as usize)
    }

    //
    pub fn size(&self) -> usize {
        self.states.len()
    }

    // TODO
    // see if a state is an accepting state
    pub fn state_is_accepting(&self, id: u32) -> bool {
        match self.get_state(id) {
            Some(state) => state.accepting,
            None => false,
        }
    }

   pub fn is_accepting(&self) -> bool {
        self.current_states.iter().any(|&id| self.state_is_accepting(id))
    }

    pub fn get_current_states_id(&self) -> HashSet<u32> {
        self.current_states.clone()
    }

    pub fn set_current_states(&mut self, state_ids: HashSet<u32>) {
        self.current_states = state_ids
    }

    pub fn get_longest_accepting_match(&self) -> String {
        self.longest_accepting_match.clone()
    }

    // TODO: this needs to be changed to work for sets
    // simulate a single step of the dfa simulation and return the current read string
    // need to manipulate current state, possibly longest accepted string, matching, and self_is_accepting
    pub fn simulate_one_step(&mut self, symbol: u8) {
        // println!("-SIMULATE ONE STEP");
        // println!("\tI am trying to transition on {}", symbol);
        // println!("\tState {} transitions len: {}, symbol index: {}", self.get_current_states_id(), self.states[self.get_current_states_id()].transitions.len(), symbol);

        // get the index of the symbol to look in the state transitions
        if !self.alphabet.contains_key(&symbol){
            eprintln!("ERROR! DFA-simulation called on nonexistant symbol {}", symbol);
            process::exit(1);
        }
        let symbol_index = match self.alphabet.get(&symbol) {
            Some(i) => *i,
            None => {
                self.will_not_match = true;
                return;
            }
        };

        // TODO
        // attempt to transition
        let mut next_states = HashSet::new();

        for state_id in &self.current_states {
            let row = &self.states[*state_id as usize];
            for next in &row.transitions[symbol_index] {
                next_states.insert(*next);
            }
        }
        
        // TODO
        // there is a transition
        if next_states.len() > 0 {
            self.current_states = next_states;

            // append the thing we just read to our read sequence
            self.current_read_sequence.push(symbol as char);

            // check acceptance
            if self.is_accepting() {
                self.longest_accepting_match = self.current_read_sequence.clone();
            }
        }
        else { // if no transition, we failed to read anything new and we cannot continue
            self.will_not_match = true;
        }
        // println!("\tMY current read sequence is {}", self.current_read_sequence);
        // println!("\t My longest accepting read sequence is {}", self.longest_accepting_match);
        // println!("\t My current sequence is longer than my longest accepting: {}", self.current_read_sequence.len() > self.longest_accepting_match.len());
        // println!("\t I am in an accepting state: {}", self.self_is_accepting);
    }

    // simulate an entire string on the dfa and return whether it was accepted or not
    pub fn simulate(&mut self, seq: &str) -> bool {
        // println!("-SIMULATE CALLED ON {}-", seq);
        // we don't exclude whitespace
        let sequence: Vec<u8> = seq.bytes().collect();

        // TODO
        // make sure we are starting in the start state
        self.current_states.clear();
        self.current_states.insert(self.start_state_id);

        self.matching = true;
        self.will_not_match = false;

        // this needs to be reset to see how far the simulation got before it read something it couldnt accept
        self.longest_accepting_match = String::new();
        self.current_read_sequence = String::new();

        // run on all symbols until we finish or cannot continue
        for symbol in sequence {
            if self.will_not_match {
                // println!("\tI couldn't keep reading :/");
                break; // we cannot transition from current state
            }
            // println!("\tI am capable of reading a symbol.");
            self.simulate_one_step(symbol);
        }
        // println!("\tSIMULATE: No more symbols. I ended in a match: {} and my longest match was {}", self.self_is_accepting, self.longest_accepting_match);
        
        // return if any prefix was accepted. Is this gonna work?
        return self.is_accepting();
    }

    // print function
    pub fn print(&self) -> String {
        let mut output = String::new();

        for state in &self.states {
            if self.state_is_accepting(state.state_id) {
                output.push_str("+ ");
            } else {
                output.push_str("- ");
            }

            output.push_str(&state.state_id.to_string());
            output.push_str(" ");

            for transition in &state.transitions {
                if transition.is_empty() {
                    output.push_str("E ");
                } else {
                    let mut ids: Vec<u32> = transition.iter().cloned().collect();
                    ids.sort();
                    let s: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
                    output.push_str(&s.join(","));
                    output.push_str(" ");
                }
            }

            output.push_str("\n");
        }

        output
    }
}
