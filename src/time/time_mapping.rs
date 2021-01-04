use std::collections::VecDeque;

use pareen::{simple_linear_regression_with_slope, Fun};

use crate::GameTimeDelta;

#[derive(Debug, Clone)]
pub struct TimeMappingConfig {
    pub max_evidence_len: usize,
    pub tick_time_delta: GameTimeDelta,
}

#[derive(Debug, Clone)]
pub struct TimeMapping<Src, Tgt> {
    config: TimeMappingConfig,
    evidence: VecDeque<(Src, Tgt)>,
}

impl<Src, Tgt> TimeMapping<Src, Tgt> {
    pub fn new(config: TimeMappingConfig) -> Self {
        TimeMapping {
            config,
            evidence: VecDeque::new(),
        }
    }

    pub fn from_evidence<I>(config: TimeMappingConfig, evidence: I) -> Self
    where
        I: IntoIterator<Item = (Src, Tgt)>,
    {
        let mut mapping = Self::new(config);
        for (src_time, tgt_time) in evidence {
            mapping.record_evidence(src_time, tgt_time);
        }
        mapping
    }

    pub fn record_evidence(&mut self, src_time: Src, tgt_time: Tgt) {
        self.evidence.push_back((src_time, tgt_time));
        if self.evidence.len() > self.config.max_evidence_len {
            self.evidence.pop_front();
        }
    }
}

impl<Src, Tgt> TimeMapping<Src, Tgt>
where
    Src: Into<f64> + Clone,
    Tgt: Into<f64> + Clone,
    f64: Into<Tgt>,
{
    pub fn eval(&self, t: Src) -> Option<Tgt> {
        Fun::eval(self, t)
    }
}

impl<Src, Tgt> TimeMapping<Src, Tgt>
where
    Src: Into<f64> + Clone,
{
    pub fn prune_evidence_older_than(&mut self, min_src_time: Src) {
        let min_src_time = min_src_time.into();
        self.evidence
            .retain(|(src_time, _)| src_time.clone().into() >= min_src_time);
    }
}

impl<Src, Tgt> Fun for TimeMapping<Src, Tgt>
where
    Src: Into<f64> + Clone,
    Tgt: Into<f64> + Clone,
    f64: Into<Tgt>,
{
    type T = Src;
    type V = Option<Tgt>;

    fn eval(&self, t: Src) -> Option<Tgt> {
        if self.evidence.len() >= 2 {
            let (front, back) = self.evidence.as_slices();
            let values = pareen::slice(front)
                .seq_with_dur(pareen::slice(back))
                .map(|(src_time, tgt_time)| (src_time.into(), tgt_time.into()));
            let line = simple_linear_regression_with_slope(1.0, values);
            Some(line.eval(t.into()).into())
        } else {
            None
        }
    }
}
