use std::{cell::Cell, rc::Rc};

use super::{LocalDt, LocalTime};

#[derive(Debug, Clone)]
pub struct LocalClock {
    local_time: Rc<Cell<Option<LocalTime>>>,
}

impl Default for LocalClock {
    fn default() -> Self {
        Self {
            local_time: Rc::new(Cell::new(None)),
        }
    }
}

impl LocalClock {
    pub fn new() -> Self {
        LocalClock::default()
    }

    pub fn local_time(&self) -> LocalTime {
        self.local_time.get().unwrap_or(LocalTime::zero())
    }

    pub fn set_local_time(&mut self, new_local_time: LocalTime) -> LocalDt {
        let dt = self
            .local_time
            .get()
            .map_or(LocalDt::zero(), |local_time| new_local_time - local_time);
        let dt = dt.max(LocalDt::zero());
        self.local_time.set(Some(new_local_time));

        dt
    }

    pub fn advance(&mut self, dt: LocalDt) {
        self.local_time.set(Some(self.local_time() + dt));
    }
}
