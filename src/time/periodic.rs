use crate::time::LocalDt;

#[derive(Debug, Clone)]
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

    pub fn period(&self) -> LocalDt {
        self.period
    }

    pub fn accumulator(&self) -> LocalDt {
        self.accumulator
    }

    pub fn percent(&self) -> f64 {
        self.accumulator / self.period
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
