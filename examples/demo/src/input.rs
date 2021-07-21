use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use malen::{InputState, Key};

use untimely::{GameTime, PlayerId};

use crate::GameInput;

pub fn current_game_input(id: PlayerId, time: GameTime, input_state: &InputState) -> GameInput {
    if id == PlayerId(0) {
        GameInput {
            left: input_state.key(Key::A),
            right: input_state.key(Key::D),
            up: input_state.key(Key::W),
            down: input_state.key(Key::S),
            shoot: input_state.key(Key::Space),
        }
    } else {
        let up = GameInput {
            up: true,
            ..GameInput::default()
        };
        let down = GameInput {
            down: true,
            ..GameInput::default()
        };
        let none = GameInput::default();

        let head = pareen::constant(down.clone()).dur(0.35);
        let tail = pareen::seq_with_dur!(
            pareen::constant(none.clone()).dur(0.5),
            pareen::constant(up).dur(0.7),
            pareen::constant(none).dur(0.5),
            pareen::constant(down).dur(0.7),
        );

        head.seq(tail.repeat()).eval(time.to_secs())
    }
}

pub fn get_param(element_id: &str) -> f64 {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let element = document.get_element_by_id(element_id).unwrap();
    let input = element.dyn_into::<HtmlInputElement>().unwrap();
    input.value_as_number()
}
