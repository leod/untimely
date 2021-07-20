use crate::time::LocalDt;

pub struct PeriodicTimer {
    period: LocalDt,
    accumulator: LocalDt,
}

impl PeriodicTimer {
    pub fn new(period: LocalDt) -> Self {
        Self {
            period,
            accumulator: LocalDt::zero(),
        }
    }

    pub fn advance(&mut self, dt: LocalDt) {
        self.accumulator += dt;
    }

    pub fn trigger(&mut self) -> bool {
        if self.accumulator >= self.period {
            self.accumulator -= self.period;
            true
        } else {
            false
        }
    }
}
