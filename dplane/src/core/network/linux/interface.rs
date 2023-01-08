// use std::ffi::CStr;
// use std::ptr::null_mut;
// use std::os::raw::c_char;
use std::mem::size_of;
use crate::core::memory::array::Array;
use crate::core::network::pktbuf;
use pnet::datalink;
use pnet::datalink::NetworkInterface;
use pnet::datalink::DataLinkSender;
use pnet::datalink::DataLinkReceiver;
use pnet::datalink::Channel::Ethernet;


#[derive(Clone)]
pub struct Interface {
    pub port: u16,
    pub queue: u16,

    // interface: *const NetworkInterface,
    // tx: *const Box<dyn DataLinkSender>,
    // rx: *const Box<dyn DataLinkReceiver>,
}

impl Interface {
    pub fn init(name: &str, mut rx_queues: u16, mut tx_queues: u16) -> (u16, u16, u16) {
        (0, 0, 0)
    }

    // pub fn new(name: &str) -> Self {
    //     let interfaces = datalink::interfaces();
    //     let interface = interfaces.into_iter().filter(|interface: &NetworkInterface| interface.name == name).next().expect("Failed get Inteface");

    //     let (tx, rx) = match datalink::channel(&interface, Default::default()) {
    //         Ok(Ethernet(tx, rx)) => (tx, rx),
    //         Ok(_) => panic!("failed create channel"),
    //         Err(e) => panic!("{}", e),
    //     };

    //     let (interface_ptr, rx_ptr, tx_ptr) = unsafe {
    //         (
    //             libc::malloc(size_of::<NetworkInterface>()) as *mut NetworkInterface,
    //             libc::malloc(size_of::<Box<dyn DataLinkReceiver>>()) as *mut Box<dyn DataLinkReceiver>,
    //             libc::malloc(size_of::<Box<dyn DataLinkSender>>()) as *mut Box<dyn DataLinkSender>
    //         )
    //     };
        
    //     unsafe {
    //         *interface_ptr = interface;
    //         *rx_ptr = rx;
    //         *tx_ptr = tx;
    //     }

    //     Interface {
    //         interface: interface_ptr,
    //         rx: rx_ptr,
    //         tx: tx_ptr,
    //     }
    // }

    pub fn debug_show_info(&self) {
    }


    // TODO

    pub fn rx(&self, pktbuf: &Array<pktbuf::PktBuf>, len: usize) -> u16 {
        0
        // unsafe {
        //     // dpdk_sys::rte_eth_rx_burst(self.port_number, 0, pktbuf.as_ptr(),  pktbuf.len() as u16);
        //     dpdk_sys::rte_eth_rx_burst(
        //         self.port_number,
        //         0,
        //         pktbuf as *mut pktbuf::PktBuf as *mut *mut dpdk_sys::rte_mbuf,
        //         len as u16
        //     )
        // }
    }

    pub fn tx(&self, pktbuf: &Array<&mut pktbuf::RawPktBuf>, len: usize) -> u16 {
        0
        // unsafe {
        //     // dpdk_sys::rte_eth_tx_burst(self.port_number, 0, pktbuf.as_ptr(),  pktbuf.pkt_count as u16);
        //     dpdk_sys::rte_eth_tx_burst(
        //         self.port_number,
        //         0,
        //         pktbuf as *mut pktbuf::PktBuf as *mut *mut dpdk_sys::rte_mbuf,
        //         1
        //     );
        // }
    }
}
