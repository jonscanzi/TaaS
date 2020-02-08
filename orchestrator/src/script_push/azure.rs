
pub struct AzureScriptPush {
    vms: Vec<super::VmSummary>,
    replace: Vec<super::ReplData>,
}

impl super::ScriptPush for AzureScriptPush {

    fn new() -> Self {
        AzureScriptPush {vms: Self::load_last_deployment_summary(), replace: Self::load_last_deployment_replacements()}
    }

    fn get_vm_summary(&self, idx: usize) -> super::VmSummary {
        self.vms[idx].clone()
    }

    fn get_vm_list(&self) -> Vec<String> {

        self.vms.iter().map(|info| info.name.to_owned()).collect()
    }

    fn get_vm_replacements(&self, name: &str) -> super::ReplData {

        let r_index = self.replace.iter().position(|i| i.name == name);
        assert!(r_index.is_some());
        self.replace[r_index.unwrap()].clone() //TODO: remove expensive clone
    }
}