use std::sync::RwLock;
use std::net::TcpStream;
use std::io::prelude::*;
use std::io::Error;
use crate::core::memory::array::Array;
use crate::controller::cmd;
use crate::controller::table_controller;
use crate::pipeline::table::Table;


/**
 * DataPlane to ControlPlane
 */
pub fn stream_handler(mut stream: TcpStream, mut table_list: Array<RwLock<Table>>) -> Result<(), Error> {
    let mut request_buffer: [u8;1024] = [0; 1024];
    let mut response_buffer: [u8;1024] = [0; 1024];
    loop {
        let request_bytes = stream.read(&mut request_buffer)?;
        if request_bytes == 0 {
            return Ok(());
        }


        let cmd = request_buffer[0];
        let mut response_bytes = 0;
        if cmd == cmd::RequestCmd::Ping as u8 {
            response_buffer[0] = cmd::RequestCmd::Ping as u8;
            response_bytes = 1;
        } else if cmd == cmd::RequestCmd::AddFlowEntry as u8 {
            let table_id = request_buffer[1];
            let table = &mut table_list[table_id as usize];
            table_controller::add_flow_entry(table, &request_buffer[2..]);
            response_buffer[0] = cmd::ResponseCmd::SuccessMessage as u8;
            response_bytes = 1;
        } else if cmd == cmd::RequestCmd::ShowFlowEntry as u8 {
            let table_id = request_buffer[1];
            let table = &table_list[table_id as usize];
            let flow_entry_str = table_controller::show_flow_entry(&*table.read().unwrap());

        } else {
            response_buffer[0] = cmd::ResponseCmd::ErrorMessage as u8;
            response_bytes = 1;
        }


        stream.write(&response_buffer[..response_bytes])?;
        stream.flush()?;
    }
}
