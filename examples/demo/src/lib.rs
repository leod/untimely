mod draw;
mod figure;
mod figures;
mod game;
mod input;

use wasm_bindgen::prelude::wasm_bindgen;

use untimely::LocalDt;

pub use draw::DrawGame;
pub use figure::Figure;
pub use game::{Game, GameInput};
pub use input::current_game_input;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    log::info!("Hi, starting the demo");

    let mut figures = figures::figures().unwrap();

    malen::main_loop(move |dt, _running| {
        let dt = LocalDt::from_duration(dt);
        let dt = dt.min(LocalDt::from_secs(1.0));

        for figure in figures.iter_mut() {
            if figure.is_active() {
                figure.update(dt);
            }
        }

        for figure in figures.iter_mut() {
            figure.draw().unwrap();
        }
    })
    .unwrap();
}
