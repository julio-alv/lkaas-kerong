#![allow(unused)]
pub struct Command([u8; 5]);

/// Data head / frame head, fixed value
const STX: u8 = 0x02;
/// Data tail / frame tail fixed value
const ETX: u8 = 0x03;

const UNLOCK_ONE: u8 = 0x31;
const QUERY_ALL: u8 = 0x32;

/// Fixed Checksum for query all command
const QTX: u8 = 0x37;

impl Command {
    pub fn unlock_one(n: u8) -> Self {
        Command([STX, n, UNLOCK_ONE, ETX, n.saturating_add(0x36)])
    }
    pub fn query_all() -> Self {
        Command([STX, 0, QUERY_ALL, ETX, QTX])
    }
}

impl AsRef<[u8]> for Command {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
