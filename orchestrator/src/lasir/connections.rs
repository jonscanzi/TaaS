use crate::utils::types::CidrIP;

/* @brief
 *
 * The interface for accessing the logical list of connections
 * of every VMs. Since there might be special cases where
 * the connections are not symmetric, i.e. vm1 -> vm2
 * does not imply vm2 -> vm1, it is left as possibility
 * offered by this interface, and the implementer is free
 * to allow this behaviour or not
 *
 * note: duplicate connections are ignored
 */

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct AbstractSubnet {
        pub address_prefix: Option<CidrIP>,
        pub connected_vm_idxs: Vec<usize>,
}

pub struct ConnectionProperties {
    pub speed_mbps: usize,
    pub latency_us: usize,
    pub drop_chance_percent: f32,
}

pub trait VmConnectionLogical {
    
    fn new(vm_num: usize) -> Self where Self: Sized; //small trick to allow using the trait as type
    fn all_connections_for_vm(&self, vm_idx: usize) -> Vec<usize>;
    fn connection_exists(&self, vm_in: usize, vm_out: usize) -> bool;
    fn add_sym_connection(&mut self, vm_a: usize, vm_b: usize);
    fn add_sym_connection_with_speed(&mut self, vm_a: usize, vm_b: usize, cp: ConnectionProperties);
    fn add_asym_connection(&mut self, vm_in: usize, vm_out: usize);
    fn add_asym_connection_with_speed(&mut self, vm_in: usize, vm_out: usize, cp: ConnectionProperties);
    fn remove_sym_connection(&mut self, _vm_a: usize, _vm_b: usize) {unimplemented!();}
    fn remove_asym_connection(&mut self, _vm_in: usize, _vm_out: usize) {unimplemented!();}
    fn prepare_asym_network(&mut self);
    // if any call to add_asym_connection_for_vm() or prepare_asym_network()
    // is made, this function should return false
    fn is_symmetric(&self) -> bool;
    fn vm_count(&self) -> usize;

    //error-handling connections, done here because it should stop
    //if it encounters them
    fn error_loopback(&self, vm_idx: usize) {
        panic!("error: vm number {} has a loopback connection", vm_idx);
    }
    fn error_no_asym_support(&self, impl_name: &str) {
        panic!("error: implementation {} does not support asymetric networks", impl_name);
    }
}

/* @brief
 *
 * Implementation of the logical connection trait
 * using rust's Vec to represent a compressed 2D matrix
 *
 */
#[derive(Debug)]
#[allow(dead_code)]
pub struct VmConnectionLogicalVec {
    connections: Vec<Vec<usize>>,
}

// ========================== V2 ========================

pub fn new_connection_vec(vm_num: usize) -> VmConnectionLogicalV2 {
        let mut ret: Vec<Vec<ConnectionTo>> = Vec::new();
        ret.resize(vm_num, Vec::new());
        VmConnectionLogicalV2{connections: ret}
}

// a connection between 2 vms is defined as: 
//  -a speed
//  -a latency
//  -a packet drop probability
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ConnectionTo {
    vm_dest: usize,
    speed_mbps: usize,
    latency_us: usize,
    drop_chance_percent: f32,
}

#[derive(Debug)]
pub struct VmConnectionLogicalV2 {
    connections: Vec<Vec<ConnectionTo>>,
}

macro_rules! create_connection_to {
    ($self: ident, $vm_from: expr, $vm_to: expr, $speed_mbps: expr, $latency_us: expr, $drop_chance_percent: expr) => {
        {
        //checking the connection does not already exist
        let vm_connections: Vec<usize> = $self.connections[$vm_from].iter().map(|x| x.vm_dest).collect(); 
        assert!(!(vm_connections.contains(&$vm_to)), format!("Error: trying to create two connections from vm {} to vm {}", $vm_from, $vm_to));
        ConnectionTo{vm_dest: $vm_to, speed_mbps: $speed_mbps, latency_us: $latency_us, drop_chance_percent: $drop_chance_percent}
        }
    }
}

const DEFAULT_CONNECTION_SPEED_MBPS: usize = 1000;
const DEFAULT_CONNECTION_LATENCY_US: usize = 0;
const DEFAULT_CONNECTION_DROP_CHANCE:f32 = 0.0f32;
impl VmConnectionLogical for VmConnectionLogicalV2 {
    fn new(vm_num: usize) -> Self {
        let mut ret: Vec<Vec<ConnectionTo>> = Vec::new();
        ret.resize(vm_num, Vec::new());
        VmConnectionLogicalV2{connections: ret}
    }

    fn all_connections_for_vm(&self, vm_idx: usize) -> Vec<usize> {
        self.connections[vm_idx].iter().map(|x| x.vm_dest).collect()
    }

    fn connection_exists(&self, vm_in: usize, vm_out: usize) -> bool {
        let vm_connections: Vec<usize> = self.connections[vm_in].iter().map(|x| x.vm_dest).collect();
        vm_connections.contains(&vm_out)
    }
    
    fn add_sym_connection(&mut self, vm_a: usize, vm_b: usize) {
       self.add_sym_connection_with_speed(vm_a, vm_b, ConnectionProperties{speed_mbps: DEFAULT_CONNECTION_SPEED_MBPS, latency_us: DEFAULT_CONNECTION_LATENCY_US, drop_chance_percent: DEFAULT_CONNECTION_DROP_CHANCE});
    }

    fn add_sym_connection_with_speed(&mut self, vm_a: usize, vm_b: usize, cp: ConnectionProperties) {
        let new_connection_a: ConnectionTo = create_connection_to!(self, vm_a, vm_b, cp.speed_mbps, cp.latency_us, cp.drop_chance_percent);
        self.connections[vm_a].push(new_connection_a);
        let new_connection_b: ConnectionTo = create_connection_to!(self, vm_b, vm_a, cp.speed_mbps, cp.latency_us, cp.drop_chance_percent);
        self.connections[vm_b].push(new_connection_b);
    }

    fn add_asym_connection(&mut self, _vm_in: usize, _vm_out: usize){unimplemented!()}
    fn add_asym_connection_with_speed(&mut self, _vm_in: usize, _vm_out: usize, _cp: ConnectionProperties){unimplemented!()}
    fn prepare_asym_network(&mut self){unimplemented!()}
    fn is_symmetric(&self) -> bool{
        true
    }
    fn vm_count(&self) -> usize{
        self.connections.len()
    }
}