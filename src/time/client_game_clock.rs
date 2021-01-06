use crate::{
    GameTime, GameTimeDelta, LocalTime, LocalTimeDelta, TickNum, TimeMapping, TimeMappingConfig,
};

pub trait ClientGameClock {
    fn record_receive_event(&mut self, local_time: LocalTime, received_tick_num: TickNum);
    fn advance_local_time(&mut self, local_time_delta: LocalTimeDelta);
    fn get_predicted_receive_game_time(&self) -> GameTime;
    fn get_game_time(&self) -> GameTime;
}

#[derive(Debug, Clone)]
pub struct DelayedGameClock {
    tick_time_delta: GameTimeDelta,
    game_time_delay: GameTimeDelta,
    time_warp_function: TimeWarpFunction,

    time_mapping: TimeMapping<LocalTime, GameTime>,

    current_local_time: LocalTime,
    current_game_time: GameTime,
    current_predicted_receive_game_time: GameTime,

    max_received_game_time: GameTime,
}

#[derive(Debug, Clone)]
pub enum TimeWarpFunction {
    Sigmoid { alpha: f64, power: i32 },
    Catcheb,
}

impl TimeWarpFunction {
    pub fn eval(&mut self, t: GameTimeDelta) -> f64 {
        match self {
            TimeWarpFunction::Sigmoid { alpha, power } => {
                let exponent = -*alpha * t.to_secs().powi(*power);
                0.5 + 1.0 / (1.0 + exponent.exp())
            }
            TimeWarpFunction::Catcheb => {
                0.5 + (2.0 - 0.5) / (1.0 + 2.0 * (-t.to_secs() / 0.005).exp())
            }
        }
    }
}

impl DelayedGameClock {
    pub fn new(
        game_time_delay: GameTimeDelta,
        time_warp_function: TimeWarpFunction,
        time_mapping_config: TimeMappingConfig,
    ) -> Self {
        assert!(game_time_delay > GameTimeDelta::ZERO);

        DelayedGameClock {
            tick_time_delta: time_mapping_config.tick_time_delta,
            game_time_delay,
            time_warp_function,
            time_mapping: TimeMapping::new(time_mapping_config),
            current_local_time: LocalTime::ZERO,
            current_game_time: GameTime::ZERO,
            current_predicted_receive_game_time: GameTime::ZERO,
            max_received_game_time: GameTime::ZERO,
        }
    }
}

impl ClientGameClock for DelayedGameClock {
    fn record_receive_event(&mut self, local_time: LocalTime, received_tick_num: TickNum) {
        let game_time = received_tick_num.to_game_time(self.tick_time_delta);
        self.time_mapping.record_evidence(local_time, game_time);

        self.max_received_game_time = self.max_received_game_time.max(game_time);
    }

    fn advance_local_time(&mut self, local_time_delta: LocalTimeDelta) {
        let target_game_time = self.current_predicted_receive_game_time - self.game_time_delay;
        let game_time_delta = target_game_time - self.current_game_time;
        let warp_factor = self.time_warp_function.eval(game_time_delta);

        let max_allowed_game_time = self.current_game_time.max(self.max_received_game_time);
        self.current_game_time += local_time_delta.to_game_time_delta() * warp_factor;
        self.current_game_time = self.current_game_time.min(max_allowed_game_time);

        self.current_local_time += local_time_delta;

        self.time_mapping.update(self.current_local_time);
        self.current_predicted_receive_game_time = self
            .time_mapping
            .eval(self.current_local_time)
            .unwrap_or(GameTime::ZERO);
    }

    fn get_predicted_receive_game_time(&self) -> GameTime {
        self.current_predicted_receive_game_time
    }

    fn get_game_time(&self) -> GameTime {
        //self.get_predicted_receive_game_time() - self.game_time_delay
        self.current_game_time
    }
}
