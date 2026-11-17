//! The play probe drives the same start/step functions and input object used by the UI.
use crate::game::{Game, Input};
use serde_json::{json, Value};
pub struct Probe {
    frame: u32,
    min_x: f32,
    max_x: f32,
    ball_travel: f32,
    last: Option<macroquad::prelude::Vec2>,
    start_score: u32,
    started: bool,
}
impl Probe {
    pub fn new() -> Self {
        Self {
            frame: 0,
            min_x: 320.,
            max_x: 320.,
            ball_travel: 0.,
            last: None,
            start_score: 0,
            started: false,
        }
    }
    pub fn drive(&mut self, g: &mut Game) -> Input {
        if !self.started {
            g.start();
            self.start_score = g.score;
            self.started = true;
        }
        self.frame += 1;
        self.min_x = self.min_x.min(g.paddle);
        self.max_x = self.max_x.max(g.paddle);
        if let Some(b) = g.balls.first() {
            if let Some(p) = self.last {
                self.ball_travel += b.p.distance(p);
            }
            self.last = Some(b.p);
        }
        // Move right, move left, return under the served ball, then launch and track it.
        let x = if self.frame < 20 {
            320. + self.frame as f32 * 4.
        } else if self.frame < 45 {
            400. - (self.frame - 20) as f32 * 5.
        } else {
            g.balls.first().map(|b| b.p.x + 4.).unwrap_or(320.)
        };
        Input {
            x,
            click: self.frame == 48 || g.balls.iter().any(|b| b.held) && self.frame > 50,
        }
    }
    pub fn done(&self) -> bool {
        self.frame >= 600
    }
    pub fn report(&self, g: &Game, hud: bool) -> Value {
        json!({"version":1,"steps":[
  {"step":"move","passed":self.max_x-self.min_x>80.,"detail":format!("Scripted mouse moved paddle over {:0.0} pixels; ball traveled {:0.0} pixels",self.max_x-self.min_x,self.ball_travel)},
  {"step":"act","passed":g.launches>0&&g.hit_count>0&&g.score>self.start_score,"detail":format!("Left-click launched {} balls; {} brick hits, {} paddle rebounds, score {}",g.launches,g.hit_count,g.rebounds,g.score)},
  {"step":"hud","passed":hud,"detail":format!("Rendered glass HUD score {}, {} reserve life indicators and board {} during actual play",g.score,(g.lives-1).max(0),g.board+1)}],"simulation_frames":self.frame,"original_runtime_compared":false})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Phase, BOARDS};
    use macroquad::prelude::*;
    #[test]
    fn original_boards_are_complete() {
        assert_eq!(BOARDS.len(), 50 * 400);
        for b in BOARDS.chunks(400) {
            assert!(b.iter().any(|&x| x != 0 && x != 2));
            assert!(b.iter().all(|&x| x <= 22));
        }
    }
    #[test]
    fn serve_mouse_and_launch() {
        let mut g = Game::new();
        g.start();
        g.step(Input {
            x: 420.,
            click: false,
        });
        assert_eq!(g.paddle, 420.);
        assert!(g.balls[0].held);
        assert_eq!(g.balls[0].p.x, 420.);
        g.step(Input {
            x: 420.,
            click: true,
        });
        assert!(!g.balls[0].held);
        assert!(g.balls[0].v.y < 0.);
        assert_eq!(g.balls[0].speed, 5);
    }
    #[test]
    fn rebound_zones_and_truncated_velocity() {
        let mut g = Game::new();
        g.start();
        let mut b = g.balls[0].clone();
        b.p.x = 320. - 36.5 - 4.5;
        b.rebound(320., 73.);
        assert_eq!(b.angle, 150);
        assert_eq!(b.v, vec2(-5., -2.));
        b.p.x = 320. - 4.5;
        b.rebound(320., 73.);
        assert_eq!(b.angle, 105);
        assert_eq!(b.v, vec2(-1., -4.));
    }
    #[test]
    fn all_balls_must_be_lost() {
        let mut g = Game::new();
        g.start();
        let mut b = g.balls[0].clone();
        b.held = false;
        b.p = vec2(100., 470.);
        b.v = vec2(0., 5.);
        g.balls.push(b);
        g.step(Input {
            x: 320.,
            click: false,
        });
        assert_eq!(g.lives, 3);
        assert_eq!(g.balls.len(), 1);
        g.balls[0].held = false;
        g.balls[0].p.y = 480.;
        g.step(Input {
            x: 320.,
            click: false,
        });
        assert_eq!(g.lives, 2);
        assert_eq!(g.phase, Phase::Lost);
        for _ in 0..72 {
            g.step(Input::default());
        }
        assert_eq!(g.phase, Phase::Playing);
        assert!(g.balls[0].held);
    }
    #[test]
    fn bonus_rules_and_limits() {
        let mut g = Game::new();
        g.start();
        g.apply_bonus(10);
        assert_eq!(g.width, 146.);
        for _ in 0..5 {
            g.apply_bonus(10);
        }
        assert_eq!(g.width, 292.);
        g.apply_bonus(16);
        assert_eq!(g.width, 36.);
        g.apply_bonus(12);
        assert_eq!(g.balls.len(), 2);
        g.apply_bonus(15);
        assert_eq!(g.balls[0].speed, 7);
        g.apply_bonus(15);
        g.apply_bonus(15);
        assert_eq!(g.balls[0].speed, 9);
        g.apply_bonus(3);
        assert_eq!(g.balls[0].speed, 4);
        g.apply_bonus(18);
        assert_eq!(g.balls[0].size(), 5.);
        g.apply_bonus(7);
        assert_eq!(g.balls[0].sprite, 61);
    }
    #[test]
    fn bullet_limit_and_scoring() {
        let mut g = Game::new();
        g.start();
        g.apply_bonus(8);
        for _ in 0..10 {
            g.step(Input {
                x: 320.,
                click: true,
            });
        }
        assert_eq!(g.bullets.len(), 6);
        assert_eq!(g.shots, 6);
    }
    #[test]
    fn script_reaches_core_loop() {
        let mut g = Game::new();
        let mut p = Probe::new();
        while !p.done() {
            let input = p.drive(&mut g);
            g.step(input);
        }
        let report = p.report(&g, true);
        assert!(
            report["steps"]
                .as_array()
                .unwrap()
                .iter()
                .all(|x| x["passed"] == true),
            "{report}"
        );
    }
}
