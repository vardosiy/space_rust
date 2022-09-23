use std::time::{Duration, Instant};

use crate::constants::{SHOT_SPEED, SHOT_WIDTH};
use crate::entities::shape::{Shape, Shaped};
use crate::entities::ship::Ship;
use crate::entities::shot::Shot;
use crate::globals::screen_rect;
use crate::math::Vec2i;

use super::boss_stages::BossStage;
use super::Boss;

//-----------------------------------------------------------------------------

const APPEAR_MOVE_SPEED: i32 = 8;
const APPEAR_TARGET_HEIGHT: i32 = 50;

const SIMPLE_SHOOTING_STAGE_MOVE_SPEED: i32 = 12;
const SIMPLE_SHOOTING_STAGE_SHOOTING_INTERVAL: Duration = Duration::from_millis(300);

const SPREAD_SHOOTING_STAGE_MOVE_SPEED: i32 = 8;
const SPREAD_SHOOTING_STAGE_SHOOTING_INTERVAL: Duration = Duration::from_millis(500);
const SPREAD_SHOOTING_ANGLE_RANGE: i32 = 120;
const SPREAD_SHOOTING_ANGLE_STEP: usize = 20;

const TARGETED_STAGE_MOVE_SPEED: i32 = 15;
const TARGETED_STAGE_SHOOTING_INTERVAL: Duration = Duration::from_millis(500);

const STAGE_1_FINISH_HP_THRESHOLD: f32 = 0.7f32;
const STAGE_2_FINISH_HP_THRESHOLD: f32 = 0.4f32;

const BOSS_DAMAGE: i32 = 10;

const ANGLE_DOWN: i32 = 180;

//-----------------------------------------------------------------------------

#[derive(Copy, Clone)]
enum Direction {
    Left,
    Right,
}

fn move_horizontally<T>(stage: &mut T, boss: &Boss, move_speed: i32) -> Vec2i {
    let x_offset = match stage.direction {
        Direction::Left => -move_speed,
        Direction::Right => move_speed,
    };

    let mut new_shape = boss.shape().clone();
    let screen_rect = screen_rect();

    new_shape.pos.x += x_offset;
    if !new_shape.in_rect(&screen_rect) {
        new_shape.pos.x -= x_offset * 2;
        stage.direction = match stage.direction {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        };
    }

    new_shape.pos
}

//-----------------------------------------------------------------------------

fn shoot_down<T>(stage: &mut T, boss: &Boss, shooting_interval: Duration) -> Option<Vec<Shot>> {
    let now = Instant::now();
    if stage.shoot_time + shooting_interval <= now {
        stage.shoot_time = now;

        let shot = make_boss_shot(&boss, ANGLE_DOWN);
        return Some(vec![shot]);
    }

    None
}

fn make_boss_shot(boss: &Boss, angle: i32) -> Shot {
    let shot_shape = Shape::new(boss.center(), SHOT_WIDTH);
    Shot::new(shot_shape, SHOT_SPEED, angle, BOSS_DAMAGE)
}

//-----------------------------------------------------------------------------

pub struct AppearStage;
impl BossStage for AppearStage {
    fn calc_new_pos(&mut self, boss: &Boss, ship: &Ship) -> Vec2i {
        let mut new_pos = boss.pos();
        if !self.completed(&boss) {
            new_pos.y += APPEAR_MOVE_SPEED;
        }

        new_pos
    }

    fn shoot(&mut self, boss: &Boss, ship: &Ship) -> Option<Vec<Shot>> {
        None
    }

    fn completed(&self, boss: &Boss) -> bool {
        boss.pos().y > APPEAR_TARGET_HEIGHT
    }
}

//-----------------------------------------------------------------------------

pub struct SimpleShootingDown {
    direction: Direction,
    shoot_time: Instant,
}

impl SimpleShootingDown {
    pub fn new() -> Self {
        SimpleShootingDown {
            direction: Direction::Right,
            shoot_time: Instant::now(),
        }
    }
}

impl BossStage for SimpleShootingDown {
    fn calc_new_pos(&mut self, boss: &Boss, ship: &Ship) -> Vec2i {
        move_horizontally(&mut self, boss, SIMPLE_SHOOTING_STAGE_MOVE_SPEED)
    }

    fn shoot(&mut self, boss: &Boss, ship: &Ship) -> Option<Vec<Shot>> {
        shoot_down(&mut self, &boss, SIMPLE_SHOOTING_STAGE_SHOOTING_INTERVAL)
    }

    fn completed(&self, boss: &Boss) -> bool {
        boss.hp_percent() < STAGE_1_FINISH_HP_THRESHOLD
    }
}

//-----------------------------------------------------------------------------

pub struct SpreadShooting {
    direction: Direction,
    shoot_time: Instant,
}

impl SpreadShooting {
    pub fn new() -> Self {
        SpreadShooting {
            direction: Direction::Right,
            shoot_time: Instant::now(),
        }
    }
}

impl BossStage for SpreadShooting {
    fn calc_new_pos(&mut self, boss: &Boss, ship: &Ship) -> Vec2i {
        move_horizontally(&mut self, boss, SPREAD_SHOOTING_STAGE_MOVE_SPEED)
    }

    fn shoot(&mut self, boss: &Boss, ship: &Ship) -> Option<Vec<Shot>> {
        let now = Instant::now();
        if self.shoot_time + SPREAD_SHOOTING_STAGE_SHOOTING_INTERVAL > now {
            return None;
        }

        self.shoot_time = now;

        let angle_start = ANGLE_DOWN - SPREAD_SHOOTING_ANGLE_RANGE / 2;
        let angle_end = ANGLE_DOWN + SPREAD_SHOOTING_ANGLE_RANGE / 2;

        let mut shots = vec![];
        for shot_angle in (angle_start..=angle_end).step_by(SPREAD_SHOOTING_ANGLE_STEP) {
            let shot = make_boss_shot(&boss, shot_angle);
            shots.push(shot);
        }

        Some(shots)
    }

    fn completed(&self, boss: &Boss) -> bool {
        boss.hp_percent() < STAGE_2_FINISH_HP_THRESHOLD
    }
}

//-----------------------------------------------------------------------------

pub struct Targeted {
    shoot_time: Instant,
}

impl Targeted {
    pub fn new() -> Self {
        Targeted {
            shoot_time: Instant::now(),
        }
    }
}

impl BossStage for Targeted {
    fn calc_new_pos(&mut self, boss: &Boss, ship: &Ship) -> Vec2i {
        let boss_center = boss.center();
        let ship_center = ship.center();
        let diff_x = boss_center.x - ship_center.x;

        let mut result = boss.pos();
        if diff_x.abs() < TARGETED_STAGE_MOVE_SPEED {
            result.x = ship_center.x - boss.width() / 2;
        } else {
            result.x += if diff_x > 0 {
                TARGETED_STAGE_MOVE_SPEED
            } else {
                -TARGETED_STAGE_MOVE_SPEED
            };
        }

        let screen_rect = screen_rect();
        result.x = result
            .x
            .clamp(screen_rect.top_left.x, screen_rect.bottom_right.x);

        result
    }

    fn shoot(&mut self, boss: &Boss, ship: &Ship) -> Option<Vec<Shot>> {
        shoot_down(&mut self, &boss, TARGETED_STAGE_SHOOTING_INTERVAL)
    }

    fn completed(&self, boss: &Boss) -> bool {
        false
    }
}

//-----------------------------------------------------------------------------
