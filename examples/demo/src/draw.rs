use malen::{
    draw::{ColPass, ColVertex, Font, LineBatch, TextBatch, TriBatch},
    AaRect, Camera, Canvas, Color4, Screen,
};
use nalgebra::{Matrix3, Point2, Point3, Vector2};

use untimely::PlayerId;

use crate::game::{Bullet, Game, Player, Wall};

pub struct DrawGame {
    font: Font,
    tri_col_batch: TriBatch<ColVertex>,
    line_col_batch: LineBatch<ColVertex>,
    text_batch: TextBatch,
    col_pass: ColPass,
}

impl DrawGame {
    pub fn new(canvas: &Canvas) -> Result<Self, malen::Error> {
        Ok(Self {
            font: Font::from_bytes(
                canvas,
                include_bytes!("../resources/Roboto-Regular.ttf").to_vec(),
                40.0,
            )?,
            tri_col_batch: TriBatch::new(canvas)?,
            line_col_batch: LineBatch::new(canvas)?,
            text_batch: TextBatch::new(canvas)?,
            col_pass: ColPass::new(canvas)?,
        })
    }

    pub fn font_mut(&mut self) -> &mut Font {
        &mut self.font
    }

    pub fn draw(
        &mut self,
        canvas: &Canvas,
        transform: &Matrix3<f32>,
        game: &Game,
    ) -> Result<(), malen::Error> {
        self.tri_col_batch.clear();
        self.line_col_batch.clear();
        self.text_batch.clear();

        self.render(game);

        self.col_pass
            .draw(transform, &self.tri_col_batch.draw_unit())?;
        self.col_pass
            .draw(transform, &self.line_col_batch.draw_unit())?;

        Ok(())
    }

    pub fn draw_multiple(
        &mut self,
        canvas: &Canvas,
        games: &[(&str, &Game)],
    ) -> Result<(), malen::Error> {
        canvas.clear(Color4::new(1.0, 1.0, 1.0, 1.0));

        let padding = 15.0;
        let mut x_start = 0.0;

        for (name, game) in games.iter() {
            let transform = canvas.screen().orthographic_projection()
                * Matrix3::new_translation(&Vector2::new(x_start, 0.0))
                * Camera::screen_view_matrix(&canvas.screen());

            self.draw(canvas, &transform, game)?;

            self.font.write(
                20.0,
                Point3::new(20.0, 20.0, 0.0),
                Color4::new(1.0, 0.0, 0.0, 1.0),
                name,
                &mut self.text_batch,
            );
            self.font
                .draw(canvas, &transform, &self.text_batch.draw_unit())?;

            x_start += Game::MAP_WIDTH + padding;
        }

        Ok(())
    }

    fn render(&mut self, game: &Game) {
        let map_rect = AaRect::from_top_left(
            Point2::origin(),
            Vector2::new(Game::MAP_WIDTH, Game::MAP_HEIGHT),
        );
        self.tri_col_batch
            .push_quad(&map_rect.into(), -1.0, Color4::new(0.9, 0.9, 0.9, 1.0));

        for (player_id, player) in game.players.iter() {
            self.render_player(*player_id, player);
        }

        for wall in game.walls.iter() {
            self.render_wall(wall);
        }

        self.line_col_batch.push_quad_outline(
            &map_rect.into(),
            0.0,
            Color4::new(0.0, 0.0, 0.0, 1.0),
        );
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

        if player_id.0 == 0 {
            self.line_col_batch.push_quad_outline(
                &player.aa_rect().into(),
                0.0,
                Color4::new(0.0, 0.0, 0.0, 1.0),
            );
        }
    }

    fn render_wall(&mut self, wall: &Wall) {
        self.tri_col_batch
            .push_quad(&wall.0.into(), 0.0, Color4::from_u8(100, 100, 100, 255));
        /*self.line_col_batch
        .push_quad_outline(&wall.0.into(), 0.0, Color4::new(0.0, 0.0, 0.0, 1.0));*/
    }
}
