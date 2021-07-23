use std::{cell::Cell, rc::Rc};

use super::{LocalDt, LocalTime};

#[derive(Debug, Clone)]
pub struct LocalClock {
    time: Rc<Cell<LocalTime>>,
}

impl LocalClock {
    pub fn new() -> Self {
        LocalClock::default()
    }

    pub fn get(&self) -> LocalTime {
        self.time.get()
    }

    pub fn set(&mut self, time: LocalTime) -> LocalDt {
        let dt = if time < self.time.get() {
            LocalDt::zero()
        } else {
            self.time.get() - time
        };

        self.time.set(time);

        dt
    }
}

impl Default for LocalClock {
    fn default() -> Self {
        Self {
            time: Rc::new(Cell::new(LocalTime::zero())),
        }
    }
}
