use std::rc::Rc;

use rand::Rng;
use rand_distr::{Distribution, Normal};

use gnuplot::{AutoOption, AxesCommon, Caption, DashType, Figure, Graph, LineStyle, LineWidth};

use pareen::AnimBox;

use untimely::{
    time::{ClientGameClock, DelayedTimeMappingClock, TimeWarpFunction},
    GameTime, GameTimeDelta, LocalTime, LocalTimeDelta, TickNum, TickNumDelta, TimeMappingConfig,
};

#[derive(Debug, Clone)]
pub struct ConnectionParams {
    pub receive_latency_mean_millis: f64,
    pub receive_latency_std_dev: f64,
    pub receive_loss: f64,
}

#[derive(Clone)]
pub struct ConnectionProfile {
    pub name: &'static str,
    pub params: Rc<AnimBox<GameTimeDelta, ConnectionParams>>,
}

impl ConnectionProfile {
    pub fn no_variance() -> Self {
        Self {
            name: "no variance connection",
            params: Rc::new(
                pareen::constant(ConnectionParams {
                    receive_latency_mean_millis: 100.0,
                    receive_latency_std_dev: 0.0,
                    receive_loss: 0.0,
                })
                .into_box(),
            ),
        }
    }

    pub fn great() -> Self {
        Self {
            name: "great connection",
            params: Rc::new(
                pareen::constant(ConnectionParams {
                    receive_latency_mean_millis: 20.0,
                    receive_latency_std_dev: 1.0,
                    receive_loss: 0.0,
                })
                .into_box(),
            ),
        }
    }

    pub fn okay() -> Self {
        Self {
            name: "okay connection",
            params: Rc::new(
                pareen::constant(ConnectionParams {
                    receive_latency_mean_millis: 60.0,
                    receive_latency_std_dev: 10.0,
                    receive_loss: 0.01,
                })
                .into_box(),
            ),
        }
    }

    pub fn okayish() -> Self {
        Self {
            name: "okayish connection",
            params: Rc::new(
                pareen::constant(ConnectionParams {
                    receive_latency_mean_millis: 100.0,
                    receive_latency_std_dev: 20.0,
                    receive_loss: 0.05,
                })
                .into_box(),
            ),
        }
    }

    pub fn great_okayish() -> Self {
        let params = Rc::try_unwrap(Self::great().params)
            .ok()
            .unwrap()
            .map_time(GameTimeDelta::from_secs)
            .dur(2.0)
            .seq_with_dur(
                Rc::try_unwrap(Self::okayish().params)
                    .ok()
                    .unwrap()
                    .map_time(GameTimeDelta::from_secs)
                    .dur(2.0),
            )
            .repeat()
            .map_time(GameTimeDelta::to_secs)
            .into_box();

        Self {
            name: "great<->okayish connection",
            params: Rc::new(params),
        }
    }
}

impl ConnectionProfile {
    /*pub fn receive_latency_distr(&self) -> impl Distribution<f64> {
        Normal::new(
            self.receive_latency_mean_millis,
            self.receive_latency_std_dev,
        )
        .unwrap()
    }*/

    pub fn sample_tick_receive_times(
        &self,
        num_ticks: usize,
        tick_time_delta: GameTimeDelta,
        start_local_time: LocalTime,
        start_tick_num: TickNum,
    ) -> Vec<(LocalTime, TickNum)> {
        let mut receive_times: Vec<_> = (0..num_ticks)
            .map(|tick_num_delta| {
                let tick_num_delta = TickNumDelta::from(tick_num_delta);
                let tick_num = start_tick_num + tick_num_delta;
                let game_time_delta = tick_num_delta.to_game_time_delta(tick_time_delta);

                // NOTE: We assume here that the server performs and sends ticks
                // with a perfectly constant period. Further, we assume that
                // GameTime progresses at the same speed as LocalTime.
                let server_local_time_delta = LocalTimeDelta::from_secs(game_time_delta.to_secs());
                let server_local_time = start_local_time + server_local_time_delta;

                let connection_params = self.params.eval(game_time_delta);
                let receive_latency = Normal::new(
                    connection_params.receive_latency_mean_millis,
                    connection_params.receive_latency_std_dev,
                )
                .unwrap()
                .sample(&mut rand::thread_rng());

                let client_receive_time =
                    server_local_time + LocalTimeDelta::from_millis(receive_latency);

                (client_receive_time, tick_num)
            })
            .filter(|(_, tick_num)| {
                rand::thread_rng().gen::<f64>()
                    >= self
                        .params
                        .eval(tick_num.to_delta().to_game_time_delta(tick_time_delta))
                        .receive_loss
            })
            .collect();

        // Due to the receive jitter, it can happen that we receive ticks out
        // of order. We re-sort here, so that the events are ordered by the
        // client's local time.
        receive_times.sort_by(|(local_time_a, _), (local_time_b, _)| {
            local_time_a.partial_cmp(local_time_b).unwrap()
        });
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
        name: "perfect client, 60Hz",
        frame_time_mean_millis: 1000.0 / 60.0,
        frame_time_std_dev: 0.0,
    };

    pub const PERFECT_32HZ: ClientProfile = ClientProfile {
        name: "perfect client, 32Hz",
        frame_time_mean_millis: 1000.0 / 32.0,
        frame_time_std_dev: 0.0,
    };

    pub const PERFECT_16HZ: ClientProfile = ClientProfile {
        name: "perfect client, 16Hz",
        frame_time_mean_millis: 1000.0 / 16.0,
        frame_time_std_dev: 0.0,
    };

    pub const SOLID_60HZ: ClientProfile = ClientProfile {
        name: "solid client",
        frame_time_mean_millis: 1000.0 / 60.0,
        frame_time_std_dev: 2.5,
    };

    pub fn frame_time_distr(&self) -> impl Distribution<f64> {
        Normal::new(self.frame_time_mean_millis, self.frame_time_std_dev).unwrap()
    }

    pub fn sample_frame_time_delta(&self) -> LocalTimeDelta {
        LocalTimeDelta::from_millis(self.frame_time_distr().sample(&mut rand::thread_rng()))
    }
}

pub fn perfect_game_time(
    profile: &ConnectionProfile,
    client_local_time: LocalTime,
    game_time_delay: GameTimeDelta,
) -> GameTime {
    // FIXME: This is assuming that both server and client local time start at zero.
    let server_game_time_delta = client_local_time.to_delta().to_game_time_delta();

    let connection_params = profile.params.eval(server_game_time_delta);
    let perfect_local_time = client_local_time
        - LocalTimeDelta::from_millis(connection_params.receive_latency_mean_millis)
        - game_time_delay.to_local_time_delta();

    GameTime::from_secs_since_start(perfect_local_time.to_secs_since_start())
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

pub struct SimulationOutput {
    pub connection_profile: ConnectionProfile,
    pub client_profile: ClientProfile,
    pub tick_time_delta: GameTimeDelta,
    pub game_time_delay: GameTimeDelta,
    pub receive_times: Vec<(LocalTime, TickNum)>,
    pub frames: Vec<SimulationFrame>,
}

pub fn simulate<Clock: ClientGameClock>(
    mut clock: Clock,
    connection_profile: &ConnectionProfile,
    client_profile: &ClientProfile,
    tick_time_delta: GameTimeDelta,
    game_time_delay: GameTimeDelta,
    num_ticks: usize,
) -> SimulationOutput {
    let receive_times = connection_profile.sample_tick_receive_times(
        num_ticks,
        tick_time_delta,
        LocalTime::ZERO,
        TickNum::ZERO,
    );

    // Sort by receive time *decreasing*, so that we can pop the earliest events
    // from the back.
    let mut receive_times_reverse = receive_times.clone();
    receive_times_reverse.reverse();

    let mut frames = Vec::new();
    let mut frame = SimulationFrame::default();

    loop {
        if receive_times_reverse.is_empty() {
            break;
        }

        // Check for the next received ticks.
        while let Some((receive_time, received_tick_num)) = receive_times_reverse.last().copied() {
            if receive_time > frame.local_time {
                // We haven't received this tick yet.
                break;
            }

            receive_times_reverse.pop();
            let received_game_time = received_tick_num.to_game_time(tick_time_delta);
            frame.max_received_game_time = frame.max_received_game_time.max(received_game_time);

            clock.record_receive_event(receive_time, received_tick_num);
            //clock.record_receive_event(frame.local_time, received_tick_num);
        }

        // Keep track of game time and deltas for simulation output.
        frame.game_time_delta = clock.get_game_time() - frame.game_time;
        frame.game_time = clock.get_game_time();

        // Remember what the clock thinks is the current receive game time.
        frame.predicted_receive_game_time = clock.get_predicted_receive_game_time();

        frames.push(frame.clone());

        // Advance the client's local time and game time.
        frame.local_time_delta = client_profile.sample_frame_time_delta();
        frame.local_time += frame.local_time_delta;
        clock.advance_local_time(frame.local_time_delta);
    }

    SimulationOutput {
        connection_profile: connection_profile.clone(),
        client_profile: client_profile.clone(),
        tick_time_delta,
        game_time_delay,
        receive_times,
        frames,
    }
}

#[derive(Debug, Clone)]
pub struct EvaluationMetrics {
    pub game_time_mean_squared_error: f64,
    pub warp_factor_mean: f64,
    pub warp_factor_variance: f64,
}

pub fn evaluate_simulation_output(output: &SimulationOutput) -> EvaluationMetrics {
    let game_time_mean_squared_error = output
        .frames
        .iter()
        .map(|frame| {
            let perfect_game_time = perfect_game_time(
                &output.connection_profile,
                frame.local_time,
                output.game_time_delay,
            );
            (perfect_game_time.to_millis_since_start() - frame.game_time.to_millis_since_start())
                .powi(2)
        })
        .sum::<f64>()
        / output.frames.len() as f64;

    let warp_factors: Vec<_> = output
        .frames
        .iter()
        .filter(|frame| frame.local_time_delta > LocalTimeDelta::ZERO)
        .map(|frame| frame.game_time_delta.to_secs() / frame.local_time_delta.to_secs())
        .collect();

    let warp_factor_mean = warp_factors.iter().sum::<f64>() / output.frames.len() as f64;
    let warp_factor_variance = warp_factors
        .iter()
        .map(|k| (k - warp_factor_mean).powi(2))
        .sum::<f64>()
        / output.frames.len() as f64;

    EvaluationMetrics {
        game_time_mean_squared_error,
        warp_factor_mean,
        warp_factor_variance,
    }
}

fn plot_simulation_output(output: &SimulationOutput, clock_name: &str, shift: bool) {
    let mut fg = Figure::new();
    let axes = fg.axes2d();

    axes.set_title(
        &format!(
            "Simulation with connection\\_profile={}, client\\_profile={}, client\\_game\\_clock={}",
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
    let perfect_game_time: Vec<_> = output
        .frames
        .iter()
        .map(|frame| {
            let perfect_game_time = perfect_game_time(
                &output.connection_profile,
                frame.local_time,
                output.game_time_delay,
            );
            perfect_game_time.to_secs_since_start() - time_shift(frame)
        })
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
    let time_delay: Vec<_> = output
        .frames
        .iter()
        .map(|frame| f64::from(frame.game_time) - f64::from(frame.predicted_receive_game_time))
        .collect();

    let receive_time_local: Vec<f64> = output
        .receive_times
        .iter()
        .map(|(local_time, _)| local_time.to_secs_since_start())
        .collect();
    let receive_time_game: Vec<f64> = output
        .receive_times
        .iter()
        .map(|(local_time, tick_num)| {
            tick_num
                .to_game_time(output.tick_time_delta)
                .to_secs_since_start()
                - if shift { f64::from(*local_time) } else { 0.0 }
        })
        .collect();

    axes.lines(
        client_local_time.as_slice(),
        server_local_time.as_slice(),
        &[Caption("server game time"), LineWidth(1.0)],
    );
    axes.lines_points(
        client_local_time.as_slice(),
        predicted_receive_game_time.as_slice(),
        &[Caption("predicted receive game time"), LineWidth(2.0)],
    );
    axes.lines_points(
        client_local_time.as_slice(),
        game_time.as_slice(),
        &[Caption("client game time"), LineWidth(2.0)],
    );
    axes.lines(
        client_local_time.as_slice(),
        perfect_game_time.as_slice(),
        &[Caption("perfect client game time"), LineWidth(1.0)],
    );
    axes.lines_points(
        client_local_time.as_slice(),
        max_received_game_time.as_slice(),
        &[Caption("max received game time")],
    );
    axes.lines_points(
        receive_time_local.as_slice(),
        receive_time_game.as_slice(),
        &[Caption("received game time")],
    );
    /*axes.lines_points(
        client_local_time.as_slice(),
        time_delay.as_slice(),
        &[Caption("time delay")],
    );*/

    fg.show().unwrap();
}

fn escape_gnuplot(s: &str) -> String {
    s.to_owned()
        .replace("_", "\\_")
        .replace("{", "\\{")
        .replace("}", "\\}")
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
        /*let y: Vec<f64> = profile
            .receive_latency_distr()
            .sample_iter(rand::thread_rng())
            .take(num_ticks)
            .collect();
        axes.lines(x.as_slice(), y.as_slice(), &[Caption(profile.name)]);*/
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

fn plot_evidence_len_vs_simulation_metrics(
    connection_profile: &ConnectionProfile,
    client_profile: &ClientProfile,
    key_name: &str,
    key_map: impl Fn(&EvaluationMetrics) -> f64,
) {
    let tick_time_delta = GameTimeDelta::from_secs(1.0 / 16.0);
    let game_time_delay = 2.0 * tick_time_delta;
    let num_ticks = 1024;
    let num_trials = 16;

    let title = format!(
        "Simulation evaluation metrics for different time mapping parameters (
    tick_time_delta={:?},
    game_time_delay={:?},
    num_ticks={:?},
    num_trials={:?},
    connection_profile={:?},
    client_profile={:?},
    client_game_clock=DelayedTimeMappingClock,
)",
        tick_time_delta,
        game_time_delay,
        num_ticks,
        num_trials,
        connection_profile.name,
        client_profile.name,
    );

    let mut fg = Figure::new();
    let axes = fg.axes2d();
    axes.set_title(&escape_gnuplot(&title), &[])
        .set_legend(Graph(0.5), Graph(0.9), &[], &[])
        .set_x_label("max evidence len", &[])
        .set_y_label(key_name, &[]);

    let settings = vec![
        (
            TimeWarpFunction::Sigmoid {
                alpha: 10.0,
                power: 1,
            },
            false,
            None,
        ),
        (
            TimeWarpFunction::Sigmoid {
                alpha: 10.0,
                power: 1,
            },
            true,
            None,
        ),
        (
            TimeWarpFunction::Sigmoid {
                alpha: 15.0,
                power: 1,
            },
            false,
            None,
        ),
        (
            TimeWarpFunction::Sigmoid {
                alpha: 15.0,
                power: 1,
            },
            false,
            Some(0.5),
        ),
        (
            TimeWarpFunction::Sigmoid {
                alpha: 25.0,
                power: 1,
            },
            false,
            None,
        ),
        (
            TimeWarpFunction::Sigmoid {
                alpha: 50.0,
                power: 1,
            },
            false,
            None,
        ),
        (
            TimeWarpFunction::Sigmoid {
                alpha: 50.0,
                power: 1,
            },
            false,
            Some(0.2),
        ),
        (
            TimeWarpFunction::Sigmoid {
                alpha: 100.0,
                power: 1,
            },
            false,
            None,
        ),
        (TimeWarpFunction::Catcheb, false, None),
    ];

    for (time_warp_function, ignore_if_out_of_order, ema_alpha) in settings.into_iter() {
        let max_evidence_len = [2, 4, 8, 16, 32, 64];
        let game_time_mean_squared_error: Vec<_> = max_evidence_len
            .iter()
            .map(|max_evidence_len| {
                (0..num_trials)
                    .map(|_| {
                        let client_game_clock = DelayedTimeMappingClock::new(
                            game_time_delay,
                            time_warp_function.clone(),
                            TimeMappingConfig {
                                max_evidence_len: *max_evidence_len,
                                tick_time_delta,
                                ignore_if_out_of_order,
                                ema_alpha,
                            },
                        );
                        let simulation_output = simulate(
                            client_game_clock,
                            connection_profile,
                            client_profile,
                            tick_time_delta,
                            game_time_delay,
                            num_ticks,
                        );
                        let metrics = evaluate_simulation_output(&simulation_output);
                        key_map(&metrics)
                    })
                    .sum::<f64>()
                    / num_trials as f64
            })
            .collect();

        let caption = format!(
            "{:?} with ignore_out_of_order={}, ema_alpha={:?}",
            time_warp_function, ignore_if_out_of_order, ema_alpha,
        );
        axes.lines_points(
            &max_evidence_len,
            game_time_mean_squared_error.as_slice(),
            &[Caption(&escape_gnuplot(&caption))],
        );
    }

    fg.show().unwrap();
}

fn main() {
    let tick_time_delta = GameTimeDelta::from_secs(1.0 / 16.0);

    /*plot_evidence_len_vs_simulation_metrics(
        &ConnectionProfile::great_okayish(),
        &ClientProfile::SOLID_60HZ,
        "warp factor variance",
        |metrics| metrics.warp_factor_variance,
    );
    plot_evidence_len_vs_simulation_metrics(
        &ConnectionProfile::great_okayish(),
        &ClientProfile::SOLID_60HZ,
        "game time MSE",
        |metrics| metrics.game_time_mean_squared_error,
    );*/

    {
        let game_time_delay = 2.0 * tick_time_delta;
        let time_warp_function = TimeWarpFunction::Sigmoid {
            alpha: 50.0,
            power: 1,
        };
        //let time_warp_function = TimeWarpFunction::Catcheb;

        let client_game_clock = DelayedTimeMappingClock::new(
            game_time_delay,
            time_warp_function,
            TimeMappingConfig {
                max_evidence_len: 8,
                tick_time_delta,
                ignore_if_out_of_order: false,
                ema_alpha: None,
            },
        );
        let num_ticks = 128;
        let simulation_output = simulate(
            client_game_clock,
            &ConnectionProfile::okayish(),
            &ClientProfile::SOLID_60HZ,
            tick_time_delta,
            game_time_delay,
            num_ticks,
        );
        println!(
            "Evaluation metrics: {:?}",
            evaluate_simulation_output(&simulation_output)
        );
        plot_simulation_output(&simulation_output, "DelayedTimeMappingClock", false);
        //plot_simulation_output(&simulation_output, "DelayedTimeMappingClock", true);
    }

    /*plot_receive_latencies(
        &[&ConnectionProfile::NO_VARIANCE, &ConnectionProfile::GREAT, &ConnectionProfile::OKAY, &ConnectionProfile::OKAYISH],
        128,
    );*/

    /*{
        let tick_receive_times = ConnectionProfile::NO_VARIANCE.sample_tick_receive_times(
            30,
            tick_time_delta,
            LocalTime::ZERO,
            TickNum::ZERO,
        );
        plot_tick_receive_times(
            &ConnectionProfile::NO_VARIANCE,
            &tick_receive_times,
            tick_time_delta,
        );
    }*/
}
