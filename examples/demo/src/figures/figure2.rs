use malen::{
    draw::plot::{Axis, Line, Plot, Plotting},
    Canvas, Color4, InputState,
};
use nalgebra::{Matrix3, Vector2};
use untimely::{
    mock::MockNet, LocalClock, LocalDt, LocalTime, Metrics, PeriodicTimer, PlayerId, TickNum,
};

use crate::{current_game_input, get_socket_params, DrawGame, Figure, Game, GameInput, GameParams};

type ServerMsg = Game;
type ClientMsg = GameInput;

struct Server {
    game: Game,
    tick_num: TickNum,
    tick_timer: PeriodicTimer,

    // Only for visualization:
    last_inputs: Vec<GameInput>,
}

struct Client {
    id: PlayerId,
    latest_server_game: Game,
    input_timer: PeriodicTimer,

    // Only for visualization:
    last_input: GameInput,
}

pub struct Figure2 {
    clock: LocalClock,

    server: Server,
    clients: Vec<Client>,
    mock_net: MockNet<ServerMsg, ClientMsg>,

    metrics: Metrics,
    canvas: Canvas,
    draw_game: DrawGame,
    plotting: Plotting,
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

        if self.tick_timer.trigger() {
            for (_, sender, input) in mock_net.receive_server() {
                self.game.run_input(sender, &input);

                // Only for visualization:
                self.last_inputs[sender.0 as usize] = input;
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
            last_input: GameInput::default(),
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

            // Only for visualization:
            self.last_input = my_input;
        }
    }
}

impl Figure2 {
    pub fn new() -> Result<Self, malen::Error> {
        let clock = LocalClock::new();

        let canvas = Canvas::from_element_id("figure2")?;
        let draw_game = DrawGame::new(&canvas)?;
        let plotting = Plotting::new(&canvas)?;

        Ok(Self {
            clock: clock.clone(),
            server: Server::new(),
            clients: vec![Client::new(0), Client::new(1)],
            mock_net: MockNet::new(&[PlayerId(0), PlayerId(1)], clock.clone()),
            metrics: Metrics::new(LocalDt::from_secs(5.0), clock.clone()),
            canvas,
            draw_game,
            plotting,
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
        for client in &mut self.clients {
            client.update(dt, self.canvas.input_state(), &mut self.mock_net);
        }

        // Record metrics for visualization.
        let time_anna = self.clients[0].latest_server_game.time.to_secs();
        let time_brad = self.clients[1].latest_server_game.time.to_secs();
        let time_server = self.server.game.time.to_secs();
        self.metrics
            .record_gauge("game_delay_anna", time_server - time_anna);
        self.metrics
            .record_gauge("game_delay_brad", time_server - time_brad);
    }

    fn draw(&mut self) -> Result<(), malen::Error> {
        let anna = &self.clients[0];
        let brad = &self.clients[1];
        let serv = &self.server;

        self.draw_game.draw(
            &self.canvas,
            &[
                (
                    "Anna",
                    &anna.latest_server_game,
                    Some(anna.last_input),
                    None,
                ),
                (
                    "Brad",
                    &brad.latest_server_game,
                    None,
                    Some(brad.last_input),
                ),
                (
                    "Server",
                    &serv.game,
                    Some(serv.last_inputs[0]),
                    Some(serv.last_inputs[1]),
                ),
            ],
        )?;
        self.draw_plot()?;

        Ok(())
    }
}

impl Figure2 {
    fn plot(&self) -> Plot {
        let mut lines = Vec::new();

        let shift = |points: &[(f64, f64)]| {
            points
                .iter()
                .map(|(x, y)| (*x - self.clock.local_time().to_secs(), *y))
                .collect::<Vec<_>>()
        };

        if let Some(gauge) = self.metrics.get_gauge("game_delay_anna") {
            lines.push(Line {
                caption: "delay_anna".to_string(),
                color: Color4::new(0.2, 0.8, 0.2, 1.0),
                points: shift(&gauge.plot_points()),
            });
        }
        if let Some(gauge) = self.metrics.get_gauge("game_delay_brad") {
            lines.push(Line {
                caption: "delay_brad".to_string(),
                color: Color4::new(0.2, 0.2, 0.8, 1.0),
                points: shift(&gauge.plot_points()),
            });
        }

        Plot {
            size: Vector2::new(990.0, 200.0),
            x_axis: Axis {
                label: "local time [s]".to_string(),
                range: Some((-5.0, 0.0)),
                tics: 1.0,
                tic_precision: 1,
            },
            y_axis: Axis {
                label: "game time [s]".to_string(),
                range: Some((0.0, 0.5)),
                tics: 0.1,
                tic_precision: 1,
            },
            axis_color: Color4::new(0.0, 0.0, 0.0, 1.0),
            background_color: None,
            text_color: Color4::new(0.0, 0.0, 0.0, 1.0),
            lines,
        }
    }

    fn draw_plot(&mut self) -> Result<(), malen::Error> {
        let transform = self.canvas.screen().orthographic_projection()
            * Matrix3::new_translation(&Vector2::new(0.0, Game::MAP_HEIGHT + 15.0));

        let plot = self.plot();
        self.plotting
            .draw(&self.canvas, self.draw_game.font_mut(), &transform, &plot)
    }
}
