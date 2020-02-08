//main.rs
//Author: Jonathan Scanzi
//Date: 09 July 2019

// =========== Libraries =============
extern crate serde;
extern crate yaml_rust;
#[macro_use]
extern crate lazy_static;
extern crate fs_extra;
extern crate base64;
extern crate rand;

// ========== General Data ===========
#[macro_use]
mod logger;
mod asir;
#[macro_use]
mod utils;
mod pipelines;
mod paths;
mod cloud_functions;
mod shell_tools;
#[macro_use]
mod post_deployment;
mod script_push;

// ============ YAMLSIR ==============
mod yamlsir;
// ============= LASIR ===============
mod lasir;
// ============= PASIR ============vpn.adnovum.ch===
mod pasir;
// ========= ORCHESTRATOR -===========
mod orchestrator;
// ========= Cloud Specific ==========
use pipelines::TaasPipeline;
    // =========== Azure =============
    mod azuresir;
    // use pipelines::azure::azure_pipeline;

use std::env;
use utils::global_config::PROVIDERS_CONFIG;
use utils::global_config::SHELL;
use script_push::ScriptPush;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1) {
        Some(arg) => {
            match arg.to_lowercase().as_ref() {
                "delete" | "remove" | "clean" | "rm" | "del" => { clean_azure(); }
                "push" => { azure_push_script(&args[2..]); }
                _ => { normal_azure(arg); }
            }
        }
        None => {
            println!("Please provide the name for a scenario to run.");
            std::process::exit(0);
        }
    }
}

fn normal_azure(scenario: &str) {
    cloud_functions::azure::check_azure_cli_install();
    //remove temp folders from potential previous run
    shell_tools::run_command_no_output(&format!("rm -rf {}", "temp_common_data temp-template-deployment test-deployment temp_deploy_scripts *-b64script.json create_orchestrator.sh"), &SHELL.shell);
    // Build the VM network, deploy it, run tests
    // azure_pipeline(scenario);
    pipelines::azure_cli::AzureTaasPipeline::run_v2(scenario);
}

fn clean_azure() {

    cloud_functions::azure::check_azure_cli_install();

    println!("Clearing temporary files...");
    shell_tools::run_command_no_output(&format!("rm -rf {}", "last_deployment_replacements.yml last_deployment_summary.yml temp_common_data temp-template-deployment test-deployment temp_deploy_scripts *-b64script.json create_orchestrator.sh"), &SHELL.shell);

    // Clear the Azure resource group
    // TODO: move it in azure pipeline
    println!("Clearing resource group {}\nThis may take a while...", PROVIDERS_CONFIG["resource-group"]);
    cloud_functions::azure::clear_resource_group_v2(&PROVIDERS_CONFIG["resource-group"]);
}

fn azure_push_script(args: &[String]) {

    let deployment_name = &args.get(0).expect("Error: please provide a name for the script setup you want to push");

    if !std::path::Path::new(&format!("push_scripts/{}", deployment_name)).is_dir() {
        panic!("Error: cannot access push_scripts/{}, please make sure the path exists and is accessible", deployment_name);
    }

    let azure_push = script_push::azure::AzureScriptPush::new();

    azure_push.push_script(deployment_name);
}
