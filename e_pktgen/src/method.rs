pub type FnMethod = fn(conf: &yaml_rust::Yaml, count: i64) -> Vec<Vec<u8>>;
pub mod tcp;
pub mod udp;
