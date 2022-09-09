use modular_bitfield_msb::bitfield;
use modular_bitfield_msb::specifiers::*;

#[bitfield]
#[derive(Clone)]
pub struct TCP {
    pub src_port: B16,
    pub dst_port: B16,
    pub sequence: B32,
    pub ack: B32,
    pub hdr_len: B4,
    pub reserved: B6,
    pub flag_urg: bool,
    pub flag_ack: bool,
    pub flag_psh: bool,
    pub flag_rst: bool,
    pub flag_syn: bool,
    pub flag_fin: bool,
    pub window_size: B16,
    pub checksum: B16,
    pub pointer: B16
}

pub fn calc_checksum(ipv4_bin: &[u8], tcp_bin: &[u8], data_bin: &[u8]) -> u16 {
    fn bin_to_u16(b1: u8, b2: u8) -> u16 {
        ((b1 as u16) << 8) + (b2 as u16)
    }

    fn add_u16(v1: u16, v2: u16) -> u16 {
        let sum: u32 = (v1 as u32) + (v2 as u32);
        if sum > 0xffff { ((sum & 0xffff) + 1) as u16 } else { sum as u16 }
    }

    let mut checksum: u16 = bin_to_u16(ipv4_bin[0], ipv4_bin[1]);
    checksum = add_u16(checksum, bin_to_u16(ipv4_bin[12], ipv4_bin[13]));
    checksum = add_u16(checksum, bin_to_u16(ipv4_bin[14], ipv4_bin[15]));
    checksum = add_u16(checksum, bin_to_u16(ipv4_bin[16], ipv4_bin[17]));
    checksum = add_u16(checksum, bin_to_u16(ipv4_bin[18], ipv4_bin[19]));
    checksum = add_u16(checksum, bin_to_u16(0, ipv4_bin[9]));
    checksum = add_u16(checksum, bin_to_u16(ipv4_bin[2], ipv4_bin[3]));

    checksum = add_u16(checksum, bin_to_u16(tcp_bin[0], tcp_bin[1]));
    checksum = add_u16(checksum, bin_to_u16(tcp_bin[2], tcp_bin[3]));
    checksum = add_u16(checksum, bin_to_u16(tcp_bin[4], tcp_bin[5]));
    checksum = add_u16(checksum, bin_to_u16(tcp_bin[6], tcp_bin[7]));
    checksum = add_u16(checksum, bin_to_u16(tcp_bin[8], tcp_bin[9]));
    checksum = add_u16(checksum, bin_to_u16(tcp_bin[10], tcp_bin[11]));
    checksum = add_u16(checksum, bin_to_u16(tcp_bin[12], tcp_bin[13]));
    checksum = add_u16(checksum, bin_to_u16(tcp_bin[14], tcp_bin[15]));
    checksum = add_u16(checksum, bin_to_u16(tcp_bin[18], tcp_bin[19]));

    for i in 0..data_bin.len() {
        if (i % 2) != 0 {
            continue;
        }
        checksum = add_u16(checksum, bin_to_u16(tcp_bin[i], tcp_bin[i + 1]));
    }
    checksum ^ 0xffff
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_calc_checksum() {
    }
}
