use crate::utils::types::CidrIP;
use std::net::Ipv4Addr;
use std::str::FromStr;

#[derive(Debug)]
pub struct Globals {
    pub location: String,
    pub resource_group: String,
    pub has_stub_network: bool,
}

#[derive(Debug)]
pub struct Vnet {
    pub name: String,
    pub address_prefixes: CidrIP, //only allow one prefix; it's the subnets jobs to do the others
}

#[derive(Debug, Clone)]    
pub struct Subnet {
    pub name: String,
    pub address_prefixes: CidrIP, //just like Vnet we only allow one prefix
    //vnet_name is implicitely set to Vnet.name
}

#[derive(Debug, Clone)]
pub struct Nic {
    pub name: String,
    pub vnet: String,
    pub subnet: String, //the reference required by azure is the name of the subnet
    pub private_ip_address: Ipv4Addr,
    pub has_public_ip_address: bool, //cannot choose what the address will be
    //vnet_name is also implicitely set to Vnet.name
}

impl Nic {
    //assume that if a VM requires a default NIC, it is not connected to the rest of the system
    //and should be attributed a public IP address
    pub fn new_with_public_ip(name: String, vnet: String, subnet: String, private_ip: String) -> Self {
        Self {
            name: name,
            vnet: vnet,
            subnet: subnet,
            private_ip_address: Ipv4Addr::from_str(&private_ip).unwrap(),
            has_public_ip_address: true,
        }
    }
}

#[derive(Debug)]
pub struct Vm {
    pub name: String,
    pub nics: Vec<Nic>,
    pub image: String, //azure-provided OSes
    pub size: String, //Azure virtual hardware config
    pub admin_username: String,
    pub admin_password: String,
    pub authentication_type: String,
    pub custom_script: String,
}

#[derive(Debug)]
pub struct WholeSystem {
    pub global_config: Globals,
    pub vnet: Vnet,
    pub subnets: Vec<Subnet>,
    pub vms: Vec<Vm>,
}