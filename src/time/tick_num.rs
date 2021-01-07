use std::ops::{Add, AddAssign};

use crate::time::{GameTime, GameTimeDelta};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TickNumDelta(u64);

impl TickNumDelta {
    pub const ZERO: TickNumDelta = TickNumDelta(0);
    pub const ONE: TickNumDelta = TickNumDelta(1);

    pub fn to_game_time_delta(self, tick_time_delta: GameTimeDelta) -> GameTimeDelta {
        self.0 as f64 * tick_time_delta
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TickNum(TickNumDelta);

impl TickNum {
    pub const ZERO: TickNum = TickNum(TickNumDelta::ZERO);

    pub fn to_game_time(self, tick_time_delta: GameTimeDelta) -> GameTime {
        GameTime::ZERO + self.0.to_game_time_delta(tick_time_delta)
    }

    pub fn to_delta(self) -> TickNumDelta {
        self.0
    }

    pub fn get_next(self) -> Self {
        self + TickNumDelta::ONE
    }
}

impl Add<u64> for TickNumDelta {
    type Output = Self;

    fn add(self, other: u64) -> Self {
        TickNumDelta(self.0 + other)
    }
}

impl Add<u64> for TickNum {
    type Output = Self;

    fn add(self, other: u64) -> Self {
        TickNum(self.0 + other)
    }
}

impl Add<TickNumDelta> for TickNum {
    type Output = Self;

    fn add(self, other: TickNumDelta) -> Self {
        TickNum(self.0 + other.0)
    }
}

impl AddAssign<u64> for TickNum {
    fn add_assign(&mut self, other: u64) {
        *self = *self + other;
    }
}

impl AddAssign<TickNumDelta> for TickNum {
    fn add_assign(&mut self, other: TickNumDelta) {
        *self = *self + other;
    }
}

impl From<u64> for TickNumDelta {
    fn from(x: u64) -> Self {
        TickNumDelta(x)
    }
}

impl From<TickNumDelta> for u64 {
    fn from(x: TickNumDelta) -> Self {
        x.0
    }
}

impl From<usize> for TickNumDelta {
    fn from(x: usize) -> Self {
        TickNumDelta(x as u64)
    }
}

impl From<TickNumDelta> for usize {
    fn from(x: TickNumDelta) -> Self {
        x.0 as usize
    }
}

impl From<u64> for TickNum {
    fn from(x: u64) -> Self {
        TickNum(x.into())
    }
}

impl From<TickNum> for u64 {
    fn from(x: TickNum) -> Self {
        x.0.into()
    }
}

impl From<usize> for TickNum {
    fn from(x: usize) -> Self {
        TickNum(x.into())
    }
}

impl From<TickNum> for usize {
    fn from(x: TickNum) -> Self {
        x.0.into()
    }
}
