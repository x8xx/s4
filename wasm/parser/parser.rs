#![no_main]

extern {
    pub fn pkt_read(packet_id: i64, offset: u8) -> u8;
}

struct Packet {
    id: i64,
    size: u8,
}

#[no_mangle]
pub fn parse(packet_id: i64, packet_size: i32) -> i32 {
    let packet = Packet {
        id: packet_id,
        size: packet_size as u8,
    };
    parse_ethernet(&packet)
}

fn parse_ethernet(packet: &Packet) -> i32 {
    let mut result = 0b1;
    let ethernet_size: u8 = 13;
    if packet.size < ethernet_size {
        return 0
    }

    unsafe {
        if pkt_read(packet.id, 12) == 0x8 && pkt_read(packet.id, 13) == 0 {
            result += parse_ipv4(packet, ethernet_size);
            result
        } else {
            0
        }
    }
}

fn parse_ipv4(packet: &Packet, base_offset: u8) -> i32 {
    let mut result = 0b10;
    let ipv4_size: u8 = 20;
    if packet.size < ipv4_size + base_offset {
        return 0
    }

    result += unsafe {
        match pkt_read(packet.id, 10 + base_offset) {
            6 => parse_tcp(packet, 20 + base_offset),
            17 => parse_tcp(packet, 20 + base_offset),
            _ => 0
        }
    };
    result
}

fn parse_tcp(packet: &Packet, base_offset: u8) -> i32 {
    let mut result = 0b100;
    let tcp_size: u8 = 20;
    if packet.size < tcp_size + base_offset {
        return 0
    }
    result
}

fn parse_udp(packet: &Packet, base_offset: u8) -> i32 {
    let mut result = 0b1000;
    let udp_size: u8 = 20;
    if packet.size < tcp_size + base_offset {
        return 0
    }
    result
}
