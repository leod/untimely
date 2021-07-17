use std::{
    marker::PhantomData,
    ops::{Add, Mul, Neg, Sub},
};

pub trait TimeTag: PartialOrd + Copy {
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Dt<Tag>(f64, PhantomData<Tag>);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Time<Tag>(Dt<Tag>);

pub struct LocalTag;
pub struct GameTag;

pub type GameTime = Time<GameTag>;
pub type GameDt = Dt<GameTag>;

pub type LocalTime = Time<LocalTag>;
pub type LocalDt = Dt<LocalTag>;

impl<Tag> Dt<Tag> {
    pub fn from_secs(secs: f64) -> Self {
        Dt(secs, PhantomData)
    }

    pub fn to_secs(self) -> f64 {
        self.0
    }
}

impl<Tag> Time<Tag> {
    pub fn from_secs(secs: f64) -> Self {
        Time(Dt::from_secs(secs))
    }

    pub fn to_secs(self) -> f64 {
        (self.0).0
    }
}

impl<Tag> Add<Dt<Tag>> for Time<Tag> {
    type Output = Self;

    fn add(self, rhs: Dt<Tag>) -> Self {
        Time(Dt::from_secs((self.0).0 + rhs.0))
    }
}

impl<Tag> Sub<Dt<Tag>> for Time<Tag> {
    type Output = Self;

    fn sub(self, rhs: Dt<Tag>) -> Self {
        Time(Dt::from_secs((self.0).0 - rhs.0))
    }
}

impl<Tag> Sub<Time<Tag>> for Time<Tag> {
    type Output = Dt<Tag>;

    fn sub(self, rhs: Time<Tag>) -> Dt<Tag> {
        Dt::from_secs((self.0).0 - (rhs.0).0)
    }
}

impl<Tag> Add<Dt<Tag>> for Dt<Tag> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Dt::from_secs(self.0 + rhs.0)
    }
}

impl<Tag> Sub<Dt<Tag>> for Dt<Tag> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Dt::from_secs(self.0 - rhs.0)
    }
}

impl<Tag> Mul<f64> for Dt<Tag> {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        Dt::from_secs(self.0 * rhs)
    }
}

impl<Tag> Neg for Dt<Tag> {
    type Output = Self;

    fn neg(self) -> Self {
        self * -1.0
    }
}
