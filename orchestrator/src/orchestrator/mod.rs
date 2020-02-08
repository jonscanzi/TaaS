use crate::pasir;
use crate::asir;
use crate::utils::types::CidrIP;
use std::collections::HashMap;

const ORCH_USER: &str = "orch";
const ORCH_PASS: &str = "asdfgDDFjklqwe1234";

/// Creates a hard-coded PASIR VM for the orchestrator (webserver)
/// It runs ubuntu and has its own, independent network
pub fn create() -> (pasir::machines::Vm, pasir::connections::Subnet) {

    let vm = pasir::machines::Vm {
        name: "orchestrator".to_string(),
        os: pasir::machines::Os {
            candidates: asir::OsCandidates::common_only("UbuntuLTS"),
        },
        hwconfig: Some(pasir::machines::HwConfig::default()),
        override_config: None,
        config_template: "".to_string(),
        role: "".to_string(),
        has_remote_access: true,
        auth: pasir::machines::Auth {
            user: ORCH_USER.to_string(),
            password: ORCH_PASS.to_string(),
        },
    };

    // For the post-deployment glogabl replacement map
    //TODO: do it more properly
    crate::post_deployment::add_global_replacement("machines/orchestrator/user", ORCH_USER);
    crate::post_deployment::add_global_replacement("machines/orchestrator/username", ORCH_USER);
    crate::post_deployment::add_global_replacement("machines/orchestrator/pass", ORCH_PASS);
    crate::post_deployment::add_global_replacement("machines/orchestrator/password", ORCH_PASS);

    let sn = pasir::connections::Subnet {
        prefix: CidrIP::from("10.1.0.0/16"),
        connected_vms: {
            let mut ret = HashMap::new();
            ret.insert(0, "10.1.0.5".parse().unwrap());
            ret
        },
    };
    (vm, sn)
}