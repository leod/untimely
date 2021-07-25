use malen::InputState;
use untimely::{mock::MockNet, LocalClock, LocalDt, Metrics, PeriodicTimer, PlayerId, TickNum};

use crate::{
    current_game_input, get_socket_params, is_active, DrawGame, Figure, Game, GameInput, GameParams,
};

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
    draw_game: DrawGame,
}

impl Figure2 {
    pub fn new() -> Result<Self, malen::Error> {
        let clock = LocalClock::new();

        let instances = &[
            ("figure2_anja", "Anja"),
            ("figure2_brad", "Brad"),
            ("figure2_server", "Server"),
        ];
        let draw_game = DrawGame::new(instances, &[("figure2_plot", "figure2_plot_div")])?;

        Ok(Self {
            clock: clock.clone(),
            server: Server::new(),
            users: vec![User::new(0), User::new(1)],
            mock_net: MockNet::new(&[PlayerId(0), PlayerId(1)], clock.clone()),
            metrics: Metrics::new(LocalDt::from_secs(5.0), clock.clone()),
            draw_game,
        })
    }
}

impl Figure for Figure2 {
    fn update(&mut self, dt: LocalDt) {
        self.draw_game.update();
        if !is_active("figure2", &self.clock) {
            return;
        }

        self.mock_net
            .set_params(PlayerId(0), get_socket_params("figure2", "anja"));
        self.mock_net
            .set_params(PlayerId(1), get_socket_params("figure2", "brad"));

        self.clock.advance(dt);
        self.server.update(dt, &mut self.mock_net);
        for client in &mut self.users {
            client.update(dt, self.draw_game.input_state(0), &mut self.mock_net);
        }

        // Record metrics for visualization.
        let time_anna = self.users[0].latest_server_game.time.to_secs();
        let time_brad = self.users[1].latest_server_game.time.to_secs();
        let time_server = self.server.game.time.to_secs();
        self.metrics
            .record_gauge("anja_server_delay", time_server - time_anna);
        self.metrics
            .record_gauge("brad_server_delay", time_server - time_brad);
    }

    fn draw(&mut self) -> Result<(), malen::Error> {
        if !is_active("figure2", &self.clock) {
            return Ok(());
        }

        let games = &[
            (
                Some(&self.users[0].latest_server_game),
                Some(self.users[0].last_input.clone()),
                None,
            ),
            (
                Some(&self.users[1].latest_server_game),
                Some(self.users[1].last_input.clone()),
                None,
            ),
            (
                Some(&self.server.game),
                Some(self.server.last_inputs[0].clone()),
                Some(self.server.last_inputs[1].clone()),
            ),
        ];
        self.draw_game.draw(games)?;
        self.draw_game
            .draw_plot(self.clock.local_time(), &self.metrics)?;

        Ok(())
    }
}
