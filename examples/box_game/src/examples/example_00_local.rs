use malen::{Camera, Canvas, Color4};
use untimely::{LocalTimeDelta, PeriodicTimer, PlayerId};

use crate::{draw::DrawGame, examples::Example, game::Game};

pub struct MyExample {
    canvas: Canvas,
    draw_game: DrawGame,

    game: Game,
    tick_timer: PeriodicTimer,
}

impl MyExample {
    pub fn new() -> Result<Self, malen::Error> {
        let canvas = Canvas::from_element_id("example_local_canvas")?;
        let draw_game = DrawGame::new(&canvas)?;

        let game = Game::default();
        let tick_timer = PeriodicTimer::new(game.tick_time_delta.to_local_time_delta());

        Ok(Self {
            canvas,
            draw_game,
            game,
            tick_timer,
        })
    }
}

impl Example for MyExample {
    fn update(&mut self, dt: LocalTimeDelta) {
        while let Some(_) = self.canvas.pop_event() {}

        self.tick_timer.add_time_delta(dt);
        if self.tick_timer.trigger() {
            let input = super::util::current_game_input(&self.canvas.input_state());
            self.game.run_input(PlayerId(0), &input);
        }
    }

    fn draw(&mut self) -> Result<(), malen::Error> {
        self.canvas.clear(Color4::new(0.0, 0.0, 0.0, 1.0));

        let view = Camera::screen_view_matrix(&self.canvas.screen_geom());
        self.draw_game.draw(&self.canvas, &self.game, &view)?;

        Ok(())
    }
}
