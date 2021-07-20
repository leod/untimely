use malen::{InputState, Key};

use crate::GameInput;

pub fn current_game_input(input_state: &InputState) -> GameInput {
    GameInput {
        left: input_state.key(Key::A),
        right: input_state.key(Key::D),
        up: input_state.key(Key::W),
        down: input_state.key(Key::S),
        shoot: input_state.key(Key::Space),
    }
}

pub fn get_param(element_id: &str) -> f64 {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let element = document.get_element_by_id(element_id).unwrap();
    let value = element.get_attribute("value").unwrap();
    value.parse().unwrap()
}
