use std::fs::File;
use std::collections::HashMap;
use std::io::{BufReader};
use std::io::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use regex::RegexSet;
use std::fmt;

const SETUP_MARKER: &str = r"¥ SETUP";
const RUN_MARKER: &str = r"¥ RUN";
const MACHINE_MARKER: &str = r"¥¥[ \t]*(.+)";

const VM_LIST_RE: &str = r"¥¥\s*((?:\S+[ \t]*)+)";
const VM_LIST_SPLIT_RE: &str = r"[ \t]+";

#[derive(Debug, Clone)]
pub enum StepType {
    Setup,
    Run,
}
// Display trait for easier printing of the different steps
impl fmt::Display for StepType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StepType::Setup => write!(f, "setup"),
            StepType::Run => write!(f, "run"),
        }
    }
}

/// Given a text file (as String) describing a sequence of machine scripts (setups and runs),
/// Returns a data structure with each step encapsulated in a Vec and each script
/// stored in a HashMap recording which machine sould run each of the scripts
pub fn parse_run_list(text_fn: &str) -> Vec<(StepType, HashMap<String, String>)> {

    let mut ret = Vec::new();
    fn is_eof(res: Result<usize, std::io::Error>) -> bool {
        let eof = res.is_err() || res.unwrap() == 0;
        eof
    }

    lazy_static! {
        static ref PARSE_RE_SET: RegexSet = RegexSet::new(&[
            SETUP_MARKER,
            RUN_MARKER,
            MACHINE_MARKER,
        ]).unwrap();

        static ref MACHINE_NAME_RE: Regex = Regex::new(MACHINE_MARKER).unwrap();
        static ref VM_RE: Regex = Regex::new(VM_LIST_RE).unwrap();
        static ref VM_SPLIT_RE: Regex = Regex::new(VM_LIST_SPLIT_RE).unwrap();
    }

    let f = File::open(text_fn).unwrap();
    let mut f = BufReader::new(f);
    let mut line_buffer = String::with_capacity(64); //creating string with some capacity to reduce the number of re-allocations

    let mut state: Option<StepType> = None;

    // run through text line by line, checking if there is a match for one of the re
    // whenever there is a new re match, absorb all lines until new re match to that specific context
    // otherwise just record the line and go to the next one
    let mut current_machine_list: Vec<String> = Vec::new();
    let mut current_script = String::from("");
    let mut current_step = HashMap::new();
    while {

        let line_res = f.read_line(&mut line_buffer);

        // check if current line is a special one
        let possible_matches: Vec<_> = PARSE_RE_SET.matches(&line_buffer).into_iter().collect();
        if !possible_matches.is_empty() {
            debug_assert!(possible_matches.len() <= 1);
            match possible_matches[0] {
                0 => { // Setup Marker
                    if !current_script.is_empty() {
                        (&current_machine_list).into_iter().for_each(|n| {
                            current_step.insert(n.to_string(), current_script.clone());
                        });

                        current_script.clear();
                    }
                    if state.is_some() {
                        ret.push( (state.unwrap(), current_step.clone()) );
                    }
                    
                    current_step.clear();
                    state = Some(StepType::Setup);
                },
                1 => { // Run marker
                    if !current_script.is_empty() {
                        (&current_machine_list).into_iter().for_each(|n| {
                            current_step.insert(n.to_string(), current_script.clone());
                        });
                        current_script.clear();
                    }
                    if state.is_some() {
                        ret.push( (state.unwrap(), current_step.clone()) );
                    }
                    current_step.clear();
                    state = Some(StepType::Run);
                },
                2 => { // Machine marker
                    if !current_script.is_empty() {
                        (&current_machine_list).into_iter().for_each(|n| {
                            current_step.insert(n.to_string(), current_script.clone());
                        });
                        current_script.clear();
                    }
                    let current_machine_string = VM_RE.captures(&line_buffer).unwrap().get(1).unwrap().as_str();
                    current_machine_list = VM_SPLIT_RE.split(current_machine_string).map(|v| v.to_string()).collect();

                },
                _ => panic!() //should not happen
            }
            line_buffer.clear() //clear the string now so special lines are not added to the scripts
        }

        // add current line to script buffer
        match &state {
            None => {},
            Some(StepType::Run) => {
                current_script.push_str(&line_buffer);
            },
            Some(StepType::Setup) => {
                current_script.push_str(&line_buffer);
            },
        };
        
        line_buffer.clear();
        let eof = is_eof(line_res);
        if eof { //add last step
            (&current_machine_list).into_iter().for_each(|n| {
                current_step.insert(n.to_string(), current_script.clone());
            });
            ret.push( (state.as_ref().unwrap().clone(), current_step.clone()) );
        }
        !eof
    }
    {}
    ret
}