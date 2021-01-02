use std::collections::BTreeMap;

use rand::Rng;
use rand_distr::{Distribution, Normal};

use untimely::{LocalTime, TickNum};

struct Profile {
    tick_duration_secs: f64,
    receive_latency_mean_millis: f64,
    receive_latency_std_dev: f64,
    receive_loss: f64,
}

impl Profile {
    const GREAT: Profile = Profile {
        tick_duration_secs: 1.0 / 60.0,
        receive_latency_mean_millis: 20.0,
        receive_latency_std_dev: 1.0,
        receive_loss: 0.0,
    };

    const OKAYISH: Profile = Profile {
        tick_duration_secs: 1.0 / 60.0,
        receive_latency_mean_millis: 100.0,
        receive_latency_std_dev: 5.0,
        receive_loss: 5.0,
    };
}

impl Profile {
    fn receive_latency_distr(&self) -> impl Distribution<f64> {
        Normal::new(
            self.receive_latency_mean_millis,
            self.receive_latency_std_dev,
        )
        .unwrap()
    }

    fn sample_tick_receive_times(
        &self,
        num_ticks: usize,
        start_local_time: LocalTime,
        start_tick_num: TickNum,
    ) -> Vec<(LocalTime, TickNum)> {
        self.receive_latency_distr()
            .sample_iter(rand::thread_rng())
            .take(num_ticks)
            .enumerate()
            .map(|(tick_num_offset, receive_latency)| {
                let tick_num = TickNum(start_tick_num.0 + tick_num_offset as u32);
                let game_time = tick_num.0 as f64 * self.tick_duration_secs;
                (LocalTime(start_local_time.0 + game_time), tick_num)
            })
            .filter(|_| rand::thread_rng().gen::<f64>() >= self.receive_loss)
            .collect()
    }
}

fn main() {
    let tick_receive_times =
        Profile::OKAYISH.sample_tick_receive_times(256, LocalTime(0.0), TickNum(0));
}
