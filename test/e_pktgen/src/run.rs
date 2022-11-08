use std::fs;
use std::collections::HashMap;
use argh::FromArgs;
use yaml_rust::YamlLoader;
use crate::method::FnMethod;
use crate::device;
use crate::gen;

#[derive(FromArgs)]
/// 
struct Args {
    /// interface name
    #[argh(option, short = 'i')]
    interface: Option<String>,
    /// config yaml file path
    #[argh(option, short = 'c')]
    config: Option<String>
}



pub fn run(methods: HashMap<&str, FnMethod>) {
    let args: Args = argh::from_env();

    let interface_name = args.interface.unwrap();
    let mut device = device::init(interface_name);

    let config_path = args.config.unwrap();
    let config = YamlLoader::load_from_str(&fs::read_to_string(config_path).unwrap().to_string()).unwrap();

    gen::gen(&mut device, &config[0], methods);
}
