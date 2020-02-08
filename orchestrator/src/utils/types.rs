use std::net::Ipv4Addr;

#[derive(Hash, Eq, PartialEq, Debug)]
#[derive(Clone)]
pub struct CidrIP {
    pub ip: Ipv4Addr,
    pub netmask: u8,
}

impl CidrIP {
    pub fn to_string(&self) -> String {
        format!("{}/{}", self.ip.to_string(), self.netmask)
    }
}

impl From<&str> for CidrIP {
    fn from(cidr: &str) -> Self {
        let elems: Vec<&str> = cidr.split('/').collect();
        assert!(elems.len() == 2);
        Self {
            ip: elems[0].parse().unwrap(),
            netmask: elems[1].parse().unwrap(),
        }
    }
}