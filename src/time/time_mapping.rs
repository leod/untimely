use std::collections::VecDeque;

use pareen::{simple_linear_regression_with_slope, Fun};

use crate::GameTimeDelta;

#[derive(Debug, Clone)]
pub struct TimeMappingConfig {
    pub max_evidence_len: usize,
    pub tick_time_delta: GameTimeDelta,
    pub ignore_if_out_of_order: bool,
    pub ema_alpha: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct TimeMapping<Src, Tgt> {
    config: TimeMappingConfig,
    evidence: VecDeque<(Src, Tgt)>,
    line: Option<pareen::stats::Line<f64>>,
}

impl<Src, Tgt> TimeMapping<Src, Tgt>
where
    Src: Into<f64> + PartialOrd + Clone,
    Tgt: Into<f64> + PartialOrd + Clone,
    f64: Into<Tgt>,
{
    pub fn new(config: TimeMappingConfig) -> Self {
        assert!(config.tick_time_delta > GameTimeDelta::ZERO);

        TimeMapping {
            config,
            evidence: VecDeque::new(),
            line: None,
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
        if self.config.ignore_if_out_of_order {
            let is_in_order =
                self.evidence
                    .back()
                    .map_or(true, |(newest_src_time, newest_tgt_time)| {
                        src_time > newest_src_time.clone() && tgt_time > newest_tgt_time.clone()
                    });
            if !is_in_order {
                return;
            }
        }

        self.evidence.push_back((src_time, tgt_time));
        if self.evidence.len() > self.config.max_evidence_len {
            self.evidence.pop_front();
        }
    }

    fn prune_evidence_older_than(&mut self, min_src_time: Src) {
        let min_src_time = min_src_time.into();
        self.evidence
            .retain(|(src_time, _)| src_time.clone().into() >= min_src_time);
    }

    pub fn update(&mut self, cur_src_time: Src) {
        self.line = if self.evidence.len() >= 2 {
            let (front, back) = self.evidence.as_slices();
            let values = pareen::slice(front)
                .seq_with_dur(pareen::slice(back))
                .map(|(src_time, tgt_time)| (src_time.into(), tgt_time.into()));
            let line = simple_linear_regression_with_slope(1.0, values);
            //let line = pareen::simple_linear_regression(values);
            if let (Some(ema_alpha), Some(old_line)) = (self.config.ema_alpha, &self.line) {
                Some(pareen::stats::Line {
                    slope: 1.0,
                    y_intercept: ema_alpha * line.0.y_intercept
                        + (1.0 - ema_alpha) * old_line.y_intercept,
                })
            } else {
                Some(line.0)
            }
        } else {
            None
        }
    }

    pub fn eval(&self, t: Src) -> Option<Tgt> {
        Fun::eval(self, t)
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
        pareen::Anim(&self.line).eval(t.into()).map(f64::into)
    }
}
