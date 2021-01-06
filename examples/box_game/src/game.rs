use std::collections::BTreeMap;

use nalgebra::{Point2, Vector2};

use malen::AaRect;

use untimely::{EntityId, GameTimeDelta, PlayerId};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub shoot: bool,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub pos: Point2<f32>,
}

impl Player {
    pub const MOVE_SPEED: f32 = 200.0;
    pub const SIZE: f32 = 25.0;

    pub fn aa_rect(&self) -> AaRect {
        AaRect {
            center: self.pos,
            size: Vector2::new(Self::SIZE, Self::SIZE),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Bullet {
    pub owner: PlayerId,
    pub pos: Point2<f32>,
    pub angle: f32,
}

#[derive(Debug, Clone)]
pub struct Wall(pub AaRect);

#[derive(Debug, Clone)]
pub struct Game {
    pub tick_time_delta: GameTimeDelta,
    pub players: BTreeMap<PlayerId, Player>,
    pub bullets: BTreeMap<EntityId, Bullet>,
    pub walls: Vec<Wall>,
}

impl Game {
    pub fn new() -> Self {
        let players = vec![
            (
                PlayerId(0),
                Player {
                    pos: Point2::new(100.0, 100.0),
                },
            ),
            (
                PlayerId(1),
                Player {
                    pos: Point2::new(300.0, 100.0),
                },
            ),
        ];

        Self {
            tick_time_delta: GameTimeDelta::from_hz(16.0),
            players: players.into_iter().collect(),
            bullets: BTreeMap::new(),
            walls: Self::walls(),
        }
    }

    pub fn run_input(&mut self, player_id: PlayerId, input: &Input) {
        let dt = self.tick_time_delta.to_secs_f32();

        if let Some(mut player) = self.players.get(&player_id).cloned() {
            let move_dir = Self::input_to_move_dir(input);
            player.pos += move_dir * Player::MOVE_SPEED * dt;

            for wall in self.walls.iter() {
                if let Some(response_vector) = Self::check_overlap(player.aa_rect(), wall.0) {
                    player.pos += response_vector;
                }
            }

            self.players.insert(player_id, player);
        }
    }

    fn check_overlap(a: AaRect, b: AaRect) -> Option<Vector2<f32>> {
        // Top left
        let a_min = a.center - a.size / 2.0;
        let b_min = b.center - b.size / 2.0;

        // Bottom right
        let a_max = a.center + a.size / 2.0;
        let b_max = b.center + b.size / 2.0;

        let overlap_x = Self::range_overlap(a_min.x, a_max.x, b_min.x, b_max.x);
        let overlap_y = Self::range_overlap(a_min.y, a_max.y, b_min.y, b_max.y);

        if overlap_x > 0.0 && overlap_y > 0.0 {
            if overlap_x < overlap_y {
                Some((a_max.x - b_max.x).signum() * Vector2::new(overlap_x, 0.0))
            } else {
                Some((a_max.y - b_max.y).signum() * Vector2::new(0.0, overlap_y))
            }
        } else {
            None
        }
    }

    fn range_overlap(min_a: f32, max_a: f32, min_b: f32, max_b: f32) -> f32 {
        (max_a.min(max_b) - min_a.max(min_b)).max(0.0)
    }

    fn input_to_move_dir(input: &Input) -> Vector2<f32> {
        let mut dir = Vector2::zeros();
        if input.left {
            dir.x -= 1.0;
        }
        if input.right {
            dir.x += 1.0;
        }
        if input.up {
            dir.y -= 1.0;
        }
        if input.down {
            dir.y += 1.0;
        }

        if dir.norm() > 0.0 {
            dir.normalize()
        } else {
            dir
        }
    }

    fn walls() -> Vec<Wall> {
        vec![
            Wall(AaRect {
                center: Point2::new(0.0, 200.0),
                size: Vector2::new(50.0, 400.0),
            }),
            Wall(AaRect {
                center: Point2::new(400.0, 200.0),
                size: Vector2::new(50.0, 400.0),
            }),
            Wall(AaRect {
                center: Point2::new(200.0, 0.0),
                size: Vector2::new(200.0, 50.0),
            }),
        ]
    }
}
