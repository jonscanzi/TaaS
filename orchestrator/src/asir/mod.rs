//asir.rs
//Author: Jonathan Scanzi
//Date 18 Jul 2019
//
//This file contains all the definitions for the various inermediate representations (IR) across
//the project.

//using vecs for future extension to the algorithm, but fow now they have 1 value
#[derive(Debug, Clone)]
pub struct OsCandidates {
    pub custom_os: Vec<String>,
    pub common_os: Vec<String>,
    pub approx_os: Vec<String>,
}

impl OsCandidates {
    pub fn common_only(os: &str) -> Self {
        Self {
            custom_os: Vec::with_capacity(0),
            common_os: vec![os.to_string()],
            approx_os: Vec::with_capacity(0),
        }
    }
}

pub trait Os {
    fn get_common(&self) -> String;
    fn get_name(&self) -> String;
    fn get_all(&self) -> Vec<String>;
}

pub trait RealHwConfig {
    type disk_type;

    fn cpu_freq_mhz(&self) -> usize;
    fn cpu_cores(&self) -> usize;
    fn ram_gb(&self) -> usize;
    fn disks(&self) -> Vec<Self::disk_type>;
}

pub trait CloudHwConfig {
    fn name(&self) -> String; //the actual name that the cloud provider will need
    fn summary(&self) -> String;
}

pub trait Auth {
    fn to_string(&self) -> String;
}

/**
 * @brief
 *
 * Interface to query information on the internal representation of a VM. This trait is here
 * because a VM in the early, platform-agnostic phase is not the same as the representation of a VM
 * for a specific cloud provider, but it still needs to be queried in a uniform way
 **/
pub trait Vm<O: Os> {
    fn get_name(&self) -> String;
    fn get_os(&self) -> &O;
    fn os_name(&self) -> String;
    fn get_auth(&self) -> String;
}