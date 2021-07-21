use std::collections::BTreeMap;

use crate::{time::LocalTag, LocalDt, LocalTime, Samples};

#[derive(Debug, Clone, Default)]
pub struct Gauge {
    samples: Samples<LocalTag, f64>,
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
    time: LocalTime,
    gauges: BTreeMap<String, Gauge>,
}

impl Metrics {
    pub fn new(max_sample_age: LocalDt) -> Self {
        Self {
            max_sample_age,
            time: LocalTime::zero(),
            gauges: BTreeMap::new(),
        }
    }

    pub fn time(&self) -> LocalTime {
        self.time
    }

    pub fn advance(&mut self, dt: LocalDt) {
        self.time += dt;

        for gauge in self.gauges.values_mut() {
            gauge
                .samples
                .retain_recent_samples(self.time - self.max_sample_age);
        }
    }

    pub fn gauges(&self) -> impl Iterator<Item = (&String, &Gauge)> {
        self.gauges.iter()
    }

    pub fn gauge_mut(&mut self, name: &str) -> &mut Gauge {
        self.gauges
            .entry(name.to_string())
            .or_insert(Gauge::default())
    }

    pub fn get_gauge(&self, name: &str) -> Option<&Gauge> {
        self.gauges.get(name)
    }

    pub fn record_gauge(&mut self, name: &str, value: f64) {
        let time = self.time;
        self.gauge_mut(name).samples.record_sample(time, value);
    }
}
