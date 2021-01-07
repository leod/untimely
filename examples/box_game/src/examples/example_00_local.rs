use malen::{Camera, Canvas, Color4};
use untimely::{LocalTimeDelta, PeriodicTimer, PlayerId};

use crate::{draw::DrawGame, examples::Example, game::Game};

pub struct MyExample {
    game: Game,
    tick_timer: PeriodicTimer,

    canvas: Canvas,
    draw_game: DrawGame,
}

impl MyExample {
    pub fn new() -> Result<Self, malen::Error> {
        let game = Game::default();
        let tick_timer = PeriodicTimer::new(game.tick_time_delta.to_local_time_delta());

        let canvas = Canvas::from_element_id("example_local_canvas")?;
        let draw_game = DrawGame::new(&canvas)?;

        Ok(Self {
            game,
            tick_timer,
            canvas,
            draw_game,
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

        let transform = self.canvas.screen_geom().orthographic_projection()
            * Camera::screen_view_matrix(&self.canvas.screen_geom());
        self.draw_game.draw(&self.canvas, &self.game, &transform)?;

        Ok(())
    }
}
