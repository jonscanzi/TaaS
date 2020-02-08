#![allow(unused)]
use std::process::Command;
use crate::utils::global_config::DEFAULT_VALUES;
use crate::utils::global_config::PROVIDERS_CONFIG;
use crate::utils::global_config::SHELL;
use std::fs;
use base64::encode;
use serde::{Deserialize, Serialize};
use crate::cloud_functions::VmHardwareProperties;
use crate::utils::macros;
use crate::shell_tools;
use crate::shell_tools::RunInfo;
use std::sync::Mutex;
use std::net::Ipv4Addr;

macro_rules! get_prop {
    ($map: ident, $name: expr, $default: expr) => {
        $map.iter().filter(|prop| prop.name == $name).nth(0).map(|prop| prop.value.parse().unwrap_or($default)).unwrap_or($default)
    }
}

pub fn find_best_matching_vm_name(core_count: Option<usize>, ram_gb: Option<usize>) -> AzureProperties {

    let parsed_azurevms: Vec<AzureVMSize> = parse_azure_skus();

    let specific: Vec<AzureProperties> = get_important_info(&parsed_azurevms);
    let specific: Vec<&AzureProperties> = vec_to_ref_vec(&specific);

    let mut lenience = 0;
    let mut ret: Option<AzureProperties>;

    loop {
        let sp = specific.clone();
        let specific_with_lenience: Vec<&AzureProperties> = match core_count {
        Some(core_count) => get_vm_with_core_count(sp, core_count, lenience/4),
        None => sp,
        };
        let specific_with_lenience: Vec<&AzureProperties> = match ram_gb {
            Some(ram_gb) => get_vm_with_ram_amount(specific_with_lenience, ram_gb, lenience),
            None => specific_with_lenience,
        };
        if (specific_with_lenience.len() == 0) {
            lenience+=1;
            continue;
        }
        else {
            ret = Some(specific_with_lenience[0].clone());
            break;
        }
    }

    //TODO: might want to make a more robust choice
    let arbitrary_choice: &AzureProperties = &ret.unwrap();
    arbitrary_choice.clone()
}

//TODO: replace these with methods inside the VmHardwareProperties struct
fn get_vm_with_ram_amount<'a, V, P: 'a>(vms: V, ram_gb: usize, deviation: usize) -> Vec<&'a P>
    where V: IntoIterator<Item = &'a P>,
          P: VmHardwareProperties {

    vms.into_iter().filter(|prop| within_bounds_incl!(ram_gb-deviation, prop.ram_gb(), ram_gb+deviation)).collect()
}

/// I could not find this feature on Rust's API
fn vec_to_ref_vec<'a, I, E: 'a>(it: I) -> Vec<&'a E>
    where I: IntoIterator<Item = &'a E> {
        it.into_iter().filter(|_| true).collect()
}

fn get_vm_with_core_count<'a, V, P: 'a>(vms: V, core_count: usize, deviation: usize) -> Vec<&'a P>
    where V: IntoIterator<Item = &'a P>,
          P: VmHardwareProperties {

    vms.into_iter().filter(|prop| within_bounds_incl!(core_count-deviation, prop.core_count(), core_count+deviation)).collect()
}

/// For the given Azure VMs, finds all properties considered as important.
/// Mostly CPU core count, RAM amount, etc. though it is meant to be augmented in the future
fn get_important_info<'a, I>(vms: I) -> Vec<AzureProperties>
    where I: IntoIterator<Item = &'a AzureVMSize> {

    let mut ret: Vec<AzureProperties> = Vec::new();
    for vm in vms {
        let cap_map: Option<_> = vm.capabilities.as_ref();
        let cap_map = unwrap_or_continue!(cap_map);
        let name = vm.name.clone().unwrap();
        let core_count: usize = get_prop!(cap_map, "vCPUs", 0);
        let ram_gb: usize = get_prop!(cap_map, "MemoryGB", 0);
        let max_disk_count = get_prop!(cap_map, "MaxDataDiskCount", 0);
        let max_disk_capacity_gb = std::cmp::min(get_prop!(cap_map, "MaxResourceVolumeMB", 0), get_prop!(cap_map, "OSVhdSizeMB", 0))/1024;

        let new = AzureProperties { name: name,
                                    core_count: core_count,
                                    ram_gb: ram_gb,
                                    max_disk_count: max_disk_count,
                                    max_disk_capacity_gb: max_disk_capacity_gb,
        };
        ret.push(new);
    }
    ret
}
/// Querries __all__ Azure VMs and collect all hardware info
//TODO: do this in a lazy_static variable to avoid calling it multiple times
fn parse_azure_skus() -> Vec<AzureVMSize> {

    // shell_tools::run_command(&format!("az vm list-skus -l {}", PROVIDERS_CONFIG["location"]));
    let cmd = Command::new("sh")
        .arg("-c")
        .arg(format!("{} vm list-skus -l {}", PROVIDERS_CONFIG["azure-cli-binary"], PROVIDERS_CONFIG["location"])).output().unwrap().stdout;
    
    let json = String::from_utf8_lossy(&cmd).to_string();
    let ret: Vec<AzureVMSize> = serde_json::from_str(&json).unwrap();
    ret
}

#[derive(Debug, Clone)]
pub struct AzureProperties {
    name: String,
    core_count: usize,
    ram_gb: usize,
    max_disk_count: usize,
    max_disk_capacity_gb: usize,
}

impl VmHardwareProperties for AzureProperties {
    make_getter!(core_count~usize, ram_gb~usize, max_disk_count~usize, max_disk_capacity_gb~usize);

    missing_getter!(max_network_throughput~usize, threads_per_core~usize);

    fn name(&self) -> String {
        self.name.clone()
    }
}

fn default_str() -> Option<String> {
    Some(String::from("default:("))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AzureVMSize {
    #[serde(default = "default_str")]
    name: Option<String>,
    #[serde(default = "default_str")]
    tier: Option<String>,
    capabilities: Option<Vec<AzureVMCapabilities>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AzureVMCapabilities {
    name: String,
    value: String,
}

/// Arcane Azure JSON for an empty resource group
const DELETE_RG: &str = "{
\"$schema\": \"https://schema.management.azure.com/schemas/2015-01-01/deploymentTemplate.json#\",
\"contentVersion\": \"1.0.0.0\",
\"parameters\": { },
\"variables\": { },
\"resources\": [ ],
\"outputs\": { }
}
";

// Get Azure public IP based on VM name
pub fn get_public_ip(vm_name: &str) -> String {

    let ha = Command::new("sh")
            .arg("-c")
            .arg(format!("{} vm show -d -g {} -n {} --query publicIps -o tsv", PROVIDERS_CONFIG["azure-cli-binary"], PROVIDERS_CONFIG["resource-group"], vm_name)).output().unwrap().stdout;

    let ha = String::from_utf8_lossy(&ha);
    let ha = ha.trim();
    ha.to_string()
}

/// Clears an entire resource group (i.e. makes it empty)
/// Note: the resource group will NOT be deleted
const CLEAR_FN: &str = "removeall.json";
pub fn clear_resource_group(rg_name: &str) {
    fs::write(CLEAR_FN, DELETE_RG);
    
    shell_tools::run_command(&format!("{} group deployment create --mode complete --template-file {} --resource-group {}", PROVIDERS_CONFIG["azure-cli-binary"], CLEAR_FN, rg_name), &SHELL.shell);
    fs::remove_file(CLEAR_FN);
}

pub fn clear_resource_group_v2(rg_name: &str) {

    let ids = shell_tools::run_command(&format!("{} resource list -g '{}' --query '[].id' -o tsv", PROVIDERS_CONFIG["azure-cli-binary"], rg_name), &SHELL.shell);
    ids.panic_on_failure();
    let mut ids = ids.stdout().trim();

    if (ids != "") {
        let mut ids = ids.to_string();
        let ids = ids.replace("\n", " ");
        shell_tools::run_command(&format!("{} resource delete --ids {}", PROVIDERS_CONFIG["azure-cli-binary"], ids), &SHELL.shell).panic_on_failure();
    }
}

/// takes a shell (or any properly shebang'ed) script, uploads
/// it to the required azure VM and executes it.
/// note that Azure limits the size of the script to 256KB
#[allow(unused)]
pub fn send_and_exec_script(machine_name: &str, script_text: &str) {
    let b64encoded = encode(script_text);
    let json_fn = format!("{}-b64script.json", machine_name);
    fs::write(&json_fn , format!("{{\n\t\"script\": \"{}\"\n}}", &b64encoded));

     shell_tools::run_command(&format!("{} vm extension set --resource-group {} --vm-name {} --name customScript --publisher Microsoft.Azure.Extensions --settings ./{}", PROVIDERS_CONFIG["azure-cli-binary"], PROVIDERS_CONFIG["resource-group"], machine_name, json_fn), &SHELL.shell).panic_on_failure();
}

pub fn send_and_exec_script_small(machine_name: &str, script_text: &str) {

     shell_tools::run_command(&format!("{} vm run-command invoke -g {} -n {} --command-id RunShellScript --scripts '{}'", PROVIDERS_CONFIG["azure-cli-binary"], PROVIDERS_CONFIG["resource-group"], machine_name, script_text), &SHELL.shell).panic_on_failure();
}

fn check_logged_in() {
    match shell_tools::run_command_no_output(&format!("{} account show", PROVIDERS_CONFIG["azure-cli-binary"]), &SHELL.shell).non_zero_exit() {
        false => (),
        true => panic!("Error: Azure CLI is installed but does not seem to be logged in. Please run \"az login\".")
    };
}

pub fn check_azure_cli_install() {
    match shell_tools::check_command_exist(&PROVIDERS_CONFIG["azure-cli-binary"]) {
        true => check_logged_in(),
        false => panic!("Error: Azure CLI (az) is not installed on this system with path '{}', please check the provided path in the config files and/or install Azure CLI.", PROVIDERS_CONFIG["azure-cli-binary"]),
    };
}