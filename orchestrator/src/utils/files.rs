use std::path::Path;
use std::path::PathBuf;
use std::fs;

/// Using rust's built-in read_dir function, add the ability to choose to list only files of a certain type
#[allow(unused)]
pub fn list_all_files_with_ext<P: AsRef<Path>>(path: P, ext: &str) -> Vec<PathBuf> {

    //TODO: handle all unwrap properly (with e.g. match), make less ugly
    let all_relevant = fs::read_dir(&path).unwrap().filter(|e| e.is_ok() && e.as_ref().unwrap().path().extension().map(|s| s.to_str().unwrap_or("err")).unwrap_or("err") == ext);
    all_relevant.map(|e| e.unwrap().path()).collect()
}