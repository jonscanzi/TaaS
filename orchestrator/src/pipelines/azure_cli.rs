use crate::pasir;
use crate::azuresir;
use std::thread;
use std::iter::Iterator;
use crate::shell_tools;
use shell_tools::RunInfo;
use crate::pipelines::TaasPipeline;
use crate::utils::global_config::SHELL;

pub struct AzureTaasPipeline {}

impl AzureTaasPipeline {
}

impl TaasPipeline for AzureTaasPipeline {

    fn create_system(pasir_vms: &Vec<pasir::machines::Vm>, pasir_network: &Vec<pasir::connections::Subnet>, system_name: &str) {

        let azuresir = azuresir::translator::pasir_to_azuresir(&pasir_vms, pasir_network, system_name);
        let emitter_system = azuresir::emitter::emit_new(&azuresir);
        let mut network_script = String::with_capacity(emitter_system.network.len());// + emitter_system.vms.len() * emitter_system.vms[0].len());
        println_with_time!("Azure - Creating network for {}", system_name);
        network_script.push_str(&emitter_system.network);
        crate::shell_tools::run_command(&format!("{}", network_script), &SHELL.shell).panic_on_failure();

        let pause_duration = Self::determine_pause_between_vms(emitter_system.vms.len());
        let mut machine_jh = Vec::with_capacity(emitter_system.vms.len());
        emitter_system.vms.iter().enumerate().for_each(|(idx, v)| {
            let v = v.to_owned(); //necessary for thread spawning
            println_with_time!("Azure - Starting creation for {} of machine {}", system_name, pasir_vms[idx].name);
            let jh = thread::spawn(move || {
                crate::shell_tools::run_command(&format!("{}", v), &SHELL.shell).panic_on_failure();
            });
            machine_jh.push(jh);
            thread::sleep(pause_duration);
        });
        for jh in machine_jh {
            jh.join().expect("Azure - Error: at least one machine failed to deploy. Please look at the program output or the Azure website to determine what went wrong.");
        }
        println_with_time!("Azure - Finished creating system {}", system_name);
    }

    fn run_script(machine_name: &str, script_text: &str) {
        crate::cloud_functions::azure::send_and_exec_script_small(machine_name, script_text);
    }

    fn get_public_ip(machine_name: &str) -> String {
        crate::cloud_functions::azure::get_public_ip(machine_name)
    }
}