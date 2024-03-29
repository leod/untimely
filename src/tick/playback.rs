use crate::{
    GameDt, GameTime, LocalClock, LocalDt, LocalTime, Metrics, PlaybackClock, PlaybackClockParams,
};

#[derive(Debug, Clone)]
pub struct TickPlaybackParams {
    pub playback_clock_params: PlaybackClockParams,
    pub max_residual: GameDt,
}

#[derive(Debug, Clone)]
pub struct Interpolation<'a, T> {
    pub current_time: GameTime,
    pub current_value: &'a T,
    pub next_time: GameTime,
    pub next_value: &'a T,
    pub alpha: f64,
}

#[derive(Debug, Clone)]
pub struct TickPlayback<T> {
    params: TickPlaybackParams,
    local_clock: LocalClock,
    playback_clock: PlaybackClock,
    ticks: Vec<(GameTime, T)>,
    current_tick: Option<(GameTime, T)>,
}

impl<T> TickPlayback<T>
where
    T: Clone,
{
    pub fn new(params: TickPlaybackParams, local_clock: LocalClock) -> Self {
        Self {
            params: params.clone(),
            local_clock: local_clock.clone(),
            playback_clock: PlaybackClock::new(params.playback_clock_params, local_clock),
            ticks: Vec::new(),
            current_tick: None,
        }
    }

    pub fn playback_clock(&self) -> &PlaybackClock {
        &self.playback_clock
    }

    pub fn playback_time(&self) -> GameTime {
        self.playback_clock.playback_time()
    }

    pub fn playback_clock_params(&self) -> &PlaybackClockParams {
        &self.playback_clock.params
    }

    pub fn playback_clock_params_mut(&mut self) -> &mut PlaybackClockParams {
        &mut self.playback_clock.params
    }

    pub fn current_tick(&self) -> Option<(GameTime, &T)> {
        self.current_tick
            .as_ref()
            .map(|(time, value)| (*time, value))
    }

    pub fn next_tick(&self) -> Option<(GameTime, &T)> {
        self.ticks.last().map(|(time, value)| (*time, value))
    }

    pub fn interpolation(&self) -> Option<Interpolation<T>> {
        self.current_tick()
            .and_then(|current_tick| self.next_tick().map(|next_tick| (current_tick, next_tick)))
            .map(|((current_time, current_value), (next_time, next_value))| {
                let alpha = (self.playback_time() - current_time) / (next_time - current_time);

                Interpolation {
                    current_time,
                    current_value,
                    next_time,
                    next_value,
                    alpha,
                }
            })
    }

    pub fn record_tick(
        &mut self,
        receive_time: LocalTime,
        receive_game_time: GameTime,
        receive_value: T,
    ) {
        self.playback_clock
            .record_stream_time(receive_time, receive_game_time);

        if receive_game_time < self.playback_clock.playback_time() {
            return;
        }

        match self
            .ticks
            .binary_search_by(|(game_time, _)| receive_game_time.partial_cmp(&game_time).unwrap())
        {
            Ok(_) => {
                // Ignore duplicate tick.
                return;
            }
            Err(index) => {
                self.ticks.insert(index, (receive_game_time, receive_value));
            }
        }
    }

    pub fn advance(&mut self, dt: LocalDt) -> Vec<(GameTime, T)> {
        let residual = self.playback_clock.advance(dt);

        if residual > self.params.max_residual {
            // We have trailed too far behind the tick stream. This can happen e.g. if there is a
            // large jump ahead in the local clock, but the local dt was smaller (e.g. because of
            // clipping the dt to a maximum).
            //
            // In this case, we simply jump to the time of the newest received tick.
            if let Some((newest_time, _)) = self.ticks.first() {
                log::info!(
                    "PlaybackClock at {:?} fell behind by {:?} (have {} ticks), jumping ahead to {:?}",
                    self.playback_clock.playback_time(),
                    residual,
                    self.ticks.len(),
                    newest_time,
                );

                self.playback_clock.set_playback_time(*newest_time);
            }

            while self.ticks.len() > 1 {
                self.ticks.pop();
            }
        }

        let mut started_ticks = Vec::new();

        while self.is_oldest_tick_ready() {
            let oldest_tick = self.ticks.pop().unwrap();

            started_ticks.push(oldest_tick.clone());
            self.current_tick = Some(oldest_tick);
        }

        started_ticks
    }

    pub fn record_metrics(&self, prefix: &str, metrics: &mut Metrics) {
        self.playback_clock.record_metrics(prefix, metrics);
    }

    fn is_oldest_tick_ready(&self) -> bool {
        self.ticks.last().map_or(false, |(oldest_time, _)| {
            self.playback_time() >= *oldest_time
        })
    }
}
