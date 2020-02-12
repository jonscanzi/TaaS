// global_config.rs
//
// Author: Jonathan Scanzi
// Date: 15 Jul 2019
//
// All the global variables that would not make sense to pass along the data hierarchy are here.
// Obviously, there should be as few of them as possible.

// - Cloud Provider
// - OS name mapping

use crate::yaml_rust::{YamlLoader, Yaml};
use crate::paths;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;

lazy_static! {
    pub static ref COMMON_OS_MAP: HashMap<String, String> = create_common_os_mapping();
    pub static ref CLOUD_PROVIDER: String = load_cloud_provider();
    pub static ref DEFAULT_VALUES: DefaultValues = load_default_values();
    /// Refer to load_providers_config() definition for details
    pub static ref PROVIDERS_CONFIG: HashMap<String, String> = load_providers_config();
    pub static ref NETWORK: HashMap<String, String> = load_network_config();
    pub static ref SSH: SshConfig = load_ssh_config();
    pub static ref SHELL: ShellConfig = load_shell_config();
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShellConfig {

    pub shell: String,
    #[serde(default = "use_default_download_tool")]
    pub download_tool: String,
}

fn use_default_download_tool() -> String {
    "curl".to_string()
}

fn load_shell_config() -> ShellConfig {
    let sc: String = fs::read_to_string(paths::SHELL_CONFIG).expect(&format!("Error: could not find shell config file at {}", paths::SHELL_CONFIG));
    let sc: ShellConfig = serde_yaml::from_str(&sc).unwrap();
    if !(sc.download_tool == "curl" || sc.download_tool == "wget") { // Only supports using either wget or curl
        panic!("Error: in config/shell.yml make sure you select either curl or wget in the field 'download_tool'");
    }
    sc
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SshConfig {

    pub custom_args: String,
}

fn load_ssh_config() -> SshConfig {
    let sc: String = fs::read_to_string(paths::SSH_CONFIG).expect(&format!("Error: could not find ssh config file at {}", paths::SSH_CONFIG));
    let mut sc: SshConfig = serde_yaml::from_str(&sc).unwrap();
    // fix for when the field is empty
    if sc.custom_args == "~" {
        sc.custom_args = "".to_owned();
    }
    sc
}

fn load_network_config() -> HashMap<String, String> {
    let nc: String = fs::read_to_string(paths::NETWORK_CONFIG).expect(&format!("Error: could not find network config file at {}", paths::NETWORK_CONFIG));
    let nc: HashMap<String, String> = serde_yaml::from_str(&nc).unwrap();
    nc
}


/// Uses a special config file, defined as paths::PROVIDER to retrieve the cloud provider chosen by the user
/// This step allows to simplify other config data structures
fn load_cloud_provider() -> String {
    let parse: &Yaml = yaml_tree_from_file!(paths::PROVIDER);

    let cp = parse["provider"].as_str().unwrap_or_else(|| panic!("Unable to find cloud provider in provider config {}", paths::PROVIDER));
    cp.to_string()
}

/// Loads the file defined in the paths module in order to create a map of configuration
/// for various cloud providers. At run time, it only keeps the config that corresponds
/// to the current cloud provider (defined as CLOUD_PROVIDER), thus avoiding a syntax like
/// PROVIDERS_CONFIG["azure"]["location"], being instead PROVIDERS_CONFIG["location"]
fn load_providers_config() -> HashMap<String, String> {
    let pc: String = fs::read_to_string(paths::PROVIDERS_CONFIG).expect(&format!("Error: could not find providers config file at {}", paths::PROVIDERS_CONFIG));
    let pc: HashMap<String, HashMap<String, String>> = serde_yaml::from_str(&pc).unwrap();
    let pc: HashMap<String, String> = pc[&CLOUD_PROVIDER.to_string()].clone();
    pc
}

/// Similar to other initialising functions, creates a data structure to easily and quickly access default values provided by the user
fn load_default_values() -> DefaultValues {
    let dv: String = fs::read_to_string(paths::DEFAULT_VALUES).unwrap_or_else(|_| panic!("Could not open default values file at {}", paths::DEFAULT_VALUES));
    let dv: DefaultValues = serde_yaml::from_str(&dv).unwrap_or_else(|_| panic!("Could not parse default values file {} into YAML", paths::DEFAULT_VALUES));
    dv
}

#[derive(Serialize, Deserialize, Debug)]
/// All the default values for a Cloud VM that the user can provide
pub struct DefaultValues {
    pub cpu_freq_mhz: usize,
    pub cpu_cores: usize,
    pub ram_gb: usize,

    pub capacity_gb: usize,
    pub r#type: String,
    pub grade: u8,

    pub os_common: String,

    pub location: String,
    pub remote_access: bool,
    pub config_template: String,
    pub custom_script: String,
}

#[inline]
fn yaml_to_str(filename: &str) -> String {
    fs::read_to_string(filename).unwrap_or_else(|_| panic!("Error: could not load config file {}", filename))
}

/// Creates a runtime mapping of common OS nicknames into the name of the image for 
/// specific cloud providers. The mapping is chosen by the user in file located in
/// paths::COMMON_OS
fn create_common_os_mapping() -> HashMap<String, String> {
    
    let mut ret: HashMap<String, String> = HashMap::new();
    let text = yaml_to_str(paths::COMMON_OS);

    let parse: &Yaml = &YamlLoader::load_from_str(&text).unwrap()[0];
    let os_list = &parse.as_hash().unwrap();
    let oses = os_list.keys();

    for os in oses {
        let cloud_specific = &os_list[os];
        let cp: &str = &*CLOUD_PROVIDER; //Is this a C++?
        let new_entry = &cloud_specific[cp];

        if !new_entry.is_badvalue() {
            ret.insert(os.as_str().unwrap().to_string(), new_entry.as_str().unwrap().to_string());
        }
    }
    ret
}