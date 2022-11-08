use modular_bitfield_msb::bitfield;
use modular_bitfield_msb::specifiers::*;

#[bitfield]
#[derive(Clone)]
pub struct UDP {
    pub src_port: B16,
    pub dst_port: B16,
    pub len: B16,
    pub checksum: B16,
}
