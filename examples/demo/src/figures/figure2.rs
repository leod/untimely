use std::collections::BTreeMap;

use malen::{draw::plot::Plotting, Camera, Canvas, Color4, InputState};
use nalgebra::Point2;
use untimely::{mock::MockNet, LocalDt, LocalTime, Metrics, PeriodicTimer, PlayerId, TickNum};

use crate::{current_game_input, get_param, DrawGame, Figure, Game, GameInput, GameParams};

type ServerMsg = Game;
type ClientMsg = GameInput;

struct Server {
    game: Game,
    tick_num: TickNum,
    tick_timer: PeriodicTimer,
}

struct Client {
    id: PlayerId,
    latest_server_game: Game,
    input_timer: PeriodicTimer,
}

pub struct Figure2 {
    time: LocalTime,
    server: Server,
    clients: Vec<Client>,
    mock_net: MockNet<ServerMsg, ClientMsg>,

    metrics: Metrics,

    canvas: Canvas,
    draw_game: DrawGame,
    plotting: Plotting,
}

const PLOT_SECS: f64 = 5.0;
const PLOT_HEIGHT: u32 = 145;

impl Server {
    fn new() -> Self {
        let game = Game::default();
        let tick_timer = PeriodicTimer::new(game.params.dt.to_local_dt());

        Self {
            game,
            tick_timer,
            tick_num: TickNum::zero(),
        }
    }

    fn update(&mut self, dt: LocalDt, mock_net: &mut MockNet<ServerMsg, ClientMsg>) {
        self.tick_timer.advance(dt);

        if self.tick_timer.trigger() {
            for (_, sender, input) in mock_net.receive_server() {
                self.game.run_input(sender, &input);
            }
            self.game.time += self.game.params.dt;
            self.tick_num = self.tick_num.succ();

            if self.tick_num.to_u32() % 3 == 0 {
                for client in self.game.players.keys() {
                    mock_net.send_to_client(*client, self.game.clone());
                }
            }
        }
    }
}

impl Client {
    fn new(id: u32) -> Self {
        Client {
            id: PlayerId(id),
            latest_server_game: Game::default(),
            input_timer: PeriodicTimer::new(GameParams::default().dt.to_local_dt()),
        }
    }

    fn update(
        &mut self,
        dt: LocalDt,
        input_state: &InputState,
        mock_net: &mut MockNet<ServerMsg, ClientMsg>,
    ) {
        let messages = mock_net.receive_client(self.id);

        // Always immediately display the latest state we receive from the
        // server.
        if let Some(last_message) = messages.last() {
            self.latest_server_game = last_message.1.clone();
        }

        // Send input periodically.
        self.input_timer.advance(dt);

        if self.input_timer.trigger() {
            let my_input = current_game_input(self.id, self.latest_server_game.time, input_state);
            mock_net.send_to_server(self.id, my_input);
        }
    }
}

impl Figure2 {
    pub fn new() -> Result<Self, malen::Error> {
        let canvas = Canvas::from_element_id("figure2")?;
        let draw_game = DrawGame::new(&canvas)?;
        let plotting = Plotting::new(&canvas)?;

        Ok(Self {
            time: LocalTime::zero(),
            server: Server::new(),
            clients: vec![Client::new(0), Client::new(1)],
            mock_net: MockNet::new(&[PlayerId(0), PlayerId(1)]),
            metrics: Metrics::new(LocalDt::from_secs(10.0)),
            canvas,
            draw_game,
            plotting,
        })
    }
}

impl Figure for Figure2 {
    fn update(&mut self, time: LocalTime, dt: LocalDt) {
        while let Some(_) = self.canvas.pop_event() {}

        self.mock_net.set_time(time);
        {
            let anna = self.mock_net.socket_mut(PlayerId(0));
            anna.server_out_params.latency_mean =
                LocalDt::from_millis(get_param("figure2_anna_ping"));
            anna.client_out_params.latency_mean =
                LocalDt::from_millis(get_param("figure2_anna_ping"));

            log::info!(
                "anna: {:?} {:?}",
                anna.server_out_params,
                anna.client_out_params
            );
        }

        self.server.update(dt, &mut self.mock_net);

        for client in &mut self.clients {
            client.update(dt, self.canvas.input_state(), &mut self.mock_net);
        }
    }

    fn draw(&mut self) -> Result<(), malen::Error> {
        self.draw_game.draw_multiple(
            &self.canvas,
            &[
                ("Anna", &self.clients[0].latest_server_game),
                ("Brad", &self.clients[1].latest_server_game),
                ("Server", &self.server.game),
            ],
        )?;

        Ok(())
    }

    fn is_active(&self) -> bool {
        self.canvas.has_focus()
    }
}
