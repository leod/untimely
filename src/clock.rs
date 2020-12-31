use std::collections::VecDeque;

use pareen::{simple_linear_regression, Anim, Fun};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct GameTime(f64);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct LocalTime(f64);

impl From<f64> for GameTime {
    fn from(t: f64) -> Self {
        GameTime(t)
    }
}

impl From<f64> for LocalTime {
    fn from(t: f64) -> Self {
        LocalTime(t)
    }
}

impl From<GameTime> for f64 {
    fn from(t: GameTime) -> Self {
        t.0
    }
}

impl From<LocalTime> for f64 {
    fn from(t: LocalTime) -> Self {
        t.0
    }
}

pub struct TimeMapping<Src, Tgt> {
    max_evidence_len: usize,
    evidence: VecDeque<(Src, Tgt)>,
}

impl<Src, Tgt> TimeMapping<Src, Tgt> {
    pub fn new(max_evidence_len: usize) -> Self {
        TimeMapping {
            max_evidence_len,
            evidence: VecDeque::new(),
        }
    }

    pub fn record_evidence(&mut self, src_time: Src, tgt_time: Tgt) {
        self.evidence.push_back((src_time, tgt_time));
        if self.evidence.len() > self.max_evidence_len {
            self.evidence.pop_front();
        }
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
            let line = simple_linear_regression(values);
            Some(line.eval(t.into()).into())
        } else {
            None
        }
    }
}

pub struct PlaybackClock {}
