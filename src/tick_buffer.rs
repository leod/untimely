use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TickNum(pub u32);

pub struct SenderTickBuffer<T> {
    last_client_ack_num: Option<TickNum>,
    sent_ticks: BTreeMap<TickNum, T>,
}

impl<T> SenderTickBuffer<T> {
    pub fn record_client_ack(&mut self, num: TickNum) {
        self.last_client_ack_num = self.last_client_ack_num.max(Some(num));
    }

    pub fn record_sent_tick(&mut self, num: TickNum, tick: T) {
        // We should never send the same tick more than once.
        assert!(!self.sent_ticks.contains_key(&num));

        self.sent_ticks.insert(num, tick);
    }

    pub fn get_last_client_ack_num(&self) -> Option<TickNum> {
        self.last_client_ack_num
    }

    pub fn get_last_client_ack_tick(&self) -> Option<&T> {
        self.last_client_ack_num
            .and_then(|num| self.sent_ticks.get(&num))
    }
}

pub struct ReceiverTickBuffer<T> {
    received_ticks: BTreeMap<TickNum, T>,
}
