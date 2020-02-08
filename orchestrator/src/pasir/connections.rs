use std::collections::HashSet;
use std::iter::FromIterator;
use crate::lasir::connections::*;
use crate::utils::math::log2_ceil;
use crate::utils::math::be_arr_as_be_u32;
use crate::utils::math::u32_as_be_arr;
use crate::utils::math::generate_right_bitmask;
use crate::utils::math::increment_at_bit_index;

#[derive(Debug)]
pub struct SubnetCandidate {
    connected_vms: HashSet<usize>,
}

/**
 * @brief
 * looks at all curent subnet candidates, and returns whether the input vm
 * should be added to one of them
 *
 **/
#[inline]
fn extend_subnet(vm_idx: usize, current_subnets: &mut Vec<SubnetCandidate>, all_neighbours: &HashSet<usize>) -> Option<usize> {
 
    // check all candidate subnets if they should add the current vm to them 
    // this is the case if all members of the subnets are neighbours of the current vm
    // if not, check the next, until one good candidate is found
    // if not, return None and that means the current vm is taken care of

    let sze: usize = current_subnets.len();
    for i in 0..sze {
        if !current_subnets[i].connected_vms.contains(&vm_idx) && current_subnets[i].connected_vms.is_subset(&all_neighbours) {
            assert!(current_subnets[i].connected_vms.len() <= all_neighbours.len(),
                    "Network generation error in pick_subnet: it shouldn't be possible to have 2 subnets with all the same VMs.");

            current_subnets[i].connected_vms.insert(vm_idx);
            return Some(i);
        }
    }
    None
}

/**
 * @brief
 * checks that all neighbours of the current vm share a subnet with it
 * 
 * returns any missing neighbour
 **/
#[inline]
fn check_missing_neighbours(vm_idx: usize, current_subnets: &Vec<SubnetCandidate>, all_neighbours: &HashSet<usize>) -> HashSet<usize> {
    let mut visited_neighbours: HashSet<usize> = HashSet::new();
    for sn_vms in current_subnets {
        let neighbours = &sn_vms.connected_vms;
        if neighbours.contains(&vm_idx) {
            for neighbour in neighbours {
                visited_neighbours.insert(*neighbour);
            }
        }
    }
    let missing_neighbours: HashSet<usize> = all_neighbours.difference(&visited_neighbours).cloned().collect();
    missing_neighbours
}

/**
 * @brief
 * for a given index in the list of current subnets, check if any
 * subnets in the list is a sub-subnet of the indexed subnet
 * then, remove all of these sub-subnets from the list
 *
 * optimisation potential: sort indices in reverse order as Vec::remove is not O(1), though it might
 * mess with caching
 * optimisation potential: replace VEC with HashSet and fast hashing function
 **/
fn remove_subsubnets(subnet_idx: usize, current_subnets: &mut Vec<SubnetCandidate>) {
    let mut indices_to_remove: Vec<usize> = Vec::new();

    //have to do it this way since with current Rust, Vec only supports removing according to index and not items
    let sze = current_subnets.len();
    for i in 0..sze {
        // Not removing the subnet itself
        if subnet_idx != i && current_subnets[i].connected_vms.is_subset(&current_subnets[subnet_idx].connected_vms) {
            indices_to_remove.push(i);
        }
    }
    indices_to_remove.sort_by(|a, b| b.partial_cmp(a).unwrap());
    for i in indices_to_remove {
        current_subnets.remove(i);
    }
}

pub fn create_network_from_logical(connections: &dyn VmConnectionLogical) -> Vec<SubnetCandidate> {
    
    let mut debug_first_iter =  true;

    if connections.is_symmetric() == false {
        panic!("Error: trying to create an asymetric network, but create_network_from_logical is meant for symmetrical ones.");
    }

    let mut current_subnets: Vec<SubnetCandidate> = Vec::new();
    for vm_idx in 0..connections.vm_count() {
        //doesn't seem to be possible to convert directly to hashset
        let all_neighbours: HashSet<usize> = HashSet::from_iter(connections.all_connections_for_vm(vm_idx).to_vec());
    
        //flow:
        //
        //for every vm:
        //  go through all subnets one at a time, try to extend the subnets with current vm
        //      as soon as a subnet is extended: 
        //          check if doing this created subset of subnets; delete them from the subnet list
        //          repeat the extending operation with the updated subnet list
        //      if no subnet was touched:
        //          the current vm should be taken care of, so we continue
        //  check current network, look for neighbours that don't share a subnet of the current vm
        //  add all missing neigbours as 2-vm subnets
        //  (repeat for next vm)
        //
        //should be done now, but TODO: review algorithm on paper
       
        //how to do a do... while() in rust
        while {
            let extend_result = extend_subnet(vm_idx, &mut current_subnets, &all_neighbours);
            if extend_result.is_some() {
                assert!(!debug_first_iter,"subnet generation should not remove subnets in first iteration");
                remove_subsubnets(extend_result.unwrap(), &mut current_subnets);
            }
            extend_result.is_some()
        }{}
        
        debug_first_iter = false;
        let unreached_neighbours = check_missing_neighbours(vm_idx, &current_subnets, &all_neighbours);

        for new_neighbour in &unreached_neighbours {
            current_subnets.push(SubnetCandidate {connected_vms: {let mut ret = HashSet::new(); ret.insert(vm_idx); ret.insert(*new_neighbour); ret}});
        }
    }
    current_subnets
}

use crate::utils::types::CidrIP;
use std::net::Ipv4Addr;
use std::collections::HashMap;
/**
 * @brief
 * for a subnet, containing a cidr address (ipv4+netmask)
 * and a list of all its connected vms with their
 * assigned IP addresses
 **/
#[derive(Debug, Clone)]
pub struct Subnet {
    pub prefix: CidrIP,
    //connected_vms: Vec<(usize, Ipv4Addr)>,
    pub connected_vms: HashMap<usize, Ipv4Addr>,
}

/**
 *
 * @brief
 * structure that holds an IPv4 address that is meant to be incremented as the programmer wants. It can
 * interpret IPv4 addresses as either u32 or 4 bytes (u8), and should work on both big and little
 * endian systems
 **/
#[derive(Debug)]
pub struct Ipv4Counter {
    curr_addr: u32, 
}

#[allow(non_snake_case)]
fn Ipv4Counter_from_ip_order(address: [u8; 4]) -> Ipv4Counter {
    let u = be_arr_as_be_u32(address);
    Ipv4Counter{curr_addr: u }
}
impl Ipv4Counter {
    #[allow(non_snake_case)]    
    fn to_Ipv4Addr(&self) -> Ipv4Addr {
        let ip_arr = u32_as_be_arr(self.curr_addr);
        Ipv4Addr::new(ip_arr[0], ip_arr[1], ip_arr[2], ip_arr[3])
    }
    
    #[inline]
    #[allow(dead_code)]
    fn as_u32(&self) -> u32 {
        self.curr_addr
    }
    
    #[inline]
    #[allow(dead_code)]
    fn network_ordering(&self) -> u32 {
        self.curr_addr.to_be()
    }
    
    //it should work, I promise
    fn trailing_zeros(&self) -> u32 {
        self.curr_addr.trailing_zeros()
    }

    #[allow(dead_code)]
    fn minimum_netmask(&self) -> u32 {
        32 - self.trailing_zeros()
    }
    
    //puts at least num_bits '0' on the right and increment the higher bit
    fn make_room_for(&mut self, num_addr: u32) {
        let log_bits = log2_ceil(num_addr as i32) as u32;
        if self.curr_addr.trailing_zeros() < log_bits {
            let bitmask = !generate_right_bitmask(log_bits);
            let masked_addr = self.curr_addr & bitmask;
            let new_addr = increment_at_bit_index(masked_addr, log_bits+1);
            self.curr_addr = new_addr;
        }
    }

    #[allow(dead_code)]
    fn byte_at(&self, idx: usize) -> u8 {
        u32_as_be_arr(self.curr_addr)[idx]
    }
}

impl std::ops::Add<u32> for Ipv4Counter {
    type Output = Self;

    fn add(self, num: u32) -> Self::Output {
        
        Self {
            curr_addr: self.curr_addr + num,
        }
    }
}

//not using the x lowest and x highest ip addresses of the subnet according to this
const IP_ADDRESSES_RESERVED: u32 = 8;
/**
 * @brief
 *
 * this function expects a correct set of subnets (does not do any checks)
 * and build actual subnets with correct masks
 **/

// it is actually much more complicated, we need, for every subnet that we consider, to find the
// first number in the address space (considering an IPv4 address as a u32) that has enough
// trailing zeroes in order to accomodate the whole subnet
//
// this can be done e.g. by incrementing the bit at pos ceil(log2(size)), and then masking the rest
pub fn assign_subnets_and_ip_v2(logical_subnets: &Vec<SubnetCandidate>) -> Vec<Subnet> {
    
    let mut final_subnets: Vec<Subnet> = Vec::new();
    let mut ip_counter = Ipv4Counter_from_ip_order([10,1,0,0]);
    
    for l_subnet in logical_subnets {
        //look for enough trailing zeroes
        let subnet_size: u32 = l_subnet.connected_vms.len() as u32 + (IP_ADDRESSES_RESERVED * 2);
        ip_counter.make_room_for(subnet_size);
        let orig_ip: Ipv4Addr = ip_counter.to_Ipv4Addr();
        ip_counter = ip_counter + IP_ADDRESSES_RESERVED;
        
        let mut connected_vms: HashMap<usize, Ipv4Addr> = HashMap::new();
        for vm in &l_subnet.connected_vms {
            connected_vms.insert(*vm, ip_counter.to_Ipv4Addr());
            ip_counter = ip_counter + 1;
        }
        ip_counter = ip_counter + IP_ADDRESSES_RESERVED;

        let netmask = 32 - log2_ceil(subnet_size as i32) as u8;
        final_subnets.push(Subnet{prefix: CidrIP{ip: orig_ip, netmask: netmask}, connected_vms: connected_vms});
    }
    final_subnets
}