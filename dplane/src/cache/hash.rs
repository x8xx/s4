use std::mem::transmute;
// use crate::core::memory::array::Array;
// use crate::parser::header;
// use crate::parser::parse_result;


/**
 * murmurhash3
 * reference: https://github.com/mhallin/murmurhash3-rs
 */

static MURMURHASH3_C1: u32 = 0xcc9e2d51u32;
static MURMURHASH3_C2: u32 = 0x1b873593u32;

pub fn l1_hash_function_murmurhash3(pkt: *const u8, hdr_len: usize, seed: u32) -> u16 {
    let pkt_u32 = unsafe { transmute::<*const u8, *const u32>(pkt) };

    let mut h1 = seed;
    let block_len = hdr_len / 4;
    for i in 0..block_len {
        let mut k1 = unsafe { *pkt_u32.offset(i as isize) };

        k1 = k1.wrapping_mul(MURMURHASH3_C1);
        k1 = k1.rotate_left(15);
        k1 = k1.wrapping_mul(MURMURHASH3_C2);

        h1 ^= k1;
        h1 = h1.rotate_left(13);
        h1 = h1.wrapping_mul(5);
        h1 = h1.wrapping_add(0xe6546b64)
    }

    let mut k1 = 0u32;
    if hdr_len & 3 == 3 { k1 ^= (unsafe { *pkt.offset((block_len * 4) as isize + 2) } as u32) << 16; }
    if hdr_len & 3 == 2 { k1 ^= (unsafe { *pkt.offset((block_len * 4) as isize + 1) } as u32) << 8; }
    if hdr_len & 3 == 1 {
        k1 ^= unsafe { *pkt.offset((block_len * 4) as isize) } as u32;
        k1 = k1.wrapping_mul(MURMURHASH3_C1);
        k1 = k1.rotate_left(15);
        k1 = k1.wrapping_mul(MURMURHASH3_C2);
        h1 ^= k1;
    }

    h1 ^= hdr_len as u32;

    h1 ^= h1 >> 16;
    h1 = h1.wrapping_mul(0x85ebca6b);
    h1 ^= h1 >> 13;
    h1 = h1.wrapping_mul(0xc2b2ae35);
    h1 ^= h1 >> 16;

    h1 << 16;
    h1 as u16
}


pub fn l2_hash_function_murmurhash3(pkt: *const u8, key_len: usize, seed: u32) -> u16 {
    let pkt_u32 = unsafe { transmute::<*const u8, *const u32>(pkt) };

    let mut h1 = seed;
    let block_len = key_len / 4;
    for i in 0..block_len {
        let mut k1 = unsafe { *pkt_u32.offset(i as isize) };

        k1 = k1.wrapping_mul(MURMURHASH3_C1);
        k1 = k1.rotate_left(15);
        k1 = k1.wrapping_mul(MURMURHASH3_C2);

        h1 ^= k1;
        h1 = h1.rotate_left(13);
        h1 = h1.wrapping_mul(5);
        h1 = h1.wrapping_add(0xe6546b64)
    }

    let mut k1 = 0u32;
    if key_len & 3 == 3 { k1 ^= (unsafe { *pkt.offset((block_len * 4) as isize + 2) } as u32) << 16; }
    if key_len & 3 == 2 { k1 ^= (unsafe { *pkt.offset((block_len * 4) as isize + 1) } as u32) << 8; }
    if key_len & 3 == 1 {
        k1 ^= unsafe { *pkt.offset((block_len * 4) as isize) } as u32;
        k1 = k1.wrapping_mul(MURMURHASH3_C1);
        k1 = k1.rotate_left(15);
        k1 = k1.wrapping_mul(MURMURHASH3_C2);
        h1 ^= k1;
    }

    h1 ^= key_len as u32;

    h1 ^= h1 >> 16;
    h1 = h1.wrapping_mul(0x85ebca6b);
    h1 ^= h1 >> 13;
    h1 = h1.wrapping_mul(0xc2b2ae35);
    h1 ^= h1 >> 16;

    h1 << 16;
    h1 as u16
}


pub fn l3_hash_function_murmurhash3(pkt: *const u8, key_len: usize, seed: u32) -> u16 {
    l2_hash_function_murmurhash3(pkt, key_len, seed)
}
