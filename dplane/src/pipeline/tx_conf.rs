pub struct TxConf {
    pub output_port: usize,
    pub is_drop: bool,
    pub is_flooding: bool,
}

impl TxConf {
    pub fn init(&mut self) {
        self.output_port = 0;
        self.is_drop = false;
        self.is_flooding = false;
    }
}
