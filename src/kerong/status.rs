use std::fmt::{Debug, Display};

#[derive(PartialEq, Debug, Clone)]
pub struct Locker {
    number: u8,
    closed: bool,
    loaded: bool,
}

#[derive(PartialEq, Default, Clone)]
pub struct Status {
    lockers: Vec<Locker>,
}

impl Status {
    pub fn new() -> Self {
        Status { lockers: vec![] }
    }
}

impl From<[u8; 9]> for Status {
    fn from(value: [u8; 9]) -> Self {
        const LATCH_INDEX: usize = 3;
        const INFRARED_INDEX: usize = 5;
        let latches = u16::from_le_bytes([value[LATCH_INDEX], value[LATCH_INDEX + 1]]);
        let infrareds = u16::from_le_bytes([value[INFRARED_INDEX], value[INFRARED_INDEX + 1]]);
        let mut lockers: Vec<Locker> = vec![];

        for i in 0..16 {
            let latch = (latches >> i) & 1;
            let infrared = (infrareds >> i) & 1;
            lockers.push(Locker {
                number: i + 1,
                closed: latch == 1,
                loaded: infrared == 1,
            })
        }
        Status { lockers }
    }
}

impl From<Status> for Vec<u8> {
    fn from(val: Status) -> Self {
        let mut resp = vec![];
        for lock in &val.lockers {
            let latch_state = if lock.closed { "C" } else { "O" };
            let infrared_state = if lock.loaded { "L" } else { "E" };
            resp.push(latch_state.to_owned() + infrared_state);
        }
        resp.join(",").into()
    }
}

impl Debug for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut resp = vec![];
        for lock in &self.lockers {
            let latch_state = if lock.closed { "C" } else { "O" };
            let infrared_state = if lock.loaded { "L" } else { "E" };
            resp.push(latch_state.to_owned() + infrared_state);
        }

        f.write_str(resp.join(",").as_str())
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut resp = vec![];
        for lock in &self.lockers {
            let latch_state = if lock.closed { "C" } else { "O" };
            let infrared_state = if lock.loaded { "L" } else { "E" };
            resp.push(latch_state.to_owned() + infrared_state);
        }

        f.write_str(resp.join(",").as_str())
    }
}