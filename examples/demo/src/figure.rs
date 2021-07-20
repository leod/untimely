use untimely::{LocalDt, LocalTime};

pub trait Figure {
    fn update(&mut self, time: LocalTime, dt: LocalDt);
    fn draw(&mut self) -> Result<(), malen::Error>;
    fn is_active(&self) -> bool;
}
