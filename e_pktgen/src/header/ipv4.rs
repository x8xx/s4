use modular_bitfield_msb::bitfield;
use modular_bitfield_msb::specifiers::*;

#[bitfield]
#[derive(Clone)]
pub struct IPv4 {
    pub version: B4,
    pub hdr_len: B4,
    pub tos: B8,
    pub len: B16,
    pub identification: B16,
    pub flags: B4,
    pub offset: B12,
    pub ttl: B8,
    pub protocol: B8,
    pub checksum: B16,
    pub src_address: B32,
    pub dst_address: B32,
}

pub fn calc_checksum(ipv4_bin: &[u8;20]) -> u16 {
    fn bin_to_u16(b1: u8, b2: u8) -> u16 {
        ((b1 as u16) << 8) + (b2 as u16)
    }

    fn add_u16(v1: u16, v2: u16) -> u16 {
        let sum: u32 = (v1 as u32) + (v2 as u32);
        if sum > 0xffff { ((sum & 0xffff) + 1) as u16 } else { sum as u16 }
    }

    let mut checksum: u16 = bin_to_u16(ipv4_bin[0], ipv4_bin[1]);
    checksum = add_u16(checksum, bin_to_u16(ipv4_bin[2], ipv4_bin[3]));
    checksum = add_u16(checksum, bin_to_u16(ipv4_bin[4], ipv4_bin[5]));
    checksum = add_u16(checksum, bin_to_u16(ipv4_bin[6], ipv4_bin[7]));
    checksum = add_u16(checksum, bin_to_u16(ipv4_bin[8], ipv4_bin[9]));
    checksum = add_u16(checksum, bin_to_u16(ipv4_bin[12], ipv4_bin[13]));
    checksum = add_u16(checksum, bin_to_u16(ipv4_bin[14], ipv4_bin[15]));
    checksum = add_u16(checksum, bin_to_u16(ipv4_bin[16], ipv4_bin[17]));
    checksum = add_u16(checksum, bin_to_u16(ipv4_bin[18], ipv4_bin[19]));
    checksum ^ 0xffff
}

pub fn ipv4_address_str_to_u32(ipv4_address: &str) -> u32 {
    let octets: Vec<&str> = ipv4_address.split('.').collect();
    let mut addr_u32: u32 = 0;
    for i in 0..octets.len() {
        addr_u32 += u32::from_str_radix(octets[i], 10).unwrap() << ((octets.len() - i - 1) * 8);
    }
    addr_u32
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ipv4_address_str_to_u32() {
        assert_eq!(ipv4_address_str_to_u32("240.10.0.1"), 4027187201);
        assert_eq!(ipv4_address_str_to_u32("1.1.1.1"), 16843009);
    }

    #[test]
    fn test_calc_checksum() {
        let mut ipv4 = IPv4::new();
        ipv4.set_version(4);
        ipv4.set_hdr_len(5);
        ipv4.set_tos(0);
        ipv4.set_len(0x16ce);
        ipv4.set_identification(0x654c);
        ipv4.set_flags(4);
        ipv4.set_offset(0);
        ipv4.set_ttl(1);
        ipv4.set_protocol(0x11);
        ipv4.set_checksum(0);
        ipv4.set_src_address(0xc0a8651f); 
        ipv4.set_dst_address(0xe000001f); 
        let ipv4_bin = ipv4.into_bytes();
        assert_eq!(calc_checksum(&ipv4_bin), 0xf7eb);
    }
}
