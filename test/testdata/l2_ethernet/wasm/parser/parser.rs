#![no_main]
mod libparser;
use libparser::*;


#[no_mangle]
pub fn parse(pkt_id: i64, pkt_len: i32, parse_result_id: i64) -> bool {
    let packet = Packet {
        id: pkt_id,
        len: pkt_len as u8,
        parse_result_id,
    };
    parse_ethernet(&packet)
}

fn parse_ethernet(pkt: &Packet) -> bool {
    let hdr_id = 0;
    let ethernet_size = 13;
    if pkt.len < ethernet_size {
        return false;
    }

    unsafe {
        extract_hdr(pkt.parse_result_id, hdr_id, 0, ethernet_size as i32);
    };
    true
}
