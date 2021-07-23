use malen::{Canvas, InputState};
use untimely::{
    mock::MockNet, LocalClock, LocalDt, LocalTime, Metrics, PeriodicTimer, PlayerId, TickNum,
};

use crate::{current_game_input, get_socket_params, DrawGame, Figure, Game, GameInput, GameParams};

type ServerMsg = Game;
type ClientMsg = GameInput;

const SEND_TICK_DELTA: u64 = 3;

struct Server {
    game: Game,
    tick_num: TickNum,
    tick_timer: PeriodicTimer,

    // Only for visualization:
    last_inputs: Vec<GameInput>,
}

struct User {
    id: PlayerId,
    latest_server_game: Game,
    input_timer: PeriodicTimer,

    // Only for visualization:
    last_input: GameInput,
}

impl Server {
    fn new() -> Self {
        let game = Game::default();
        let tick_timer = PeriodicTimer::new(game.params.dt.to_local_dt());

        Self {
            game,
            tick_timer,
            tick_num: TickNum::zero(),
            last_inputs: vec![GameInput::default(); 2],
        }
    }

    fn update(&mut self, dt: LocalDt, mock_net: &mut MockNet<ServerMsg, ClientMsg>) {
        self.tick_timer.advance(dt);

        while self.tick_timer.trigger() {
            for (_, sender, input) in mock_net.receive_from_clients() {
                self.game.run_input(sender, &input);

                // Only for visualization:
                self.last_inputs[sender.0 as usize] = input;
            }
            self.game.time += self.game.params.dt;
            self.tick_num = self.tick_num.succ();

            if self.tick_num.to_u64() % SEND_TICK_DELTA == 0 {
                for client in self.game.players.keys() {
                    mock_net.send_to_client(*client, self.game.clone());
                }
            }
        }
    }
}

impl User {
    fn new(id: u32) -> Self {
        User {
            id: PlayerId(id),
            latest_server_game: Game::default(),
            input_timer: PeriodicTimer::new(GameParams::default().dt.to_local_dt()),
            last_input: GameInput::default(),
        }
    }

    fn update(
        &mut self,
        dt: LocalDt,
        input_state: &InputState,
        mock_net: &mut MockNet<ServerMsg, ClientMsg>,
    ) {
        let messages = mock_net.receive_from_server(self.id);

        // Always immediately display the latest state we receive from the
        // server.
        if let Some(last_message) = messages.last() {
            self.latest_server_game = last_message.1.clone();
        }

        // Send input periodically.
        self.input_timer.advance(dt);

        while self.input_timer.trigger() {
            let my_input = current_game_input(self.id, self.latest_server_game.time, input_state);
            mock_net.send_to_server(self.id, my_input.clone());

            // Only for visualization:
            self.last_input = my_input;
        }
    }
}

pub struct Figure2 {
    clock: LocalClock,

    server: Server,
    users: Vec<User>,
    mock_net: MockNet<ServerMsg, ClientMsg>,

    metrics: Metrics,
    canvas: Canvas,
    draw_game: DrawGame,
}

impl Figure2 {
    pub fn new() -> Result<Self, malen::Error> {
        let clock = LocalClock::new();

        let canvas = Canvas::from_element_id("figure2")?;
        let draw_game = DrawGame::new(&canvas)?;

        Ok(Self {
            clock: clock.clone(),
            server: Server::new(),
            users: vec![User::new(0), User::new(1)],
            mock_net: MockNet::new(&[PlayerId(0), PlayerId(1)], clock.clone()),
            metrics: Metrics::new(LocalDt::from_secs(5.0), clock.clone()),
            canvas,
            draw_game,
        })
    }
}

impl Figure for Figure2 {
    fn update(&mut self, time: LocalTime) {
        while let Some(_) = self.canvas.pop_event() {}

        let dt = self.clock.set_local_time(time).min(LocalDt::from_secs(1.0));

        self.mock_net
            .set_params(PlayerId(0), get_socket_params("figure2", "anna"));
        //self.mock_net.set_params(PlayerId(1), get_socket_params("figure2", "brad"));

        self.server.update(dt, &mut self.mock_net);
        for client in &mut self.users {
            client.update(dt, self.canvas.input_state(), &mut self.mock_net);
        }

        // Record metrics for visualization.
        let time_anna = self.users[0].latest_server_game.time.to_secs();
        let time_brad = self.users[1].latest_server_game.time.to_secs();
        let time_server = self.server.game.time.to_secs();
        self.metrics
            .record_gauge("anna_server_delay", time_server - time_anna);
        self.metrics
            .record_gauge("brad_server_delay", time_server - time_brad);
    }

    fn draw(&mut self) -> Result<(), malen::Error> {
        let anna = &self.users[0];
        let brad = &self.users[1];
        let serv = &self.server;

        self.draw_game.draw(
            &self.canvas,
            &[
                (
                    "Anna",
                    Some(&anna.latest_server_game),
                    Some(anna.last_input.clone()),
                    None,
                ),
                (
                    "Brad",
                    Some(&brad.latest_server_game),
                    Some(brad.last_input.clone()),
                    None,
                ),
                (
                    "Server",
                    Some(&serv.game),
                    Some(serv.last_inputs[0].clone()),
                    Some(serv.last_inputs[1].clone()),
                ),
            ],
        )?;
        self.draw_game
            .draw_plot(&self.canvas, self.clock.local_time(), &self.metrics)?;

        Ok(())
    }
}
