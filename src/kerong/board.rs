#![allow(unused)]
use serial::{prelude::*, unix::TTYPort};

use super::{command::Command, status::Status};
use std::io::prelude::*;
use std::io::Error;

pub struct CU16 {
    port: TTYPort,
}

impl CU16 {
    pub fn initialize(path: &str) -> Result<Self, Error> {
        let mut port = serial::open(&path).unwrap();
        port.reconfigure(&|settings| {
            settings.set_baud_rate(serial::Baud19200)?;
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
        })?;

        Ok(CU16 { port })
    }
    pub fn open(&mut self, n: u8) -> std::io::Result<()> {
        self.port.write_all(Command::unlock_one(n).as_ref())
    }
    pub fn query_all(&mut self) -> Result<Status, Error> {
        let mut buffer = [0u8; 9];
        self.port.write_all(Command::query_all().as_ref())?;
        match self.port.read_exact(&mut buffer) {
            Ok(()) => Ok(Status::from(buffer)),
            Err(e) => Err(e),
        }
    }
}
