use modular_bitfield_msb::bitfield;
use modular_bitfield_msb::specifiers::*;

#[bitfield]
#[derive(Clone)]
pub struct Ethernet {
    pub dst_address: B48,
    pub src_address: B48,
    pub ether_type: B16
}

pub fn mac_address_str_to_u64(mac_address: &str) -> u64 {
    let octets: Vec<&str> = mac_address.split(':').collect();
    let mut addr_u64: u64 = 0;
    for i in 0..octets.len() {
        addr_u64 += u64::from_str_radix(octets[i], 16).unwrap() << ((octets.len() - i - 1) * 8);
    }
    addr_u64 
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mac_address_str_to_u64() {
        assert_eq!(mac_address_str_to_u64("ff:ff:ff:ff:ff:ff"), 281474976710655);
        assert_eq!(mac_address_str_to_u64("01:02:03:04:05:06"), 1108152157446);
    }
}
