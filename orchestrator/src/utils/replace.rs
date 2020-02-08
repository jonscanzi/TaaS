use std::fs;
use std::collections::HashMap;
use std::collections::HashSet;

const SPECIAL_CHAR: char = '¥';
const USUAL_IDENT_SIZE: usize = 20; //the expected length of a special identifier (inside ¥{  })

pub enum ReplaceFailPolicy {
    #[allow(unused)]
    Ignore,
    Warn,
    #[allow(unused)]
    Panic,
}

/// Takes an existing file name, opens it, replace special identifiers (using the 'replace' function below),
/// then write a new file with the replaced content
pub fn copy_and_replace_v2(orig_fn: &str, target_fn: &str, replacement: &HashMap<String, String>, fail_policy: ReplaceFailPolicy) {
    let orig: String = fs::read_to_string(orig_fn).unwrap_or_else(|_| panic!("Could not find file {}", orig_fn));
    let replaced = replace(&orig, replacement, fail_policy, Some(orig_fn.to_string()));
    fs::write(target_fn, replaced).expect(&format!("Could not create replaced file {}. Check permissions and whether the parent directory exists.", target_fn));
}

pub fn replace_and_write(orig_txt: &str, target_fn: &str, replacement: &HashMap<String, String>, fail_policy: ReplaceFailPolicy, associated_filename: Option<String>) {
    let replaced = replace(&orig_txt, replacement, fail_policy, associated_filename);
    fs::write(target_fn, replaced).expect(&format!("Could not create replaced file {}. Check permissions and whether the parent directory exists.", target_fn));
}

/// Runs through a given text and looks for ¥ symbols. When one is found, checks if the next
/// character is '{'. If that is the case, collects all characters until a '}' is found. Once all
/// characters are collected, tries to replace the collected characters with the replacement map,
/// removing the whole '¥{...}'. If any of these checks fail, the function will silently continue
/// looking for matches. If multiple '¥{' are found, the function will search for a new match, and
/// once that is done, it will go back the the previous '¥' and try matching again.
pub fn replace(text: &str, replacement: &HashMap<String, String>, fail_policy: ReplaceFailPolicy, associated_filename: Option<String>) -> String {

    fn focus_replace(text: &str, _start: usize, map: &HashMap<String, String>, fail_policy: ReplaceFailPolicy, associated_filename: &Option<String>) -> String {

        let mut has_warned = false;
        let mut warned_tokens: HashSet<String> = HashSet::new();
        let mut state: usize = 0;
        let mut ident = String::with_capacity(USUAL_IDENT_SIZE);
        let mut ret = String::with_capacity(USUAL_IDENT_SIZE);

        for (idx, chr) in text.chars().enumerate() {

            //check if we have reached the special symbol, and if there is still room in the string
            //for '{}'
            match state {
                0 => {
                     if chr == SPECIAL_CHAR && text.len() > idx+2 {
                         state = 1;//ParserState::cheking_bracket;
                     }
                    else {
                        ret.push(chr);
                    }
                },

                1 => {
                    if chr == '{' {
                        state = 2;//ParserState::collecting;
                    }
                    else {
                        state = 0;//ParserState::waiting;
                        ret.push(SPECIAL_CHAR);
                        ret.push(chr);
                        ident.clear();

                    }
                },

                2 => {
                    if chr == '}' {

                        let map_res = map.get(&ident);
                        match map_res {

                            Some(s) => ret.push_str(s),
                            None => {
                                match fail_policy  {
                                    ReplaceFailPolicy::Panic => {
                                        match associated_filename {
                                                Some(filename) => panic!("Error: in text file {}, found replacement token \"{}\", but could not find a suitable replacement.", filename, ident),
                                                None => panic!("\nError: found replacement token \"{}\", but could not find a suitable replacement. Text file:\n{}\n", ident, text),
                                            };
                                    }
                                    ReplaceFailPolicy::Warn => { //Only print the whole text once, o/w just mention which token was not found
                                        if !has_warned {
                                            match associated_filename {
                                                Some(filename) => println!("Warning: in text file {}, found replacement token \"{}\", but could not find a suitable replacement.", filename, ident),
                                                None => println!("\nWarning: found replacement token \"{}\", but could not find a suitable replacement. Text file:\n{}\n", ident, text),
                                            };
                                            warned_tokens.insert(ident.clone());
                                            has_warned = true;
                                        }
                                        else {
                                            if !warned_tokens.contains(&ident) {
                                                println!("For the same text file, the token {} was also found to have no match.", ident);
                                                warned_tokens.insert(ident.clone());
                                            }
                                        }
                                    },
                                    ReplaceFailPolicy::Ignore => (),
                                };
                                ret.push_str(&format!("{}{}{}{}", SPECIAL_CHAR, "{", ident, "}"));
                            }
                        };

                        // ret.push_str(map.get(&ident).unwrap_or(&format!("{}{}{}{}", SPECIAL_CHAR, "{", ident, "}")));
                        ident.clear();
                        state = 0;//ParserState::waiting;
                    }
                    else {
                        ident.push(chr);
                    }
                },
                _=> panic!(),
            }
        }
        ret
    }
    let ret = focus_replace(text, 0, &replacement, fail_policy, &associated_filename);
    ret
}