mod figure1;
mod figure2;
mod figure3;

use crate::Figure;

pub fn figures() -> Result<Vec<Box<dyn Figure>>, malen::Error> {
    Ok(vec![
        Box::new(figure1::Figure1::new()?),
        Box::new(figure2::Figure2::new()?),
        //Box::new(figure3::Figure3::new()?),
    ])
}
