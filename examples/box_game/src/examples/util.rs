use malen::{InputState, Key};

use crate::game::Input;

pub fn current_game_input(input_state: &InputState) -> Input {
    Input {
        left: input_state.key(Key::A),
        right: input_state.key(Key::D),
        up: input_state.key(Key::W),
        down: input_state.key(Key::S),
        shoot: input_state.key(Key::Space),
    }
}
