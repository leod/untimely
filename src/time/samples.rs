use std::collections::VecDeque;

use super::{LocalClock, LocalDt, LocalTime};

#[derive(Debug, Clone)]
pub struct Samples<Value> {
    max_age: LocalDt,
    clock: LocalClock,
    samples: VecDeque<(LocalTime, Value)>,
}

impl<Value> Samples<Value> {
    pub fn new(max_age: LocalDt, clock: LocalClock) -> Self {
        Self {
            max_age,
            clock,
            samples: VecDeque::new(),
        }
    }

    pub fn set_max_age(&mut self, max_age: LocalDt) {
        self.max_age = max_age;
    }

    pub fn iter(&self) -> impl Iterator<Item = (LocalTime, &Value)> {
        self.samples.iter().map(|(time, value)| (*time, value))
    }

    pub fn times(&self) -> impl Iterator<Item = LocalTime> + '_ {
        self.samples.iter().map(|(time, _)| *time)
    }

    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.samples.iter().map(|(_, value)| value)
    }

    pub fn front(&self) -> Option<&(LocalTime, Value)> {
        self.samples.front()
    }

    pub fn back(&self) -> Option<&(LocalTime, Value)> {
        self.samples.back()
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn record(&mut self, sample_time: LocalTime, sample_value: Value) {
        let local_time = self.clock.local_time();
        let max_age = self.max_age;

        self.samples.push_back((sample_time, sample_value));
        self.samples
            .retain(|&(time, _)| local_time - time <= max_age);
    }
}
