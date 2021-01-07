mod example;
mod example_00_local;
mod example_01_server_client;

pub(self) mod util;

pub use example::Example;

pub fn new_examples() -> Result<Vec<Box<dyn Example>>, malen::Error> {
    Ok(vec![
        Box::new(example_00_local::MyExample::new()?),
        Box::new(example_01_server_client::MyExample::new()?),
    ])
}
