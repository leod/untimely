use malen::{Canvas, Color4};
use untimely::{LocalDt, LocalTime, PeriodicTimer, PlayerId};

use crate::{current_game_input, DrawGame, Figure, Game, GameInput};

pub struct Figure1 {
    game: Game,
    tick_timer: PeriodicTimer,

    canvas: Canvas,
    draw_game: DrawGame,
    last_input: GameInput,
}

impl Figure1 {
    pub fn new() -> Result<Self, malen::Error> {
        let game = Game::default();
        let tick_timer = PeriodicTimer::new(game.params.dt.to_local_dt());

        let canvas = Canvas::from_element_id("figure1")?;
        let draw_game = DrawGame::new(&canvas)?;

        Ok(Self {
            game,
            tick_timer,
            canvas,
            draw_game,
            last_input: GameInput::default(),
        })
    }
}

impl Figure for Figure1 {
    fn update(&mut self, time: LocalTime, dt: LocalDt) {
        while let Some(_) = self.canvas.pop_event() {}

        self.tick_timer.advance(dt);
        if self.tick_timer.trigger() {
            self.last_input =
                current_game_input(PlayerId(0), self.game.time, &self.canvas.input_state());

            self.game.run_input(PlayerId(0), &self.last_input);
            self.game.time += self.game.params.dt;
        }
    }

    fn draw(&mut self) -> Result<(), malen::Error> {
        self.canvas.clear(Color4::new(1.0, 1.0, 1.0, 1.0));

        self.draw_game.draw(
            &self.canvas,
            &[("Anna", &self.game, Some(self.last_input), None)],
        )?;

        Ok(())
    }

    fn is_active(&self) -> bool {
        self.canvas.has_focus()
    }
}
