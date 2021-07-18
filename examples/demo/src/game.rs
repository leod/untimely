use std::collections::BTreeMap;

use nalgebra::{Point2, Vector2};

use malen::AaRect;

use untimely::{EntityId, GameDt, GameTime, PlayerId, TickNum};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct GameInput {
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
    pub const SIZE: f32 = 15.0;

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
pub struct GameParams {
    pub dt: GameDt,
}

impl Default for GameParams {
    fn default() -> Self {
        Self {
            dt: GameDt::from_hz(20.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    pub params: GameParams,
    pub time: GameTime,
    pub players: BTreeMap<PlayerId, Player>,
    pub bullets: BTreeMap<EntityId, Bullet>,
    pub walls: Vec<Wall>,
}

impl Default for Game {
    fn default() -> Self {
        let players = vec![
            (
                PlayerId(0),
                Player {
                    pos: Point2::new(50.0, Game::MAP_HEIGHT / 2.0),
                },
            ),
            (
                PlayerId(1),
                Player {
                    pos: Point2::new(Game::MAP_WIDTH - 50.0, Game::MAP_HEIGHT / 2.0),
                },
            ),
        ];

        Self {
            params: GameParams::default(),
            time: GameTime::zero(),
            players: players.into_iter().collect(),
            bullets: BTreeMap::new(),
            walls: Self::walls(),
        }
    }
}

impl Game {
    pub const MAP_WIDTH: f32 = 420.0;
    pub const MAP_HEIGHT: f32 = 315.0;

    pub fn run_input(&mut self, player_id: PlayerId, input: &GameInput) {
        let dt = self.params.dt.to_secs() as f32;

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

    fn input_to_move_dir(input: &GameInput) -> Vector2<f32> {
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
        // Border size
        let b = 15.0;

        let w = Self::MAP_WIDTH;
        let h = Self::MAP_HEIGHT;

        vec![
            Wall(AaRect {
                center: Point2::new(b / 2.0, h / 2.0),
                size: Vector2::new(b, h - 2.0 * b),
            }),
            Wall(AaRect {
                center: Point2::new(w - b / 2.0, h / 2.0),
                size: Vector2::new(b, h - 2.0 * b),
            }),
            Wall(AaRect {
                center: Point2::new(w / 2.0, b / 2.0),
                size: Vector2::new(w, b),
            }),
            Wall(AaRect {
                center: Point2::new(w / 2.0, h - b / 2.0),
                size: Vector2::new(w, b),
            }),
            Wall(AaRect {
                center: Point2::new(w / 2.0, h / 2.0),
                size: Vector2::new(b * 2.0, h * 0.618),
            }),
        ]
    }
}
