use crate::time::{Time, TimeTag};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct TickNum(u64);

#[derive(Debug, Clone, Copy)]
pub struct TickTag;
impl TimeTag for TickTag {}

pub type TickTime = Time<TickTag>;

impl TickNum {
    pub fn zero() -> Self {
        TickNum(0)
    }

    pub fn from_tick_time(tick_time: TickTime) -> Self {
        // In TickNum space, we interpret one tick as meaning "one second",
        // although this of course does not match with the actual game time.
        TickNum(tick_time.to_secs() as u64)
    }

    pub fn succ(self) -> Self {
        TickNum(self.0 + 1)
    }

    pub fn to_u64(self) -> u64 {
        self.0
    }

    pub fn to_tick_time(self) -> TickTime {
        TickTime::from_secs(self.0 as f64)
    }
}
