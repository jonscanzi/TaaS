
use crate::lasir::connections::VmConnectionLogical;
use crate::lasir;

pub fn lasir_network_to_pasir_network(connections: &dyn VmConnectionLogical) -> Vec<super::connections::Subnet>{
    let subnet_candidates = super::connections::create_network_from_logical(connections);
    let real_subnets = super::connections::assign_subnets_and_ip_v2(&subnet_candidates);
    real_subnets
}

pub fn lasir_vms_to_pasir_vms<'a, I>(vms: I) -> Vec<super::machines::Vm>
    where I: IntoIterator<Item = &'a lasir::machines::Vm>
{

    //TODO: consider implementing From<lasir::machines> on pasir::machines for easier conversion
    fn translate_vm(vm: &lasir::machines::Vm) -> super::machines::Vm {
        let in_hwc = &vm.hwconfig;
        let out_hwc = in_hwc.as_ref().map(|hwc| gather_hwconfig(&hwc));
        let auth = super::machines::Auth {
            user: vm.auth.user.clone(),
            password: vm.auth.password.clone(),
        };
        super::machines::Vm {
            name: vm.name.clone(),
            os: super::machines::Os {candidates: vm.os.candidates.clone()},
            override_config: vm.override_config.as_ref().map(|o| o.to_owned()),
            hwconfig: out_hwc,
            config_template: vm.config_template.clone(),
            has_remote_access: vm.has_remote_access,
            role: vm.role.clone(),
            auth: auth,
        }
    }

    //TODO: replace with map below
    let mut ret: Vec<super::machines::Vm> = Vec::new();
    for vm in vms {
        ret.push(translate_vm(&vm));
    }
    // vms.into_iter().map(|vm| translate_vm(vm));
    ret
}

fn gather_hwconfig(lasir_hwconfig: &lasir::machines::HwConfig) -> super::machines::HwConfig {

    let l = lasir_hwconfig;
    super::machines::HwConfig {
        cpu_freq_mhz: l.cpu_freq_mhz,
        cpu_cores: l.cpu_cores,
        ram_gb: l.ram_gb,
        storage: l.storage.iter().map(|s| translate_storage(s)).collect(),
    }
}

fn translate_storage(lasir_storage: &lasir::machines::Disk) -> super::machines::Disk {
    let dsk = lasir_storage;
    super::machines::Disk {
                            is_main: dsk.is_main,
                            capacity_gb: dsk.capacity_gb,
                            tpe: {
                                match dsk.tpe {
                                    lasir::machines::DiskType::HDD => super::machines::DiskType::HDD,
                                    lasir::machines::DiskType::SSD => super::machines::DiskType::SSD,
                                    lasir::machines::DiskType::NVM => super::machines::DiskType::NVM,
                                    lasir::machines::DiskType::OTHER1 => super::machines::DiskType::OTHER1,
                                    lasir::machines::DiskType::OTHER2 => super::machines::DiskType::OTHER2,
                                    lasir::machines::DiskType::OTHER3 => super::machines::DiskType::OTHER3,
                                }
                            },
                            grade: dsk.grade,
                        }
}