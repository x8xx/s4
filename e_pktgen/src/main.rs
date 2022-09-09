mod run;
mod method;
mod header;
mod device;
mod gen;
use std::collections::HashMap;
use crate::method::FnMethod;

fn main() {
    let mut methods: HashMap<&str, FnMethod> = HashMap::new();
    methods.insert("tcp", method::tcp::gen_tcp_packet);
    run::run(methods);
}
