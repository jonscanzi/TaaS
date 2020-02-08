//TODO: create macros to not simply duplicate lasir VM

use crate::asir;
use crate::asir::Os as AsirOs;

#[derive(Debug, Clone)]
pub struct Os {
    pub candidates: asir::OsCandidates,
}

impl AsirOs for Os {

    fn get_common(&self) -> String {
        self.candidates.common_os[0].clone()
    }
    
    fn get_name(&self) -> String {
        self.get_all()[0].clone()
    }
    
    fn get_all(&self) -> Vec<String> {
        let mut ret: Vec<String> = Vec::new();
        push_all_clone!(ret, &self.candidates.custom_os,
                             &self.candidates.common_os,
                             &self.candidates.approx_os);
        ret
    }
}

#[derive(Debug, Clone)]
pub enum DiskType {
    HDD,
    SSD,
    NVM,
    OTHER1,
    OTHER2,
    OTHER3,
}

#[derive(Debug, Clone)]
pub struct Disk {
    pub is_main: bool,
    pub capacity_gb: usize,
    pub tpe: DiskType,
    pub grade: u8, //arbitrary number between 0 and 255 to be used by cloud genrators if they provide storage speed tiers
}

impl Disk {
    pub fn default() -> Self {
        Self {
            is_main: true,
            capacity_gb: 1000,
            tpe: DiskType::SSD,
            grade: 255,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HwConfig {
    pub cpu_freq_mhz: usize,
    pub cpu_cores: usize,
    pub ram_gb: usize,
    pub storage: Vec<Disk>, 
}

impl HwConfig {
    pub fn default() -> Self{
        Self {
            cpu_freq_mhz: 5000,
            cpu_cores: 4,
            ram_gb: 16,
            storage: vec![Disk::default()],
        }
    }
}

impl asir::RealHwConfig for HwConfig {
    type disk_type = Disk;

    fn cpu_freq_mhz(&self) -> usize {self.cpu_freq_mhz}
    fn cpu_cores(&self) -> usize {self.cpu_cores}
    fn ram_gb(&self) -> usize {self.ram_gb}
    fn disks(&self) -> Vec<Disk> {self.storage.clone()}
}

#[derive(Debug, Clone)]
pub struct Vm {
    pub name: String,
    pub os: Os,
    pub hwconfig: Option<HwConfig>,
    pub override_config: Option<String>,
    pub config_template: String,
    pub has_remote_access: bool,
    pub role: String,
    pub auth: Auth,
}

#[derive(Debug, Clone)]
pub struct Auth {
    pub user: String,
    pub password: String,
}

impl asir::Vm<Os> for Vm {
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_os(&self) -> &Os {
        &self.os
    }
    fn os_name(&self) -> String {
        self.os.get_name()
    }
    fn get_auth(&self) -> String {
        unimplemented!();
    }
}