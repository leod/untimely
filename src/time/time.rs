use std::{
    cmp::Ordering,
    marker::PhantomData,
    ops::{Add, AddAssign, Mul, Neg, Sub},
};

pub trait TimeTag: Copy {}

#[derive(Debug, Copy)]
pub struct Dt<Tag>(f64, PhantomData<Tag>);

#[derive(Debug, Copy)]
pub struct Time<Tag>(Dt<Tag>);

#[derive(Debug, Clone, Copy)]
pub struct LocalTag;

#[derive(Debug, Clone, Copy)]
pub struct GameTag;

impl TimeTag for LocalTag {}
impl TimeTag for GameTag {}

pub type GameTime = Time<GameTag>;
pub type GameDt = Dt<GameTag>;

pub type LocalTime = Time<LocalTag>;
pub type LocalDt = Dt<LocalTag>;

impl<Tag> Dt<Tag> {
    pub fn from_secs(secs: f64) -> Self {
        Dt(secs, PhantomData)
    }

    pub fn from_millis(millis: f64) -> Self {
        Self::from_secs(millis / 1000.0)
    }

    pub fn zero() -> Self {
        Self::from_secs(0.0)
    }

    pub fn to_secs(self) -> f64 {
        self.0
    }

    pub fn max(self, rhs: Self) -> Self {
        Self::from_secs(self.to_secs().max(rhs.to_secs()))
    }

    pub fn min(self, rhs: Self) -> Self {
        Self::from_secs(self.to_secs().min(rhs.to_secs()))
    }
}

impl LocalDt {
    pub fn to_game_dt(self) -> GameDt {
        GameDt::from_secs(self.to_secs())
    }
}

impl<Tag> Time<Tag> {
    pub fn from_dt(dt: Dt<Tag>) -> Self {
        Time(dt)
    }

    pub fn from_secs(secs: f64) -> Self {
        Self::from_dt(Dt::from_secs(secs))
    }

    pub fn zero() -> Self {
        Self::from_dt(Dt::zero())
    }

    pub fn to_dt(self) -> Dt<Tag> {
        self.0
    }

    pub fn to_secs(self) -> f64 {
        self.to_dt().to_secs()
    }

    pub fn max(self, rhs: Self) -> Self {
        Self::from_dt(self.to_dt().max(rhs.to_dt()))
    }

    pub fn min(self, rhs: Self) -> Self {
        Self::from_dt(self.to_dt().min(rhs.to_dt()))
    }
}

// Due to the Tag type parameter, derive does not work, so we need to derive
// some trait implementations by hand.

impl<Tag> Clone for Dt<Tag> {
    fn clone(&self) -> Self {
        self.clone()
    }
}

impl<Tag> Clone for Time<Tag> {
    fn clone(&self) -> Self {
        self.clone()
    }
}

impl<Tag> Default for Dt<Tag> {
    fn default() -> Self {
        Self::zero()
    }
}

impl<Tag> Default for Time<Tag> {
    fn default() -> Self {
        Self::zero()
    }
}

impl<Tag> PartialEq for Dt<Tag> {
    fn eq(&self, other: &Dt<Tag>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<Tag> PartialEq for Time<Tag> {
    fn eq(&self, other: &Time<Tag>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<Tag> PartialOrd for Dt<Tag> {
    fn partial_cmp(&self, other: &Dt<Tag>) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<Tag> PartialOrd for Time<Tag> {
    fn partial_cmp(&self, other: &Time<Tag>) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<Tag> Add<Dt<Tag>> for Time<Tag> {
    type Output = Self;

    fn add(self, rhs: Dt<Tag>) -> Self {
        Time(Dt::from_secs((self.0).0 + rhs.0))
    }
}

impl<Tag> AddAssign<Dt<Tag>> for Time<Tag>
where
    Tag: Copy,
{
    fn add_assign(&mut self, rhs: Dt<Tag>) {
        self.0 = self.0 + rhs;
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
