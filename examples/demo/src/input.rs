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
