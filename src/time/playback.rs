use super::{predict_stream_time, GameDt, GameTime, LocalClock, LocalDt, LocalTime, Samples};

pub fn time_warp(residual: GameDt) -> f64 {
    0.5 + (2.0 - 0.5) / (1.0 + 2.0 * (-residual.to_secs() / 0.005).exp())
}

pub struct PlaybackParams {
    pub delay: GameDt,
    pub max_overtake: GameDt,
    pub max_sample_age: LocalDt,
}

pub struct PlaybackClock {
    pub params: PlaybackParams,
    clock: LocalClock,
    stream_samples: Samples<GameTime>,
    playback_time: GameTime,
}

impl PlaybackClock {
    pub fn new(params: PlaybackParams, clock: LocalClock) -> Self {
        let stream_samples = Samples::new(LocalDt::from_secs(1.0), clock.clone());

        Self {
            params,
            clock,
            stream_samples,
            playback_time: GameTime::zero(),
        }
    }

    pub fn playback_time(&self) -> GameTime {
        self.playback_time
    }

    pub fn record_stream_time(&mut self, receive_time: LocalTime, stream_time: GameTime) {
        self.stream_samples.record(receive_time, stream_time);
    }

    pub fn advance(&mut self, dt: LocalDt) {
        let stream_time = predict_stream_time(&self.stream_samples, self.clock.local_time())
            .unwrap_or(GameTime::zero());
        let target_time = stream_time - self.params.delay;
        let residual = target_time - self.playback_time;
        let max_stream_time = self
            .stream_samples
            .iter()
            .map(|(_, stream_time)| *stream_time)
            .max_by(|time1, time2| time1.partial_cmp(time2).unwrap())
            .unwrap_or(GameTime::zero());
        let max_playback_time = max_stream_time + self.params.max_overtake;

        self.playback_time += dt.to_game_dt() * time_warp(residual);
        self.playback_time = self.playback_time.min(max_playback_time);
    }
}
