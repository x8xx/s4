#![no_main]

/**
 * 0: ethernet, 1: IPv4, 2, TCP, 3, UDP
 */


extern {
    pub fn read_pkt(packet_id: i64, offset: u8) -> u8;
    pub fn extract_hdr(parse_result_id: i64, hdr_id: i64, offset: i32);
    pub fn set_hdr_len(parse_result_id: i64, hdr_len: usize);
}

pub struct Packet {
    id: i64,
    len: u8,
    parse_result_id: i64,
}


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
    let ethernet_size: u8 = 13;
    if pkt.len < ethernet_size {
        return false;
    }

    unsafe {
        extract_hdr(pkt.parse_result_id, hdr_id, 0);
        if read_pkt(pkt.id, 12) == 0x8 && read_pkt(pkt.id, 13) == 0 {
            return parse_ipv4(pkt, ethernet_size);
        }
        set_hdr_len(pkt.parse_result_id, ethernet_size as usize);
    };
    true
}

fn parse_ipv4(pkt: &Packet, offset: u8) -> bool {
    let hdr_id = 1;
    let ipv4_size: u8 = 20;
    if pkt.len < ipv4_size + offset {
        return false;
    }

    unsafe {
        extract_hdr(pkt.parse_result_id, hdr_id, offset as i32);
        return match read_pkt(pkt.id, 10 + offset) {
            6 => parse_tcp(pkt, 20 + offset),
            17 => parse_udp(pkt, 20 + offset),
            _ => {
                set_hdr_len(pkt.parse_result_id, (offset + ipv4_size) as usize);
                true
            },
        };
    }
}

fn parse_tcp(pkt: &Packet, offset: u8) -> bool {
    let hdr_id = 2;
    let tcp_size: u8 = 20;
    if pkt.len < tcp_size + offset {
        return false;
    }
    unsafe {
        extract_hdr(pkt.parse_result_id, hdr_id, offset as i32);
        set_hdr_len(pkt.parse_result_id, (offset + tcp_size) as usize);
    }
    true
}

fn parse_udp(pkt: &Packet, offset: u8) -> bool {
    let hdr_id = 3;
    let udp_size: u8 = 8;
    if pkt.len < udp_size + offset {
        return false;
    }
    unsafe {
        extract_hdr(pkt.parse_result_id, hdr_id, offset as i32);
        set_hdr_len(pkt.parse_result_id, (offset + udp_size) as usize);
    }
    true
}
