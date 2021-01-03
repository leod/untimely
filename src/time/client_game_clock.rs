use crate::{
    GameTime, GameTimeDelta, LocalTime, LocalTimeDelta, TickNum, TimeMapping, TimeMappingConfig,
};

pub trait ClientGameClock {
    fn record_receive_event(&mut self, local_time: LocalTime, received_tick_num: TickNum);
    fn advance_local_time(&mut self, local_time_delta: LocalTimeDelta);
    fn get_game_time(&self) -> GameTime;
}

pub struct DelayedTimeMappingClock {
    tick_time_delta: GameTimeDelta,
    game_time_delay: GameTimeDelta,

    time_mapping: TimeMapping<LocalTime, GameTime>,

    current_local_time: LocalTime,
}

impl DelayedTimeMappingClock {
    pub fn new(
        tick_time_delta: GameTimeDelta,
        game_time_delay: GameTimeDelta,
        time_mapping_config: TimeMappingConfig,
    ) -> Self {
        DelayedTimeMappingClock {
            tick_time_delta,
            game_time_delay,
            time_mapping: TimeMapping::new(time_mapping_config),
            current_local_time: LocalTime::ZERO,
        }
    }
}

impl ClientGameClock for DelayedTimeMappingClock {
    fn record_receive_event(&mut self, local_time: LocalTime, received_tick_num: TickNum) {
        let game_time = received_tick_num.to_game_time(self.tick_time_delta);
        self.time_mapping.record_evidence(local_time, game_time);
    }

    fn advance_local_time(&mut self, local_time_delta: LocalTimeDelta) {
        self.current_local_time += local_time_delta;
    }

    fn get_game_time(&self) -> GameTime {
        self.time_mapping
            .eval(self.current_local_time)
            .map_or(GameTime::ZERO, |receive_game_time| {
                receive_game_time - self.game_time_delay
            })
    }
}
