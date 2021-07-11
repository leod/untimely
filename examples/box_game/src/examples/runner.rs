use malen::{Canvas, draw::plot::Plotting};
use untimely::channel::{SimNetChannel, LocalTime};

use crate::{game::GameInput, draw::DrawGame, examples::{UpdateParams, Example}};

#[derive(Clone)]
struct Conn {
    client_to_server: SimNetChannel<ClientMsg>,
    server_to_client: SimNetChannel<ServerMsg>,
}

#[derive(Clone)]
struct State<E: Example> {
    current_time: LocalTime,
    conns: BTreeMap<PlayerId, Conn>,
    state: State,
}

pub struct Runner<E: Example> {
    canvas: Canvas,
    draw_game: DrawGame,
    plotting: Plotting,
    state: State,
}

impl<E: Example> Runner<E> {
    pub fn new(element_id: &str) -> Result<Self, malen::Error> {
        let canvas = Canvas::from_element_id(element_id)?;
        let draw_game = DrawGame::new(&canvas)?;
        let plotting = Plotting::new(&canvas)?;
        let state = E::default();
        let state = State {
            current_time: LocalTime::ZERO,
            client_to_server: SimNetChannel::new(NetProfile::wonky_slow_profile()),
            server_to_client: SimNetChannel::new(NetProfile::wonky_slow_profile()),
            state,
        };

        Ok(Self {
            canvas,
            draw_game,
            plotting,
            state,
        })
    }

    pub fn update(&mut self, dt: LocalTimeDelta) {
        let update_params = UpdateParams {
            client_outbox: &mut Vec::new(),
            server_outbox: &mut Vec::new(),
            client_outbox: &
        };
    }
}
