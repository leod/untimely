use malen::{
    draw::{ColPass, ColVertex, Font, TextBatch, TriBatch},
    AaRect, Camera, Canvas, Color4, ScreenGeom,
};
use nalgebra::{Matrix3, Point2, Vector2};

use untimely::PlayerId;

use crate::game::{Bullet, Game, Player, Wall};

pub struct DrawGame {
    tri_col_batch: TriBatch<ColVertex>,
    col_pass: ColPass,
}

impl DrawGame {
    pub fn new(canvas: &Canvas) -> Result<Self, malen::Error> {
        Ok(Self {
            tri_col_batch: TriBatch::new(canvas)?,
            col_pass: ColPass::new(canvas)?,
        })
    }

    pub fn draw(
        &mut self,
        canvas: &Canvas,
        game: &Game,
        transform: &Matrix3<f32>,
    ) -> Result<(), malen::Error> {
        self.tri_col_batch.clear();
        self.render(game);

        self.col_pass
            .draw(transform, &self.tri_col_batch.draw_unit())?;

        Ok(())
    }

    pub fn draw_multiple(
        &mut self,
        canvas: &Canvas,
        games: &[(&str, &Game)],
    ) -> Result<(), malen::Error> {
        // TODO: Need to consider device_pixel_ratio here?
        let screen_geom = ScreenGeom {
            size: Vector2::new(Game::MAP_WIDTH as u32, Game::MAP_HEIGHT as u32),
            device_pixel_ratio: canvas.screen_geom().device_pixel_ratio,
        };

        for (i, (name, game)) in games.iter().enumerate() {
            canvas.set_viewport(
                Point2::new(i as u32 * Game::MAP_WIDTH as u32, 0),
                screen_geom.size,
            );
            let transform =
                screen_geom.orthographic_projection() * Camera::screen_view_matrix(&screen_geom);

            self.draw(canvas, game, &transform)?;
        }

        Ok(())
    }

    fn render(&mut self, game: &Game) {
        for (player_id, player) in game.players.iter() {
            self.render_player(*player_id, player);
        }

        for wall in game.walls.iter() {
            self.render_wall(wall);
        }
    }

    fn render_player(&mut self, player_id: PlayerId, player: &Player) {
        let color = if player_id.0 % 2 == 0 {
            // Green-ish for player 1
            Color4::from_u8(100, 200, 100, 255)
        } else {
            // Blue-ish for player 2
            Color4::from_u8(100, 100, 200, 255)
        };

        self.tri_col_batch
            .push_quad(&player.aa_rect().into(), 0.0, color);
    }

    fn render_wall(&mut self, wall: &Wall) {
        self.tri_col_batch
            .push_quad(&wall.0.into(), 0.0, Color4::from_u8(100, 100, 100, 255));
    }
}
