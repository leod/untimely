use super::TickTime;
use crate::{time, LocalClock, LocalDt, LocalTime, Samples, TickNum};

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

    pub fn insert(&mut self, receive_time: LocalTime, receive_num: TickNum, value: T) {
        let is_outdated = self
            .last_popped_num
            .map_or(false, |last_popped_num| receive_num <= last_popped_num);
        if is_outdated {
            return;
        }

        self.time_samples.record(receive_time, receive_num.to_tick_time());

        match self
            .buffer
            .binary_search_by_key(&receive_num, |(tick_num, _)| *tick_num)
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
        let stream_num = time::predict_stream_time(&self.time_samples, self.clock.local_time())
            .map(|stream_time| TickNum::from_tick_time(stream_time));

        let oldest_item_is_ready = self.buffer.first().map_or(false, |(oldest_num, _)| {
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

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}
