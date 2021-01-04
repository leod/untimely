macro_rules! impl_time_type {
    ($time:ident, $delta:ident) => {
        #[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
        pub struct $delta(f64);

        #[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
        pub struct $time($delta);

        impl $delta {
            pub const ZERO: $delta = $delta(0.0);

            pub fn from_secs(secs: f64) -> Self {
                assert!(secs.is_finite());
                $delta(secs)
            }

            pub fn from_mins(mins: f64) -> Self {
                Self::from_secs(mins * 60.0)
            }

            pub fn from_millis(millis: f64) -> Self {
                Self::from_secs(millis / 1000.0)
            }

            pub fn to_secs(self) -> f64 {
                self.0
            }

            pub fn to_mins(self) -> f64 {
                self.0 / 60.0
            }

            pub fn to_millis(self) -> f64 {
                self.0 * 1000.0
            }

            pub fn min(self, other: Self) -> Self {
                $delta(self.0.max(other.0))
            }

            pub fn max(self, other: Self) -> Self {
                $delta(self.0.max(other.0))
            }
        }

        impl $time {
            pub const ZERO: $time = $time($delta(0.0));

            pub fn from_secs_since_start(secs_since_start: f64) -> Self {
                $time($delta::from_secs(secs_since_start))
            }

            pub fn from_mins_since_start(mins_since_start: f64) -> Self {
                $time($delta::from_mins(mins_since_start))
            }

            pub fn from_millis_since_start(millis_since_start: f64) -> Self {
                $time($delta::from_millis(millis_since_start))
            }

            pub fn to_secs_since_start(self) -> f64 {
                self.0.to_secs()
            }

            pub fn to_mins_since_start(self) -> f64 {
                self.0.to_mins()
            }

            pub fn to_millis_since_start(self) -> f64 {
                self.0.to_millis()
            }

            pub fn to_delta(self) -> $delta {
                self.0
            }

            pub fn min(self, other: Self) -> Self {
                $time(self.0.max(other.0))
            }

            pub fn max(self, other: Self) -> Self {
                $time(self.0.max(other.0))
            }
        }

        impl From<f64> for $delta {
            fn from(secs: f64) -> Self {
                assert!(secs.is_finite());
                $delta::from_secs(secs)
            }
        }

        impl From<$delta> for f64 {
            fn from(delta: $delta) -> Self {
                delta.0
            }
        }

        impl From<f64> for $time {
            fn from(secs_since_start: f64) -> Self {
                $time($delta::from_secs(secs_since_start))
            }
        }

        impl From<$time> for f64 {
            fn from(delta: $time) -> Self {
                (delta.0).0
            }
        }

        impl std::ops::Add<$delta> for $time {
            type Output = Self;

            fn add(self, other: $delta) -> Self {
                $time(self.0 + other)
            }
        }

        impl std::ops::AddAssign<$delta> for $time {
            fn add_assign(&mut self, other: $delta) {
                *self = *self + other;
            }
        }

        impl std::ops::Sub<$delta> for $time {
            type Output = Self;

            fn sub(self, other: $delta) -> Self {
                $time(self.0 - other)
            }
        }

        impl std::ops::Sub<$time> for $time {
            type Output = $delta;

            fn sub(self, other: $time) -> $delta {
                self.0 - other.0
            }
        }

        impl std::ops::Sub<$delta> for $delta {
            type Output = $delta;

            fn sub(self, other: $delta) -> Self {
                $delta(self.0 - other.0)
            }
        }

        impl std::ops::Add<$delta> for $delta {
            type Output = $delta;

            fn add(self, other: $delta) -> Self {
                $delta(self.0 + other.0)
            }
        }

        impl std::ops::Mul<$delta> for f64 {
            type Output = $delta;

            fn mul(self, other: $delta) -> $delta {
                $delta(self * other.0)
            }
        }

        impl std::ops::Mul<f64> for $delta {
            type Output = $delta;

            fn mul(self, other: f64) -> $delta {
                other * self
            }
        }
    };
}

impl_time_type!(GameTime, GameTimeDelta);
impl_time_type!(LocalTime, LocalTimeDelta);

impl LocalTimeDelta {
    pub fn to_game_time_delta(&self) -> GameTimeDelta {
        GameTimeDelta::from_secs(self.to_secs())
    }
}

impl GameTimeDelta {
    pub fn to_local_time_delta(&self) -> LocalTimeDelta {
        LocalTimeDelta::from_secs(self.to_secs())
    }
}
