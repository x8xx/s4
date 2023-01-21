use crate::dpdk;


pub fn main(interface: String, args: Option<Vec<String>>) {
    let (port_number, max_rx_queues, max_tx_queues) = dpdk::interface::Interface::init(&interface);
    // let max_tx_queues = 14;
    println!("Port{}: max_rx_queues {}, max_tx_queues {}", port_number, max_rx_queues, max_tx_queues);


}
