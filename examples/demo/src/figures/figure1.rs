use untimely::{LocalClock, LocalDt, LocalTime, PeriodicTimer, PlayerId};

use crate::{current_game_input, DrawGame, Figure, Game, GameInput};

pub struct Figure1 {
    clock: LocalClock,

    game: Game,
    tick_timer: PeriodicTimer,

    draw_game: DrawGame,
    last_input: GameInput,
}

impl Figure1 {
    pub fn new() -> Result<Self, malen::Error> {
        let clock = LocalClock::new();

        let game = Game::default();
        let tick_timer = PeriodicTimer::new(game.params.dt.to_local_dt());

        let draw_game = DrawGame::new(&[("figure1", "Anja")], &[])?;

        Ok(Self {
            game,
            clock,
            tick_timer,
            draw_game,
            last_input: GameInput::default(),
        })
    }
}

impl Figure for Figure1 {
    fn update(&mut self, time: LocalTime) {
        self.draw_game.update();

        let dt = self.clock.set_local_time(time).min(LocalDt::from_secs(1.0));
        self.tick_timer.advance(dt);

        if self.tick_timer.trigger() {
            self.last_input =
                current_game_input(PlayerId(0), self.game.time, &self.draw_game.input_state(0));

            self.game.run_input(PlayerId(0), &self.last_input);
            self.game.time += self.game.params.dt;
        }
    }

    fn draw(&mut self) -> Result<(), malen::Error> {
        let games = &[(Some(&self.game), Some(self.last_input.clone()), None)];
        self.draw_game.draw(games)?;

        Ok(())
    }
}
