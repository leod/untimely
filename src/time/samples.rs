use std::collections::VecDeque;

use super::{Time, TimeTag};

#[derive(Debug, Clone)]
pub struct Samples<Tag, Value> {
    samples: VecDeque<(Time<Tag>, Value)>,
}

impl<Tag, Value> Default for Samples<Tag, Value> {
    fn default() -> Self {
        Self {
            samples: VecDeque::new(),
        }
    }
}

impl<Tag, Value> Samples<Tag, Value>
where
    Tag: TimeTag,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn iter(&self) -> impl Iterator<Item = &(Time<Tag>, Value)> {
        self.samples.iter()
    }

    pub fn front(&self) -> Option<&(Time<Tag>, Value)> {
        self.samples.front()
    }

    pub fn back(&self) -> Option<&(Time<Tag>, Value)> {
        self.samples.back()
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn record_sample(&mut self, time: Time<Tag>, value: Value) {
        self.samples.push_back((time, value));
    }

    pub fn retain_recent_samples(&mut self, min_time: Time<Tag>) {
        self.samples.retain(|&(time, _)| time > min_time);
    }
}