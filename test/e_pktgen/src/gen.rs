// use std::collections::HashMap;
// use crate::device::Device;
// use crate::method::FnMethod;

// const GENERAL_CONFIG_KEY: &str = "General";
// const METHOD_CONFIG_KEY: &str = "Methods";

// pub fn gen(device: &mut Device, config: &yaml_rust::Yaml, methods: HashMap<&str, FnMethod>) {
//     let general_config = &config[GENERAL_CONFIG_KEY];
//     let packets = methods["tcp"](&config[METHOD_CONFIG_KEY]["tcp"], general_config["count"].as_i64().unwrap());
//     // println!("{}", config["General"]["count"].as_i64().unwrap());
//     // println!("{}", config["Environment"]["tcp"]["count"].as_i64().unwrap());
//     // println!("{}", config.len());
//     for packet in packets.iter() {
//         // println!("{:?}", packet);
//         device.send(packet);
//     }
// }

use std::ffi::c_void;
use std::mem::transmute;


#[repr(C)]
pub struct GenArgs {
}


pub extern "C" fn start_gen(gen_args_ptr: *mut c_void) -> i32 {
    println!("start gen thread");
    0
}
