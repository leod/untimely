use std::{cmp::Ordering, collections::BinaryHeap};

use rand::Rng;
use rand_distr::{Distribution, Normal};

use crate::time::{LocalDt, LocalTime};

#[derive(Debug, Clone)]
pub struct MockChannelParams {
    pub latency_mean_millis: f64,
    pub latency_std_dev: f64,
    pub loss: f64,
}

impl MockChannelParams {
    pub const PERFECT: Self = Self {
        latency_mean_millis: 0.0,
        latency_std_dev: 0.0,
        loss: 0.0,
    };

    pub fn sample_residual<R: Rng>(&self, rng: &mut R) -> Option<LocalDt> {
        if rng.gen::<f64>() < self.loss {
            None
        } else {
            let distribution = Normal::new(self.latency_mean_millis, self.latency_std_dev).unwrap();
            let residual = distribution.sample(rng);
            Some(LocalDt::from_millis(residual))
        }
    }
}

#[derive(Clone)]
struct Message<T>(LocalTime, T);

#[derive(Clone)]
pub struct MockChannel<T> {
    messages_in_transit: BinaryHeap<Message<T>>,
}

impl<T> Default for MockChannel<T> {
    fn default() -> Self {
        MockChannel {
            messages_in_transit: BinaryHeap::new(),
        }
    }
}

impl<T> MockChannel<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn send(&mut self, params: &MockChannelParams, current_time: LocalTime, message: T) {
        let rng = &mut rand::thread_rng();

        if let Some(residual) = params.sample_residual(rng) {
            let arrival_time = current_time + residual;
            self.messages_in_transit
                .push(Message(arrival_time, message));
        }
    }

    pub fn receive(&mut self, current_time: LocalTime) -> Option<(LocalTime, T)> {
        if let Some(Message(oldest_arrival_time, _)) = self.messages_in_transit.peek() {
            if current_time > *oldest_arrival_time {
                let oldest_entry = self.messages_in_transit.pop().unwrap();
                Some((oldest_entry.0, oldest_entry.1))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<T> PartialOrd for Message<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl<T> Eq for Message<T> {}

impl<T> PartialEq for Message<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(&other) == Ordering::Equal
    }
}

impl<T> Ord for Message<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.partial_cmp(&self.0).unwrap()
    }
}
