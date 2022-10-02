use std::fs;
use getopts::Options;
use yaml_rust::YamlLoader;

pub struct SwitchConfig {
    pub parser_path: String,
    pub listen_address: String,
    pub pipeline_core_num: u8,
    pub interface_configs: Vec<InterfaceConfig>,
}

pub struct InterfaceConfig {
    pub if_name: String,
    pub cache_core_num: u8,
}

const GENERAL_CONFIG_KEY: &str = "general";
const INTERFACES_CONFIG_KEY: &str = "interfaces";

pub fn parse_switch_args(args: &[String]) -> SwitchConfig {
    let mut opts = Options::new();
    opts.optopt("c", "config", "yaml switch config path", "");

    let matches = opts.parse(args).unwrap();
    let config_path: String = matches.opt_get::<String>("p").unwrap().unwrap();

    let yaml_config = YamlLoader::load_from_str(&fs::read_to_string(config_path).unwrap().to_string()).unwrap()[0];
    let yaml_general = yaml_config[GENERAL_CONFIG_KEY];
    let yaml_interfaces = yaml_config[INTERFACES_CONFIG_KEY];

    let parser_path = yaml_general["parser_path"].into_string().unwrap();
    let listen_address = yaml_general["listen_address"].into_string().unwrap();
    let pipeline_core_num = yaml_general["pipeline_core_num"].as_i64().unwrap() as u8;

    
    let interface_configs = Vec::new();
    for yaml_interface in yaml_interfaces.as_hash().unwrap() {
        interface_configs.push(InterfaceConfig{
            if_name: yaml_interface.0.into_string().unwrap(),
            cache_core_num: yaml_interface.1["cache_core_num"].as_i64().unwrap() as u8,
        });
        
    }

    SwitchConfig { parser_path, listen_address, pipeline_core_num, interface_configs }
}
