use rand_distr::{Distribution, Normal};

use untimely::{LocalTime, TickNum};

struct TimeProfile {
    receive_latency_mean_millis: f64,
    receive_latency_std_dev: f64,
    receive_loss: f64,
}

impl TimeProfile {}

impl TimeProfile {
    fn sample_tick_receive_times(&self, n: usize) -> BTreeMap<(LocalTime, TickNum)> {
        let distr = Normal::new(
            self.receive_latency_mean_millis,
            self.receive_latency_std_dev,
        );
        BTreeMap::new()
    }
}

fn main() {}
