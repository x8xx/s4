use getopts::Options;

pub struct SwitchConfig {
    pub rx_cores: u8,
    pub fib_cores: u8,
}


pub fn parse_switch_args(args: &[String]) -> SwitchConfig {
    let mut opts = Options::new();
    opts.optopt("r", "rx-cores", "number of rx cores to allocate", "");
    opts.optopt("f", "fib-cores", "number of fib cores to allocate", "");

    let matches = opts.parse(args).unwrap();
    let rx_cores: u8 = matches.opt_get::<u8>("r").unwrap().unwrap();
    let fib_cores: u8 = matches.opt_get::<u8>("f").unwrap().unwrap();

    SwitchConfig { rx_cores, fib_cores }
}
