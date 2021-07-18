mod figure1;

use crate::Figure;

pub fn figures() -> Result<Vec<Box<dyn Figure>>, malen::Error> {
    Ok(vec![Box::new(figure1::Figure1::new()?)])
}
