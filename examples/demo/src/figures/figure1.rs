use untimely::{LocalClock, LocalDt, PeriodicTimer, PlayerId};

use crate::{current_game_input, is_active, DrawGame, Figure, Game, GameInput};

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

        let draw_game = DrawGame::new(&[("figure1_anja", "Anja")], &[])?;

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
    fn update(&mut self, dt: LocalDt) {
        self.draw_game.update();
        if !is_active("figure1", &self.clock) {
            return;
        }

        self.clock.advance(dt);
        self.tick_timer.advance(dt);

        while self.tick_timer.trigger() {
            self.last_input =
                current_game_input(PlayerId(0), self.game.time, &self.draw_game.input_state(0));

            self.game.run_input(PlayerId(0), &self.last_input);
            self.game.time += self.game.params.dt;
        }
    }

    fn draw(&mut self) -> Result<(), malen::Error> {
        if !is_active("figure1", &self.clock) {
            return Ok(());
        }

        let games = &[(Some(&self.game), Some(self.last_input.clone()), None)];
        self.draw_game.draw(games)?;

        Ok(())
    }
}
