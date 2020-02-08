extern crate regex;
use regex::Regex;

/* @brief
 *
 * compare two strings and gives a matching score
 * the higher the score, the closer the two strings are
 * 
 * for now implemented as a simple left substring search
 * where it tries to find the largest substring of orig
 * that is present in candidate
 *
 */
fn matching_score(orig: &str, candidate: &str) -> usize {
    let mut best_score: usize = 0;
    let orig: String = orig.to_lowercase();
    let candidate: String = candidate.to_lowercase();
    lazy_static! {
        static ref re_ws: Regex = Regex::new(r"\s+").unwrap();
    }
    let orig: String = re_ws.replace_all(&orig, "").to_string();
    let candidate: String = re_ws.replace_all(&candidate, "").to_string();
    
    //left substring compare 
    let max_range = orig.len();
    for range in 0..max_range {
        if candidate.contains(&orig[0..range]) {
            best_score = range+1;
        }
    }
    best_score
}

/// Uses simple heuristics to find which string in candidates is the closest to os (the given OS name)
pub fn find_best_matching_os(os: &str, candidates: &Vec<String>) -> String {
    let mut best_score: usize = 0;
    
    let mut all_best_matches: Vec<&str> = Vec::new();
    for candidate in candidates {
        let new_score = matching_score(&os, &candidate);
        if new_score > 0 && new_score == best_score {
            all_best_matches.push(&candidate)
        }
        else if new_score > best_score {
            best_score = new_score;
            all_best_matches.clear();
            all_best_matches.push(&candidate);
        }
    }
    if all_best_matches.len() == 0 {
        panic!("error: could not find any matchig os for {}", os);
    }
    else if all_best_matches.len() > 1 {
        println!("Warning: found multiple OS candidates for {}, chose {}", os, all_best_matches[0]);
    }
    String::from(all_best_matches[0])
}
