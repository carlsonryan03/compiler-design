#[derive(Debug, Clone)] // debug lets you print StateRow with :? for debugging purposes. clone lets you deep copy
pub struct StateRow {
    pub accepting: bool,
    pub state_id: u32,
    pub transitions: Vec<Option<u32>>,
}

#[derive(Clone)]
pub struct DFA {
    pub states: Vec<StateRow>,
    pub len_alphabet: usize,
}

// impl block for dfa minimize
impl DFA {
    // Create dfa from rows
    pub fn new(states: Vec<StateRow>) -> Result<Self, String> {
        if states.is_empty() {
            return Err("DFA must contain at least one state".to_string());
        }

        let len_alphabet = states[0].transitions.len(); 

        Ok(DFA {
            states,
            len_alphabet,
        })
    }

    // Get state from ID: Reference
    pub fn get_state(&self, id: u32) -> Option<&StateRow> {
        self.states.get(id as usize)
    }

    //
    pub fn num_states(&self) -> usize {
        self.states.len()
    }

    // see if marked by + or -
    pub fn is_accepting(&self, id: u32) -> bool {
        match self.get_state(id) {
            Some(state) => state.accepting,
            None => false,
        }
    }

    pub fn remove_state(&mut self, state_to_remove: usize, state_to_keep: Option<usize>) {
        // remove the row from the transition table
        self.states.remove(state_to_remove);

        // Fix state IDs after removal. we need to shift the states so we don't try to access an index out of bounds after removing an earlier state
        for (i, state) in self.states.iter_mut().enumerate() {
            state.state_id = i as u32;

            for transition in &mut state.transitions {
                // remove references to the state we just removed
                if *transition == Some(state_to_remove as u32) {
                    // if we have a state to keep, point to it; else set to None
                    *transition = state_to_keep.map(|k| k as u32);
                } else if let Some(t) = transition {
                    // decrement state IDs larger than removed index
                    if *t > state_to_remove as u32 {
                        *t -= 1;
                    }
                }
            }

        }
    }

    // private: states should only be merged if they are found to be equivalent in the minimize algorithm
    fn merge_states(&mut self, state1: u32, state2: u32) {
        // println!("Result of merging states: {} and {}", state1, state2); // DEBUG

        // always choose state2 to remove. It's fine to choose state 1 but it doesn't matter which
        let state_to_remove = state2 as usize;
        let state_to_keep = state1 as usize;

        // Give state 1 all of state 2's in-transitions (if the table has state2, change it to state1)
        for state in &mut self.states {
            for transition in &mut state.transitions {
                if *transition == Some(state_to_remove as u32) {
                    *transition = Some(state_to_keep as u32);
                }
            }
        }
        
        self.remove_state(state_to_remove, Some(state_to_keep));
        // println!("{}", self.print()); // DEBUG
    }

    // 
    pub fn minimize(&mut self) {
        // M = sets to merge
        let mut merge_sets: Vec<Vec<u32>> = Vec::new();

        // L = stack of (state_set, alphabet)
        let mut stack: Vec<(Vec<u32>, Vec<usize>)> = Vec::new();

        // initial partition: split accepting and nonaccepting as they will never be equivalent
        // we will assume that accepting could me a merge set, and we will look for states that are not equivalent, and refine partitions
        let accepting: Vec<u32> = self.states
            .iter()
            .filter(|s| s.accepting)
            .map(|s| s.state_id)
            .collect();
        // ^ iterates over all states, finds the accepting states, and collects the state id's

        let non_accepting: Vec<u32> = self.states
            .iter()
            .filter(|s| !s.accepting)
            .map(|s| s.state_id)
            .collect();

        // represents the columns in the transition table. All the symbols we need to look at
        let alphabet: Vec<usize> = (0..self.len_alphabet).collect();

        stack.push((accepting, alphabet.clone()));
        stack.push((non_accepting, alphabet.clone()));

        // iterate over combinations of states transitioning on certain symbols
        while let Some((states, mut chars)) = stack.pop() {
            // we have looked at all alphabet symbols for the popped states
            if chars.is_empty() {
                continue;
            }

            // consider one symbol of the alphabet
            let c = chars.pop().unwrap();

            // partitions: looks for states that have the same transition on c
            let mut partitions: std::collections::HashMap<Option<u32>, Vec<u32>> =
                std::collections::HashMap::new();

            for s in &states {
                let row = self.get_state(*s).unwrap();
                let dest = row.transitions[c];
                // find all states that have the same transition on c as s does
                partitions.entry(dest).or_default().push(*s);
            }

            // for any partitions with more than one state, we have states to merge
            for (_, group) in partitions {
                if group.len() > 1 {
                    // if there are no more symbols to check for a partition, it is impossible to refine it further and it is a merge_set
                    if chars.is_empty() {
                        merge_sets.push(group);
                    } else {
                        // else we need to revisit it when we look at a new alphabet symbol, because the states may not all be equivalent
                        stack.push((group, chars.clone()));
                    }
                }
            }
        }

        // Merge the partition sets with multiple states (equivalent)
        // we do this in descending order because as we remove states, we want to avoid index out of bounds errors
        for i in 0..merge_sets.len() {
            let mut sorted_set = merge_sets[i].clone();
            sorted_set.sort_unstable_by(|a, b| b.cmp(a)); // descending order within a merge set

            let first = sorted_set.pop().unwrap(); // keep the larger
            for &s in &sorted_set {
                self.merge_states(first, s);

                // After merging s, all remaining IDs > s need to be decremented in all sets. This sucks
                for j in i+1..merge_sets.len() {
                    merge_sets[j] = merge_sets[j].iter().map(|id| if *id > s { id - 1 } else { *id }).collect();
                }
            }
        }

        // // DEBUG
        // if merge_sets.is_empty() {
        //     println!("There was nothing to merge. DFA is already minimized");
        // }
    }


    // pub fn simulate(&self, input: &[usize]) -> bool {
    //     // TODO: will be good for debugging, needed for luthor
    // }

    // print function
    pub fn print(&self) -> String {
        let mut output = String::new();

        for state in &self.states {
            if self.is_accepting(state.state_id) {
                output.push_str("+ ");
            } else {
                output.push_str("- ");
            }

            output.push_str(&state.state_id.to_string());
            output.push_str(" ");

            for transition in &state.transitions {
                match transition {
                    Some(t) => output.push_str(&t.to_string()),
                    None => output.push_str("E"),
                }
                output.push_str(" ");
            }

            output.push_str("\n");
        }

        output
    }
}
