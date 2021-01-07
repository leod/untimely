mod draw;
mod examples;
mod game;

use wasm_bindgen::prelude::wasm_bindgen;

use untimely::LocalTimeDelta;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    log::info!("Hi, starting the example");

    let mut examples = examples::new_examples().unwrap();

    malen::main_loop(move |dt, _running| {
        let dt = LocalTimeDelta::from_duration(dt);
        let dt = dt.min(LocalTimeDelta::from_secs(10.0));

        for example in examples.iter_mut() {
            example.update(dt);
        }

        for example in examples.iter_mut() {
            example.draw().unwrap();
        }
    })
    .unwrap();
}
