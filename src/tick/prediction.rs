use std::collections::BTreeMap;

use super::TickNum;

pub trait Predictable: Clone {
    type Input: Clone;

    fn run_input(&mut self, input: &Self::Input);
    fn apply_correction(&mut self, reference: &Self);
}

#[derive(Debug, Default)]
pub struct Prediction<P: Predictable> {
    last_input: P::Input,
    predicted_state: P,
}

#[derive(Default)]
pub struct ClientSidePrediction<P: Predictable> {
    predictions: BTreeMap<TickNum, Prediction<P>>,
}

impl<P: Predictable> ClientSidePrediction<P> {
    pub fn new() -> Self {
        Self {
            predictions: BTreeMap::new(),
        }
    }

    pub fn start_tick(
        &mut self, 
        tick_num: TickNum,
        input: &P::Input,
        reference: Option<&P>,
        my_last_input_num: Option<TickNum>,
    ) {
        if self.predictions.contains_key(&tick_num) {
            self.predictions.clear();
        }
    }
}