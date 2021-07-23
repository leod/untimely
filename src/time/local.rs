use std::{cell::Cell, rc::Rc};

use super::{LocalDt, LocalTime};

#[derive(Debug, Clone)]
pub struct LocalClock {
    local_time: Rc<Cell<LocalTime>>,
}

impl LocalClock {
    pub fn new() -> Self {
        LocalClock::default()
    }

    pub fn local_time(&self) -> LocalTime {
        self.local_time.get()
    }

    pub fn set_local_time(&mut self, new_local_time: LocalTime) -> LocalDt {
        let dt = (new_local_time - self.local_time.get()).max(LocalDt::zero());
        self.local_time.set(new_local_time);

        dt
    }
}

impl Default for LocalClock {
    fn default() -> Self {
        Self {
            local_time: Rc::new(Cell::new(LocalTime::zero())),
        }
    }
}
