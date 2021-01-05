use wasm_bindgen::prelude::wasm_bindgen;

use malen::Canvas;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    log::info!("Hi, starting the example");

    let mut canvas = Canvas::from_element_id("canvas").unwrap();
    log::info!("Initialized malen context");

    malen::main_loop(move |dt, _running| {}).unwrap();
}
