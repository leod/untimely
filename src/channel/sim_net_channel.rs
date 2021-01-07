use std::{cmp::Ordering, collections::BinaryHeap};

use rand::Rng;
use rand_distr::{Distribution, Normal};

use pareen::{Anim, AnimBox, Fun};

use crate::{LocalTime, LocalTimeDelta};

#[derive(Debug, Clone)]
pub struct NetParams {
    pub latency_mean_millis: f64,
    pub latency_std_dev: f64,
    pub loss: f64,
}

impl NetParams {
    pub fn sample<R: Rng>(&self, rng: &mut R) -> Option<LocalTimeDelta> {
        if rng.gen::<f64>() < self.loss {
            None
        } else {
            let distr = Normal::new(self.latency_mean_millis, self.latency_std_dev).unwrap();
            Some(LocalTimeDelta::from_millis(distr.sample(rng)))
        }
    }
}

pub struct NetProfile(AnimBox<LocalTimeDelta, NetParams>);

impl NetProfile {
    pub fn new<F, A>(anim: A) -> Self
    where
        F: Fun<T = LocalTimeDelta, V = NetParams> + 'static,
        A: Into<Anim<F>>,
    {
        NetProfile(anim.into().into_box())
    }

    pub fn perfect_profile() -> Self {
        Self::new(NetParams {
            latency_mean_millis: 0.0,
            latency_std_dev: 0.0,
            loss: 0.0,
        })
    }

    pub fn zen_fast_profile() -> Self {
        Self::new(NetParams {
            latency_mean_millis: 20.0,
            latency_std_dev: 0.0,
            loss: 0.0,
        })
    }

    pub fn zen_slow_profile() -> Self {
        Self::new(NetParams {
            latency_mean_millis: 150.0,
            latency_std_dev: 0.0,
            loss: 0.0,
        })
    }

    pub fn wonky_fast_profile() -> Self {
        Self::new(NetParams {
            latency_mean_millis: 20.0,
            latency_std_dev: 5.0,
            loss: 0.025,
        })
    }

    pub fn wonky_slow_profile() -> Self {
        Self::new(NetParams {
            latency_mean_millis: 100.0,
            latency_std_dev: 10.0,
            loss: 0.025,
        })
    }

    pub fn net_params(&self, time_delta: LocalTimeDelta) -> NetParams {
        self.0.eval(time_delta)
    }
}

struct Entry<T>(LocalTime, T);

pub struct SimNetChannel<T> {
    profile: NetProfile,
    messages_in_transit: BinaryHeap<Entry<T>>,
}

impl<T> SimNetChannel<T> {
    pub fn new(profile: NetProfile) -> Self {
        Self {
            profile,
            messages_in_transit: BinaryHeap::new(),
        }
    }

    pub fn send(&mut self, current_time: LocalTime, message: T) {
        // TODO: Do we care about reproducibility here? Does not seem worth the
        // trouble.
        let rng = &mut rand::thread_rng();

        let net_params = self.profile.net_params(current_time.to_delta());
        let arrival_time = net_params.sample(rng);

        if let Some(arrival_time) = net_params.sample(rng) {
            self.messages_in_transit
                .push(Entry(LocalTime::ZERO + arrival_time, message));
        }
    }

    pub fn receive(&mut self, current_time: LocalTime) -> Option<(LocalTime, T)> {
        if let Some(Entry(oldest_arrival_time, _)) = self.messages_in_transit.peek() {
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

impl<T> PartialOrd for Entry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl<T> Eq for Entry<T> {}

impl<T> PartialEq for Entry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(&other) == Ordering::Equal
    }
}

impl<T> Ord for Entry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.partial_cmp(&self.0).unwrap()
    }
}
