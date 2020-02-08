use std::io::{Write, stdin, stdout};
use crate::utils::global_config::SSH;
use crate::utils::global_config::SHELL;
use crate::shell_tools;
use crate::shell_tools::RunInfo;
use serde::{Serialize, Deserialize};
pub mod azure;

pub trait ScriptPush {

    fn prepare_replaced(&self, vm_name: &str, repl_path: &std::path::Path) {
        //TODO: replace this with some kind of do_for_all_files() function
        let all_content = repl_path.read_dir();
        let _all_content = all_content.map( |c|
            for elem in c {
                elem.map( |e|
                    e.file_type().map( |tpe|
                    
                        match tpe.is_file() {
                            true => {

                                // file name manipulation
                                let mut mut_path = repl_path.to_path_buf();
                                let inn: String = e.path().to_string_lossy().to_string();
                                mut_path.pop();
                                mut_path.push("repl_temp");
                                std::fs::create_dir_all(&mut_path).expect(&format!("Error: could not create directory for temporary replaced files '{}'. Check user permissions.", mut_path.to_string_lossy().to_string()));
                                mut_path.push(e.file_name());
                                let out: String = mut_path.to_string_lossy().to_string();

                                // recover replacement map
                                let repl_data = self.get_vm_replacements(vm_name);
                                println!("{}", out);
                                
                                crate::utils::replace::copy_and_replace_v2(&inn, &out, &repl_data.replacements, crate::utils::replace::ReplaceFailPolicy::Warn)
                            }
                            false => {}
                        }
                    )
                ).unwrap().unwrap();
            }
        );
    }

    fn new() -> Self;
    fn push_script(&self, deployment_name: &str) {

        //TODO: check folder existence
        let vms = self.get_vm_list();

        println!("Available VMs:");
        self.print_vm_list(&vms);
        let chosen_vm_idxs = self.ask_vm_selection();
        self.send_script_for(&chosen_vm_idxs, ScriptRunType::SSH, deployment_name)
    }

    fn print_vm_list(&self, vms: &Vec<String>) {
        for (count, vm) in vms.iter().enumerate() {
            println!("{}: {}", count, vm);
        }
    }

    fn ask_vm_selection(&self) -> Vec<usize> {

        //inner function do parse, for the while loop
        fn parse(max_val: usize) -> Option<Vec<usize>> {

            print!("Choose VMs: ");
            let mut buffer = String::new();
            stdout().flush().unwrap();
            let parsed = stdin().read_line(&mut buffer);
            match parsed {
                Ok(_) => (),
                Err(_) => { return None; },
            }
            let buffer = buffer.trim();

            //remove extra spaces and tabs
            let buffer = buffer.replace('\t', " ");
            let vecc: Vec<&str> = buffer.split(" ").filter(|s| s != &"").collect();

            //Give up and return None if any of the "numbers" is badly formatted or is beyond the list range
            let ret = {
                let mut ret = Vec::new();
                for idx in vecc {
                    let newnum: usize = match idx.parse::<usize>() {
                        Ok(num) => {
                            match num <= max_val {
                                true => num,
                                false => { return None },
                            }
                        },
                        Err(_) => { return None; },
                    };
                    ret.push(newnum);
                }
                ret
            };
            match ret.len() {
                0 => None,
                _ => Some(ret),
            }
        }

        #[allow(unused_assignments)] //have to do this unless there is a way to not assign a value
        let mut arr: Option<Vec<usize>> = None;
        let arr_len = self.get_vm_list().len();
        while {
            arr = parse(arr_len - 1);
            if arr.is_none() {
                println!("Please input a sequence of numbers within the range.");
            }
            arr.is_none()
        } {}

        arr.unwrap()
    }

    fn get_vm_list(&self) -> Vec<String>;
    fn get_vm_summary(&self, idx: usize) -> VmSummary;
    fn get_vm_replacements(&self, name: &str) -> ReplData;

    fn send_script_for(&self, idxs: &Vec<usize>, run_type: ScriptRunType, deployment_name: &str) {

        let all_vms = self.get_vm_list();
    
        for idx in idxs {
            debug_assert!(*idx < all_vms.len());
                println!("Running on machine {}...", idx);
                match run_type {
                    ScriptRunType::SSH => {
                        self.send_script_ssh(&self.get_vm_summary(*idx), deployment_name);
                    },
                    // _ => unimplemented!(),
                }
                println!("Finished running on machine {}.", idx);
        }
    }

    fn send_script_ssh(&self, vm_summary: &VmSummary, deployment_name: &str) {

        //first, delete push_scripts/'deployment_name' in remote machine (as sudo) (in case it exists)
        //then, copy the 'deployment_name' to the machine
        //then, run the script

        shell_tools::run_command(&format!("ssh {} -oStrictHostKeyChecking=no -oUserKnownHostsFile=/dev/null {}@{} \"cd ~; echo {} | sudo -S rm -rf push_scripts/{}\"", SSH.custom_args, vm_summary.username, vm_summary.hostname, vm_summary.password, deployment_name), &SHELL.shell).panic_on_failure();
        shell_tools::run_command(&format!("ssh {} -oStrictHostKeyChecking=no -oUserKnownHostsFile=/dev/null {}@{} \"cd ~; mkdir -p push_scripts/{}\"", SSH.custom_args, vm_summary.username, vm_summary.hostname, deployment_name), &SHELL.shell).panic_on_failure();
        shell_tools::run_command(&format!("scp {} -oStrictHostKeyChecking=no -oUserKnownHostsFile=/dev/null push_scripts/{}/run.sh {}@{}:~/push_scripts/{}/", SSH.custom_args, deployment_name, vm_summary.username, vm_summary.hostname, deployment_name), &SHELL.shell).panic_on_failure();
        
        let data_str = format!("push_scripts/{}/data", deployment_name);
        let data_path = std::path::Path::new(&data_str);
        if data_path.is_dir() && data_path.read_dir().unwrap().count() > 0 {
            shell_tools::run_command(&format!("scp -r {} -oStrictHostKeyChecking=no -oUserKnownHostsFile=/dev/null push_scripts/{}/data/* {}@{}:~/", SSH.custom_args, deployment_name, vm_summary.username, vm_summary.hostname), &SHELL.shell).panic_on_failure();
        }
        let repl_str = format!("push_scripts/{}/replace", deployment_name);
        let repl_path = std::path::Path::new(&repl_str);
        if repl_path.is_dir() && repl_path.read_dir().unwrap().count() > 0 {
            self.prepare_replaced(&vm_summary.name, &repl_path);
            shell_tools::run_command(&format!("scp {} -oStrictHostKeyChecking=no -oUserKnownHostsFile=/dev/null push_scripts/{}/repl_temp/* {}@{}:~/", SSH.custom_args, deployment_name, vm_summary.username, vm_summary.hostname), &SHELL.shell).panic_on_failure();
            shell_tools::run_command_no_output(&format!("rm -rf push_scripts/{}/repl_temp", deployment_name), &SHELL.shell);
        }
        shell_tools::run_command(&format!("ssh {} -oStrictHostKeyChecking=no -oUserKnownHostsFile=/dev/null {}@{} \"cd ~; echo {} | sudo -S sh ~/push_scripts/{}/run.sh\"", SSH.custom_args, vm_summary.username, vm_summary.hostname, vm_summary.password, deployment_name), &SHELL.shell).panic_on_failure();
    }

    fn load_last_deployment_summary() -> Vec<VmSummary> {
        let last_deployment_summary = std::fs::read_to_string("last_deployment_summary.yml").expect("Error: could not open last_deployment_summary.yml. Make sure you have run the deployment first, as well as the file permissions."); //TODO: don't use hard-coded value
        let parse: Result<Vec<VmSummary>, _> = serde_yaml::from_str(&last_deployment_summary); 

        //TODO: handle unwrap
        parse.unwrap()
    }

    fn load_last_deployment_replacements() -> Vec<ReplData> {
        let last_deployment_replacements = std::fs::read_to_string("last_deployment_replacements.yml").expect("Error: could not open last_deployment_replacements.yml. Make sure you have run the deployment first, as well as the file permissions."); //TODO: don't use hard-coded value
        let parse: Result<Vec<ReplData>, _> = serde_yaml::from_str(&last_deployment_replacements);

        //TODO: handle unwrap
        parse.unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmSummary {
    name: String,
    hostname: String,
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReplData {
    name: String,
    replacements: std::collections::HashMap<String, String>,
}

#[derive(Clone)]
pub enum ScriptRunType {
    SSH,
    // CloudSpecific,
}