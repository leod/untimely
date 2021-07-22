use malen::{
    draw::{
        ColPass, ColVertex, Font, Light, LineBatch, OccluderBatch, ShadowColPass, ShadowMap,
        TextBatch, TriBatch,
    },
    AxisRect, Camera, Canvas, Color3, Color4, Screen,
};
use nalgebra::{Matrix3, Point2, Point3, Vector2};

use untimely::PlayerId;

use crate::game::{Bullet, Game, Player, Wall};

pub struct DrawGame {
    font: Font,

    occluder_batch: OccluderBatch,
    shadowed_tri_col_batch: TriBatch<ColVertex>,
    plain_tri_col_batch: TriBatch<ColVertex>,
    line_col_batch: LineBatch<ColVertex>,
    text_batch: TextBatch,
    lights: Vec<Light>,

    col_pass: ColPass,
    shadow_col_pass: ShadowColPass,

    shadow_map: ShadowMap,
}

impl DrawGame {
    pub fn new(canvas: &Canvas) -> Result<Self, malen::Error> {
        Ok(Self {
            font: Font::from_bytes(
                canvas,
                include_bytes!("../resources/Roboto-Regular.ttf").to_vec(),
                40.0,
            )?,
            occluder_batch: OccluderBatch::new(canvas)?,
            shadowed_tri_col_batch: TriBatch::new(canvas)?,
            plain_tri_col_batch: TriBatch::new(canvas)?,
            line_col_batch: LineBatch::new(canvas)?,
            text_batch: TextBatch::new(canvas)?,
            col_pass: ColPass::new(canvas)?,
            lights: Vec::new(),
            shadow_col_pass: ShadowColPass::new(canvas)?,
            shadow_map: ShadowMap::new(canvas, 512, 32)?,
        })
    }

    pub fn font_mut(&mut self) -> &mut Font {
        &mut self.font
    }

    pub fn draw(&mut self, canvas: &Canvas, games: &[(&str, &Game)]) -> Result<(), malen::Error> {
        self.occluder_batch.clear();
        self.shadowed_tri_col_batch.clear();
        self.plain_tri_col_batch.clear();
        self.line_col_batch.clear();
        self.text_batch.clear();
        self.lights.clear();

        {
            let padding = 15.0;
            let mut x_start = 0.0;

            for (name, game) in games.iter() {
                self.render_game(game, Vector2::new(x_start, 0.0));
                self.font.write(
                    20.0,
                    Point3::new(x_start + 10.0, 7.5, 0.0),
                    Color4::new(1.0, 0.0, 0.0, 1.0),
                    name,
                    &mut self.text_batch,
                );

                x_start += Game::MAP_WIDTH + padding;
            }
        }

        let view = Camera::screen_view_matrix(&canvas.screen());
        let transform = canvas.screen().orthographic_projection() * view;

        canvas.clear(Color4::new(1.0, 1.0, 1.0, 1.0));
        self.shadow_map
            .build(canvas, &view, &self.lights)?
            .draw_occluders(&self.occluder_batch.draw_unit())?
            .finish()?;
        self.shadow_col_pass.draw(
            &transform,
            Color3::new(0.4, 0.4, 0.4),
            &self.shadow_map,
            &self.shadowed_tri_col_batch.draw_unit(),
        )?;
        self.col_pass
            .draw(&transform, &self.plain_tri_col_batch.draw_unit())?;
        self.col_pass
            .draw(&transform, &self.line_col_batch.draw_unit())?;
        self.font
            .draw(canvas, &transform, &self.text_batch.draw_unit())?;

        Ok(())
    }

    fn render_game(&mut self, game: &Game, offset: Vector2<f32>) {
        let map_rect = AxisRect::from_top_left(
            Point2::origin(),
            Vector2::new(Game::MAP_WIDTH, Game::MAP_HEIGHT),
        );
        self.shadowed_tri_col_batch.push_quad(
            &map_rect.translate(offset).into(),
            -1.0,
            Color4::new(0.9, 0.9, 0.9, 1.0),
        );

        for (player_id, player) in game.players.iter() {
            self.render_player(*player_id, player, offset);
        }

        for wall in game.walls.iter() {
            self.render_wall(wall, offset);
        }

        self.line_col_batch.push_quad_outline(
            &map_rect.translate(offset).into(),
            0.0,
            Color4::new(0.0, 0.0, 0.0, 1.0),
        );
    }

    fn render_player(&mut self, player_id: PlayerId, player: &Player, offset: Vector2<f32>) {
        let color = if player_id.0 % 2 == 0 {
            // Green-ish for player 1
            Color4::from_u8(100, 200, 100, 255)
        } else {
            // Blue-ish for player 2
            Color4::from_u8(100, 100, 200, 255)
        };
        let rect = player.axis_rect().translate(offset);

        self.plain_tri_col_batch.push_quad(&rect.into(), 0.0, color);
        self.lights.push(Light {
            world_pos: player.pos + offset,
            radius: 150.0,
            angle: 0.0,
            angle_size: 2.0 * std::f32::consts::PI,
            color: Color3::new(color.r, color.g, color.b),
        });
        self.occluder_batch.push_occluder_quad(
            &rect.into(),
            Some(self.shadow_map.light_offset(self.lights.len() - 1)),
        );

        if player_id.0 == 0 {
            self.line_col_batch.push_quad_outline(
                &player.axis_rect().translate(offset).into(),
                0.0,
                Color4::new(0.0, 0.0, 0.0, 1.0),
            );
        }
    }

    fn render_wall(&mut self, wall: &Wall, offset: Vector2<f32>) {
        self.plain_tri_col_batch.push_quad(
            &wall.0.translate(offset).into(),
            0.0,
            Color4::from_u8(75, 75, 75, 255),
        );
        self.occluder_batch
            .push_occluder_quad(&wall.0.translate(offset).into(), None);
        /*self.line_col_batch
        .push_quad_outline(&wall.0.into(), 0.0, Color4::new(0.0, 0.0, 0.0, 1.0));*/
    }
}
