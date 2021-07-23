use untimely::LocalTime;

pub trait Figure {
    fn update(&mut self, time: LocalTime);
    fn draw(&mut self) -> Result<(), malen::Error>;
    fn is_active(&self) -> bool;
}
