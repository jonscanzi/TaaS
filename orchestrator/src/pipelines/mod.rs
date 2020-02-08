pub mod azure_cli;

use crate::yamlsir;
use crate::lasir;
use crate::pasir;
use crate::paths;
use crate::utils::run_parser;

use lasir::machines::LogicalSystem as lasirSystem;
use pasir::machines::Vm as pasirVm;
use run_parser::StepType;

use std::fs;
use std::path::Path;
use crate::cloud_functions;
use std::thread;
use std::time;
use std::collections::HashSet;
use std::collections::HashMap;
use crate::utils::replace;
use std::time::Duration;
use crate::utils;
use std::iter::Iterator;
use crate::shell_tools;
use crate::utils::global_config::SSH;
use crate::utils::global_config::SHELL;

use shell_tools::RunInfo;
use std::iter::FromIterator;
use std::convert::TryInto;

const ONE_SEC: Duration = time::Duration::from_millis(1000);
// const TWO_SECS: Duration = time::Duration::from_millis(2000);
// const THREE_SECS: Duration = time::Duration::from_millis(3000);
// const FOUR_SECS: Duration = time::Duration::from_millis(4000);
// const EIGHT_SECS: Duration = time::Duration::from_millis(5000);
const RUN_STEP_PREFIX: &str = "step_run";
const SETUP_STEP_PREFIX: &str = "step_setup";

pub trait TaasPipeline {

    /// Given a set of PASIR VMs, returns the associate hostname / public IP address of each that has one
    /// Will also add then to the global map
    fn map_public_hostname_with_global(pasir_vms: &Vec<pasir::machines::Vm>) -> HashMap<String, String> {

        fn global_repl_host(machine_name: &str, public_hostname: &str) {
        
            crate::post_deployment::add_global_replacement(&format!("machines/{}/public_ip", &machine_name), &format!("{}", &public_hostname));
            crate::post_deployment::add_global_replacement(&format!("machines/{}/public_ip_address", &machine_name), &format!("{}", &public_hostname));
            crate::post_deployment::add_global_replacement(&format!("machines/{}/public_host", &machine_name), &format!("{}", &public_hostname));
            crate::post_deployment::add_global_replacement(&format!("machines/{}/public_hostname", &machine_name), &format!("{}", &public_hostname));
        }

        let mut ret: HashMap<String, String> = HashMap::new();
        pasir_vms.iter().filter(|vm| vm.has_remote_access).for_each(|vm| {
            let phn = Self::get_public_ip(&vm.name);
            ret.insert(vm.name.clone(), phn.clone());
            global_repl_host(&vm.name, &phn);
        });
        ret
    }

    // ====================================== ORCHESTRATOR ==========================================================

    /// Uses the newly created deployment folder containing the config for all vms,
    /// then pushes all of these files to the orchestrator with scp
    fn prepare_ws<'a, V: Clone>(ip: &str, machine_names: V)
        where V: IntoIterator<Item = String> {

            let mut success = false;
            let mut counter = 4;

            while counter > 0 {
                //TODO: get rid of clone
                Self::setup_webserv(ip, machine_names.clone());

                let fail1 = crate::shell_tools::run_command_no_output(&format!("ssh {} -oStrictHostKeyChecking=no -oUserKnownHostsFile=/dev/null orch@{} \"echo asdfgDDFjklqwe1234 | sudo -S systemctl status orche\"", SSH.custom_args, ip), &SHELL.shell).failure();
                let fail2 = crate::shell_tools::run_command_no_output(&format!("ssh {} -oStrictHostKeyChecking=no -oUserKnownHostsFile=/dev/null orch@{} \"echo asdfgDDFjklqwe1234 | sudo -S ls /home/orch/machine_reports\"", SSH.custom_args, ip), &SHELL.shell).failure();

                if !fail1 && !fail2 {
                    success = true;
                    break;
                }
                println_with_time!("Webserver configuration failed, trying again...");
                counter-=1;
            }

        if !success {
            panic!();
        }
    }

    fn setup_webserv<'a, V>(ip: &str, machine_names: V)
        where V: IntoIterator<Item = String> {

        std::fs::create_dir_all("test-deployment").unwrap_or_else(|_| panic!("Error: could not create temporary folder for deployment templates."));

        std::fs::copy("webserver/webserver", "test-deployment/ws").unwrap_or_else(|_| panic!("Error: could not copy the webserver binary to the temporary folder. Make sure it exists at ./webserver/webserver, and the user has access to it."));

        // Common data
        let common_data_dir = fs::read_dir("temp_common_data");
        if common_data_dir.is_ok() && common_data_dir.unwrap().count() > 0 {
            crate::shell_tools::run_command(&format!("cd temp_common_data; tar czf common_data.tgz ./*"), &SHELL.shell).panic_on_failure();
            crate::shell_tools::run_command_try_again(&format!("ssh {} -oStrictHostKeyChecking=no -oUserKnownHostsFile=/dev/null orch@{} \"mkdir -p common_data\"", SSH.custom_args, ip), &SHELL.shell, 8, None).panic_on_failure();
            crate::shell_tools::run_command_try_again(&format!("scp {} -oStrictHostKeyChecking=no -oUserKnownHostsFile=/dev/null temp_common_data/common_data.tgz orch@{}:~/common_data/", SSH.custom_args, ip), &SHELL.shell, 8, Some(ONE_SEC));
            crate::shell_tools::run_command_try_again(&format!("ssh {} -oStrictHostKeyChecking=no -oUserKnownHostsFile=/dev/null orch@{} \"cd common_data; tar xzf common_data.tgz\"", SSH.custom_args, ip), &SHELL.shell, 8, None);
        }
        
        crate::shell_tools::run_command_try_again(&format!("scp {} -oStrictHostKeyChecking=no -oUserKnownHostsFile=/dev/null test-deployment/ws orch@{}:~/ws", SSH.custom_args, ip), &SHELL.shell, 8, Some(ONE_SEC));
        crate::shell_tools::run_command_no_output("rm test-deployment/ws", &SHELL.shell);
        crate::shell_tools::run_command_try_again(&format!("scp {} -oStrictHostKeyChecking=no -oUserKnownHostsFile=/dev/null webserver/webserver_setup.sh orch@{}:~/webserver_setup.sh", SSH.custom_args, ip), &SHELL.shell, 8, Some(ONE_SEC));
        crate::shell_tools::run_command_try_again(&format!("scp {} -oStrictHostKeyChecking=no -oUserKnownHostsFile=/dev/null webserver/orche.service orch@{}:~/orche.service", SSH.custom_args, ip), &SHELL.shell, 8, Some(ONE_SEC));

        crate::shell_tools::run_command_try_again(&format!("ssh {} -oStrictHostKeyChecking=no -oUserKnownHostsFile=/dev/null orch@{} \"mkdir -p machine_reports\"", SSH.custom_args, ip), &SHELL.shell, 8, Some(ONE_SEC));

        let dirs: Vec<String> = machine_names.into_iter().collect();
        let dirs = dirs.join(" machine_reports/");
        let dirs = "machine_reports/".to_owned() + &dirs;

        crate::shell_tools::run_command_try_again(&format!("ssh {} -oStrictHostKeyChecking=no -oUserKnownHostsFile=/dev/null orch@{} \"mkdir -p {}\"", SSH.custom_args, ip, dirs), &SHELL.shell, 8, Some(ONE_SEC));
        crate::shell_tools::run_command_try_again(&format!("ssh {} -oStrictHostKeyChecking=no -oUserKnownHostsFile=/dev/null orch@{} \"echo asdfgDDFjklqwe1234 | sudo -S sh ~/webserver_setup.sh\"", SSH.custom_args, ip), &SHELL.shell, 8, Some(ONE_SEC));
    }

    // ====================================================================================================================

    // ================================================ TEMPLATES =========================================================

    fn generate_replacement_yml(vm_name: &str, replacement_map: &HashMap<String, String>) -> String {
        let mut ret = String::with_capacity(vm_name.len() + replacement_map.len()*32); //assume on average 32 characters for each replacement in the yml

        ret.push_str("  {\n");
        ret.push_str(&format!("    name: {},\n", vm_name));
        ret.push_str("    replacements:\n");
        ret.push_str("      {\n");
        for (elem, repl) in replacement_map {
            ret.push_str(&format!("        '{}': '{}',\n", elem.replace("'", "''"), repl.replace("'", "''"))); //replace ' with '' for yaml escaping
        }
        ret.push_str("      },\n");
        ret.push_str("  },\n");
        ret
    }

    #[inline]
    fn parse_yaml(scenario: &str) -> yamlsir::Root {

        let system_yaml_fn = format!("{}/{}/{}", paths::SCENARIO_PATH, scenario, paths::SYSTEM_YAML_NAME);

        //transforming a yaml config file to internal representations
        let parsed_yaml = yamlsir::parse_yaml(&system_yaml_fn);
        parsed_yaml
    }

    #[inline]
    fn yamlsir_to_lasir(yamlsir: &yamlsir::Root) -> lasirSystem<lasir::connections::VmConnectionLogicalV2> {
        lasir::translator::yamlsir_to_lasir(yamlsir)
    }

    fn lasir_to_pasir(lasir_system: &lasirSystem<lasir::connections::VmConnectionLogicalV2>) -> (Vec<pasirVm>, Vec<pasir::connections::Subnet>){

        //translate LASIR to PASIR with subnets
        let vms: &Vec<lasir::machines::Vm> = &lasir_system.vms;
        let connections = &lasir_system.network;
        let pasir_network = pasir::translator::lasir_network_to_pasir_network(connections);
        let pasir_vms = pasir::translator::lasir_vms_to_pasir_vms(vms);

        (pasir_vms, pasir_network)
    }

    // Pipeline idea:

    // Parse yaml
    // Make it LASIR
    // Make it PASIR
    // Give LASIR and PASIR to cloud-specific pipeline steps (using abstraction)
    // Get back new data: Public IPs
    // Run Post-Deployment

    fn run_v2(scenario: &str) {
        println_with_time!("Generating system internal representation...");
        let yamlsir_root = Self::parse_yaml(scenario);
        let lasir_system = Self::yamlsir_to_lasir(&yamlsir_root);
        let (pasir_vms, pasir_subnet) = Self::lasir_to_pasir(&lasir_system);

        //TODO: move this in function
        let common_data_map: HashMap<String, Vec<String>> = {

            let mut ret = HashMap::new();
            for pasir_vm in pasir_vms.clone() {
                let common_data_yaml = std::fs::read_to_string(&format!("scenarios/{}/deployment_templates/{}/common_data.yml", scenario, pasir_vm.config_template));
                match common_data_yaml {
                    Ok(raw_yaml) => {
                        let common_data_list: Result<Vec<String>, _> = serde_yaml::from_str(&raw_yaml);
                        match common_data_list {
                            Ok(lst) => { ret.insert(pasir_vm.name.clone(), lst); () },
                            Err(e) => panic!("Error: common_data.yml for deployment template {} is not proper YAML. Issue:\n{}\n", pasir_vm.config_template, e),
                        }
                    },
                    Err(_) => (),
                }
            }
            ret
        };
        Self::prepare_common_data(&common_data_map);
        println_with_time!("Creating orchestrator webserver...");
        let ws_handle = thread::spawn(move || {
            Self::create_orchestrator();
        });

        // Recover run steps (if they exist)
        let pipeline_fn = format!("scenarios/{}/{}", scenario, paths::RUN_STEPS_FN);
        let run_steps = match Path::new(&pipeline_fn).exists() {
            true => {
                run_parser::parse_run_list(&pipeline_fn)
            },
            false => {
                println_with_time!("Warning: pipeline file is missing");
                Vec::new()
            }
        };
        if run_steps.len() == 0 {
            println_with_time!("Warning: pipeline file does not contain any steps");
        }

        // CREATE SYSTEM
        // This is the specific functions which must take the pasir, and create itw own internal representation
        println_with_time!("Creating machines...");
        let pasir_clones = pasir_vms.clone();
        let pasir_subnet_clone = pasir_subnet.clone();
        let system_jh = thread::spawn(move || {
            Self::create_system(&pasir_clones, &pasir_subnet_clone, "taas_run");
        });

        // Prepare and run webserver
        ws_handle.join().expect("Error: The webserver could not be setup. Please check that ssh is properly configured on your machine by manually connecting to the orchestrator.");
        let orch_ip = Self::get_public_ip("orchestrator");
        println_with_time!("Preparing webserver files...");
        Self::prepare_ws(&orch_ip, pasir_vms.iter().map(|v| v.name.to_owned()));
        system_jh.join().expect("Error: Could not properly create machines. Please look at the cloud-sepcific errors on the program and the website.");
        let public_hostname_map = Self::map_public_hostname_with_global(&pasir_vms);
        Self::prepare_from_templates(scenario, &lasir_system, &pasir_vms, &pasir_subnet, &public_hostname_map, &run_steps);

        // Get data for each machine
        Self::push_data_to_machines(&pasir_vms, &common_data_map, &orch_ip);

        let mut last_deployment_info = String::with_capacity(pasir_vms.len() * 48); //about 48 characters per machine in the yml file
        last_deployment_info.push_str("[\n");
        for vm in &pasir_vms {

            if vm.has_remote_access {

                let pip = cloud_functions::azure::get_public_ip(&vm.name);
                println_with_time!("Public IP address of {}: {}\n  Password: {}", &vm.name, &pip, &vm.auth.password);
                let yml = format!("  {{\n      name: {},\n      username: {},\n      password: {},\n      hostname: {},\n  }},\n", vm.name, vm.auth.user, vm.auth.password, pip);
                last_deployment_info.push_str(&yml);
            }
        }

        last_deployment_info.push_str("]");
        fs::write("last_deployment_summary.yml", last_deployment_info).expect("Error: could not write new file last_deployment_summary.yml. Please check permissions");

        let machine_name_user_map: HashMap<String, String> = pasir_vms.iter().map(|v| (v.name.clone(), v.auth.user.clone())).collect();
        for (step_index, (step_type, all_scripts)) in run_steps.iter().enumerate() {
            println_with_time!("Pipeline - Running step {} ({})", step_index, step_type);

            let mut saved_jh = Vec::new();
            for (machine, _) in all_scripts {
                let file_suffix = match step_type {
                    StepType::Run => RUN_STEP_PREFIX,
                    StepType::Setup => SETUP_STEP_PREFIX,
                };

                //saving unique values for thread ownership
                let user = machine_name_user_map[machine].clone(); 
                let machine = machine.to_string();
                // script filename must the the same as what is in the temporary files
                let filename = format!("{}.{}.sh", file_suffix, step_index);
                let jh = thread::spawn(move || {
                    Self::run_script(&machine, &format!("cd /home/{}; sudo sh {}", user, filename));
                });
                saved_jh.push(jh);

                // add small pause when doing setups to limit load on cloud provider (not on setup to reduce delay between each machine's start)
                match step_type {
                    StepType::Run => (),
                    StepType::Setup => thread::sleep(ONE_SEC),
                }
            }

            for jh in saved_jh {
                jh.join().expect("Error: One of the pipeline scripts failed to execute, please look at the program output to see what went wrong.");
            }
            println_with_time!("Pipeline - Finished running step {} ({})", step_index, step_type);
        }

        // Handle post deployement (if user supplied a post-deployment script file)
        let post_deployment_fn = format!("{}/{}/{}", paths::SCENARIO_PATH, scenario, paths::POST_DEPLOYMENT_SCRIPT_FN);
        let post = fs::read(post_deployment_fn);

        match post {
            Ok(filename) => {
                println_with_time!("Running post_deployment script...");

                let postd_script: String = String::from_utf8(filename).unwrap_or_else(|_| panic!("Error: could not coerce post-deployment script into valid utf-8"));
                let exit_code = crate::post_deployment::run(&postd_script).exit_code();
                println_with_time!("Finished executing post_deployment script. Now exiting with the script's exit code...");
                std::process::exit(exit_code as i32);
            },
            Err(_) => (),
        }
    }

    fn push_data_to_machines(vms: &Vec<pasirVm>, common_data_map: &HashMap<String, Vec<String>>, orch_pip: &str) {
        println_with_time!("Running setup scripts on VMs...");

        let mut jhs = Vec::new();
        let pause_duration = Self::determine_pause_between_vms(vms.len());
        vms.iter().for_each(|v| {
            let common_datas = common_data_map.get(&v.name).map(|m| m.clone());
            //clones in order for threads to have their own copies
            let v = v.clone();
            let orch_pip = orch_pip.to_owned();
            let jh = thread::spawn(move || {
                Self::get_all_vm_files_v2(&v.name, &v.auth.user, &orch_pip, &common_datas);
            });
            jhs.push(jh);
            thread::sleep(pause_duration);
        });

        for j in jhs {
            j.join().unwrap_or_else(|_|
            panic!("Error: At least one setup script failed to be executed properly. Please look at the program's output to determine what went wrong.")
            );
        }
    }

    #[inline]
    fn get_all_vm_files_v2(vm_name: &str, vm_username: &str, ws_ip: &str, common_data: &Option<Vec<String>>) {

        let all_files = std::fs::read_dir(format!("test-deployment/{}", vm_name)).unwrap();
        let mut get_script = String::new();

        get_script.push_str(&format!("cd /home/{}\n", vm_username));

        for file in all_files {
            get_script.push_str(&format!("wget http://{}:8000/{}/{}\n", ws_ip, vm_name, file.unwrap().file_name().to_str().unwrap()));
        }

        match common_data {
            Some(cd) => {
                for common_file in cd {
                    get_script.push_str(&format!("wget http://{}:8000/common_data/{}\n", ws_ip, common_file));
                }
            },
            None => (),
        }

        get_script.push_str(&format!("tar xzf {}.tgz\n", vm_name));

        get_script.push_str(&format!("sudo chown {}:{} ./*\n", vm_username, vm_username));
        get_script.push_str(&format!("rm -f {}.tgz\n", vm_name));
        Self::run_script(&vm_name, &get_script);
    }

    fn prepare_from_templates(
        scenario_name: &str,
        lasir_system: &lasirSystem<lasir::connections::VmConnectionLogicalV2>,
        pasir_vms: &Vec<pasirVm>,
        pasir_network: &Vec<pasir::connections::Subnet>,
        hostname_map: &HashMap<String, String>,
        run_steps_map: &Vec<(run_parser::StepType, HashMap<String, String>)>,
    ) {
        println_with_time!("Creating VM Role -> IP map...");
        let (full_ip_repl_map, singleton_ip_repl_map) = utils::roles::create_vm_local_ip_mapping(lasir_system, &pasir_vms, &pasir_network);
        let mut vm_specific_repl_map = utils::roles::create_ip_string_replacement_map(full_ip_repl_map, singleton_ip_repl_map);

        println_with_time!("Sending scripts and data to Webserver...");
        //TODO; make this in special function
        for idx in 0..vm_specific_repl_map.len() {
            vm_specific_repl_map[idx].insert("NAME".to_string(), pasir_vms[idx].name.clone());
            vm_specific_repl_map[idx].insert("name".to_string(), pasir_vms[idx].name.clone());
            vm_specific_repl_map[idx].insert("VM_NAME".to_string(), pasir_vms[idx].name.clone());
            vm_specific_repl_map[idx].insert("vm_name".to_string(), pasir_vms[idx].name.clone());
            vm_specific_repl_map[idx].insert("machine_name".to_string(), pasir_vms[idx].name.clone());
            vm_specific_repl_map[idx].insert("MACHINE_NAME".to_string(), pasir_vms[idx].name.clone());
            vm_specific_repl_map[idx].insert("vm name".to_string(), pasir_vms[idx].name.clone());
            vm_specific_repl_map[idx].insert("machine name".to_string(), pasir_vms[idx].name.clone());
            vm_specific_repl_map[idx].insert("VM NAME".to_string(), pasir_vms[idx].name.clone());
            vm_specific_repl_map[idx].insert("MACHINE NAME".to_string(), pasir_vms[idx].name.clone());
            vm_specific_repl_map[idx].insert("USER".to_string(), pasir_vms[idx].auth.user.clone());
            vm_specific_repl_map[idx].insert("user".to_string(), pasir_vms[idx].auth.user.clone());
            vm_specific_repl_map[idx].insert("USERNAME".to_string(), pasir_vms[idx].auth.user.clone());
            vm_specific_repl_map[idx].insert("username".to_string(), pasir_vms[idx].auth.user.clone());
            vm_specific_repl_map[idx].insert("password".to_string(), pasir_vms[idx].auth.password.clone());
            vm_specific_repl_map[idx].insert("PASSWORD".to_string(), pasir_vms[idx].auth.user.clone());
            vm_specific_repl_map[idx].insert("pass".to_string(), pasir_vms[idx].auth.user.clone());
            vm_specific_repl_map[idx].insert("PASS".to_string(), pasir_vms[idx].auth.user.clone());
        }

        let mut replacement_map: HashMap<String, String> = HashMap::new();
        let deployment_templates_folder = format!("{}/{}/{}", paths::SCENARIO_PATH, scenario_name, paths::DEPLOYMENT_TEMPLATES_PATH); //TODO: put this string in global file
        Self::replace_templates_for_deploy(&deployment_templates_folder, &pasir_vms, hostname_map, &mut replacement_map, &vm_specific_repl_map, run_steps_map);
    }

    fn replace_templates_for_deploy(
                                deployment_templates_folder: &str,
                                pasir_vms: &Vec<crate::pasir::machines::Vm>,
                                pip_map: &HashMap<String, String>,
                                replacement_map: &mut HashMap<String, String>,
                                vm_specific_repl_map: &Vec<HashMap<String, String>>,
                                run_steps_map: &Vec<(run_parser::StepType, HashMap<String, String>)>,
                            ) {

        let public_ip_str = format!("{}", Self::get_public_ip("orchestrator"));
        replacement_map.insert("ORCHESTRATOR_IP".to_string(), public_ip_str.clone());
        replacement_map.insert("WEBSERVER_IP".to_string(), public_ip_str.clone());
 
        pip_map.iter().for_each(|(k, v)| { replacement_map.insert(k.to_string(), v.to_string()); });
        Self::prepare_template_configs_for_vms(&deployment_templates_folder, &pasir_vms, &replacement_map, vm_specific_repl_map, run_steps_map);

        shell_tools::run_command_try_again(&format!("scp {} -oStrictHostKeyChecking=no -oUserKnownHostsFile=/dev/null -r test-deployment/* orch@{}:~", SSH.custom_args, public_ip_str), &SHELL.shell, 8, Some(ONE_SEC));
    }

    #[inline]
    /// Create configs for all the VMs. This includes the config files, with generic parameters replaced by actual values (e.g. connections -> IP addresses)
    fn prepare_template_configs_for_vms(
        deployment_templates_folder: &str,
        vms: &Vec<pasir::machines::Vm>,
        replacement_map: &HashMap<String, String>,
        vm_specific_repl_map: &Vec<HashMap<String, String>>,
        run_steps_map: &Vec<(run_parser::StepType, HashMap<String, String>)>,
    ) {

        let vm_count: usize = vms.len();

        let mut replacement_yml = String::with_capacity(vm_count * replacement_map.len()*64); //assume 64 characters per vm for yml file
        replacement_yml.push_str("[\n");

        let mut all_machine_specific_replacement_maps = HashMap::new();
        //TODO: replace this loop with .iter().enumerate()
        for vm_idx in 0..vm_count {
            let vm = &vms[vm_idx];
            let mut temp_replacement_map = replacement_map.clone();
            let temp_vm_specific_repl_map: HashMap<String, String> = vm_specific_repl_map[vm_idx].clone();
            temp_replacement_map.extend(temp_vm_specific_repl_map);
            all_machine_specific_replacement_maps.insert(vm.name.clone(), temp_replacement_map.clone()); // copying the temporary replacement map for later use in pipeline run
            if vm.config_template != "" {
                std::fs::create_dir_all(format!("temp-template-deployment/{}", vm.name)).unwrap_or_else(|_| panic!("Error: could not create temporary folder for deployment templates."));
                std::fs::create_dir_all(format!("test-deployment/{}", vm.name)).unwrap_or_else(|_| panic!("Error: could not create temporary folder for deployment templates."));

                //TODO: put this in a function in replace
                let all_to_replace = std::fs::read_dir(format!("{}/{}/replace", deployment_templates_folder, vm.config_template));
                
                //TODO: use a match statement with the Result<_>
                if all_to_replace.is_ok() {
                    let list_replace = all_to_replace.unwrap();
                    for fil in list_replace {
                        let fil = fil.unwrap();
                        let path = fil.path();
                        let filename = path.file_name().unwrap().to_str().unwrap();
                        replace::copy_and_replace_v2(&format!("{}", &path.display()),
                                                &format!("temp-template-deployment/{}/{}", vm.name, &filename),
                                                &temp_replacement_map,
                                                replace::ReplaceFailPolicy::Warn);
                    }
                }

                shell_tools::run_command_no_output(&format!("cp -rf {}/{}/data/* temp-template-deployment/{}/", deployment_templates_folder, vm.config_template, vm.name), &SHELL.shell);
                shell_tools::run_command_no_output(&format!("cd temp-template-deployment/{}; tar czf {}.tgz ./*; cp {}.tgz ../../test-deployment/{}/", vm.name, vm.name, vm.name, vm.name), &SHELL.shell);
                replacement_yml.push_str(&Self::generate_replacement_yml(&vm.name, &temp_replacement_map))
            }
        }

        //handle run steps
        let valid_vm_names: Vec<_> = vms.iter().map(|v| v.name.clone()).collect();
        for (run_index, (script_type, steps)) in run_steps_map.iter().enumerate() {
            for (machine_name, script) in steps {
                // check if machine name given in step run exists
                valid_vm_names.iter().position(|x| x == machine_name).expect(&format!("Error: the run step file references machine '{}', but it is not declared in the system description.", machine_name));
                let file_suffix = match script_type {
                    StepType::Run => RUN_STEP_PREFIX,
                    StepType::Setup => SETUP_STEP_PREFIX,
                };
                
                let filename = format!("test-deployment/{}/{}.{}.sh", machine_name, file_suffix, run_index);
                
                replace::replace_and_write(script,
                                            &filename,
                                            &all_machine_specific_replacement_maps[machine_name],
                                            replace::ReplaceFailPolicy::Warn,
                                            Some(paths::RUN_STEPS_FN.to_string()));
            }
        }

        replacement_yml.push_str("]");
        std::fs::write("last_deployment_replacements.yml", replacement_yml).expect("Could not write last_deployment_replacements.yml. Please check file permissions");
    }

    fn create_orchestrator() {
        let (machine, subnet) = crate::orchestrator::create();
        Self::create_system(&vec![machine], &vec![subnet], "webserver");
    }

    #[inline]
    fn prepare_common_data(common_data_map: &HashMap<String, Vec<String>>) {

        //takes all common datas for all machines and make sure they are only referenced once
        let res: HashSet<String> = HashSet::from_iter(common_data_map.values().fold(Vec::new(), |mut acc, vec| { acc.extend_from_slice(vec.as_slice()); acc }));

        shell_tools::run_command("mkdir -p temp_common_data", &SHELL.shell).panic_on_failure();

        for data in res {
            shell_tools::run_command(&format!("cp -rf scenarios/common_data/{} temp_common_data/", data), &SHELL.shell).panic_on_failure();
        }
    }

    fn run_script_from_file(machine_name: &str, script_fn: &str) {

        let script_text = fs::read_to_string(script_fn).expect(&format!("Error: cannot open {} properly. Check file permission and text encoding.", script_fn));
        Self::run_script(machine_name, &script_text);
    }

    /// Small helper function intended to be used to make pauses between the creation of multiple machines in parralel
    /// This is done so to not overwhelm Azure with a huge number of VMs created at the same time.
    /// The idea is that the pause increases as the number of machines to be created increases.
    #[inline]
    fn determine_pause_between_vms(vm_num: usize) -> time::Duration {
        assert!(vm_num < 2 << 30);
        let base: usize = 1000; //ms
        let val = 2 * base * crate::utils::math::log2_ceil(vm_num as i32);
        time::Duration::from_millis(val.try_into().unwrap())
    }

    // ================================== CLOUD SPECIFIC ======================================

    fn run_script(machine_name: &str, script_text: &str);

    fn create_system(pasir_vms: &Vec<pasir::machines::Vm>, pasir_network: &Vec<pasir::connections::Subnet>, system_name: &str);

    fn get_public_ip(machine_name: &str) -> String;
}