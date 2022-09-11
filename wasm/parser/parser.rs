#![no_main]

extern {
    pub fn pkt_read(packet_id: i64, offset: u8) -> u8;
    pub fn extract_header(parse_id: i64, hdr_id: i64, base_offset: i64);
}

pub struct Packet {
    id: i64,
    parse_id: i64,
    size: u8,
}

#[no_mangle]
pub fn parse(packet_id: i64, packet_size: i32, parse_id: i64) -> u8 {
    let packet = Packet {
        id: packet_id,
        parse_id,
        size: packet_size as u8,
    };
    parse_ethernet(&packet)
}

fn parse_ethernet(packet: &Packet) -> u8 {
    let hdr_id = 0;
    let ethernet_size: u8 = 13;
    if packet.size < ethernet_size {
        return 0;
    }

    unsafe {
        extract_header(packet.parse_id, hdr_id, 0);
        if pkt_read(packet.id, 12) == 0x8 && pkt_read(packet.id, 13) == 0 {
            return parse_ipv4(packet, ethernet_size);
        }
    };
    ethernet_size
}

fn parse_ipv4(packet: &Packet, base_offset: u8) -> u8 {
    let hdr_id = 1;
    let ipv4_size: u8 = 20;
    if packet.size < ipv4_size + base_offset {
        return base_offset;
    }

    unsafe {
        extract_header(packet.parse_id, hdr_id, base_offset as i64);
        match pkt_read(packet.id, 10 + base_offset) {
            6 => parse_tcp(packet, 20 + base_offset),
            17 => parse_udp(packet, 20 + base_offset),
            _ => base_offset + ipv4_size,
        }
    }
}

fn parse_tcp(packet: &Packet, base_offset: u8) -> u8 {
    let hdr_id = 2;
    let tcp_size: u8 = 20;
    if packet.size < tcp_size + base_offset {
        return base_offset;
    }
    unsafe { extract_header(packet.parse_id, hdr_id, base_offset as i64); }
    base_offset + tcp_size
}

fn parse_udp(packet: &Packet, base_offset: u8) -> u8 {
    let hdr_id = 3;
    let udp_size: u8 = 8;
    if packet.size < udp_size + base_offset {
        return base_offset - 1;
    }
    unsafe { extract_header(packet.parse_id, hdr_id, base_offset as i64); }
    base_offset + udp_size
}
