use crate::yamlsir;
use crate::asir;
use std::collections::HashMap;
use crate::lasir::connections::ConnectionProperties;
use crate::lasir::connections::VmConnectionLogical;
use crate::yamlsir::default;
use crate::post_deployment;

/**
 * @brief
 * transforms the near-raw YAMLSIR to LASIR, which contains a couple more
 * information for making PASIR generation easier
 *
 * Note: this translator is the part that is/should be the main way to add new syntax to the system description. PASIR and YAMLSIR itself should be changed as minimally as possible.
 **/
pub fn yamlsir_to_lasir(root: &yamlsir::Root) -> super::machines::LogicalSystem<super::connections::VmConnectionLogicalV2> {

    let mut vm_indices: HashMap<String, usize> = HashMap::new(); //used for vm references in the connection matrix
    let mut lasir_vms: Vec<super::machines::Vm> = Vec::new();

    let chosen_cloud = crate::utils::global_config::CLOUD_PROVIDER.to_string();

    let machines = &root.machines;
    for machine in machines {
        let vm_name = &machine.name;
        let os_name = super::machines::Os {
            candidates: asir::OsCandidates {
                custom_os: vec!(),
                approx_os: vec!(),
                common_os: vec!(machine.os_common.clone())
            }
        };
        let hwc = machine.hwconfig.as_ref().map(|conf| gather_hwconfig(&conf));
        let override_conf = match &machine.override_config {
            Some(map) => match map.get(&chosen_cloud) {
                Some(config) => Some(config.to_string()),
                None => {
                    println!("Warning: machine {} has a config override, but not for the currently chose cloud provider ({})", machine.name, chosen_cloud);
                    None
                },
            },
            None => None,
        };

        let auth = super::machines::Auth {
            user: machine.auth.username.clone(),
            password: machine.auth.password.clone(),
        };

        // Add usernames and passwords of each VM in the global replacement map for post-deployement scripts
        //TODO: do it more cleanly
        post_deployment::add_global_replacement(&format!("machines/{}/user", &vm_name), &format!("{}", &auth.user));
        post_deployment::add_global_replacement(&format!("machines/{}/username", &vm_name), &format!("{}", &auth.user));
        post_deployment::add_global_replacement(&format!("machines/{}/pass", &vm_name), &format!("{}", &auth.password));
        post_deployment::add_global_replacement(&format!("machines/{}/password", &vm_name), &format!("{}", &auth.password));

        vm_indices.insert(vm_name.to_string(), lasir_vms.len());

        let new_vm = super::machines::Vm {
            name: vm_name.to_string(),
            os: os_name,
            hwconfig: hwc,
            override_config: override_conf,
            /*auth_type: auth,*/
            config_template: machine.config_template.to_string(),
            has_remote_access: machine.remote_access,
            role: machine.role.clone(),
            auth: auth,
        };
        lasir_vms.push(new_vm);
    }

    //connections part
    let use_full_network = root.options.as_ref().map(|o| o.contains("full_network")).unwrap_or(false); //TODO: put this string in some kind of constants file
    let mut vm_connections = super::connections::new_connection_vec(lasir_vms.len());

    match use_full_network {
        true => {
            for a in 0..lasir_vms.len() {
                for b in a..lasir_vms.len() {
                    if a != b {
                        vm_connections.add_sym_connection(a, b);
                    }
                }
            }
        },
        false => {
            for connection in &root.connections {
                vm_connections.add_sym_connection_with_speed (
                    vm_indices[&connection.a],
                    vm_indices[&connection.b],
                    ConnectionProperties {
                        speed_mbps: connection.speed_mbps,
                        latency_us: connection.latency_us,
                        drop_chance_percent: connection.packet_drop_percent as f32,
                    },
                );
            }
        },
    }
    
    let ret = super::machines::LogicalSystem {vms: lasir_vms, network: vm_connections};
    ret
}

fn gather_hwconfig(yamlsir_hwconfig: &yamlsir::HwConfig) -> super::machines::HwConfig {

    let mut vm_storage: Vec<super::machines::Disk> = Vec::new();
        for disk in &yamlsir_hwconfig.storage {

            vm_storage.push(
                super::machines::Disk {
                    is_main: disk.is_os_disk,
                    capacity_gb: disk.capacity_gb,
                    tpe: match disk.r#type.as_ref() {
                        "ssd" => super::machines::DiskType::SSD,
                        "hdd" => super::machines::DiskType::HDD,
                        "nvm" => super::machines::DiskType::NVM,
                        _ => super::machines::DiskType::OTHER1,
                    },
                    grade: disk.grade,
                }
            );
        }

        let freq_with_def = yamlsir_hwconfig.cpu_freq_mhz.unwrap_or(default::cpu_freq_mhz());
        let cores_with_def = yamlsir_hwconfig.cpu_cores.unwrap_or(default::cpu_cores());
        let ram_with_def = yamlsir_hwconfig.ram_gb.unwrap_or(default::ram_gb());
        let vm_hwconf = super::machines::HwConfig {
            cpu_freq_mhz: freq_with_def,
            cpu_cores: cores_with_def,
            ram_gb: ram_with_def,
            storage: vm_storage,
        };

        vm_hwconf
}


// ================== OS CONFIG ====================

#[inline]
#[allow(dead_code)]
fn find_custom_os<'a>(custom_name: Option<&'a str>, _: &'a str) -> Option<String> {
    match custom_name {
        Some(n) => Some(n.to_string()),
        None => None,
    }
}

#[inline]
#[allow(dead_code)]
fn find_approximate_os<'a>(_approx_name: Option<&'a str>, _cloud_provider: &'a str) -> Option<String> {
    None
}