use std::collections::BTreeMap;

use malen::{Camera, Canvas, Color4, InputState};
use untimely::{
    channel::{NetProfile, SimNetChannel},
    LocalTime, LocalTimeDelta, PeriodicTimer, PlayerId, TickNum,
};

use crate::{
    draw::DrawGame,
    examples::Example,
    game::{Game, GameInput},
};

type ClientMsg = (PlayerId, GameInput);
type ServerMsg = (PlayerId, (TickNum, Game));

pub struct MyExample {
    current_time: LocalTime,

    server: MyServer,
    clients: BTreeMap<PlayerId, MyClient>,

    client_to_server: SimNetChannel<ClientMsg>,
    server_to_client: SimNetChannel<ServerMsg>,

    canvas: Canvas,
    draw_game: DrawGame,
}

struct MyServer {
    game: (TickNum, Game),
    tick_timer: PeriodicTimer,
}

struct MyClient {
    game: Option<(TickNum, Game)>,
}

impl MyServer {
    fn new() -> Self {
        let game = (TickNum::ZERO, Game::default());
        let tick_timer = PeriodicTimer::new(game.1.tick_time_delta.to_local_time_delta());

        Self { game, tick_timer }
    }

    fn update(&mut self, dt: LocalTimeDelta, received_msgs: &[ClientMsg]) -> Vec<ServerMsg> {
        for (player_id, game_input) in received_msgs {
            self.game.1.run_input(*player_id, game_input);
        }

        self.tick_timer.add_time_delta(dt);
        if self.tick_timer.trigger() {
            self.game
                .1
                .players
                .iter()
                .map(|(player_id, _)| (*player_id, self.game.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl MyClient {
    fn new() -> Self {
        Self { game: None }
    }

    fn update(
        &mut self,
        _: LocalTimeDelta,
        my_player_id: PlayerId,
        received_msgs: &[ServerMsg],
        input_state: &InputState,
    ) -> Vec<ClientMsg> {
        let current_game_input = super::util::current_game_input(input_state);

        let mut msgs_to_send = Vec::new();

        // In this example, we only send input for the first player.
        // We simply send input whenever we receive a new tick to start.
        for server_msg in received_msgs {
            if my_player_id == PlayerId(0) {
                msgs_to_send.push((my_player_id, current_game_input.clone()));
            }
        }

        msgs_to_send
    }
}

impl MyExample {
    pub fn new() -> Result<Self, malen::Error> {
        let current_time = LocalTime::ZERO;

        let server = MyServer::new();
        let clients = server
            .game
            .1
            .players
            .iter()
            .map(|(player_id, _)| (*player_id, MyClient::new()))
            .collect();

        let client_to_server = SimNetChannel::new(NetProfile::wonky_slow_profile());
        let server_to_client = SimNetChannel::new(NetProfile::wonky_slow_profile());

        let canvas = Canvas::from_element_id("example_server_client")?;
        let draw_game = DrawGame::new(&canvas)?;

        Ok(Self {
            current_time,
            server,
            clients,
            client_to_server,
            server_to_client,
            canvas,
            draw_game,
        })
    }
}

impl Example for MyExample {
    fn update(&mut self, dt: LocalTimeDelta) {
        while let Some(_) = self.canvas.pop_event() {}

        self.current_time += dt;

        let client_msgs: Vec<_> = self
            .client_to_server
            .receive_all(self.current_time)
            .into_iter()
            .map(|(_, msg)| msg)
            .collect();
        let server_msgs: Vec<_> = self
            .server_to_client
            .receive_all(self.current_time)
            .into_iter()
            .map(|(_, msg)| msg)
            .collect();

        let sent_server_msgs = self.server.update(dt, client_msgs.as_slice());
        self.server_to_client
            .send_iter(self.current_time, sent_server_msgs);

        for (player_id, client) in self.clients.iter_mut() {
            let server_msgs_for_client: Vec<_> = server_msgs
                .clone()
                .into_iter()
                .filter(|(receiver_id, _)| receiver_id == player_id)
                .collect();

            let sent_client_msgs = client.update(
                dt,
                *player_id,
                server_msgs_for_client.as_slice(),
                self.canvas.input_state(),
            );

            self.client_to_server
                .send_iter(self.current_time, sent_client_msgs);
        }
    }

    fn draw(&mut self) -> Result<(), malen::Error> {
        self.canvas.clear(Color4::new(0.0, 0.0, 0.0, 1.0));

        let view = Camera::screen_view_matrix(&self.canvas.screen_geom());
        self.draw_game
            .draw(&self.canvas, &self.server.game.1, &view)?;

        Ok(())
    }
}
