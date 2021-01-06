mod example;
mod example_00_local;

pub(self) mod util;

pub use example::Example;

pub fn new_examples() -> Result<Vec<Box<dyn Example>>, malen::Error> {
    Ok(vec![Box::new(example_00_local::MyExample::new()?)])
}
