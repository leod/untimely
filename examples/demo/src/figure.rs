use untimely::LocalDt;

pub trait Figure {
    fn update(&mut self, dt: LocalDt);
    fn draw(&mut self) -> Result<(), malen::Error>;
}
