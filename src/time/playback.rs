use super::{predict_stream_time, GameDt, GameTime, LocalDt, LocalTag, LocalTime, Samples};

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

    stream_samples: Samples<LocalTag, GameTime>,

    local_time: LocalTime,
    playback_time: GameTime,
}

impl PlaybackClock {
    pub fn new(params: PlaybackParams, local_time: LocalTime) -> Self {
        PlaybackClock {
            params,
            stream_samples: Samples::new(),
            local_time,
            playback_time: GameTime::zero(),
        }
    }

    pub fn local_time(&self) -> LocalTime {
        self.local_time
    }

    pub fn playback_time(&self) -> GameTime {
        self.playback_time
    }

    pub fn record_stream_sample(&mut self, local_time: LocalTime, stream_time: GameTime) {
        self.stream_samples.record_sample(local_time, stream_time);
        self.stream_samples
            .retain_recent_samples(local_time - self.params.max_sample_age);
    }

    pub fn advance(&mut self, dt: LocalDt) {
        let stream_time =
            predict_stream_time(&self.stream_samples, self.local_time).unwrap_or(GameTime::zero());
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

        self.local_time += dt;
    }
}
