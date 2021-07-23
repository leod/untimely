use crate::time::{Time, TimeTag};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct TickNum(u32);

#[derive(Debug, Clone, Copy)]
pub struct TickTag;
impl TimeTag for TickTag {}

pub type TickTime = Time<TickTag>;

impl TickNum {
    pub fn zero() -> Self {
        TickNum(0)
    }

    pub fn succ(self) -> Self {
        TickNum(self.0 + 1)
    }

    pub fn to_u32(self) -> u32 {
        self.0
    }

    pub fn to_tick_time(self) -> TickTime {
        // In TickNum space, we interpret one tick as meaning "one second",
        // although this of course does not match with the actual game time.
        TickTime::from_secs(self.0 as f64)
    }
}
