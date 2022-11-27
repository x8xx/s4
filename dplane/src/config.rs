use std::fs;
use std::io::Read;
// use std::collections::HashMap;
use getopts::Options;
use yaml_rust::YamlLoader;
use serde::Deserialize;

pub struct SwitchConfig {
    pub dataplane: DpConfig,

    pub parser_wasm: Vec<u8>,
    pub pipeline_wasm: Vec<u8>,

    pub listen_address: String,

    pub cache_core_num: u8,
    pub pipeline_core_num: u8,

    pub l1_cache_size: usize,
    pub l2_cache_size: usize,
    pub l3_cache_tuple_size: usize,

    pub interface_configs: Vec<InterfaceConfig>,

    pub initial_table_data: Vec<u8>,
}

pub struct InterfaceConfig {
    pub if_name: String,
    pub cache_core_num: u8,
}


/**
 * dataplan_config.json mapping
 */

#[derive(Deserialize)]
pub struct DpConfig {
    pub headers: Vec<DpConfigHeader>,
    pub header_max_size: usize,
    pub tables: Vec<DpConfigTable>,
}

#[derive(Deserialize)]
pub struct DpConfigHeader {
    pub fields: Vec<u16>,
    pub used_fields: Vec<u16>,
    pub parse_fields: Vec<u16>,
}

#[derive(Deserialize)]
pub struct DpConfigTable {
    pub keys: Vec<DpConfigTableKey>,
    pub default_action_id: u64,
    pub max_size: u64,
}

#[derive(Deserialize)]
pub struct DpConfigTableKey {
    pub match_kind: String,
    pub header_id: u64,
    pub field_id: u64,
}


const GENERAL_CONFIG_KEY: &str = "general";
const INTERFACES_CONFIG_KEY: &str = "interfaces";

pub fn parse_switch_args(args: &[String]) -> SwitchConfig {
    // getopts
    let mut opts = Options::new();
    opts.optopt("c", "config", "yaml switch config path", "");

    let matches = opts.parse(args).unwrap();
    let config_path: String = matches.opt_get::<String>("c").unwrap().unwrap();

    // yaml_rust load config
    let yaml_configs = YamlLoader::load_from_str(&fs::read_to_string(config_path).unwrap().to_string()).unwrap();
    let yaml_config = &yaml_configs[0];
    let yaml_config_general = &yaml_config[GENERAL_CONFIG_KEY];
    let yaml_config_interfaces = &yaml_config[INTERFACES_CONFIG_KEY];


    // dataplane config
    let dataplane_json_path = get_string_from_yaml_value(yaml_config_general, "dataplane_config_path");
    let dataplane_json = fs::read_to_string(dataplane_json_path).unwrap();
    let dataplane: DpConfig = serde_json::from_str(&dataplane_json).unwrap();


    // parser wasm
    let parser_wasm_path= get_string_from_yaml_value(yaml_config_general, "parser_wasm_path");
    let parser_wasm = {
        let mut f = fs::File::open(&parser_wasm_path).unwrap();
        let metadata = std::fs::metadata(&parser_wasm_path).unwrap();
        let mut parser_wasm = vec![0;metadata.len() as usize];
        f.read(&mut parser_wasm).unwrap();
        parser_wasm
    };

    // pipeline wasm
    let pipeline_wasm_path= get_string_from_yaml_value(yaml_config_general, "pipeline_wasm_path");
    let pipeline_wasm = {
        let mut f = fs::File::open(&pipeline_wasm_path).unwrap();
        let metadata = std::fs::metadata(&pipeline_wasm_path).unwrap();
        let mut pipeline_wasm = vec![0;metadata.len() as usize];
        f.read(&mut pipeline_wasm).unwrap();
        pipeline_wasm
    };

    // initial table data
    let initial_table_data_path = get_string_from_yaml_value(yaml_config_general, "initial_table_data_path");
    let initial_table_data = {
        let mut f = fs::File::open(&initial_table_data_path).unwrap();
        let metadata = std::fs::metadata(&initial_table_data_path).unwrap();
        let mut initial_table_data  = vec![0;metadata.len() as usize];
        f.read(&mut initial_table_data).unwrap();
        initial_table_data
    };


    let listen_address = get_string_from_yaml_value(yaml_config_general, "listen_address");
    let cache_core_num = yaml_config_general["cache_core_num"].clone().as_i64().unwrap() as u8;
    let pipeline_core_num = yaml_config_general["pipeline_core_num"].clone().as_i64().unwrap() as u8;
    let l1_cache_size = yaml_config_general["l1_cache_size"].clone().as_i64().unwrap() as usize;
    let l2_cache_size = yaml_config_general["l2_cache_size"].clone().as_i64().unwrap() as usize;
    let l3_cache_tuple_size = yaml_config_general["l3_cache_tuple_size"].clone().as_i64().unwrap() as usize;


    let mut interface_configs = Vec::new();
    for yaml_config_interface in yaml_config_interfaces.as_hash().unwrap() {
        interface_configs.push(InterfaceConfig{
            if_name: yaml_config_interface.0.clone().into_string().unwrap(),
            cache_core_num: yaml_config_interface.1.clone()["cache_core_num"].as_i64().unwrap() as u8,
        });
    }

    SwitchConfig {
        dataplane,
        parser_wasm,
        pipeline_wasm,
        listen_address,
        cache_core_num,
        pipeline_core_num,
        l1_cache_size,
        l2_cache_size,
        l3_cache_tuple_size,
        interface_configs,
        initial_table_data,
    }
}


fn get_string_from_yaml_value(yaml: &yaml_rust::Yaml, key: &str) -> String {
    return yaml[key].clone().into_string().unwrap();
}
