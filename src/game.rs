//! Original rules reimplemented from the evidence in MECHANICS.md.
use macroquad::prelude::*;

pub const BOARDS: &[u8] = include_bytes!("../assets/boards.bin");
pub const BONUS_NAMES: [&str; 19] = [
    "EXTRA LIFE",
    "LEVEL WARP",
    "ZAP BRICKS",
    "SLOW BALL",
    "MULTIPLY EXPLODING",
    "THRU BRICK",
    "SET OFF EXPLODING",
    "FIREBALL",
    "SHOOTING PADDLE",
    "GRAB PADDLE",
    "EXPAND PADDLE",
    "SHRINK PADDLE",
    "SPLIT BALL",
    "KILL PADDLE",
    "EXPAND PADDLE",
    "FAST BALL",
    "SUPER SHRINK",
    "FALLING BRICKS",
    "SHRINK BALL",
];
#[derive(Clone, Debug)]
pub struct Ball {
    pub p: Vec2,
    pub v: Vec2,
    pub speed: i32,
    pub angle: i32,
    pub held: bool,
    pub offset: f32,
    pub sprite: u32,
    pub bounces: u32,
    pub stuck: u32,
}
impl Ball {
    pub fn size(&self) -> f32 {
        if self.sprite == 55 {
            5.
        } else {
            9.
        }
    }
    pub fn velocity(&mut self) {
        let a = (self.angle as f64).to_radians();
        let c = (a.cos() * 1024.).trunc() / 1024.;
        let s = (a.sin() * 1024.).trunc() / 1024.;
        self.v = vec2(
            (c * self.speed as f64 * 1.2).trunc() as f32,
            (-s * self.speed as f64).trunc() as f32,
        );
    }
    pub fn rebound(&mut self, paddle: f32, width: f32) {
        let f = (self.p.x + self.size() / 2. - (paddle - width / 2.)) / width;
        self.angle = [150, 135, 120, 105, 75, 60, 45, 30][[0.07, 0.2, 0.35, 0.5, 0.65, 0.8, 0.93]
            .iter()
            .filter(|&&t| f > t)
            .count()];
        self.velocity();
        self.p.y = 450. - self.size();
    }
}
#[derive(Clone)]
pub struct Bonus {
    pub p: Vec2,
    pub v: Vec2,
    pub kind: usize,
    pub age: u32,
}
#[derive(Clone)]
pub struct Burst {
    pub p: Vec2,
    pub age: u32,
    pub sprite: u32,
}
#[derive(Clone)]
pub struct Chain {
    pub cell: usize,
    pub delay: u32,
}
#[derive(Default, Clone, Copy)]
pub struct Input {
    pub x: f32,
    pub click: bool,
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Phase {
    Menu,
    Playing,
    Paused,
    Lost,
    Transition,
    GameOver,
    Won,
    Help,
    Scores,
    Editor,
}
pub struct Game {
    pub phase: Phase,
    pub return_phase: Phase,
    pub board: usize,
    pub bricks: [u8; 400],
    pub paddle: f32,
    pub width: f32,
    pub balls: Vec<Ball>,
    pub bonuses: Vec<Bonus>,
    pub bullets: Vec<Vec2>,
    pub bursts: Vec<Burst>,
    pub chains: Vec<Chain>,
    pub score: u32,
    pub lives: i32,
    pub ticks: u64,
    pub timer: u32,
    pub shooting: bool,
    pub grab: bool,
    pub thru: bool,
    pub falling: bool,
    pub message: String,
    pub message_time: u32,
    pub sounds: Vec<usize>,
    pub hit_count: u32,
    pub launches: u32,
    pub catches: u32,
    pub rebounds: u32,
    pub shots: u32,
    pub last_brick_timer: u32,
    rng: u64,
}
impl Game {
    pub fn new() -> Self {
        let mut g = Self {
            phase: Phase::Menu,
            return_phase: Phase::Menu,
            board: 0,
            bricks: [0; 400],
            paddle: 320.,
            width: 73.,
            balls: vec![],
            bonuses: vec![],
            bullets: vec![],
            bursts: vec![],
            chains: vec![],
            score: 0,
            lives: 3,
            ticks: 0,
            timer: 0,
            shooting: false,
            grab: false,
            thru: false,
            falling: false,
            message: String::new(),
            message_time: 0,
            sounds: vec![],
            hit_count: 0,
            launches: 0,
            catches: 0,
            rebounds: 0,
            shots: 0,
            last_brick_timer: 0,
            rng: 0x36b12478,
        };
        g.load_board();
        g.serve();
        g
    }
    fn random(&mut self, n: u32) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng % n as u64) as u32
    }
    pub fn start(&mut self) {
        self.board = 0;
        self.score = 0;
        self.lives = 3;
        self.load_board();
        self.reset_bonuses();
        self.serve();
        self.phase = Phase::Playing;
        self.notice("CLICK TO LAUNCH");
    }
    pub fn load_board(&mut self) {
        self.bricks
            .copy_from_slice(&BOARDS[self.board * 400..(self.board + 1) * 400]);
        self.last_brick_timer = 0;
    }
    pub fn reset_bonuses(&mut self) {
        self.width = 73.;
        self.shooting = false;
        self.grab = false;
        self.thru = false;
        self.falling = false;
        self.bonuses.clear();
        self.bullets.clear();
        self.chains.clear();
        self.bursts.clear();
    }
    pub fn serve(&mut self) {
        self.balls = vec![Ball {
            p: vec2(self.paddle, 441.),
            v: vec2(0., 0.),
            speed: 5,
            angle: 75,
            held: true,
            offset: 0.,
            sprite: 1,
            bounces: 0,
            stuck: 0,
        }];
    }
    pub fn notice(&mut self, s: &str) {
        self.message = s.into();
        self.message_time = 150;
    }
    pub fn remaining(&self) -> usize {
        self.bricks.iter().filter(|&&b| b != 0 && b != 2).count()
    }
    pub fn step(&mut self, input: Input) {
        self.ticks += 1;
        if self.message_time > 0 {
            self.message_time -= 1;
        }
        for e in &mut self.bursts {
            e.age += 1;
        }
        self.bursts.retain(|e| e.age < 36);
        if matches!(self.phase, Phase::Lost | Phase::Transition) {
            self.timer = self.timer.saturating_sub(1);
            if self.timer == 0 {
                if self.phase == Phase::Transition {
                    self.board += 1;
                    if self.board >= 50 {
                        self.phase = Phase::Won;
                        return;
                    }
                    self.load_board();
                }
                self.reset_bonuses();
                self.serve();
                self.phase = Phase::Playing;
                self.notice("CLICK TO LAUNCH");
            }
            return;
        }
        if self.phase != Phase::Playing {
            return;
        }
        self.paddle = input
            .x
            .round()
            .clamp(21. + self.width / 2., 618. - self.width / 2.);
        let mut active = std::mem::take(&mut self.balls);
        for b in &mut active {
            if b.held {
                b.p = vec2(self.paddle + b.offset, 450. - b.size());
                if input.click {
                    b.held = false;
                    b.rebound(self.paddle, self.width);
                    self.launches += 1;
                    self.message_time = 0;
                }
                continue;
            }
            let prev = b.p;
            b.p += b.v;
            let size = b.size();
            let mut wall = false;
            if b.p.x < 20. {
                b.p.x = 20.;
                b.v.x = b.v.x.abs();
                wall = true;
            }
            if b.p.x > 619. - size {
                b.p.x = 619. - size;
                b.v.x = -b.v.x.abs();
                wall = true;
            }
            if b.p.y < 0. {
                b.p.y = 0.;
                b.v.y = b.v.y.abs();
                wall = true;
            }
            if wall {
                self.sounds.push(4);
                b.bounces += 1;
                b.stuck += 1;
            }
            if b.v.y > 0.
                && b.p.y + size >= 450.
                && prev.y + size <= 457.
                && b.p.x + size >= self.paddle - self.width / 2.
                && b.p.x <= self.paddle + self.width / 2.
            {
                if b.bounces > 40 {
                    b.speed = (b.speed + 1).min(9);
                    b.bounces = 0;
                }
                b.stuck = 0;
                self.rebounds += 1;
                self.sounds.push(if self.grab { 18 } else { 0 });
                if self.falling {
                    self.drop_bricks();
                }
                if self.grab {
                    b.held = true;
                    b.offset =
                        (b.p.x - self.paddle).clamp(-self.width / 2., self.width / 2. - size);
                    b.p.y = 450. - size;
                } else {
                    b.rebound(self.paddle, self.width);
                }
            }
            // Leading-edge probes, like the original's integer grid sampler.
            let leading_y = if b.v.y < 0. { b.p.y } else { b.p.y + size };
            if let Some(i) = self.cell(b.p.x + size / 2., leading_y) {
                if b.sprite == 61 {
                    self.bricks[i] = 8;
                }
                self.hit(i, b.speed * 2, b.v);
                if !self.thru {
                    b.v.y = -b.v.y;
                    if prev.y + size <= 50. + (i / 20) as f32 * 15. {
                        b.p.y = 50. + (i / 20) as f32 * 15. - size;
                    } else if prev.y >= 65. + (i / 20) as f32 * 15. {
                        b.p.y = 65. + (i / 20) as f32 * 15.;
                    } else {
                        b.p = prev;
                    }
                }
                b.bounces += 1;
                b.stuck += 1;
            } else {
                let x = if b.v.x < 0. {
                    b.p.x - 2.
                } else {
                    b.p.x + size + 2.
                };
                if let Some(i) = self.cell(x, b.p.y + size / 2.) {
                    if b.sprite == 61 {
                        self.bricks[i] = 8;
                    }
                    self.hit(i, b.speed * 2, b.v);
                    if !self.thru {
                        b.v.x = -b.v.x;
                        b.p.x = prev.x;
                    }
                    b.bounces += 1;
                    b.stuck += 1;
                }
            }
            if b.stuck > 300 {
                self.zap();
                b.stuck = 0;
            }
        }
        active.retain(|b| {
            if b.p.y > 479. - b.size() {
                self.sounds.push(5);
                false
            } else {
                true
            }
        });
        self.balls = active;
        if self.balls.is_empty() {
            self.lose_life();
            return;
        }
        if input.click && self.shooting && self.bullets.len() < 6 {
            for d in [-1., 1.] {
                self.bullets
                    .push(vec2(self.paddle + d * (self.width / 2. - 7.) - 2., 443.));
            }
            self.shots += 2;
            self.sounds.push(17);
        }
        let bullets = std::mem::take(&mut self.bullets);
        for mut p in bullets {
            p.y -= 8.;
            if p.y < 0. {
                continue;
            }
            if let Some(i) = self.cell(p.x + 2., p.y) {
                self.hit(i, 4, vec2(0., -2.));
                if self.thru {
                    self.bullets.push(p);
                }
            } else {
                self.bullets.push(p);
            }
        }
        let chains = std::mem::take(&mut self.chains);
        for c in chains {
            if c.delay > 0 {
                self.chains.push(Chain {
                    delay: c.delay - 1,
                    ..c
                });
            } else {
                let i = c.cell;
                let old = self.bricks[i];
                if old == 0 {
                    continue;
                }
                self.bricks[i] = 0;
                self.score = self.score.wrapping_add(4);
                self.hit_count += 1;
                self.bursts.push(Burst {
                    p: Self::cell_pos(i),
                    age: 0,
                    sprite: 145,
                });
                self.maybe_bonus(i, vec2(0., -2.));
                if old == 8 {
                    let sound = 30 + self.random(3) as usize;
                    self.sounds.push(sound);
                    let x = (i % 20) as i32;
                    let y = (i / 20) as i32;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            let nx = x + dx;
                            let ny = y + dy;
                            if (0..20).contains(&nx) && (0..20).contains(&ny) {
                                let j = (ny * 20 + nx) as usize;
                                if self.bricks[j] != 0 && !self.chains.iter().any(|c| c.cell == j) {
                                    self.chains.push(Chain { cell: j, delay: 4 });
                                }
                            }
                        }
                    }
                }
            }
        }
        let bonuses = std::mem::take(&mut self.bonuses);
        for mut b in bonuses {
            b.p += b.v;
            b.age += 1;
            if b.age % 21 == 0 {
                b.v.y += 1.;
            }
            if b.p.x < 20. {
                b.p.x = 20.;
                b.v.x = b.v.x.abs();
            }
            if b.p.x > 587. {
                b.p.x = 587.;
                b.v.x = -b.v.x.abs();
            }
            if b.p.y < 0. {
                b.p.y = 0.;
                b.v.y = b.v.y.abs();
            }
            if b.p.y > 448. {
                continue;
            }
            if b.p.y + 30. >= 450.
                && b.p.y <= 457.
                && b.p.x + 32. >= self.paddle - self.width / 2. + 5.
                && b.p.x <= self.paddle + self.width / 2. - 5.
            {
                self.score += 100;
                self.catches += 1;
                self.apply_bonus(b.kind);
            } else {
                self.bonuses.push(b);
            }
        }
        let remaining = self.remaining();
        if remaining == 1 {
            if self.last_brick_timer == 0 {
                self.last_brick_timer = 2400 + self.random(1200);
            }
            self.last_brick_timer -= 1;
            if self.last_brick_timer == 0 {
                if let Some(i) = self.bricks.iter().position(|&b| b != 0 && b != 2) {
                    self.chains.push(Chain { cell: i, delay: 0 });
                    self.bursts.push(Burst {
                        p: Self::cell_pos(i),
                        age: 0,
                        sprite: 145,
                    });
                    self.sounds.push(22);
                }
            }
        } else {
            self.last_brick_timer = 0;
        }
        if remaining == 0 && self.chains.is_empty() {
            self.advance();
        }
        if self.score > 999999999 {
            self.score = 0;
        }
    }
    pub fn cell(&self, x: f32, y: f32) -> Option<usize> {
        if !(20.0..620.).contains(&x) || !(50.0..350.).contains(&y) {
            return None;
        }
        let i = ((y - 50.) as usize / 15) * 20 + (x - 20.) as usize / 30;
        if self.bricks[i] != 0 {
            Some(i)
        } else {
            None
        }
    }
    pub fn cell_pos(i: usize) -> Vec2 {
        vec2(20. + (i % 20) as f32 * 30., 50. + (i / 20) as f32 * 15.)
    }
    fn hit(&mut self, i: usize, points: i32, v: Vec2) {
        if self.chains.iter().any(|c| c.cell == i) {
            return;
        }
        let old = self.bricks[i];
        if old == 0 {
            return;
        }
        let successful = old != 2 || self.thru;
        if successful {
            self.score += points as u32;
        }
        self.hit_count += 1;
        match old {
            2 => {
                if self.thru {
                    self.bricks[i] = 0;
                }
                self.sounds.push(3);
            }
            3 | 4 | 7 => {
                self.bricks[i] = if self.thru {
                    0
                } else {
                    match old {
                        3 => 4,
                        4 => 5,
                        _ => 6,
                    }
                };
                self.sounds.push(1);
            }
            8 => {
                self.chains.push(Chain { cell: i, delay: 4 });
                self.sounds.push(30);
            }
            21 => {
                self.bricks[i] = if self.thru { 0 } else { 2 };
                self.sounds.push(1);
            }
            _ => {
                self.bricks[i] = 0;
                self.sounds.push(7);
            }
        }
        if self.bricks[i] == 0 {
            self.bursts.push(Burst {
                p: Self::cell_pos(i),
                age: 0,
                sprite: 20,
            });
            self.maybe_bonus(i, v);
        } else if old != 8 {
            self.bursts.push(Burst {
                p: Self::cell_pos(i),
                age: 0,
                sprite: 19,
            });
        }
    }
    fn maybe_bonus(&mut self, i: usize, v: Vec2) {
        if self.bonuses.is_empty() && self.random(10) < 2 {
            let mut kind = self.random(19) as usize;
            if kind < 2 && self.random(5) != 1 {
                kind = 13;
            }
            if kind == 14 {
                kind = 10;
            }
            self.bonuses.push(Bonus {
                p: Self::cell_pos(i) - vec2(1., 0.),
                v: vec2(v.x, (v.y / 2.).trunc()),
                kind,
                age: 0,
            });
            self.sounds.push(2);
        }
    }
    pub fn lose_life(&mut self) {
        self.lives -= 1;
        self.balls.clear();
        self.sounds.push(14);
        self.phase = if self.lives < 1 {
            Phase::GameOver
        } else {
            Phase::Lost
        };
        self.timer = 72;
        self.notice(if self.lives < 1 {
            "GAME OVER"
        } else {
            "PADDLE LOST"
        });
    }
    pub fn advance(&mut self) {
        self.phase = Phase::Transition;
        self.timer = 72;
        self.sounds.push(13);
        self.notice("BOARD COMPLETE");
    }
    fn release(&mut self) {
        for b in &mut self.balls {
            if b.held {
                b.held = false;
                b.rebound(self.paddle, self.width);
            }
        }
    }
    pub fn zap(&mut self) {
        for b in &mut self.bricks {
            *b = match *b {
                2 | 21 => 20,
                7 => 6,
                3 | 4 => 5,
                x => x,
            };
        }
        self.falling = false;
    }
    fn drop_bricks(&mut self) {
        for x in 0..20 {
            for y in (0..19).rev() {
                let i = y * 20 + x;
                if self.bricks[i] != 0 && self.bricks[i + 20] == 0 {
                    self.bricks[i + 20] = self.bricks[i];
                    self.bricks[i] = 0;
                }
            }
        }
        self.sounds.push(20);
    }
    pub fn apply_bonus(&mut self, k: usize) {
        self.notice(BONUS_NAMES[k]);
        self.sounds.push(8);
        match k {
            0 => {
                self.lives = (self.lives + 1).min(20);
                self.shooting = false;
                self.grab = false;
                self.thru = false;
                self.sounds.push(13);
            }
            1 => self.advance(),
            2 => self.zap(),
            3 => {
                for b in &mut self.balls {
                    let signs = vec2(b.v.x.signum(), b.v.y.signum());
                    b.speed = 4;
                    b.velocity();
                    b.v = vec2(b.v.x.abs() * signs.x, b.v.y.abs() * signs.y);
                    if b.sprite != 61 {
                        b.sprite = 1;
                    }
                }
            }
            4 => {
                let old = self.bricks;
                for y in 0..20 {
                    for x in 0..20 {
                        if old[y * 20 + x] == 8 {
                            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                                let nx = x as i32 + dx;
                                let ny = y as i32 + dy;
                                if (0..20).contains(&nx) && (0..20).contains(&ny) {
                                    self.bricks[ny as usize * 20 + nx as usize] = 8;
                                }
                            }
                        }
                    }
                }
            }
            5 => self.thru = true,
            6 => {
                for i in 0..400 {
                    if self.bricks[i] == 8 {
                        self.chains.push(Chain { cell: i, delay: 4 });
                    }
                }
            }
            7 => {
                for b in &mut self.balls {
                    b.sprite = 61;
                }
            }
            8 => self.shooting = true,
            9 => self.grab = true,
            10 | 14 => {
                self.release();
                self.width = if self.width < 73. {
                    73.
                } else {
                    (self.width + 73.).min(292.)
                };
            }
            11 => {
                self.release();
                self.width = if self.width > 73. {
                    self.width - 73.
                } else {
                    36.
                };
            }
            12 => {
                let mirrored: Vec<_> = self
                    .balls
                    .iter()
                    .cloned()
                    .map(|mut b| {
                        b.v.x = -b.v.x;
                        b.angle = 180 - b.angle;
                        if b.held {
                            b.speed = (b.speed - 1).max(4);
                        }
                        b
                    })
                    .collect();
                self.balls.extend(mirrored);
            }
            13 => self.lose_life(),
            15 => {
                for b in &mut self.balls {
                    let signs = vec2(b.v.x.signum(), b.v.y.signum());
                    b.speed = (b.speed + 2).min(9);
                    b.velocity();
                    b.v = vec2(b.v.x.abs() * signs.x, b.v.y.abs() * signs.y);
                }
            }
            16 => {
                self.release();
                self.width = 36.;
            }
            17 => self.falling = true,
            18 => {
                for b in &mut self.balls {
                    b.sprite = 55;
                }
            }
            _ => {}
        }
    }
}
