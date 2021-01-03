use rand::Rng;
use rand_distr::{Distribution, Normal};

use gnuplot::{AutoOption, AxesCommon, Caption, Figure, Graph};

use untimely::{
    time::{ClientGameClock, DelayedTimeMappingClock},
    GameTime, GameTimeDelta, LocalTime, LocalTimeDelta, TickNum, TickNumDelta, TimeMappingConfig,
};

#[derive(Debug, Clone)]
pub struct ConnectionProfile {
    pub name: &'static str,
    pub receive_latency_mean_millis: f64,
    pub receive_latency_std_dev: f64,
    pub receive_loss: f64,
}

impl ConnectionProfile {
    pub const GREAT: ConnectionProfile = ConnectionProfile {
        name: "great connection",
        receive_latency_mean_millis: 20.0,
        receive_latency_std_dev: 1.0,
        receive_loss: 0.0,
    };

    pub const OKAYISH: ConnectionProfile = ConnectionProfile {
        name: "okayish connection",
        receive_latency_mean_millis: 100.0,
        receive_latency_std_dev: 20.0,
        receive_loss: 0.05,
    };
}

impl ConnectionProfile {
    pub fn receive_latency_distr(&self) -> impl Distribution<f64> {
        Normal::new(
            self.receive_latency_mean_millis,
            self.receive_latency_std_dev,
        )
        .unwrap()
    }

    pub fn sample_tick_receive_times(
        &self,
        num_ticks: usize,
        tick_time_delta: GameTimeDelta,
        start_local_time: LocalTime,
        start_tick_num: TickNum,
    ) -> Vec<(LocalTime, TickNum)> {
        let mut receive_times: Vec<_> = self
            .receive_latency_distr()
            .sample_iter(rand::thread_rng())
            .take(num_ticks)
            .enumerate()
            .map(|(tick_num_delta, receive_latency)| {
                let tick_num_delta = TickNumDelta::from(tick_num_delta);
                let tick_num = start_tick_num + tick_num_delta;

                // NOTE: We assume here that the server performs and sends ticks
                // with a perfectly constant period. Further, we assume that
                // GameTime progresses at the same speed as LocalTime.
                let game_time_delta = tick_num_delta.to_game_time_delta(tick_time_delta);
                let local_time_delta = LocalTimeDelta::from_secs(game_time_delta.to_secs());
                let local_time = start_local_time + local_time_delta;
                let local_arrival_time = local_time + LocalTimeDelta::from_millis(receive_latency);

                (local_arrival_time, tick_num)
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

#[derive(Debug, Clone)]
pub struct ClientProfile {
    pub name: &'static str,
    pub frame_time_mean_millis: f64,
    pub frame_time_std_dev: f64,
}

impl ClientProfile {
    pub const PERFECT_60HZ: ClientProfile = ClientProfile {
        name: "solid client",
        frame_time_mean_millis: 1000.0 / 60.0,
        frame_time_std_dev: 0.0,
    };

    pub fn frame_time_distr(&self) -> impl Distribution<f64> {
        Normal::new(self.frame_time_mean_millis, self.frame_time_std_dev).unwrap()
    }

    pub fn sample_frame_time_delta(&self) -> LocalTimeDelta {
        LocalTimeDelta::from_millis(self.frame_time_distr().sample(&mut rand::thread_rng()))
    }
}

#[derive(Debug, Clone, Default)]
pub struct SimulationFrame {
    pub local_time: LocalTime,
    pub game_time: GameTime,
    pub local_time_delta: LocalTimeDelta,
    pub game_time_delta: GameTimeDelta,
    pub max_received_game_time: GameTime,
    pub predicted_receive_game_time: GameTime,
    pub waiting_for_tick_data: bool,
}

#[derive(Debug, Clone)]
pub struct SimulationOutput {
    pub connection_profile: ConnectionProfile,
    pub client_profile: ClientProfile,
    pub receive_times: Vec<(LocalTime, TickNum)>,
    pub frames: Vec<SimulationFrame>,
}

pub fn simulate<Clock: ClientGameClock>(
    mut clock: Clock,
    connection_profile: &ConnectionProfile,
    client_profile: &ClientProfile,
    tick_time_delta: GameTimeDelta,
    num_ticks: usize,
) -> SimulationOutput {
    let mut receive_times = connection_profile.sample_tick_receive_times(
        num_ticks,
        tick_time_delta,
        LocalTime::ZERO,
        TickNum::ZERO,
    );

    // Sort by receive time *decreasing*, so that we can pop the earliest events
    // from the back.
    receive_times.reverse();

    let mut frames = Vec::new();
    let mut frame = SimulationFrame::default();

    loop {
        if receive_times.is_empty() {
            break;
        }

        // Advance the client's local time.
        frame.local_time_delta = client_profile.sample_frame_time_delta();
        frame.local_time += frame.local_time_delta;
        clock.advance_local_time(frame.local_time_delta);

        // Check for the next received ticks.
        while let Some((receive_time, received_tick_num)) = receive_times.last().copied() {
            if receive_time > frame.local_time {
                // We haven't received this tick yet.
                break;
            }

            receive_times.pop();
            let receive_game_time = received_tick_num.to_game_time(tick_time_delta);
            frame.max_received_game_time = frame.max_received_game_time.max(receive_game_time);

            clock.record_receive_event(receive_time, received_tick_num)
        }

        // Advance the client's game time acording to our clock.
        let next_game_time = clock.get_game_time();
        frame.game_time_delta = next_game_time - frame.game_time;
        frame.game_time = next_game_time;

        // Remember what the clock thinks is the current receive game time.
        frame.predicted_receive_game_time = clock.get_predicted_receive_game_time();

        frames.push(frame.clone());
    }

    // Restore original receive times for output...
    receive_times.reverse();

    SimulationOutput {
        connection_profile: connection_profile.clone(),
        client_profile: client_profile.clone(),
        receive_times,
        frames,
    }
}

fn plot_simulation_output(output: &SimulationOutput, clock_name: &str, shift: bool) {
    let mut fg = Figure::new();
    let axes = fg.axes2d();

    axes.set_title(
        &format!(
            "Simulation with connection={}, client={}, client\\_game\\_clock={}",
            output.connection_profile.name, output.client_profile.name, clock_name
        ),
        &[],
    )
    .set_legend(Graph(0.5), Graph(0.9), &[], &[])
    .set_x_label("client's local time [s]", &[])
    .set_y_label("game time [s]", &[]);

    let time_shift = |frame: &SimulationFrame| {
        if shift {
            f64::from(frame.local_time)
        } else {
            0.0
        }
    };

    let client_local_time: Vec<_> = output
        .frames
        .iter()
        .map(|frame| f64::from(frame.local_time))
        .collect();

    let server_local_time: Vec<_> = output
        .frames
        .iter()
        .map(|frame| f64::from(frame.local_time) - time_shift(frame))
        .collect();

    let game_time: Vec<_> = output
        .frames
        .iter()
        .map(|frame| f64::from(frame.game_time) - time_shift(frame))
        .collect();
    let max_received_game_time: Vec<_> = output
        .frames
        .iter()
        .map(|frame| f64::from(frame.max_received_game_time) - time_shift(frame))
        .collect();
    let predicted_receive_game_time: Vec<_> = output
        .frames
        .iter()
        .map(|frame| f64::from(frame.predicted_receive_game_time) - time_shift(frame))
        .collect();

    axes.lines(
        client_local_time.as_slice(),
        server_local_time.as_slice(),
        &[Caption("server game time")],
    );
    axes.lines(
        client_local_time.as_slice(),
        game_time.as_slice(),
        &[Caption("client game time")],
    );
    axes.lines(
        client_local_time.as_slice(),
        max_received_game_time.as_slice(),
        &[Caption("max received game time")],
    );
    axes.lines(
        client_local_time.as_slice(),
        predicted_receive_game_time.as_slice(),
        &[Caption("predicted receive game time")],
    );

    fg.show().unwrap();
}

fn plot_receive_latencies(profiles: &[&ConnectionProfile], num_ticks: usize) {
    let x: Vec<f64> = (0..num_ticks).map(|n| n as f64).collect();

    let mut fg = Figure::new();
    let axes = fg.axes2d();
    axes.set_title(
        "Samples drawn from client's receive latency distribution",
        &[],
    )
    .set_legend(Graph(0.5), Graph(0.9), &[], &[])
    .set_x_label("n", &[])
    .set_y_label("receive latency [ms]", &[])
    .set_x_range(
        AutoOption::Fix(0.0),
        AutoOption::Fix((num_ticks - 1) as f64),
    );

    for profile in profiles {
        let y: Vec<f64> = profile
            .receive_latency_distr()
            .sample_iter(rand::thread_rng())
            .take(num_ticks)
            .collect();
        axes.lines(x.as_slice(), y.as_slice(), &[Caption(profile.name)]);
    }
    fg.show().unwrap();
}

fn plot_tick_receive_times(
    profile: &ConnectionProfile,
    receive_times: &[(LocalTime, TickNum)],
    tick_time_delta: GameTimeDelta,
) {
    let x: Vec<f64> = receive_times
        .iter()
        .map(|(local_time, _)| local_time.to_secs_since_start())
        .collect();
    let y: Vec<f64> = receive_times
        .iter()
        .map(|(_, tick_num)| tick_num.to_game_time(tick_time_delta).to_secs_since_start())
        .collect();

    let mut fg = Figure::new();
    fg.axes2d()
        .set_title(
            "Local client time vs. game time in client's incoming tick stream",
            &[],
        )
        .set_legend(Graph(0.5), Graph(0.9), &[], &[])
        .set_x_label("local time [s]", &[])
        .set_y_label("game time [s]", &[])
        .lines(x.as_slice(), y.as_slice(), &[Caption(profile.name)]);
    fg.show().unwrap();
}

fn main() {
    let tick_time_delta = GameTimeDelta::from_secs(1.0 / 20.0);

    {
        let game_time_delay = 2.0 * tick_time_delta;
        let client_game_clock = DelayedTimeMappingClock::new(
            tick_time_delta,
            game_time_delay,
            TimeMappingConfig {
                max_evidence_len: 8,
            },
        );
        let num_ticks = 256;
        let simulation_output = simulate(
            client_game_clock,
            &ConnectionProfile::OKAYISH,
            &ClientProfile::PERFECT_60HZ,
            tick_time_delta,
            num_ticks,
        );
        plot_simulation_output(&simulation_output, "DelayedTimeMappingClock", true);
        plot_simulation_output(&simulation_output, "DelayedTimeMappingClock", false);
    }

    /*plot_receive_latencies(
        &[&ConnectionProfile::GREAT, &ConnectionProfile::OKAYISH],
        128,
    );

    {
        let tick_receive_times = ConnectionProfile::OKAYISH.sample_tick_receive_times(
            30,
            tick_time_delta,
            LocalTime::ZERO,
            TickNum::ZERO,
        );
        plot_tick_receive_times(
            &ConnectionProfile::OKAYISH,
            &tick_receive_times,
            tick_time_delta,
        );
    }*/
}
