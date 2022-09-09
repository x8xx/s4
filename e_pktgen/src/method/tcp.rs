use crate::header::*;
use rand::Rng;
use rand::seq::SliceRandom;

fn get_ip_port_list(conf: &yaml_rust::Yaml, count: i64, is_src: bool) -> Vec<(u32, u16)> {
    let mut rng = rand::thread_rng();
    let mut ip_list = Vec::new();

    let unit_count = count as i64 / conf["weight"].as_i64().unwrap();
    let ipv4_range_list = conf["ipv4"].as_vec().unwrap();
    for range in ipv4_range_list {
        for _ in 0..(unit_count*range["weight"].as_i64().unwrap()) {
            let ip = gen_ipv4_random_host_addr(range["range"].as_str().unwrap());
            let port = if is_src { rng.gen_range(32768..61000) } else { range["port"].as_i64().unwrap() as u16 };
            ip_list.push((ip, port));
        }
    }
    ip_list.shuffle(&mut rng);
    ip_list
}

fn gen_ipv4_random_host_addr(addr: &str) -> u32 {
    let mut rng = rand::thread_rng();
    let addr_vec: Vec<&str> = addr.split('/').collect();

    let network_addr: u32 = ipv4::ipv4_address_str_to_u32(addr_vec[0]);
    let prefix = u32::from_str_radix(addr_vec[1], 10).unwrap();
    if  prefix == 32 {
        return network_addr;
    }
    let host_addr: u32 = rng.gen_range(1..(2_u32.pow(32 - prefix)));

    network_addr + host_addr
}

pub fn gen_tcp_packet(conf: &yaml_rust::Yaml, count: i64) -> Vec<Vec<u8>> {
    let mut packets = Vec::new();

    let mut ethernet = ethernet::Ethernet::new();
    let ethernet_dst_addr = ethernet::mac_address_str_to_u64(conf["dst"]["ethernet"].as_str().unwrap());
    let ethernet_src_addr = ethernet::mac_address_str_to_u64(conf["src"]["ethernet"].as_str().unwrap());
    ethernet.set_dst_address(ethernet_dst_addr);
    ethernet.set_src_address(ethernet_src_addr);
    ethernet.set_ether_type(0x800);

    let ipv4_src_list = get_ip_port_list(&conf["src"], count, true);
    let ipv4_dst_list = get_ip_port_list(&conf["dst"], count, false);

    for i in 0..count {
        // create ip header
        let mut ipv4 = ipv4::IPv4::new();
        ipv4.set_version(4);
        ipv4.set_hdr_len(5);
        ipv4.set_tos(0);
        ipv4.set_len(48);
        ipv4.set_identification(0);
        ipv4.set_flags(0);
        ipv4.set_offset(0);
        ipv4.set_ttl(15);
        ipv4.set_protocol(6);
        ipv4.set_src_address(ipv4_src_list[i as usize].0);
        ipv4.set_dst_address(ipv4_dst_list[i as usize].0);
        let ipv4_bin = ipv4.into_bytes();
        let checksum = ipv4::calc_checksum(&ipv4_bin);
        ipv4 = ipv4::IPv4::from_bytes(ipv4_bin);
        ipv4.set_checksum(checksum);

        // tcp
        let mut tcp = tcp::TCP::new();
        tcp.set_src_port(ipv4_src_list[i as usize].1);
        tcp.set_dst_port(ipv4_dst_list[i as usize].1);


        // println!("{:048b},{:048b},{:016b},{:08b},{:032b},{:032b},{:016b},{:016b}",
        //          ethernet.dst_address(),
        //          ethernet.src_address(),
        //          ethernet.ether_type(),
        //          ipv4.protocol(),
        //          ipv4.src_address(),
        //          ipv4.dst_address(),
        //          tcp.src_port(),
        //          tcp.dst_port(),
        //          );
        

        let eth_bin = ethernet.clone().into_bytes();
        let ipv4_bin = ipv4.into_bytes();
        let tcp_bin = tcp.into_bytes();
        let mut packet: Vec<u8> = Vec::new();
        for bin in eth_bin.iter() {
            packet.push(*bin);
        }
        for bin in ipv4_bin.iter() {
            packet.push(*bin);
        }
        for bin in tcp_bin.iter() {
            packet.push(*bin);
        }

        packets.push(packet);
    }
    packets
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn is_correct_ip_addr() {
        assert_eq!(gen_ipv4_random_host_addr("240.10.0.1/32"), 4027187201);
        for _ in 0..1000 {
            let addr1 = gen_ipv4_random_host_addr("192.168.0.0/23");
            assert!((addr1 < 3232235520 + 512) && (addr1 > 3232235520));
        }
    }
}
