use crate::utils::global_config::PROVIDERS_CONFIG;
use std::sync::atomic::{AtomicUsize, Ordering};

// Counter to make sure all public IP addresses have public names
static PUBLIC_IP_COUNT: AtomicUsize = AtomicUsize::new(0);

pub struct EmitterSystem {
    pub network: String,
    pub vms: Vec<String>,
}

/// Given an entire system description, generate string for a shell script for the VMs and the network
pub fn emit_new(ws: &super::system::WholeSystem) -> EmitterSystem {
    //emit shell script:
    
    let mut vm_scripts: Vec<String> = Vec::with_capacity(ws.vms.len());
    let mut net_script = String::new();

    // ============================= Create vnet =========================================
    net_script.push_str(&generate_vnet_script(&ws));

    // =================== Create subnets ===================
    net_script.push_str(&generate_subnet_scripts(&ws));

    // =============== Create public IP, NICs, VMs =================
    // doing it at the same time as they are heavily interdependent in Azure
    for vm in &ws.vms {
        let new_vm = generate_whole_vm_script(vm, ws);
        vm_scripts.push(new_vm);
        
    }
    EmitterSystem {
        network: net_script,
        vms: vm_scripts,
    }
}

/* @brief
 * takes a command name (should be either a single program such as 'ls'
 * or a string of fixed multi-word command (e.g. 'ip addr') and arguments,
 * and build a string with the command
 *
 * format of the arguments: (is_named, parameter, argument)
 * (true, "ab", "cd") would translate to "--ab cd" while
 * (false, "a", "cd") would translate to "-a cd"
 */
pub fn generate_shell_command(name: &str, args: Vec<(bool, &str, &str)>) -> String {
    let mut ret = String::new();
    ret.push_str(name);
    ret.push_str(" \\\n");
    for arg in &args {
        let (is_named, param, arg) = arg;
        if *param == "" && *arg == "" {
            continue;
        }
        if *is_named {
            ret.push_str("\t--");
        }
        else {
            ret.push_str("\t-");
        }
        //trim param and args for pretty-printing
        ret.push_str(param.trim());
        ret.push_str(" ");
        ret.push_str(arg.trim());
        ret.push_str(" \\\n");
    }
    ret.truncate(ret.len()-2); // remove trailing '\'
    ret.push('\n'); //put back last newline
    ret
}

/// Given an AzureSIR VM description and global variables, generate a set of Azure CLI shell commands to create
/// a VM and its attached resources (NICs, Public IPs)
fn generate_whole_vm_script(vm: &super::system::Vm, ws: &super::system::WholeSystem) -> String {
    let mut sh_script = String::new();
    let mut nics_tmp_shell = String::new(); //temp string to insert all NICs at once
            // need to record public and private nics separately because Azure wants public NICs to
            // be attached first to a VM...
            let mut public_nic_names = String::new();
            let mut private_nic_names = String::new();
            for nic in &vm.nics {
                //NIC and public IP address
                let mut nic_params: Vec<(bool, &str, &str)> = Vec::new(); //params for the nics, done this way because not all NICs will have public IP
                nic_params.push((true, "resource-group", &ws.global_config.resource_group));
                nic_params.push((true, "name", &nic.name));
                nic_params.push((true, "vnet-name", &nic.vnet));

                nic_params.push((true, "subnet", &nic.subnet));
                //nic_params.push((true, "subnet", &Some(nic.subnet)));
                let private_ip = &nic.private_ip_address.to_string();
                nic_params.push((true, "private-ip-address", &private_ip));

                // using global counter to make sure all public ip addresses are unique
                let curr_pip_count = PUBLIC_IP_COUNT.fetch_add(1, Ordering::Relaxed);
                let pip_name = &format!("public-ip-{}", curr_pip_count);
                if nic.has_public_ip_address {

                    let tmp = generate_shell_command(&format!("\n{} network public-ip create", PROVIDERS_CONFIG["azure-cli-binary"]), 
                                                     vec![(true, "resource-group", &ws.global_config.resource_group),
                                                          (true, "dns-name", &format!("{}-{}-{}", crate::utils::global_config::PROVIDERS_CONFIG["resource-group"].trim().to_ascii_lowercase(), crate::utils::global_config::NETWORK["dns_prefix"].trim().to_ascii_lowercase(), vm.name.trim().to_ascii_lowercase())),
                                                          (true, "name", pip_name)]);

                    nic_params.push((true, "public-ip-address", pip_name));
                    sh_script.push_str(&tmp);
                    public_nic_names.push_str(&nic.name);
                    public_nic_names.push_str(" ");

                }
                else {
                    private_nic_names.push_str(&nic.name);
                    private_nic_names.push_str(" ");
                }
                let tmp = generate_shell_command(&format!("\n{} network nic create", PROVIDERS_CONFIG["azure-cli-binary"]), nic_params);
                nics_tmp_shell.push_str(&tmp);
            }
            sh_script.push_str(&nics_tmp_shell);

            // VM creation
            let dns_name = format!("{}.taas", &vm.name);
            let nics = public_nic_names + &private_nic_names; //creating this variable because of borrowing rules
            let vm_params: Vec<(bool, &str, &str)> = vec![(true, "resource-group", &ws.global_config.resource_group),
                                                    (true, "name", &vm.name),
                                                    (true, "nics", &nics),
                                                    (true, "image", &vm.image),
                                                    (true, "size", &vm.size),
                                                    (true, "admin-username", &vm.admin_username),
                                                    (true, "admin-password", &vm.admin_password),
                                                    (true, "authentication-type", &vm.authentication_type),
                                                    (true, "public-ip-address-dns-name", &dns_name),
                                                ];

            let tmp = generate_shell_command(&format!("\n{} vm create", PROVIDERS_CONFIG["azure-cli-binary"]), vm_params);
            sh_script.push_str(&tmp);

            sh_script
}

fn generate_vnet_script(ws: &super::system::WholeSystem) -> String {
    let mut ret = String::new();

    let tmp = generate_shell_command(&format!("\n{} network vnet create", PROVIDERS_CONFIG["azure-cli-binary"]),
                                        vec![(true, "resource-group", &ws.global_config.resource_group),
                                            (true, "name", &ws.vnet.name),
                                            (true, "address-prefixes", &ws.vnet.address_prefixes.to_string())]);
    ret.push_str(&tmp);
    ret
}

fn generate_subnet_scripts(ws: &super::system::WholeSystem) -> String {
    let mut ret = String::new();

    for subnet in &ws.subnets {
        let tmp = generate_shell_command(&format!("\n{} network vnet subnet create", PROVIDERS_CONFIG["azure-cli-binary"]), 
                                        vec![(true, "resource-group", &ws.global_config.resource_group),
                                            (true, "name", &subnet.name),
                                            (true, "vnet-name", &ws.vnet.name),
                                            (true, "address-prefixes", &subnet.address_prefixes.to_string())]);
        ret.push_str(&tmp);
    }
    ret
}