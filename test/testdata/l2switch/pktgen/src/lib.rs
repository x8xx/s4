#![no_main]
use rand::Rng;

#[no_mangle]
pub extern "C" fn pktgen(buf_list: *mut *mut u8) -> usize {
    let mut rng = rand::thread_rng();
    for i in 0..64 {
        unsafe {
            let buf = *buf_list.offset(i);
            *buf.offset(5) = rng.gen_range(0..48) as u8;
        }
    }
    64
}
