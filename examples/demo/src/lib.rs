mod draw;
mod figure;
mod figures;
mod game;
mod input;

use wasm_bindgen::prelude::wasm_bindgen;

use untimely::{LocalDt, LocalTime};

pub use draw::DrawGame;
pub use figure::Figure;
pub use game::{Game, GameInput, GameParams};
pub use input::{current_game_input, get_param};

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    log::info!("Hi, starting the demo");

    let mut figures = figures::figures().unwrap();
    let mut prev_time = None;

    malen::main_loop(move |timestamp_secs, _running| {
        let time = LocalTime::from_secs(timestamp_secs);
        let dt = prev_time.map_or(LocalDt::zero(), |prev_time| time - prev_time);
        let dt = dt.min(LocalDt::from_secs(1.0)).max(LocalDt::from_secs(0.0));
        prev_time = Some(time);

        for figure in figures.iter_mut() {
            if figure.is_active() {
                figure.update(time, dt);
            }
        }

        for figure in figures.iter_mut() {
            figure.draw().unwrap();
        }
    })
    .unwrap();
}
