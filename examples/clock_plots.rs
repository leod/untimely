use rand::Rng;
use rand_distr::{Distribution, Normal};

use gnuplot::{AutoOption, AxesCommon, Caption, Figure, Graph};

use untimely::{LocalTime, TickNum};

struct Profile {
    name: &'static str,
    tick_duration_secs: f64,
    receive_latency_mean_millis: f64,
    receive_latency_std_dev: f64,
    receive_loss: f64,
}

impl Profile {
    const GREAT: Profile = Profile {
        name: "great connection",
        tick_duration_secs: 1.0 / 60.0,
        receive_latency_mean_millis: 20.0,
        receive_latency_std_dev: 1.0,
        receive_loss: 0.0,
    };

    const OKAYISH: Profile = Profile {
        name: "okayish connection",
        tick_duration_secs: 1.0 / 60.0,
        receive_latency_mean_millis: 100.0,
        receive_latency_std_dev: 20.0,
        receive_loss: 0.05,
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
        let mut receive_times: Vec<_> = self
            .receive_latency_distr()
            .sample_iter(rand::thread_rng())
            .take(num_ticks)
            .enumerate()
            .map(|(tick_num_offset, receive_latency)| {
                let tick_num = TickNum(start_tick_num.0 + tick_num_offset as u32);
                let game_time = tick_num.0 as f64 * self.tick_duration_secs;
                let local_time = start_local_time.0 + game_time + receive_latency / 1000.0;
                (LocalTime(local_time), tick_num)
            })
            .filter(|_| rand::thread_rng().gen::<f64>() >= self.receive_loss)
            .collect();

        // Due to the receive jitter, it can happen that we receive ticks out
        // of order. We re-sort here, so that the events are ordered by the
        // client's local time.
        receive_times.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
        receive_times
    }
}

fn plot_receive_latencies(profiles: &[&Profile], n: usize) {
    let x: Vec<f64> = (0..n).map(|n| n as f64).collect();

    let mut fg = Figure::new();
    let axes = fg.axes2d();
    axes.set_title(
        "Samples drawn from client's receive latency distribution",
        &[],
    )
    .set_legend(Graph(0.5), Graph(0.9), &[], &[])
    .set_x_label("n", &[])
    .set_y_label("receive latency", &[])
    .set_x_range(AutoOption::Fix(0.0), AutoOption::Fix((n - 1) as f64));

    for profile in profiles {
        let y: Vec<f64> = profile
            .receive_latency_distr()
            .sample_iter(rand::thread_rng())
            .take(n)
            .collect();
        axes.lines(x.as_slice(), y.as_slice(), &[Caption(profile.name)]);
    }
    fg.show().unwrap();
}

fn plot_tick_receive_times(profile: &Profile, receive_times: &[(LocalTime, TickNum)]) {
    let x: Vec<f64> = receive_times
        .iter()
        .map(|(local_time, _)| local_time.0)
        .collect();
    let y: Vec<f64> = receive_times
        .iter()
        .map(|(_, tick_num)| tick_num.0 as f64 * profile.tick_duration_secs)
        .collect();

    let mut fg = Figure::new();
    fg.axes2d()
        .set_title(
            "Local client time vs. game time in client's incoming tick stream",
            &[],
        )
        .set_legend(Graph(0.5), Graph(0.9), &[], &[])
        .set_x_label("local time", &[])
        .set_y_label("game time", &[])
        .lines(x.as_slice(), y.as_slice(), &[Caption(profile.name)]);
    fg.show().unwrap();
}

fn main() {
    plot_receive_latencies(&[&Profile::GREAT, &Profile::OKAYISH], 128);

    let tick_receive_times =
        Profile::OKAYISH.sample_tick_receive_times(64, LocalTime(0.0), TickNum(0));
    plot_tick_receive_times(&Profile::OKAYISH, &tick_receive_times);
}
