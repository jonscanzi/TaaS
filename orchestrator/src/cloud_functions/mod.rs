//TODO: create interface to query things in a cloud-agnostic fashion, such as a VM public IP address.
pub mod azure;
// use std::net::Ipv4Addr;

pub trait VmHardwareProperties {
    fn name(&self) -> String;
    fn core_count(&self) -> usize;
    fn ram_gb(&self) -> usize;
    fn threads_per_core(&self) -> usize;
    fn max_disk_count(&self) -> usize;
    fn max_disk_capacity_gb(&self) -> usize;
    fn max_network_throughput(&self) -> usize;
}

// ==== EXPERIMENTAL ====
// pub trait SystemInfo {
//     fn get_machine_public_ip(&self, machine_name: &str) -> Option<Ipv4Addr>;
//     // fn get_machine_dns_name(machine_name: &str) -> Option<String>;
//     // fn get_machine_username(machine_name: &str) -> Option<String>;
//     // fn get_machine_password(machine_name: &str) -> Option<String>;
//     fn test(&self) -> String;
// }