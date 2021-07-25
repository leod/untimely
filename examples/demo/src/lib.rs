mod draw;
mod figure;
mod figures;
mod game;
mod input;

use wasm_bindgen::prelude::wasm_bindgen;

use untimely::{LocalClock, LocalDt, LocalTime};

pub use draw::DrawGame;
pub use figure::Figure;
pub use game::{Game, GameInput, GameParams};
pub use input::{current_game_input, get_socket_params, is_active};

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    log::info!("Hi, starting the demo");

    let mut clock = LocalClock::new();
    let mut figures = figures::figures().unwrap();

    malen::main_loop(move |timestamp_secs, _running| {
        let time = LocalTime::from_secs(timestamp_secs);
        let dt = clock.set_local_time(time).min(LocalDt::from_secs(1.0));

        for figure in figures.iter_mut() {
            figure.update(dt);
        }

        for figure in figures.iter_mut() {
            figure.draw().unwrap();
        }
    })
    .unwrap();
}
