use malen::InputState;
use untimely::{
    mock::MockNet, DejitterBuffer, LocalClock, LocalDt, LocalTime, Metrics, PeriodicTimer,
    PlaybackClockParams, PlayerId, TickNum, TickPlayback,
};

use crate::{current_game_input, get_socket_params, DrawGame, Figure, Game, GameInput, GameParams};

type ServerMsg = (TickNum, Game);
type ClientMsg = (TickNum, GameInput);

const NUM_SEND_TICKS: usize = 3;

struct Client {
    inputs: DejitterBuffer<GameInput>,
    last_input: GameInput,
}

struct Server {
    game: Game,
    tick_num: TickNum,
    tick_timer: PeriodicTimer,

    clients: Vec<Client>,
}

struct User {
    id: PlayerId,
    name: String,
    playback: TickPlayback<(TickNum, Game)>,

    // Only for visualization:
    last_input: GameInput,
}

impl Client {
    pub fn new(tick_dt: LocalDt, clock: LocalClock) -> Self {
        Self {
            inputs: DejitterBuffer::new(tick_dt, LocalDt::from_secs(5.0), clock),
            last_input: GameInput::default(),
        }
    }
}

impl Server {
    fn new(clock: LocalClock) -> Self {
        let game = Game::default();
        let tick_timer = PeriodicTimer::new(game.params.dt.to_local_dt());

        Self {
            game: game.clone(),
            tick_timer,
            tick_num: TickNum::zero(),
            clients: vec![
                Client::new(game.params.dt.to_local_dt(), clock.clone()),
                Client::new(game.params.dt.to_local_dt(), clock.clone()),
            ],
        }
    }

    fn update(&mut self, dt: LocalDt, mock_net: &mut MockNet<ServerMsg, ClientMsg>) {
        for (receive_time, sender, (input_num, input)) in mock_net.receive_from_clients() {
            self.clients[sender.to_usize()]
                .inputs
                .insert(receive_time, input_num, input);
        }

        self.tick_timer.advance(dt);

        while self.tick_timer.trigger() {
            for (index, client) in self.clients.iter_mut().enumerate() {
                let player_id = PlayerId(index as u32);
                let mut any_input = false;

                while let Some((_tick_num, input)) = client.inputs.pop() {
                    self.game.run_input(player_id, &input);
                    client.last_input = input.clone();
                    any_input = true;
                }

                if !any_input {
                    self.game.run_input(player_id, &client.last_input);
                }
            }

            if self.tick_num.to_usize() % NUM_SEND_TICKS == 0 {
                for client in self.game.players.keys() {
                    mock_net.send_to_client(*client, (self.tick_num, self.game.clone()));
                }
            }

            self.game.time += self.game.params.dt;
            self.tick_num = self.tick_num.succ();
        }
    }
}

impl User {
    fn new(id: u32, name: String, clock: LocalClock) -> Self {
        User {
            id: PlayerId(id),
            name,
            playback: TickPlayback::new(
                clock,
                PlaybackClockParams::for_interpolation(
                    GameParams::default().dt * NUM_SEND_TICKS as f64,
                ),
            ),
            last_input: GameInput::default(),
        }
    }

    fn update(
        &mut self,
        dt: LocalDt,
        input_state: &InputState,
        mock_net: &mut MockNet<ServerMsg, ClientMsg>,
    ) {
        for (receive_time, tick) in mock_net.receive_from_server(self.id) {
            self.playback.record_tick(receive_time, tick.1.time, tick);
        }

        let started_ticks = self.playback.advance(dt);

        let my_input = current_game_input(self.id, self.playback.playback_time(), input_state);
        for (_, (tick_num, _)) in started_ticks {
            mock_net.send_to_server(self.id, (tick_num, my_input.clone()));
            self.last_input = my_input.clone();
        }
    }

    fn game(&self) -> Option<Game> {
        self.playback
            .interpolation()
            .map(|interp| {
                interp
                    .current_value
                    .1
                    .interpolate(&interp.next_value.1, interp.alpha)
            })
            .or_else(|| {
                self.playback
                    .current_tick()
                    .map(|(_, (_, game))| game.clone())
            })
    }
}

pub struct Figure3 {
    clock: LocalClock,

    server: Server,
    users: Vec<User>,
    mock_net: MockNet<ServerMsg, ClientMsg>,

    metrics: Metrics,
    draw_game: DrawGame,
}

impl Figure3 {
    pub fn new() -> Result<Self, malen::Error> {
        let clock = LocalClock::new();

        let instances = &[
            ("figure3_anja", "Anja"),
            ("figure3_brad", "Brad"),
            ("figure3_server", "Server"),
        ];
        let draw_game = DrawGame::new(instances, &[("figure3_plot", "figure3_plot_div")])?;

        Ok(Self {
            clock: clock.clone(),
            server: Server::new(clock.clone()),
            users: vec![
                User::new(0, "anja".to_string(), clock.clone()),
                User::new(1, "brad".to_string(), clock.clone()),
            ],
            mock_net: MockNet::new(&[PlayerId(0), PlayerId(1)], clock.clone()),
            metrics: Metrics::new(LocalDt::from_secs(5.0), clock.clone()),
            draw_game,
        })
    }

    fn record_metrics(&mut self) {
        for user in self.users.iter() {
            let time_user = user.playback.playback_time().to_secs();
            let time_server = (self.server.game.time
                + self.server.tick_timer.accumulator().to_game_dt())
            .to_secs();

            user.playback.record_metrics(&user.name, &mut self.metrics);
            self.metrics.record_gauge(
                &format!("{}_server_delay", user.name),
                time_server - time_user,
            );
            self.metrics.record_gauge(
                &format!("{}_stream_server_delay", user.name),
                time_server - user.playback.playback_clock().stream_time().to_secs(),
            );
        }
    }
}

impl Figure for Figure3 {
    fn update(&mut self, time: LocalTime) {
        self.draw_game.update();
        self.mock_net
            .set_params(PlayerId(0), get_socket_params("figure3", "anja"));
        //self.mock_net.set_params(PlayerId(1), get_socket_params("figure3", "brad"));

        let dt = self.clock.set_local_time(time).min(LocalDt::from_secs(1.0));
        self.server.update(dt, &mut self.mock_net);
        for client in &mut self.users {
            client.update(dt, self.draw_game.input_state(0), &mut self.mock_net);
        }

        self.record_metrics();
    }

    fn draw(&mut self) -> Result<(), malen::Error> {
        let anja_game = self.users[0].game();
        let brad_game = self.users[0].game();

        let games = &[
            (
                anja_game.as_ref(),
                Some(self.users[0].last_input.clone()),
                None,
            ),
            (
                brad_game.as_ref(),
                Some(self.users[1].last_input.clone()),
                None,
            ),
            (
                Some(&self.server.game),
                Some(self.server.clients[0].last_input.clone()),
                Some(self.server.clients[1].last_input.clone()),
            ),
        ];

        self.draw_game.draw(games)?;
        self.draw_game
            .draw_plot(self.clock.local_time(), &self.metrics)?;

        Ok(())
    }
}
