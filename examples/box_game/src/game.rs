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
    pub const MOVE_SPEED: f32 = 100.0;
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

            self.players.insert(player_id, player);
        }
    }

    fn correct_delta_for_collision(&self, a: AaRect, b: AaRect, a_delta: &mut Vector2<f32>) {
        let a_start_pos = a.center;
        let a_end_pos = a.center + *a_delta;
        let b_pos = b.center;

        //let overlap_x =
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
