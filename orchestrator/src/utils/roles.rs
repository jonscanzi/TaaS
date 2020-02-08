use crate::lasir;
use crate::pasir;
use lasir::machines::Vm;
use lasir::connections::VmConnectionLogical;
use lasir::machines::LogicalSystem;
use rand::distributions::{Distribution, Uniform};
use std::collections::HashMap;
use std::net::Ipv4Addr;

pub fn create_ip_string_replacement_map(complete_map: Vec<HashMap<String, Vec<(Ipv4Addr, Ipv4Addr)>>>, singleton_map: Vec<HashMap<String, (Ipv4Addr, Ipv4Addr)>>) -> Vec<HashMap<String, String>> {
    let mut ret: Vec<HashMap<String, String>> = Vec::with_capacity(complete_map.len());
    for idx in 0..complete_map.len() {
        // let complete = &complete_map[idx];
        let singleton = &singleton_map[idx];
        let mut new_complete_replacement: HashMap<String, String> = HashMap::new();
        let mut new_random_replacement: HashMap<String, String> = HashMap::new();

        for (role, (ipv4_self, ipv4_out)) in singleton {
            new_random_replacement.insert(format!("=>{}", role), ipv4_out.to_string());
            new_random_replacement.insert(format!("=> {}", role), ipv4_out.to_string());
            new_random_replacement.insert(format!("=>  {}", role), ipv4_out.to_string());
            new_random_replacement.insert(format!("<={}", role), ipv4_self.to_string());
            new_random_replacement.insert(format!("<= {}", role), ipv4_self.to_string());
            new_random_replacement.insert(format!("<=  {}", role), ipv4_self.to_string());
        }
        new_complete_replacement.extend(new_random_replacement);
        ret.push(new_complete_replacement);
    }
    ret
}

//TODO: move in some util file
fn pick_random(max: usize) -> usize {
        assert!(max > 0);

        let mut rng = rand::thread_rng();
        let rnd = Uniform::from(0..max);
        let rnd = rnd.sample(&mut rng);
        rnd
}

pub fn create_vm_local_ip_mapping<'a, C: VmConnectionLogical>(lasir_system: &LogicalSystem<C>, _pasir_vms: &Vec<pasir::machines::Vm>, pasir_subnets: &Vec<pasir::connections::Subnet>) -> (Vec<HashMap<String, Vec<(Ipv4Addr, Ipv4Addr)>>>, Vec<HashMap<String, (Ipv4Addr, Ipv4Addr)>>) {

    let all_roles = find_all_roles(&lasir_system.vms);

    let mut vm_connected_roles_map: Vec<HashMap<String, Vec<usize>>> = Vec::with_capacity(lasir_system.vms.len());
    for (idx, _vm) in lasir_system.vms.iter().enumerate() {
        let mut hm: HashMap<String, Vec<usize>> = HashMap::new();
        for role in &all_roles {
            let vm_idxs = find_all_connected_vms_with_role(idx, &role, &lasir_system);
            hm.insert(role.to_string(), vm_idxs);
        }
        vm_connected_roles_map.push(hm);
    }

    let hm_len = vm_connected_roles_map.len();
    let (mut complete_map, mut random_map): (Vec<HashMap<String, Vec<(Ipv4Addr, Ipv4Addr)>>>, Vec<HashMap<String, (Ipv4Addr, Ipv4Addr)>>) = (Vec::with_capacity(hm_len), Vec::with_capacity(hm_len));
    
    for (curr_vm, map) in vm_connected_roles_map.iter().enumerate() {
        let (mut new_complete_map, mut new_random_map): (HashMap<String, Vec<(Ipv4Addr, Ipv4Addr)>>, HashMap<String, (Ipv4Addr, Ipv4Addr)>) = (HashMap::new(), HashMap::new());

        for (role, vms) in map {
            if vms.len() != 0 {
                let random_singleton = vms[pick_random(vms.len())];
                assert!(random_singleton != curr_vm, "A random VM chosen for IP mapping was the same as the source VM, index {}", random_singleton);

                let all_ipv4: Vec<(Ipv4Addr, Ipv4Addr)> = vms.iter().map(|vm| lasir_connection_to_ip(curr_vm, *vm, pasir_subnets)
                                                .unwrap_or_else(|| panic!("Failed to find in pasir two VMs connected in lasir, indices {} and {}", curr_vm, *vm))).collect();
                let random_ipv4: (Ipv4Addr, Ipv4Addr) = lasir_connection_to_ip(curr_vm, random_singleton, pasir_subnets)
                                    .unwrap_or_else(|| panic!("Failed to find in pasir two VMs connected in lasir (one chosen randomly), indices {} and {}", curr_vm, random_singleton));
                new_complete_map.insert(role.clone(), all_ipv4);
                new_random_map.insert(role.clone(), random_ipv4);
            }
        }
        complete_map.push(new_complete_map);
        random_map.push(new_random_map);
    }
    (complete_map, random_map)
}

#[inline]
// Peppered with many asserts to make sure everythong is fine
fn lasir_connection_to_ip(vm_a: usize, vm_b: usize, pasir_subnets: &Vec<pasir::connections::Subnet>) -> Option<(Ipv4Addr, Ipv4Addr)> {
    assert!(vm_a != vm_b, "Called lasir_connection_to_ip with the same VM for both ends of the connection: {}", vm_a);
    let mut ret: Option<(Ipv4Addr, Ipv4Addr)> = None;
    // let mut match_count = 0;
    for subnet in pasir_subnets {
        let (mut found_a, mut found_b): (Option<usize>, Option<usize>) = (None, None);
        for vm in subnet.connected_vms.keys() {
            if *vm == vm_a {
                if found_a.is_some() {
                    panic!("Found the vm with index {} twice in a subnet", vm);
                }
                found_a = Some(*vm);
            }
            else if *vm == vm_b {
                if found_b.is_some() {
                    panic!("Found the vm with index {} twice in a subnet", vm);
                }
                found_b = Some(*vm);
            }
        }
        if found_a.is_some() && found_b.is_some() {
            match ret {
                Some(_) => panic!("Two subnets have the same 2 VMs in them, with at least indices {}, {}", found_a.unwrap(), found_b.unwrap()),
                None => ret = Some( (subnet.connected_vms[&found_a.unwrap()], subnet.connected_vms[&found_b.unwrap()]) ),
            }
        }
    }
    ret
}

#[inline]
fn find_all_roles(vms: &Vec<Vm>) -> Vec<String> {

    // fn update<'a>(ret: &'a mut Vec<String>, stri: &String) -> &'a Vec<String> {
    //     if !ret.contains(&stri) {
    //         ret.push(stri.to_string());
    //     }
    //     ret
    // }

    let mut ret = Vec::new();

    for vm in vms {
        if !ret.contains(&vm.role) {
            ret.push(vm.role.to_string())
        }
    }
    ret
}

/// Looks at all connections of a VM, and returns all VMs that have a specific role.
#[inline]
fn find_all_connected_vms_with_role<'a, C: VmConnectionLogical>(source_vm: usize, role: &str, system: &'a LogicalSystem<C>) -> Vec<usize> {

    let all_connected_vms: Vec<usize> = system.network.all_connections_for_vm(source_vm);
    let all_relevant_vm_idxs: Vec<usize> = all_connected_vms.iter().map(|idx| (idx, system.vms.get(*idx).unwrap())).filter(|(_, vm)| vm.role == role).map(|(idx, _)| *idx).collect();
    all_relevant_vm_idxs
}