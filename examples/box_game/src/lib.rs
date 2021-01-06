mod draw;
mod game;

use nalgebra::Point2;
use wasm_bindgen::prelude::wasm_bindgen;

use malen::{Camera, Canvas, Color4, InputState, Key};

use untimely::PlayerId;

use draw::DrawGame;
use game::{Game, Input};

fn current_game_input(input_state: &InputState) -> Input {
    Input {
        left: input_state.key(Key::A),
        right: input_state.key(Key::D),
        up: input_state.key(Key::W),
        down: input_state.key(Key::S),
        shoot: input_state.key(Key::Space),
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    log::info!("Hi, starting the example");

    let mut canvas = Canvas::from_element_id("canvas").unwrap();
    log::info!("Initialized malen context");

    let mut draw_game = DrawGame::new(&canvas).unwrap();

    let mut game = Game::new();

    malen::main_loop(move |dt, _running| {
        while let Some(_) = canvas.pop_event() {}

        let input = current_game_input(canvas.input_state());
        game.run_input(PlayerId(0), &input);

        let screen_geom = canvas.screen_geom();
        canvas.clear(Color4::new(0.0, 0.0, 0.0, 1.0));
        let camera = Camera {
            center: Point2::origin(),
            angle: 0.0,
            zoom: 1.0,
        };
        draw_game.draw(&canvas, &game, &camera).unwrap();
    })
    .unwrap();
}
