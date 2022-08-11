use getopts::Options;

pub struct SwitchConfig {
    pub if_name: String,
    pub rx_cores: u8,
    pub fib_cores: u8,
}


pub fn parse_switch_args(args: &[String]) -> SwitchConfig {
    let mut opts = Options::new();
    opts.optopt("i", "interface", "used interface name", "");
    opts.optopt("r", "rx-cores", "number of rx cores to allocate", "");
    opts.optopt("f", "fib-cores", "number of fib cores to allocate", "");

    let matches = opts.parse(args).unwrap();
    let if_name: String = matches.opt_get::<String>("i").unwrap().unwrap();
    let rx_cores: u8 = matches.opt_get::<u8>("r").unwrap().unwrap();
    let fib_cores: u8 = matches.opt_get::<u8>("f").unwrap().unwrap();

    SwitchConfig { if_name, rx_cores, fib_cores }
}
