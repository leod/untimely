use untimely::LocalTimeDelta;

pub trait Example {
    fn update(&mut self, dt: LocalTimeDelta);
    fn draw(&mut self) -> Result<(), malen::Error>;
    fn is_active(&self) -> bool;
}
