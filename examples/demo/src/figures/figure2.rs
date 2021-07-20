use std::collections::BTreeMap;

use malen::{draw::plot::Plotting, Camera, Canvas, Color4};
use nalgebra::Point2;
use untimely::{mock::MockNet, LocalDt, LocalTime, Metrics, PeriodicTimer, PlayerId};

use crate::{current_game_input, DrawGame, Figure, Game, GameInput};

type ServerMsg = Game;
type ClientMsg = GameInput;

struct Server {
    game: Game,
    tick_timer: PeriodicTimer,
}

struct Client {
    id: PlayerId,
    latest_server_game: Game,
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

        Self { game, tick_timer }
    }

    fn update(&mut self, dt: LocalDt, mock_net: &mut MockNet<ServerMsg, ClientMsg>) {}
}

impl Client {
    fn new(id: u32) -> Self {
        Client {
            id: PlayerId(id),
            latest_server_game: Game::default(),
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
    fn update(&mut self, dt: LocalDt) {
        while let Some(_) = self.canvas.pop_event() {}
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
