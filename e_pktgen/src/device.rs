use pnet::datalink;
use pnet::datalink::NetworkInterface;
use pnet::datalink::DataLinkSender;
use pnet::datalink::DataLinkReceiver;
use pnet::datalink::Channel::Ethernet;

pub struct Device {
    interface: NetworkInterface,
    tx: Box<dyn DataLinkSender>,
    rx: Box<dyn DataLinkReceiver>,
}

pub fn init(interface_name: String) -> Device {
    let interfaces = datalink::interfaces();
    let interface = interfaces.into_iter().filter(|interface: &NetworkInterface| interface.name == interface_name).next().expect("Failed get Inteface");

    let (tx, rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("failed create channel"),
        Err(e) => panic!("{}", e),
    };

    let device = Device{
        interface,
        tx,
        rx,
    };
    device
}

impl Device {
    pub fn send(&mut self, packet: &[u8]) {
        println!("Inteface:{}", self.interface.name);
        self.tx.send_to(packet, None);
    }
}
