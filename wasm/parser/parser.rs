#![no_main]

extern {
    pub fn read(packet_id: i64, offset: u8) -> i32;
}

#[no_mangle]
pub fn parse(packet_id: i64, test: u8) -> i32 {
    // let bin = 10;
    let bin = unsafe { read(packet_id, 3) };
    test as i32 +  bin
}
