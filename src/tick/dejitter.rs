use super::TickTime;
use crate::{time::predict_stream_time, LocalClock, LocalDt, LocalTime, Samples, TickNum};

#[derive(Debug, Clone)]
pub struct DejitterBuffer<T> {
    delay: LocalDt,
    clock: LocalClock,
    time_samples: Samples<TickTime>,
    buffer: Vec<(TickNum, T)>,
    last_popped_num: Option<TickNum>,
}

impl<T> DejitterBuffer<T> {
    pub fn new(delay: LocalDt, max_sample_age: LocalDt, clock: LocalClock) -> Self {
        Self {
            delay,
            clock: clock.clone(),
            time_samples: Samples::new(max_sample_age, clock.clone()),
            buffer: Vec::new(),
            last_popped_num: None,
        }
    }

    pub fn delay(&self) -> LocalDt {
        self.delay
    }

    pub fn set_delay(&mut self, delay: LocalDt) {
        self.delay = delay;
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn insert(&mut self, receive_time: LocalTime, receive_num: TickNum, value: T) {
        let is_outdated = self
            .last_popped_num
            .map_or(false, |last_popped_num| receive_num <= last_popped_num);
        if is_outdated {
            return;
        }

        self.time_samples
            .record(receive_time, receive_num.to_tick_time());

        match self
            .buffer
            .binary_search_by(|(tick_num, _)| receive_num.cmp(tick_num))
        {
            Ok(_) => {
                // Ignore duplicate tick.
                return;
            }
            Err(index) => {
                self.buffer.insert(index, (receive_num, value));
            }
        }
    }

    pub fn pop(&mut self) -> Option<(TickNum, T)> {
        let delayed_time = self.clock.local_time() - self.delay;
        let stream_num =
            predict_stream_time(&self.time_samples, delayed_time).map(TickNum::from_tick_time);

        let oldest_item_is_ready = self.buffer.last().map_or(false, |(oldest_num, _)| {
            stream_num.map_or(false, |stream_num| stream_num >= *oldest_num)
        });

        if oldest_item_is_ready {
            let (oldest_num, oldest_value) = self.buffer.pop().unwrap();

            assert!(self
                .last_popped_num
                .map_or(true, |last_popped_num| last_popped_num < oldest_num));
            self.last_popped_num = Some(oldest_num);

            Some((oldest_num, oldest_value))
        } else {
            None
        }
    }
}
