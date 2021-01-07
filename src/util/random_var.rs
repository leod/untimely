use std::collections::VecDeque;

use crate::{LocalTime, LocalTimeDelta};

pub struct SlidingWindowRandomVar {
    max_age: LocalTimeDelta,
    entries: VecDeque<(LocalTime, f64)>,
}

impl SlidingWindowRandomVar {
    pub fn new(max_age: LocalTimeDelta) -> Self {
        Self {
            max_age,
            entries: VecDeque::new(),
        }
    }

    pub fn record(&mut self, current_time: LocalTime, value: f64) {
        // TODO: Not sure what the best place is to pop old values
        while self.entries.front().map_or(false, |(oldest_time, _)| current_time >= *oldest_time + self.max_age) {
            self.entries.pop_front();
        }

        assert!(self.entries.back().map_or(true, |(newest_time, _)| current_time >= *newest_time));

        self.entries.push_back((current_time, value));
    }

    // TODO: mean, median, std_dev, min, max, percentile

    pub fn to_plot_points(&self, min_time: LocalTime) -> Vec<(f64, f64)> {
        self
            .entries
            .iter()
            .map(|(time, value)| ((*time - min_time).to_secs(), *value))
            .collect()
    }
}