use std::{cmp::Ordering, collections::BinaryHeap};

use rand::Rng;
use rand_distr::{Distribution, Normal};

use crate::{LocalClock, LocalDt, LocalTime};

#[derive(Debug, Clone)]
pub struct MockChannelParams {
    pub latency_mean: LocalDt,
    pub latency_std_dev: LocalDt,
    pub loss: f64,
}

impl MockChannelParams {
    pub fn perfect() -> Self {
        Self {
            latency_mean: LocalDt::zero(),
            latency_std_dev: LocalDt::zero(),
            loss: 0.0,
        }
    }

    pub fn sample_residual<R: Rng>(&self, rng: &mut R) -> Option<LocalDt> {
        if rng.gen::<f64>() < self.loss {
            None
        } else {
            let distribution =
                Normal::new(self.latency_mean.to_secs(), self.latency_std_dev.to_secs()).unwrap();
            let residual = distribution.sample(rng);
            Some(LocalDt::from_secs(residual))
        }
    }
}

#[derive(Clone)]
struct Message<T>(LocalTime, T);

#[derive(Clone)]
pub struct MockChannel<T> {
    clock: LocalClock,
    messages_in_transit: BinaryHeap<Message<T>>,
}

impl<T> MockChannel<T> {
    pub fn new(clock: LocalClock) -> Self {
        Self {
            clock,
            messages_in_transit: BinaryHeap::new(),
        }
    }

    pub fn send(&mut self, params: &MockChannelParams, message: T) {
        let rng = &mut rand::thread_rng();

        if let Some(residual) = params.sample_residual(rng) {
            let arrival_time = self.clock.local_time() + residual;
            self.messages_in_transit
                .push(Message(arrival_time, message));
        }
    }

    pub fn receive(&mut self) -> Option<(LocalTime, T)> {
        if let Some(Message(oldest_arrival_time, _)) = self.messages_in_transit.peek() {
            if self.clock.local_time() > *oldest_arrival_time {
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
