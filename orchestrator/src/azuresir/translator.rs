use crate::pasir;
use crate::utils::types::CidrIP;
use std::net::Ipv4Addr;
use crate::utils::global_config;
use crate::asir::Os;
use crate::cloud_functions::azure::find_best_matching_vm_name;
use crate::cloud_functions::VmHardwareProperties;
use std::sync::atomic::{AtomicU8, Ordering};

//TODO: make it more robust for users that would
// create thousands of machines
static IP_ADDR_COUNTER: AtomicU8 = AtomicU8::new(5);

const RG: &str = "resource-group";

pub fn pasir_to_azuresir(vms: &Vec<pasir::machines::Vm>,
                         network: &Vec<pasir::connections::Subnet>,
                         vnet_name: &str) -> super::system::WholeSystem {
    //order: SAME AS SHELL CREATION
    //Vnet
    //Subnets(s)
    //NIC(s)
    //VM(s)

    //Networking
    //for every subnet, create:
    //  the azure subnet
    //  all the attached NICs

    let vnet = super::system::Vnet{name: vnet_name.to_string(), address_prefixes:  CidrIP {ip: Ipv4Addr::new(10,0,0,0), netmask: 8}  };

    let mut subnet_idx = 0;
    let mut all_nics: Vec<Vec<super::system::Nic>> = Vec::new(); //nics for every vm, by vm index
    all_nics.resize(vms.len(), Vec::<super::system::Nic>::new());

    let mut subnets: Vec<super::system::Subnet> = Vec::new();

    for lasir_subnet in network {
        let subnet_name = format!("{}_subnet-{}", vnet.name, subnet_idx);
        for vm_idx in lasir_subnet.connected_vms.keys() {

            let nic_num = all_nics[*vm_idx].len();
            all_nics[*vm_idx].push(super::system::Nic {
                name: format!("{}-nic{}",
                vms[*vm_idx].name.clone(), nic_num),
                vnet: vnet.name.clone(),
                subnet: subnet_name.clone(),
                private_ip_address: lasir_subnet.connected_vms[vm_idx],
                has_public_ip_address: nic_num == 0 && vms[*vm_idx].has_remote_access,
                // ^ only set first NIC of a VM to have a public address, required because
                //the public IP address must be assigned to the first NIC due to a quirk in Azure
            });
        }

        subnets.push( super::system::Subnet{name: subnet_name, address_prefixes: lasir_subnet.prefix.clone()} );
        subnet_idx+=1;
    }

    subnets.push( super::system::Subnet{name: format!("{}-stub-subnet", vnet_name), address_prefixes: CidrIP::from("10.0.255.0/24")} );

    let mut vm_idx: usize = 0;
    //VMs
    //TODO: make stub network (network for VMs with no connections) clearer in terms of naming
    let mut all_vms: Vec<super::system::Vm> = Vec::new();
    for pasir_vm in vms {

        // if the VM has no connections, as chosen by the user
        let vm_nics = if all_nics[vm_idx].is_empty() {
            let curr_ip_addr_count = IP_ADDR_COUNTER.fetch_add(1, Ordering::Relaxed);
            let stub_nic_name: String = format!("{}-stub-nic-{}", pasir_vm.name, curr_ip_addr_count);
            //TODO: remove hard-coded name and IP
            vec![super::system::Nic::new_with_public_ip(stub_nic_name,
                                                        format!("{}", vnet_name),
                                                        format!("{}-stub-subnet", vnet_name),
                                                        format!("10.0.255.{}", curr_ip_addr_count))]
        } else {
            all_nics[vm_idx].clone()
        };
        let vm_size = find_most_fitting_vm(pasir_vm);
        all_vms.push( super::system::Vm {
            name: format!("{}", pasir_vm.name),
            nics: vm_nics, //TODO: should use a better system than just cloning the vec
            image: global_config::COMMON_OS_MAP[&pasir_vm.os.get_common()].clone(),
            size: vm_size.to_string(),
            admin_username: pasir_vm.auth.user.to_string(),
            admin_password: pasir_vm.auth.password.to_string(),
            authentication_type: "all".to_string(),
            custom_script: {
                    if pasir_vm.config_template != "" {
                        format!("test-deployment/{}/script.sh", pasir_vm.name)
                    }
                    else {
                        "".to_string()
                    }
                },
        });
    vm_idx+=1;
    }
    let rg: String = crate::utils::global_config::PROVIDERS_CONFIG[RG].clone();
    let gc = super::system::Globals {
        location: "westeurope".to_string(),
        resource_group: rg,
        has_stub_network: true,
    };
    let ret = super::system::WholeSystem{
        global_config: gc,
        vnet: vnet,
        subnets: subnets.clone(),
        vms: all_vms
    };
    ret
}

fn find_most_fitting_vm(vm: &pasir::machines::Vm) -> String {

    match &vm.override_config {
        Some(over) => over.clone(),
        None => match &vm.hwconfig {
            Some(conf) => find_best_matching_vm_name(Some(conf.cpu_cores), Some(conf.ram_gb)).name(),
            None => panic!("Error: machine {} does not have a hardware description (either a cloud-specific or a regular one).", vm.name),
        }
    }
}