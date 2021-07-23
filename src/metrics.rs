use std::collections::BTreeMap;

use crate::{LocalClock, LocalDt, Samples};

#[derive(Debug, Clone)]
pub struct Gauge {
    samples: Samples<f64>,
}

impl Gauge {
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn mean(&self) -> f64 {
        self.samples.values().sum::<f64>() / self.samples.len() as f64
    }

    pub fn std(&self) -> f64 {
        // FIXME: Numerical stability
        let mean = self.mean();
        let var = self
            .samples
            .values()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / self.samples.len() as f64;

        var.sqrt()
    }

    pub fn min(&self) -> f64 {
        self.samples
            .values()
            .copied()
            .min_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap_or(f64::NAN)
    }

    pub fn max(&self) -> f64 {
        self.samples
            .values()
            .copied()
            .max_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap_or(f64::NAN)
    }

    pub fn plot_points(&self) -> Vec<(f64, f64)> {
        self.samples
            .iter()
            .map(|(time, value)| (time.to_secs(), *value))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Metrics {
    max_sample_age: LocalDt,
    clock: LocalClock,
    gauges: BTreeMap<String, Gauge>,
}

impl Metrics {
    pub fn new(max_sample_age: LocalDt, clock: LocalClock) -> Self {
        Self {
            max_sample_age,
            clock,
            gauges: BTreeMap::new(),
        }
    }

    pub fn gauges(&self) -> impl Iterator<Item = (&String, &Gauge)> {
        self.gauges.iter()
    }

    pub fn gauge_mut(&mut self, name: &str) -> &mut Gauge {
        let max_sample_age = self.max_sample_age;
        let clock = self.clock.clone();

        self.gauges
            .entry(name.to_string())
            .or_insert_with(|| Gauge {
                samples: Samples::new(max_sample_age, clock),
            })
    }

    pub fn get_gauge(&self, name: &str) -> Option<&Gauge> {
        self.gauges.get(name)
    }

    pub fn record_gauge(&mut self, name: &str, value: f64) {
        let time = self.clock.local_time();
        self.gauge_mut(name).samples.record(time, value);
    }
}
