use malen::{
    draw::{
        plot::{Axis, Line, Plot, Plotting},
        ColPass, ColVertex, Font, Light, LineBatch, OccluderBatch, ShadowColPass, ShadowMap,
        TextBatch, TriBatch,
    },
    AxisRect, Camera, Canvas, Color3, Color4,
};
use nalgebra::{Matrix3, Point2, Point3, Vector2};

use untimely::{LocalTime, Metrics, PlayerId};

use crate::game::{Game, GameInput, Player, Wall};

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

    plotting: Plotting,
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
            plotting: Plotting::new(canvas)?,
        })
    }

    pub fn font_mut(&mut self) -> &mut Font {
        &mut self.font
    }

    pub fn draw(
        &mut self,
        canvas: &Canvas,
        games: &[(&str, Option<&Game>, Option<GameInput>, Option<GameInput>)],
    ) -> Result<(), malen::Error> {
        self.occluder_batch.clear();
        self.shadowed_tri_col_batch.clear();
        self.plain_tri_col_batch.clear();
        self.line_col_batch.clear();
        self.text_batch.clear();
        self.lights.clear();

        {
            let padding = 15.0;
            let mut x_start = 0.0;

            for (name, game, input1, input2) in games.iter() {
                if let Some(game) = game {
                    self.render_game(game, Vector2::new(x_start, 0.0));
                }

                self.font.write(
                    20.0,
                    Point3::new(x_start + 10.0, 7.5, 0.0),
                    //Color4::new(1.0, 0.0, 0.0, 1.0),
                    Color4::new(1.0, 1.0, 1.0, 1.0),
                    name,
                    &mut self.text_batch,
                );
                if let Some(input1) = input1.as_ref() {
                    self.render_input(input1, Vector2::new(x_start + 10.0, 32.5));
                }
                if let Some(input2) = input2.as_ref() {
                    self.render_input(input2, Vector2::new(x_start + 10.0, 51.25));
                }

                x_start += Game::MAP_WIDTH + padding;
            }
        }

        let view = Camera::screen_view_matrix(&canvas.screen());
        let transform = canvas.screen().orthographic_projection() * view;

        canvas.clear(Color4::from_u8(248, 249, 250, 255));
        self.shadow_map
            .build(canvas, &view, &self.lights)?
            .draw_occluders(&self.occluder_batch.draw_unit())?
            .finish()?;
        self.shadow_col_pass.draw(
            &transform,
            Color3::new(0.2, 0.2, 0.2),
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

    pub fn draw_plot(
        &mut self,
        canvas: &Canvas,
        max_time: LocalTime,
        metrics: &Metrics,
    ) -> Result<(), malen::Error> {
        let transform = canvas.screen().orthographic_projection()
            * Matrix3::new_translation(&Vector2::new(0.0, Game::MAP_HEIGHT + 15.0));

        let plot = self.metrics_plot(max_time, metrics);
        self.plotting
            .draw(canvas, &mut self.font, &transform, &plot)
    }

    fn render_game(&mut self, game: &Game, offset: Vector2<f32>) {
        let map_rect = AxisRect::from_top_left(
            Point2::origin(),
            Vector2::new(Game::MAP_WIDTH, Game::MAP_HEIGHT),
        );
        self.shadowed_tri_col_batch.push_quad(
            &map_rect.translate(offset).into(),
            -1.0,
            //Color4::new(0.9, 0.9, 0.9, 1.0),
            Color4::from_u8(175, 238, 238, 255),
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

    fn render_input(&mut self, input: &GameInput, mut offset: Vector2<f32>) {
        let letters = vec![
            (input.up, "W"),
            (input.left, "A"),
            (input.down, "S"),
            (input.right, "D"),
        ];

        for (is_active, letter) in letters {
            let color = if is_active {
                Color4::from_u8(255, 255, 255, 255)
            } else {
                Color4::from_u8(150, 150, 150, 255)
            };

            offset.x += self
                .font
                .write(
                    15.0,
                    Point3::new(offset.x, offset.y, 0.0),
                    color,
                    &letter,
                    &mut self.text_batch,
                )
                .x;
            offset.x += 5.0;
        }
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
            color: Color3::new(color.r / 3.0, color.g / 3.0, color.b / 3.0),
        });
        self.occluder_batch.push_occluder_quad(
            &rect.into(),
            Some(self.shadow_map.light_offset(self.lights.len() - 1)),
        );

        self.line_col_batch.push_quad_outline(
            &player.axis_rect().translate(offset).into(),
            0.0,
            Color4::new(0.0, 0.0, 0.0, 1.0),
        );
    }

    fn render_wall(&mut self, wall: &Wall, offset: Vector2<f32>) {
        self.plain_tri_col_batch.push_quad(
            &wall.0.translate(offset).into(),
            0.0,
            Color4::from_u8(0, 0, 0, 255),
        );
        self.occluder_batch
            .push_occluder_quad(&wall.0.translate(offset).into(), None);
        self.line_col_batch.push_quad_outline(
            &wall.0.translate(offset).into(),
            0.0,
            Color4::new(0.8, 0.8, 0.8, 1.0),
        );
    }

    fn metrics_plot(&self, max_time: LocalTime, metrics: &Metrics) -> Plot {
        let mut lines = Vec::new();

        let shift = |points: &[(f64, f64)]| {
            points
                .iter()
                .map(|(x, y)| (*x - max_time.to_secs(), *y))
                .collect::<Vec<_>>()
        };

        if let Some(gauge) = metrics.get_gauge("anna_server_delay") {
            lines.push(Line {
                caption: "delay anna to server".to_string(),
                color: Color4::new(0.2, 0.8, 0.2, 1.0),
                points: shift(&gauge.plot_points()),
            });
        }
        if let Some(gauge) = metrics.get_gauge("anna_stream_delay") {
            lines.push(Line {
                caption: "delay anna to stream".to_string(),
                color: Color4::new(0.8, 0.8, 0.2, 1.0),
                points: shift(&gauge.plot_points()),
            });
        }
        if let Some(gauge) = metrics.get_gauge("brad_server_delay") {
            lines.push(Line {
                caption: "delay brad to server".to_string(),
                color: Color4::new(0.2, 0.2, 0.8, 1.0),
                points: shift(&gauge.plot_points()),
            });
        }
        if let Some(gauge) = metrics.get_gauge("brad_stream_delay") {
            lines.push(Line {
                caption: "delay brad to stream".to_string(),
                color: Color4::new(0.2, 0.8, 0.8, 1.0),
                points: shift(&gauge.plot_points()),
            });
        }
        /*if let Some(gauge) = metrics.get_gauge("anna_stream_server_delay") {
            lines.push(Line {
                caption: "anna_stream_server_delay".to_string(),
                color: Color4::new(0.8, 0.0, 0.8, 1.0),
                points: shift(&gauge.plot_points()),
            });
        }
        if let Some(gauge) = metrics.get_gauge("brad_stream_server_delay") {
            lines.push(Line {
                caption: "brad_stream_server_delay".to_string(),
                color: Color4::new(0.2, 0.8, 0.8, 1.0),
                points: shift(&gauge.plot_points()),
            });
        }*/

        Plot {
            size: Vector2::new(990.0, 200.0),
            x_axis: Axis {
                label: "local time [s]".to_string(),
                range: Some((-5.0, 0.0)),
                tics: 1.0,
                tic_precision: 1,
            },
            y_axis: Axis {
                label: "game time [s]".to_string(),
                range: Some((0.0, 0.5)),
                tics: 0.1,
                tic_precision: 1,
            },
            axis_color: Color4::new(0.0, 0.0, 0.0, 1.0),
            background_color: None,
            text_color: Color4::new(0.0, 0.0, 0.0, 1.0),
            lines,
        }
    }
}
