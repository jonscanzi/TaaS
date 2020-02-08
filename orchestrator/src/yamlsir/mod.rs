//yamlsir/mod.rs
//experimentations on how to use the serde library to ease-up yaml parsing
use serde::{Serialize, Deserialize};
use std::fs;
use crate::utils::global_config::DEFAULT_VALUES;
use std::io::Read;
use std::collections::HashMap;
use std::collections::HashSet;
#[macro_use]
pub mod default;

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {

    pub version: usize,
    pub machines: Vec<Machine>,
    pub options: Option<HashSet<String>>,
    #[serde(default = "no_connections")]
    pub connections: Vec<Connection>,
}

fn no_connections() -> Vec<Connection> {
    Vec::new()
}

make_default!(config_template, String);
#[derive(Serialize, Deserialize, Debug)]
pub struct Machine {

    pub name: String,
    pub os_common: String,
    pub hwconfig: Option<HwConfig>,
    pub override_config: Option<HashMap<String, String>>,
    pub auth: Auth,
    pub remote_access: bool,
    #[serde(default = "config_template")]
    pub config_template: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Auth {

    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HwConfig {

    pub cpu_freq_mhz: Option<usize>,
    pub cpu_cores: Option<usize>,
    pub ram_gb: Option<usize>,
    pub storage: Vec<Disk>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Disk {

    pub name: String,
    pub is_os_disk: bool,
    pub capacity_gb: usize,
    pub r#type: String,
    pub grade: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Connection {

    pub a: String,
    pub b: String,
    pub speed_mbps: usize,
    pub packet_drop_percent: f64,
    pub latency_us: usize,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
fn DEFAULT() -> String {

    "DEFAULT".to_string()
}

/// From a given YAML file, create the internal representation of the system tree
pub fn parse_yaml(filename: &str) -> Root {

    let mut yml: String = String::new();
    let mut fl = fs::File::open(filename).unwrap_or_else(|_| panic!("Could not open file {}", filename));
    fl.read_to_string(&mut yml).unwrap_or_else(|_| panic!("Could not read text from file {}", filename));
    let parse: Result<Root, _> = serde_yaml::from_str(&yml); 
    match parse {
        Ok(_) => (),
        Err(x) => panic!("Could not parse file {} into YAML. Error:\n\n{}\n\n", filename, x),
    }
    // let parse: Root = serde_yaml::from_str(&yml).unwrap_or_else(|_| panic!("Could not parse file {} into YAML", filename));
    parse.unwrap()
}