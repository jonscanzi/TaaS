use crate::utils::replace;
use std::collections::HashMap;
use std::sync::Mutex;
use crate::shell_tools;
use crate::utils::global_config::SHELL;

lazy_static! {
    static ref GLOBAL_REPL_MAP: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

/// Add given string (in the order `from`, then `to`) to the global replacement map,
/// meant to be used in the post_deployment script (e.g. to replace a symbolic name 
/// for a DNS by the actual DNS name of the deployed machine)
#[inline]
pub fn add_global_replacement(from: &str, to: &str) {
    // Do the string copy first in order to make the lock as short as possible
    let from = from.to_string();
    let to = to.to_string();
    GLOBAL_REPL_MAP.lock().unwrap().insert(from, to);
}

pub fn replace(script: &str) -> String {

    add_global_replacement("SSH_OPTIONS", &crate::utils::global_config::SSH.custom_args);
    add_global_replacement("SSH_CONFIG", &crate::utils::global_config::SSH.custom_args);

    let ret = replace::replace(script, &*GLOBAL_REPL_MAP.lock().unwrap(), replace::ReplaceFailPolicy::Warn, Some("post_deployment.sh".to_string()));

    ret
}

pub fn run(script: &str) -> crate::shell_tools::RunSummary {
    let script = replace(script);
    shell_tools::run_command_interactive(&script, &SHELL.shell)
}