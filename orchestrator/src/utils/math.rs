use std::mem::transmute;
use std::mem::size_of;

/**
 * @brief
 * returns the ceiling of the log2 of an integer
 * implemented only for i32 as it is easier to work with
 * but it shouldn't be a problem in most cases
 *
 */
pub fn log2_ceil(x: i32) -> usize {
    
    const fn num_bits<T>() -> usize {
            size_of::<T>() * 8
    }

    assert!(x > 0);
    let ret = (num_bits::<i32>() as u32 - x.leading_zeros()) as usize;
    ret
}

// ===== BYTEORDER_CONVENIENCE ======
#[inline]
#[allow(dead_code)]
pub fn be_arr_as_native_u32(arr: [u8; 4]) -> u32 {
    let be_num = unsafe{ transmute::<[u8; 4], u32>(arr) };
    be_num
}

#[inline]
pub fn be_arr_as_be_u32(arr: [u8; 4]) -> u32 {
    unsafe{ transmute::<[u8; 4], u32>(arr) }.to_be()
}

#[inline]
pub fn u32_as_be_arr(num: u32) -> [u8; 4] {
    unsafe{ transmute::<u32, [u8; 4]>(num.to_be()) }
}
// ==================================

// ============ BIT OPS =============
const RIGHTMOST_ONE_32: u32 = 0b1;
#[inline]
pub fn generate_right_bitmask(num: u32) -> u32 {

    let mut ret: u32 = 0;
    for _ in 0..num {
        ret = ret << 1;
        ret = ret | RIGHTMOST_ONE_32;
    }
    ret
}

#[inline]
pub fn increment_at_bit_index(num: u32, index: u32) -> u32 {
    let bitmask = 0b1 << index;
    num + bitmask
}
// ==================================