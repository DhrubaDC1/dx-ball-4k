//! Procedural 4K materials. Lighting/reflections are artistic screen-space
//! approximations; gameplay coordinates and collision geometry stay in game.rs.
use crate::game::{Game, Phase};
use macroquad::{miniquad::*, prelude::*};

pub const ICE: Color = Color::new(0.30, 0.91, 0.95, 1.);
pub const INK: Color = Color::new(0.055, 0.08, 0.15, 1.);

pub struct Visuals {
    material: Material,
}

impl Visuals {
    pub fn new() -> Self {
        let material = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: FRAGMENT,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("Lamp", UniformType::Float4),
                    UniformDesc::new("Clock", UniformType::Float1),
                ],
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .expect("compile procedural glass material");
        Self { material }
    }

    pub fn begin(&self, lamp: Vec2) {
        self.begin_at(lamp, get_time() as f32);
    }

    fn begin_at(&self, lamp: Vec2, time: f32) {
        self.material
            .set_uniform("Lamp", (lamp.x, lamp.y, 0.0f32, 0.0f32));
        self.material.set_uniform("Clock", time);
        gl_use_material(&self.material);
    }

    pub fn end(&self) {
        gl_use_default_material();
    }

    // Shape parameters travel with each vertex, keeping hundreds of bricks in
    // one GPU batch instead of changing uniforms for every object.
    fn shape(&self, rect: Rect, radius: f32, mode: f32, color: Color) {
        let mut vertices = vec![
            Vertex::new(rect.x, rect.y, 0., 0., 0., color),
            Vertex::new(rect.x + rect.w, rect.y, 0., 1., 0., color),
            Vertex::new(rect.x + rect.w, rect.y + rect.h, 0., 1., 1., color),
            Vertex::new(rect.x, rect.y + rect.h, 0., 0., 1., color),
        ];
        for vertex in &mut vertices {
            vertex.normal = vec4(rect.w, rect.h, radius, mode);
        }
        draw_mesh(&Mesh {
            vertices,
            indices: vec![0, 1, 2, 0, 2, 3],
            texture: None,
        });
    }

    pub fn glass(&self, rect: Rect, radius: f32, color: Color) {
        self.shape(rect, radius, 0., color);
    }

    // Interface surfaces have a quiet satin finish; reserve bright reflections
    // for the objects the player needs to follow.
    pub fn panel(&self, rect: Rect, radius: f32, color: Color) {
        self.shape(rect, radius, 6., color);
    }

    pub fn shadow(&self, rect: Rect, radius: f32) {
        self.shape(
            Rect::new(
                rect.x - 8. + 2.,
                rect.y - 8. + 5.,
                rect.w + 16.,
                rect.h + 16.,
            ),
            radius,
            3.,
            Color::new(0., 0., 0., 0.35),
        );
    }

    pub fn glow(&self, center: Vec2, radius: f32, color: Color) {
        self.shape(
            Rect::new(
                center.x - radius,
                center.y - radius,
                radius * 2.,
                radius * 2.,
            ),
            0.,
            4.,
            color,
        );
    }

    pub fn floor(&self, width: f32, height: f32) {
        self.shape(Rect::new(0., 0., width, height), 0., 5., WHITE);
    }

    pub fn brick(&self, kind: u8, rect: Rect) {
        if kind == 0 || kind == 7 {
            return;
        }
        let tint = match kind {
            2 | 21 => Color::new(0.62, 0.71, 0.82, 1.),
            3 => Color::new(0.72, 0.57, 1., 1.),
            4 => Color::new(0.54, 0.67, 1., 1.),
            8 => Color::new(1., 0.43, 0.20, 1.),
            9 | 14 | 18 => Color::new(0.22, 0.57, 1., 1.),
            10 | 15 | 19 => Color::new(0.60, 0.34, 1., 1.),
            11 | 16 | 20 => Color::new(1., 0.28, 0.53, 1.),
            12 | 17 | 22 => Color::new(1., 0.73, 0.20, 1.),
            _ => ICE,
        };
        let mode = match kind {
            2 | 21 => 1.,
            8 => 8.,
            9..=22 => 7.,
            _ => 0.,
        };
        self.shape(
            Rect::new(rect.x + 0.9, rect.y + 0.8, rect.w - 1.8, rect.h - 1.6),
            1.8,
            mode,
            tint,
        );
    }

    pub fn paddle(&self, x: f32, y: f32, width: f32, grab: bool, shooting: bool) {
        let r = Rect::new(x - width / 2., y, width, 14.);
        self.glow(
            vec2(x, y + 10.),
            width * 0.7,
            Color::new(0.1, 0.8, 0.9, 0.12),
        );
        self.shadow(r, 6.);
        self.shape(r, 5., 1., Color::new(0.70, 0.80, 0.92, 1.));
        let tint = if grab {
            Color::new(0.8, 0.55, 1., 1.)
        } else {
            ICE
        };
        self.glass(Rect::new(r.x + 10., y + 3., width - 20., 7.), 2., tint);
        for side in [-1., 1.] {
            let sx = x + side * (width / 2. - 7.);
            self.glass(
                Rect::new(sx - 2., y + 4., 4., 6.),
                1.,
                Color::new(1., 0.55, 0.28, 1.),
            );
            if shooting {
                self.shape(Rect::new(sx - 2., y - 12., 4., 18.), 1.5, 1., WHITE);
                self.glow(vec2(sx, y - 11.), 7., Color::new(1., 0.45, 0.3, 0.7));
            }
        }
    }

    pub fn ball(&self, pos: Vec2, size: f32, fire: bool) {
        let tint = if fire {
            Color::new(1., 0.43, 0.17, 1.)
        } else {
            ICE
        };
        self.glow(
            pos + Vec2::splat(size / 2.),
            size * 2.8,
            Color::new(tint.r, tint.g, tint.b, 0.4),
        );
        self.shadow(Rect::new(pos.x, pos.y + 2., size, size), size / 2.);
        self.shape(Rect::new(pos.x, pos.y, size, size), size / 2., 2., tint);
    }

    pub fn game(&self, g: &Game) {
        let lamp = g
            .balls
            .first()
            .map(|b| b.p + Vec2::splat(b.size() / 2.))
            .unwrap_or(vec2(320., 420.));
        // Simulation time makes captures deterministic at any rendering speed.
        self.begin_at(lamp, g.ticks as f32 / 60.);
        self.floor(640., 480.);
        // Recessed rails sit just beyond the original playable area.
        for x in [8., 624.] {
            self.panel(Rect::new(x, 38., 8., 423.), 4., INK);
            self.glass(
                Rect::new(x + 3., 48., 1.2, 390.),
                0.5,
                Color::new(0.20, 0.55, 0.72, 0.45),
            );
        }
        for (i, &kind) in g.bricks.iter().enumerate() {
            if kind != 0 && kind != 7 {
                let p = Game::cell_pos(i);
                self.shadow(Rect::new(p.x + 1., p.y + 1., 28., 13.), 2.5);
            }
        }
        for (i, &kind) in g.bricks.iter().enumerate() {
            let p = Game::cell_pos(i);
            self.brick(kind, Rect::new(p.x, p.y, 30., 15.));
        }
        for e in &g.bursts {
            let life = 1. - e.age as f32 / if e.sprite == 145 { 44. } else { 24. };
            if life <= 0. {
                continue;
            }
            let center = e.p + vec2(15., 7.5);
            self.glow(
                center,
                10. + (1. - life) * 35.,
                Color::new(0.5, 0.9, 1., life * 0.35),
            );
            for n in 0..8 {
                let angle = n as f32 * std::f32::consts::TAU / 8. + e.p.x;
                let p = center + vec2(angle.cos(), angle.sin()) * e.age as f32 * 1.3;
                self.glass(
                    Rect::new(p.x, p.y, 2.5 * life + 0.5, 1.5 * life + 0.5),
                    0.5,
                    Color::new(0.65, 0.94, 1., life),
                );
            }
        }
        for b in &g.bonuses {
            let rect = Rect::new(b.p.x, b.p.y, 32., 30.);
            self.shadow(rect, 4.);
            self.bonus(b.kind, rect);
        }
        for b in &g.balls {
            self.ball(b.p, b.size(), b.sprite == 61);
        }
        for p in &g.bullets {
            self.glow(*p, 9., Color::new(0.3, 0.9, 1., 0.45));
            self.glass(Rect::new(p.x, p.y, 2., 9.), 1., WHITE);
        }
        if !matches!(g.phase, Phase::Lost | Phase::GameOver) {
            self.paddle(g.paddle, 450., g.width, g.grab, g.shooting);
        }
        self.end();
        for b in &g.bonuses {
            Self::bonus_icon(b.kind, Rect::new(b.p.x, b.p.y, 32., 30.));
        }
    }

    pub fn bonus(&self, kind: usize, rect: Rect) {
        let tint = if matches!(kind, 11 | 13 | 15 | 16 | 17 | 18) {
            Color::new(1., 0.36, 0.39, 1.)
        } else if matches!(kind, 4 | 12) {
            Color::new(0.70, 0.75, 0.82, 1.)
        } else {
            Color::new(0.30, 0.67, 1., 1.)
        };
        self.glass(rect, 4., tint);
    }

    // Shared vector pictograms stay crisp at 4K and match the bonus guide.
    // Coordinates are in a 24x24 square, centered inside either sized badge.
    pub fn bonus_icon(kind: usize, rect: Rect) {
        let scale = rect.w.min(rect.h) / 28.;
        let origin = vec2(rect.x + rect.w / 2., rect.y + rect.h / 2.) - vec2(12., 12.) * scale;
        let point = |x, y| origin + vec2(x, y) * scale;
        let line = |x1, y1, x2, y2| {
            let a = point(x1, y1);
            let b = point(x2, y2);
            draw_line(a.x, a.y, b.x, b.y, 1.9 * scale, WHITE);
        };
        let circle = |x, y, r| {
            let p = point(x, y);
            draw_circle(p.x, p.y, r * scale, WHITE);
        };
        let arrow = |x: f32, y: f32, direction: f32| {
            line(x - direction * 4., y, x, y);
            line(x - direction * 3., y - 3., x, y);
            line(x - direction * 3., y + 3., x, y);
        };
        let paddle = |left, right, y| line(left, y, right, y);
        match kind {
            0 => {
                // Life: heart.
                circle(8., 9., 4.);
                circle(16., 9., 4.);
                draw_triangle(point(4., 10.), point(20., 10.), point(12., 21.), WHITE);
            }
            1 => {
                // Warp portal and forward arrow.
                let p = point(8., 12.);
                draw_ellipse_lines(p.x, p.y, 4. * scale, 9. * scale, 0., 1.7 * scale, WHITE);
                line(7., 12., 21., 12.);
                arrow(21., 12., 1.);
            }
            2 => {
                // Lightning zap.
                draw_triangle(point(14., 2.), point(5., 14.), point(14., 12.), WHITE);
                draw_triangle(point(10., 12.), point(19., 10.), point(10., 22.), WHITE);
            }
            3 | 15 => {
                // Slow/fast ball, one or three speed chevrons.
                circle(17., 12., 3.);
                for n in 0..if kind == 3 { 1 } else { 3 } {
                    arrow(12. - n as f32 * 4., 12., 1.);
                }
            }
            4 | 6 => {
                // Explosives: spread or detonate.
                circle(11., 14., 5.);
                line(12., 9., 15., 5.);
                line(15., 5., 19., 6.);
                if kind == 4 {
                    line(17., 15., 23., 15.);
                    line(20., 12., 20., 18.);
                } else {
                    for (x, y) in [(3., 4.), (20., 3.), (22., 19.)] {
                        line(x, y, x + 2., y + 2.);
                    }
                }
            }
            5 => {
                // Ball passes through a brick.
                let a = point(9., 4.);
                draw_rectangle_lines(a.x, a.y, 7. * scale, 16. * scale, 1.5 * scale, WHITE);
                line(2., 12., 22., 12.);
                arrow(22., 12., 1.);
            }
            7 => {
                // Fireball with flame tongues.
                circle(12., 16., 5.);
                draw_triangle(point(7., 16.), point(9., 3.), point(14., 16.), WHITE);
                draw_triangle(point(11., 17.), point(18., 6.), point(17., 17.), WHITE);
            }
            8 => {
                // Shooting paddle and twin bolts.
                paddle(4., 20., 20.);
                for x in [6., 18.] {
                    line(x, 17., x, 10.);
                    line(x, 6., x, 2.);
                }
            }
            9 => {
                // Magnet/grab paddle.
                paddle(4., 20., 21.);
                circle(12., 5., 2.5);
                line(6., 8., 6., 15.);
                line(6., 15., 18., 15.);
                line(18., 15., 18., 8.);
                line(6., 8., 9., 8.);
                line(15., 8., 18., 8.);
            }
            10 | 14 | 11 | 16 => {
                // Paddle width arrows.
                let expand = matches!(kind, 10 | 14);
                paddle(
                    if kind == 16 { 10. } else { 7. },
                    if kind == 16 { 14. } else { 17. },
                    18.,
                );
                if expand {
                    arrow(3., 10., -1.);
                    arrow(21., 10., 1.);
                } else {
                    arrow(9., 10., 1.);
                    arrow(15., 10., -1.);
                }
                if kind == 16 {
                    line(2., 4., 7., 4.);
                    line(17., 4., 22., 4.);
                }
            }
            12 => {
                // One ball splits into two.
                circle(12., 19., 3.);
                line(12., 15., 6., 8.);
                line(12., 15., 18., 8.);
                circle(5., 5., 3.);
                circle(19., 5., 3.);
            }
            13 => {
                // Skull and crossbones.
                circle(12., 8., 6.);
                let eye = |x| {
                    let p = point(x, 7.);
                    draw_circle(p.x, p.y, 1.5 * scale, INK);
                };
                eye(9.5);
                eye(14.5);
                line(9., 13., 15., 13.);
                line(5., 17., 19., 22.);
                line(5., 22., 19., 17.);
            }
            17 => {
                // Falling brick and down arrow.
                let p = point(4., 3.);
                draw_rectangle_lines(p.x, p.y, 16. * scale, 6. * scale, 1.5 * scale, WHITE);
                line(12., 12., 12., 22.);
                line(7., 17., 12., 22.);
                line(17., 17., 12., 22.);
            }
            18 => {
                // Small ball between inward arrows.
                circle(12., 12., 2.);
                arrow(7., 12., 1.);
                arrow(17., 12., -1.);
            }
            _ => {}
        }
    }
}

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
attribute vec4 normal;
uniform mat4 Model;
uniform mat4 Projection;
varying highp vec2 uv;
varying highp vec2 world;
varying highp vec4 shape;
varying lowp vec4 tint;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
    world = position.xy;
    shape = normal;
    tint = color0 / 255.0;
}"#;

const FRAGMENT: &str = r#"#version 100
precision highp float;
varying highp vec2 uv;
varying highp vec2 world;
varying highp vec4 shape;
varying lowp vec4 tint;
uniform vec4 Lamp;
uniform float Clock;

float box(vec2 p, vec2 halfSize, float radius) {
    vec2 q = abs(p) - halfSize + radius;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - radius;
}
void main() {
    vec2 size = shape.xy;
    vec2 p = (uv - 0.5) * size;
    bool animated = shape.w > 6.5;
    bool explosive = shape.w > 7.5;
    float mode = animated ? 0.0 : shape.w;
    if (mode > 5.5) {
        float d = box(p, size * 0.5 - 0.25, shape.z);
        float alpha = 1.0 - smoothstep(-0.2, 0.2, d);
        float edge = exp(-abs(d + 0.5) * 2.5);
        vec3 color = tint.rgb * (1.15 - uv.y * 0.18);
        color += vec3(0.16, 0.24, 0.38) * edge * 0.45;
        gl_FragColor = vec4(color, alpha * tint.a);
        return;
    }
    if (mode > 4.5) {
        vec2 q = uv;
        float teal = exp(-length((q - vec2(0.13, 0.10)) * vec2(2.2, 2.0)) * 2.0);
        float violet = exp(-length((q - vec2(0.93, 0.70)) * vec2(2.6, 2.0)) * 2.0);
        vec3 color = vec3(0.030, 0.044, 0.090) + vec3(0.025, 0.10, 0.18) * teal + vec3(0.12, 0.045, 0.20) * violet;
        vec2 grid = abs(fract(world / 30.0 - 0.5) - 0.5) * 30.0;
        float line = 1.0 - smoothstep(0.0, 0.38, min(grid.x, grid.y));
        color += vec3(0.035, 0.075, 0.12) * line * 0.18;
        float pool = exp(-length(world - Lamp.xy) / 65.0);
        color += vec3(0.018, 0.12, 0.14) * pool;
        float vignette = clamp(1.0 - length((q - 0.5) * 0.75), 0.45, 1.0);
        gl_FragColor = vec4(color * vignette, 1.0);
        return;
    }
    if (mode > 3.5) {
        float falloff = max(exp(-dot((uv - 0.5) * 5.0, (uv - 0.5) * 5.0)) - 0.002, 0.0);
        gl_FragColor = vec4(tint.rgb, tint.a * falloff);
        return;
    }
    if (mode > 2.5) {
        float d = box(p, size * 0.5 - 8.0, shape.z);
        gl_FragColor = vec4(0.0, 0.005, 0.015, exp(-max(d, 0.0) / 3.0) * tint.a);
        return;
    }
    float d = box(p, size * 0.5 - 0.25, shape.z);
    float alpha = 1.0 - smoothstep(-0.2, 0.2, d);
    if (alpha <= 0.0) discard;
    vec2 grad = vec2(
        box(p + vec2(0.2, 0.0), size * 0.5 - 0.25, shape.z) - d,
        box(p + vec2(0.0, 0.2), size * 0.5 - 0.25, shape.z) - d) / 0.2;
    float bevel = exp(min(d, 0.0) / 0.65);
    vec3 n = normalize(vec3(grad * bevel * 1.5, 1.0));
    if (mode > 1.5) n = normalize(vec3((uv - 0.5) * 1.8, sqrt(max(0.02, 1.0 - dot((uv - 0.5) * 2.0, (uv - 0.5) * 2.0)))));
    vec3 light = normalize(vec3(-0.5, -0.8, 1.3));
    float diffuse = max(dot(n, light), 0.0);
    float spec = pow(max(dot(n, normalize(light + vec3(0.0, 0.0, 1.0))), 0.0), 42.0);
    float rim = pow(1.0 - n.z, 2.0);
    float strip = exp(-pow((uv.y + uv.x * 0.26 - 0.22) * 17.0, 2.0));
    float fineRim = exp(-abs(d + 0.65) * 2.2);
    float lowerRim = exp(-abs(d + 2.0) * 2.5) * smoothstep(0.35, 1.0, uv.y);
    float pool = exp(-length(world - Lamp.xy) / 70.0);
    vec3 color = tint.rgb * (0.68 + diffuse * 0.20 - uv.y * 0.17);
    color += tint.rgb * lowerRim * 0.10;
    color += vec3(0.65, 0.9, 1.0) * (spec * 0.06 + rim * 0.12 + fineRim * 0.14 + strip * 0.08);
    color += vec3(0.15, 0.6, 0.65) * pool * (0.08 + rim * 0.3);
    if (animated) {
        float phase = Clock * 2.8 + world.x * 0.012;
        float wave = 0.5 + 0.5 * sin(uv.x * 9.0 - uv.y * 4.0 - phase);
        float sweep = pow(max(0.0, sin(uv.x * 4.0 + uv.y * 2.0 - phase)), 8.0);
        color *= 0.76 + wave * 0.34;
        color += mix(tint.rgb, vec3(0.7, 0.95, 1.0), 0.45) * sweep * 0.38;
        if (explosive) {
            float diamond = abs(uv.x - 0.5) * 2.0 + abs(uv.y - 0.5);
            float ring = exp(-abs(diamond - (0.25 + wave * 0.55)) * 16.0);
            color += vec3(1.0, 0.65, 0.16) * ring * 0.45;
        }
    }
    if (mode > 0.5 && mode < 1.5) {
        float reflection = exp(-pow((uv.y - 0.24) * 5.0, 2.0));
        color = tint.rgb * (0.40 + reflection * 0.32 + diffuse * 0.20) + fineRim * 0.18 + spec * 0.10;
    }
    if (mode > 1.5) color = mix(tint.rgb * 0.5, vec3(0.94, 1.0, 1.0), diffuse) + spec * 0.5;
    gl_FragColor = vec4(color, alpha * tint.a);
}"#;
