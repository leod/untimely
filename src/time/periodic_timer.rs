use crate::LocalTimeDelta;

pub struct PeriodicTimer {
    period: LocalTimeDelta,
    accumulator: LocalTimeDelta,
}

impl PeriodicTimer {
    pub fn new(period: LocalTimeDelta) -> Self {
        Self {
            period,
            accumulator: LocalTimeDelta::ZERO,
        }
    }

    pub fn add_time_delta(&mut self, delta: LocalTimeDelta) {
        self.accumulator += delta;
    }

    pub fn trigger(&mut self) -> bool {
        if self.accumulator >= self.period {
            self.accumulator -= self.period;
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.accumulator = LocalTimeDelta::ZERO;
    }
}
