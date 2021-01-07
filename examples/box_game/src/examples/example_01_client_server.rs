use std::collections::BTreeMap;

use malen::{
    draw::plot::{Axis, LinePlotData, Plot, Plotting},
    Canvas, Color4, InputState, ScreenGeom,
};
use nalgebra::{Matrix3, Point2, Vector2, Vector3};
use untimely::{
    channel::{NetProfile, SimNetChannel},
    util::SlidingWindowRandomVar,
    LocalTime, LocalTimeDelta, PeriodicTimer, PlayerId, TickNum,
};

use crate::{
    draw::DrawGame,
    examples::Example,
    game::{Game, GameInput},
};

type ClientMsg = (PlayerId, GameInput);
type ServerMsg = (PlayerId, Game);

const PLOT_SECS: f64 = 5.0;
const PLOT_HEIGHT: u32 = 148;

pub struct MyExample {
    current_time: LocalTime,

    server: MyServer,
    clients: BTreeMap<PlayerId, MyClient>,

    client_to_server: SimNetChannel<ClientMsg>,
    server_to_client: SimNetChannel<ServerMsg>,

    canvas: Canvas,
    draw_game: DrawGame,
    plotting: Plotting,

    server_game_time_var: SlidingWindowRandomVar,
    client_vars: BTreeMap<PlayerId, ClientRandomVars>,
}

struct MyServer {
    game: Game,
    tick_timer: PeriodicTimer,
}

struct MyClient {
    game: Game,
}

struct ClientRandomVars {
    game_time_var: SlidingWindowRandomVar,
}

impl ClientRandomVars {
    fn new(random_var_max_age: LocalTimeDelta) -> Self {
        Self {
            game_time_var: SlidingWindowRandomVar::new(random_var_max_age),
        }
    }
}

impl MyServer {
    fn new() -> Self {
        let game = Game::default();
        let tick_timer = PeriodicTimer::new(game.tick_time_delta.to_local_time_delta());

        Self { game, tick_timer }
    }

    fn update(&mut self, dt: LocalTimeDelta, received_msgs: &[ClientMsg]) -> Vec<ServerMsg> {
        for (player_id, game_input) in received_msgs {
            self.game.run_input(*player_id, game_input);
        }

        self.tick_timer.add_time_delta(dt);
        let sent_server_msgs = if self.tick_timer.trigger() {
            self.game.tick_num = self.game.tick_num.get_next();
            self.game
                .players
                .keys()
                .map(|player_id| (*player_id, self.game.clone()))
                .collect()
        } else {
            Vec::new()
        };

        sent_server_msgs
    }
}

impl MyClient {
    fn new() -> Self {
        Self {
            game: Game::default(),
        }
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

        for server_msg in received_msgs {
            // Ignore out-of-order messages:
            if server_msg.1.tick_num >= self.game.tick_num {
                self.game = server_msg.1.clone();
            }

            // In this example, we only send input for the first player.
            // We simply send input whenever we receive a new tick to start.
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
            .players
            .keys()
            .map(|player_id| (*player_id, MyClient::new()))
            .collect();

        let client_to_server = SimNetChannel::new(NetProfile::wonky_slow_profile());
        let server_to_client = SimNetChannel::new(NetProfile::wonky_slow_profile());

        let canvas = Canvas::from_element_id("example_client_server")?;
        let draw_game = DrawGame::new(&canvas)?;
        let plotting = Plotting::new(&canvas)?;

        let random_var_max_age = LocalTimeDelta::from_secs(PLOT_SECS);
        let server_game_time_var = SlidingWindowRandomVar::new(random_var_max_age);
        let client_vars = server
            .game
            .players
            .keys()
            .map(|player_id| (*player_id, ClientRandomVars::new(random_var_max_age)))
            .collect();

        Ok(Self {
            current_time,
            server,
            clients,
            client_to_server,
            server_to_client,
            canvas,
            draw_game,
            plotting,
            server_game_time_var,
            client_vars,
        })
    }
}

impl Example for MyExample {
    fn update(&mut self, dt: LocalTimeDelta) {
        while let Some(_) = self.canvas.pop_event() {}

        self.current_time += dt;

        // Account for artifical lag and loss, which messages are arriving?
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

        // Update the server and "send" its messages to the clients.
        let sent_server_msgs = self.server.update(dt, client_msgs.as_slice());
        self.server_to_client
            .send_iter(self.current_time, sent_server_msgs);

        // Update the clients and "send" their messages to the clients.
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

        // Keep some statistics for plotting.
        for (player_id, client) in self.clients.iter() {
            self.client_vars
                .get_mut(player_id)
                .unwrap()
                .game_time_var
                .record(
                    self.current_time,
                    client.game.game_time().to_secs_since_start(),
                );
        }

        self.server_game_time_var.record(
            self.current_time,
            self.server.game.game_time().to_secs_since_start(),
        );
    }

    fn draw(&mut self) -> Result<(), malen::Error> {
        self.canvas
            .set_viewport(Point2::origin(), self.canvas.screen_geom().size);
        self.canvas.clear(Color4::new(0.8, 0.8, 0.8, 1.0));

        self.draw_game.draw_multiple(
            &self.canvas,
            PLOT_HEIGHT,
            &[
                ("Client 0", &self.clients[&PlayerId(0)].game),
                ("Client 1", &self.clients[&PlayerId(1)].game),
                ("Server", &self.server.game),
            ],
        )?;

        self.draw_plot()?;

        Ok(())
    }

    fn is_active(&self) -> bool {
        self.canvas.has_focus()
    }
}

impl MyExample {
    fn describe_plot(&self) -> Plot {
        let min_time = self.current_time - LocalTimeDelta::from_secs(PLOT_SECS);

        let server_game_time_points = self.server_game_time_var.to_plot_points(min_time);

        let delay_vs_server = |var: &SlidingWindowRandomVar| {
            let points = var.to_plot_points(min_time);
            server_game_time_points
                .iter()
                .zip(points.iter())
                .map(|(server_time, other_time)| {
                    assert_eq!(server_time.0, other_time.0);
                    (server_time.0, server_time.1 - other_time.1)
                })
                .collect()
        };

        Plot {
            size: Vector2::new(Game::MAP_WIDTH as f64 * 3.0, PLOT_HEIGHT as f64),
            x_axis: Axis {
                label: "real time [s]".into(),
                range: Some((-PLOT_SECS as f64, 0.0)),
                tics: 1.0,
                tic_precision: 0,
            },
            y_axis: Axis {
                label: "game time [s]".into(),
                range: None,
                //range: Some((0.0, 0.3)),
                tics: 0.1,
                tic_precision: 2,
            },
            axis_color: Color4::new(1.0, 1.0, 1.0, 1.0),
            background_color: Some(Color4::from_u8(50, 50, 50, 255)),
            text_color: Color4::from_u8(180, 180, 180, 255),
            lines: vec![
                /*LinePlotData {
                    caption: "Server".into(),
                    color: Color4::from_u8(200, 100, 100, 255),
                    points: delay_vs_server(&self.server_game_time_var),
                },*/
                LinePlotData {
                    caption: "Client 0".into(),
                    color: Color4::from_u8(100, 200, 100, 255),
                    points: delay_vs_server(&self.client_vars[&PlayerId(0)].game_time_var),
                },
                LinePlotData {
                    caption: "Client 1".into(),
                    color: Color4::from_u8(100, 100, 200, 255),
                    points: delay_vs_server(&self.client_vars[&PlayerId(1)].game_time_var),
                },
            ],
        }
    }

    fn draw_plot(&mut self) -> Result<(), malen::Error> {
        let screen_geom = ScreenGeom {
            size: Vector2::new(960, PLOT_HEIGHT),
            device_pixel_ratio: self.canvas.screen_geom().device_pixel_ratio,
        };
        self.canvas.set_viewport(Point2::origin(), screen_geom.size);
        let transform = screen_geom.orthographic_projection();

        let plot = self.describe_plot();
        self.plotting
            .draw(&self.canvas, self.draw_game.font_mut(), &transform, &plot)
    }
}
