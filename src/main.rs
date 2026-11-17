#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod assets;
mod game;
mod verify;
mod visuals;
use assets::Assets;
use game::{Game, Input, Phase};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use visuals::{Visuals, ICE, INK};

const VIEW_WIDTH: f32 = 960.;
const VIEW_HEIGHT: f32 = 540.;
// Keep gameplay in its original coordinates, but rasterize the entire layout at 4K.
const RENDER_SCALE: f32 = 4.;
const OUTPUT_WIDTH: u32 = (VIEW_WIDTH * RENDER_SCALE) as u32;
const OUTPUT_HEIGHT: u32 = (VIEW_HEIGHT * RENDER_SCALE) as u32;
const ARENA: Rect = Rect::new(24., 30., 640., 480.);

// Share the fitted viewport between drawing and pointer input at every window size.
fn viewport(width: f32, height: f32) -> (f32, Vec2) {
    let scale = (width / VIEW_WIDTH).min(height / VIEW_HEIGHT).max(0.001);
    (
        scale,
        vec2(
            (width - VIEW_WIDTH * scale) / 2.,
            (height - VIEW_HEIGHT * scale) / 2.,
        ),
    )
}

fn automated() -> bool {
    std::env::args().any(|a| a == "--screenshot" || a == "--self-test")
}
fn conf() -> Conf {
    let mut c = Conf {
        window_title: "Brick Breaker".into(),
        window_width: OUTPUT_WIDTH as i32,
        window_height: OUTPUT_HEIGHT as i32,
        window_resizable: true,
        high_dpi: true,
        sample_count: 1,
        ..Default::default()
    };
    c.platform.blocking_event_loop = false;
    if automated() {
        c.platform.swap_interval = Some(0);
        c.high_dpi = false;
        #[cfg(target_os = "linux")]
        {
            std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
            std::env::set_var("GALLIUM_DRIVER", "llvmpipe");
        }
    }
    #[cfg(target_os = "linux")]
    {
        c.platform.linux_backend = macroquad::miniquad::conf::LinuxBackend::X11Only;
    }
    c
}
fn arg_value(flag: &str) -> Option<PathBuf> {
    let args: Vec<_> = std::env::args_os().collect();
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| PathBuf::from(&w[1]))
}
fn save_file(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(p) = path.parent() {
        if !p.as_os_str().is_empty() {
            std::fs::create_dir_all(p)?;
        }
    }
    std::fs::write(path, bytes)
}
fn data_path() -> PathBuf {
    if let Some(p) = std::env::var_os("BRICK_BREAKER_DATA") {
        return PathBuf::from(p);
    }
    #[cfg(target_os = "windows")]
    let root = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let root =
        std::env::var_os("HOME").map(|p| PathBuf::from(p).join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let root = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")));
    root.unwrap_or_else(|| PathBuf::from("."))
        .join("brick-breaker")
}
#[derive(Default, Serialize, Deserialize)]
struct Saved {
    scores: Vec<u32>,
    sound: bool,
    music: bool,
}
struct Ui {
    selected: usize,
    editor_kind: u8,
    editor_board: usize,
    editor: [u8; 400],
    saved: Saved,
}
impl Ui {
    fn new(quiet: bool) -> Self {
        let saved = if quiet {
            Saved::default()
        } else {
            std::fs::read(data_path().join("settings.json"))
                .ok()
                .and_then(|b| serde_json::from_slice(&b).ok())
                .unwrap_or(Saved {
                    scores: vec![],
                    sound: true,
                    music: false,
                })
        };
        let mut editor = [0; 400];
        editor.copy_from_slice(&game::BOARDS[..400]);
        Self {
            selected: 0,
            editor_kind: 1,
            editor_board: 0,
            editor,
            saved,
        }
    }
}
thread_local! {
    // Fonts belong to the render thread and are loaded once after GL starts.
    static UI_FONTS: std::cell::RefCell<Option<(Font, Font)>> = const { std::cell::RefCell::new(None) };
}
// Drop GPU font atlases with the main future, before miniquad unloads GL.
// Leaving them in thread-local storage until process exit calls glDeleteTextures
// after the GL library has gone away.
struct UiFontGuard;
impl Drop for UiFontGuard {
    fn drop(&mut self) {
        UI_FONTS.with(|fonts| {
            fonts.borrow_mut().take();
        });
    }
}
fn with_font<T>(size: f32, draw: impl FnOnce(&Font) -> T) -> T {
    UI_FONTS.with(|fonts| {
        let fonts = fonts.borrow();
        let (regular, bold) = fonts.as_ref().expect("interface fonts initialized");
        draw(if size >= 24. { bold } else { regular })
    })
}
fn prepare_fonts() {
    // Growing an atlas while queued labels still refer to its old texture can
    // leave black glyph rectangles. Allocate the UI sizes before drawing a scene.
    let characters: Vec<char> = (' '..='~').collect();
    UI_FONTS.with(|fonts| {
        let fonts = fonts.borrow();
        let (regular, bold) = fonts.as_ref().expect("interface fonts initialized");
        for (font, sizes) in [
            (
                regular,
                &[9., 10., 11., 12., 13., 14., 15., 16., 17., 18.][..],
            ),
            (bold, &[25., 27., 28., 29., 32., 35., 42.][..]),
        ] {
            for &size in sizes {
                font.populate_font_cache(
                    &characters,
                    (size * RENDER_SCALE * macroquad::miniquad::window::dpi_scale()).ceil() as u16,
                );
            }
            draw_text_ex(
                "A",
                -1000.,
                -1000.,
                TextParams {
                    font: Some(font),
                    font_size: (sizes[0] * RENDER_SCALE) as u16,
                    ..Default::default()
                },
            );
        }
    });
    unsafe {
        macroquad::window::get_internal_gl().flush();
    }
}
fn text(s: &str, x: f32, y: f32, size: f32, color: Color) {
    with_font(size, |font| {
        draw_text_ex(
            s,
            x,
            y,
            TextParams {
                font: Some(font),
                font_size: (size * RENDER_SCALE) as u16,
                font_scale: 1. / RENDER_SCALE,
                color,
                ..Default::default()
            },
        );
    });
}
fn centered(s: &str, y: f32, size: f32, color: Color) {
    let w = with_font(size, |font| {
        measure_text(
            s,
            Some(font),
            (size * RENDER_SCALE) as u16,
            1. / RENDER_SCALE,
        )
        .width
    });
    text(s, (640. - w) / 2., y, size, color);
}
const CYAN: Color = Color::new(0.26, 0.92, 0.94, 1.);
fn button(v: &Visuals, label: &str, r: Rect, selected: bool) {
    v.begin(vec2(320., 120.));
    v.shadow(r, 7.);
    v.panel(
        r,
        7.,
        if selected {
            Color::new(0.06, 0.25, 0.33, 1.)
        } else {
            INK
        },
    );
    v.end();
    text(
        label,
        r.x + 17.,
        r.y + r.h * 0.66,
        15.,
        if selected {
            WHITE
        } else {
            Color::new(0.61, 0.71, 0.8, 1.)
        },
    );
    if selected {
        draw_line(r.x + 5., r.y + 11., r.x + 5., r.y + r.h - 11., 2., ICE);
        text(">", r.x + r.w - 25., r.y + r.h * 0.66, 16., ICE);
    }
}
fn backdrop(v: &Visuals) {
    v.begin(vec2(320., 100.));
    v.floor(640., 480.);
    v.end();
}
fn draw_widescreen(g: &Game, a: &Assets, arena: &Texture2D, v: &Visuals) {
    let muted = Color::new(0.49, 0.61, 0.72, 1.);
    let edge = Color::new(0.15, 0.25, 0.33, 1.);
    clear_background(Color::new(0.015, 0.022, 0.04, 1.));
    v.begin(vec2(350., 130.));
    v.floor(VIEW_WIDTH, VIEW_HEIGHT);
    v.glow(vec2(380., 0.), 420., Color::new(0.12, 0.46, 0.52, 0.15));
    v.shadow(Rect::new(14., 20., 660., 500.), 12.);
    v.panel(Rect::new(14., 20., 660., 500.), 12., INK);
    v.shadow(Rect::new(688., 20., 258., 500.), 14.);
    v.panel(
        Rect::new(688., 20., 258., 500.),
        14.,
        Color::new(0.045, 0.065, 0.12, 1.),
    );
    v.panel(
        Rect::new(701., 166., 232., 175.),
        9.,
        Color::new(0.065, 0.09, 0.16, 1.),
    );
    v.end();
    draw_texture_ex(
        arena,
        ARENA.x,
        ARENA.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(ARENA.w, ARENA.h)),
            flip_y: true,
            ..Default::default()
        },
    );
    draw_line(40., 21., 298., 21., 0.7, Color::new(0.4, 0.8, 0.86, 0.6));
    draw_circle(714., 46., 2.5, ICE);
    text("THE ARCADE COLLECTION", 726., 50., 9., ICE);
    text("BRICK", 710., 88., 32., WHITE);
    text("BREAKER", 710., 119., 32., WHITE);
    text("50 BOARDS. ONE MORE TRY.", 710., 142., 10., muted);
    let playing = matches!(
        g.phase,
        Phase::Playing
            | Phase::Paused
            | Phase::Lost
            | Phase::Transition
            | Phase::GameOver
            | Phase::Won
    );
    if playing {
        text("TOTAL SCORE", 716., 189., 10., muted);
        text(&format!("{:06}", g.score), 714., 227., 35., WHITE);
        draw_line(716., 242., 918., 242., 0.6, edge);
        text("BOARD", 716., 264., 10., muted);
        text(&format!("{:02}", g.board + 1), 716., 292., 25., ICE);
        text("/ 50", 755., 291., 13., muted);
        text("LIVES", 851., 264., 10., muted);
        for i in 0..g.lives.clamp(0, 5) {
            draw_rectangle(851. + i as f32 * 13., 281., 9., 4., ICE);
        }
        text(
            &format!("{} BRICKS REMAINING", g.remaining()),
            716.,
            317.,
            10.,
            muted,
        );
        draw_line(716., 329., 918., 329., 2., edge);
        draw_line(
            716.,
            329.,
            716. + 202. * (g.board + 1) as f32 / 50.,
            329.,
            2.,
            ICE,
        );
    } else {
        let (title, lines) = match g.phase {
            Phase::Help => (
                "HOW TO PLAY",
                [
                    "Keep the ball in play.",
                    "Blue bonuses help.",
                    "Watch out for red ones.",
                ],
            ),
            Phase::Scores => (
                "YOUR BEST RUNS",
                [
                    "Every brick counts.",
                    "Beat your high score.",
                    "Clear all 50 boards.",
                ],
            ),
            Phase::Editor => (
                "MAKE IT YOURS",
                [
                    "Paint your own board.",
                    "Save it for later.",
                    "Press Enter to play.",
                ],
            ),
            _ => (
                "READY TO PLAY?",
                [
                    "Move. Aim. Break through.",
                    "Collect bonuses as you go.",
                    "Clear a board to advance.",
                ],
            ),
        };
        text(title, 716., 197., 17., ICE);
        for (i, line) in lines.iter().enumerate() {
            text(line, 716., 234. + i as f32 * 26., 13., muted);
        }
    }
    text("QUICK CONTROLS", 710., 371., 10., muted);
    let controls = if g.phase == Phase::Editor {
        [
            ("CLICK / RIGHT", "Paint / erase"),
            ("S / L", "Save / load"),
            ("ENTER", "Play board"),
            ("F11", "Fullscreen"),
        ]
    } else {
        [
            ("MOUSE", "Move paddle"),
            ("CLICK / SPACE", "Launch / fire"),
            ("P", "Pause game"),
            ("F11", "Fullscreen"),
        ]
    };
    for (i, (key, action)) in controls.iter().enumerate() {
        let y = 396. + i as f32 * 24.;
        text(key, 710., y, 10., WHITE);
        text(action, 817., y, 11., muted);
    }
    draw_line(710., 485., 924., 485., 0.6, edge);
    text(
        if a.music_on { "MUSIC ON" } else { "MUSIC OFF" },
        710.,
        504.,
        9.,
        muted,
    );
    text("4K UHD", 883., 504., 9., ICE);
}
fn draw_game(g: &Game, v: &Visuals) -> bool {
    v.game(g);
    let muted = Color::new(0.45, 0.58, 0.68, 1.);
    text(&format!("{:06}", g.score), 25., 24., 17., WHITE);
    text(
        &format!("BOARD {:02} / 50", g.board + 1),
        270.,
        24.,
        11.,
        ICE,
    );
    for i in 0..(g.lives - 1).clamp(0, 19) {
        draw_rectangle(
            603. - (i % 10) as f32 * 13.,
            14. + (i / 10) as f32 * 6.,
            9.,
            3.,
            ICE,
        );
    }
    for (i, &b) in g.bricks.iter().enumerate() {
        let p = Game::cell_pos(i);
        if b == 2 || b == 21 {
            draw_line(p.x + 11., p.y + 6., p.x + 19., p.y + 6., 0.7, WHITE);
            draw_line(p.x + 11., p.y + 9., p.x + 19., p.y + 9., 0.7, WHITE);
        } else if b == 3 || b == 4 {
            for n in 0..if b == 3 { 3 } else { 2 } {
                draw_circle(p.x + 12. + n as f32 * 3., p.y + 8., 0.7, WHITE);
            }
        } else if b == 8 {
            // A small ember identifies explosive bricks without covering their color.
            draw_poly(
                p.x + 15.,
                p.y + 7.5,
                4,
                2.,
                0.,
                Color::new(1., 0.91, 0.62, 1.),
            );
        }
    }
    text("F1  GUIDE", 25., 474., 9., muted);
    text("P  PAUSE", 280., 474., 9., muted);
    text("ESC  MENU", 558., 474., 9., muted);
    if g.message_time > 0 {
        centered(&g.message, 404., 14., ICE);
    }
    if g.balls.iter().any(|b| b.held) && g.message_time == 0 {
        centered("CLICK TO RELEASE", 415., 12., muted);
    }
    if g.phase == Phase::Paused {
        v.begin(vec2(320., 220.));
        v.shadow(Rect::new(164., 190., 312., 112.), 12.);
        v.panel(
            Rect::new(164., 190., 312., 112.),
            12.,
            Color::new(0.035, 0.07, 0.11, 0.98),
        );
        v.end();
        centered("PAUSED", 237., 28., WHITE);
        centered("P OR RIGHT CLICK TO CONTINUE", 275., 12., ICE);
    }
    if matches!(g.phase, Phase::GameOver | Phase::Won) {
        v.begin(vec2(320., 220.));
        v.shadow(Rect::new(112., 181., 416., 130.), 12.);
        v.panel(
            Rect::new(112., 181., 416., 130.),
            12.,
            Color::new(0.035, 0.07, 0.11, 0.98),
        );
        v.end();
        centered(
            if g.phase == Phase::Won {
                "ALL 50 BOARDS CLEARED"
            } else {
                "GAME OVER"
            },
            225.,
            25.,
            WHITE,
        );
        centered(&format!("FINAL SCORE  {}", g.score), 258., 18., ICE);
        centered("CLICK TO RETURN TO MENU", 290., 12., muted);
    }
    true
}
fn draw_menu(v: &Visuals, a: &Assets, ui: &Ui) {
    backdrop(v);
    v.begin(vec2(320., 90.));
    v.glow(vec2(320., 80.), 250., Color::new(0.2, 0.7, 0.8, 0.18));
    for row in 0..3 {
        for x in 0..(12 - row * 2) {
            let r = Rect::new(
                137. + row as f32 * 31. + x as f32 * 31.,
                18. + row as f32 * 16.,
                30.,
                15.,
            );
            v.shadow(r, 1.8);
            v.brick([1, 9, 10, 11, 12, 1][(x + row) % 6], r);
        }
    }
    v.paddle(320., 366., 90., false, false);
    v.ball(vec2(315.5, 346.), 9., false);
    v.end();
    centered("BRICK BREAKER", 101., 42., WHITE);
    centered("FIND YOUR FLOW. CLEAR ALL 50.", 128., 11., ICE);
    let options = [
        "PLAY GAME",
        "BONUSES & CONTROLS",
        "HIGH SCORES",
        "BOARD WORKSHOP",
    ];
    for (i, label) in options.iter().enumerate() {
        button(
            v,
            label,
            Rect::new(191., 158. + i as f32 * 44., 258., 34.),
            ui.selected == i,
        );
    }
    centered("MOVE THE MOUSE  /  CLICK TO LAUNCH", 414., 13., WHITE);
    centered(
        "BLUE: GOOD   RED: DANGER   GREY: MIXED",
        437.,
        10.,
        Color::new(0.48, 0.61, 0.72, 1.),
    );
    centered(
        &format!(
            "M  MUSIC {}     S  SOUND {}     F11  FULLSCREEN",
            if a.music_on { "ON" } else { "OFF" },
            if a.sound_on { "ON" } else { "OFF" }
        ),
        462.,
        10.,
        GRAY,
    );
}
fn draw_help(v: &Visuals) {
    backdrop(v);
    centered("BONUSES & CONTROLS", 43., 27., WHITE);
    centered("MOUSE: MOVE   CLICK / SPACE: LAUNCH OR FIRE", 72., 12., ICE);
    centered(
        "P / RIGHT CLICK: PAUSE   M: MUSIC   N: TRACK   S: SOUND",
        94.,
        10.,
        GRAY,
    );
    centered("BLUE: GOOD   RED: DANGER   GREY: MIXED", 118., 10., GRAY);
    v.begin(vec2(320., 100.));
    for k in 0..19 {
        let x = 26. + (k / 10) as f32 * 304.;
        let y = 140. + (k % 10) as f32 * 28.;
        v.panel(Rect::new(x, y, 288., 24.), 4., INK);
        v.bonus(k, Rect::new(x + 3., y + 1., 30., 22.));
    }
    v.end();
    for k in 0..game::BONUS_NAMES.len() {
        let x = 26. + (k / 10) as f32 * 304.;
        let y = 140. + (k % 10) as f32 * 28.;
        Visuals::bonus_icon(k, Rect::new(x + 3., y + 1., 30., 22.));
    }
    for (k, label) in game::BONUS_NAMES.iter().enumerate() {
        let x = 26. + (k / 10) as f32 * 304.;
        let y = 140. + (k % 10) as f32 * 28.;
        text(label, x + 40., y + 16., 11., WHITE);
    }
    centered(
        "ARROW KEYS ALSO STEER  /  CLICK ANYWHERE TO RETURN",
        461.,
        11.,
        ICE,
    );
}
fn draw_scores(v: &Visuals, ui: &Ui) {
    backdrop(v);
    centered("YOUR BEST RUNS", 65., 29., WHITE);
    centered("EVERY BRICK COUNTS", 94., 11., ICE);
    v.begin(vec2(320., 120.));
    for i in 0..8 {
        v.panel(Rect::new(166., 120. + i as f32 * 35., 308., 29.), 5., INK);
    }
    v.end();
    for i in 0..8 {
        text(
            &format!("{:02}", i + 1),
            183.,
            140. + i as f32 * 35.,
            12.,
            ICE,
        );
        text(
            &format!("{:09}", ui.saved.scores.get(i).copied().unwrap_or(0)),
            328.,
            140. + i as f32 * 35.,
            17.,
            WHITE,
        );
    }
    centered("CLICK TO RETURN", 448., 12., ICE);
}
fn draw_editor(v: &Visuals, ui: &Ui) {
    backdrop(v);
    centered(
        &format!("BOARD WORKSHOP  {:02} / 50", ui.editor_board + 1),
        25.,
        18.,
        ICE,
    );
    v.begin(vec2(320., 200.));
    for (i, &b) in ui.editor.iter().enumerate() {
        let p = Game::cell_pos(i);
        if b > 0 && b != 7 {
            v.brick(b, Rect::new(p.x, p.y, 30., 15.));
        }
    }
    for i in 1..23 {
        v.brick(i as u8, Rect::new(32. + i as f32 * 25., 370., 24., 15.));
    }
    v.end();
    for (i, &b) in ui.editor.iter().enumerate() {
        let p = Game::cell_pos(i);
        if b == 0 || b == 7 {
            draw_rectangle_lines(
                p.x,
                p.y,
                30.,
                15.,
                0.4,
                if b == 7 {
                    ICE
                } else {
                    Color::new(0.12, 0.2, 0.25, 1.)
                },
            );
        }
        if b == 7 {
            text("?", p.x + 12., p.y + 11., 10., ICE);
        }
    }
    for i in 0..23 {
        let r = Rect::new(32. + i as f32 * 25., 370., 24., 22.);
        if i == ui.editor_kind as usize {
            draw_rectangle_lines(r.x - 1., r.y - 1., r.w + 2., r.h + 2., 1., ICE);
        }
        if i == 0 || i == 7 {
            text(
                if i == 0 { "X" } else { "?" },
                r.x + 7.,
                r.y + 12.,
                12.,
                ICE,
            );
        }
    }
    centered(
        "CLICK: PAINT   RIGHT CLICK: ERASE   BACKSPACE: CLEAR",
        416.,
        12.,
        GRAY,
    );
    centered(
        "+ / - BOARD   S SAVE   L LOAD   ENTER PLAY   ESC MENU",
        442.,
        12.,
        WHITE,
    );
    centered(
        "YOUR BOARDS ARE SAVED SEPARATELY FROM THE ORIGINALS",
        466.,
        10.,
        GRAY,
    );
}
fn record_score(g: &Game, ui: &mut Ui, quiet: bool) {
    if !quiet {
        ui.saved.scores.push(g.score);
        ui.saved.scores.sort_unstable_by(|a, b| b.cmp(a));
        ui.saved.scores.truncate(8);
    }
}

#[macroquad::main(conf)]
async fn main() {
    let screenshot = arg_value("--screenshot");
    let self_test = arg_value("--self-test");
    let quiet = automated();
    let mut a = Assets::new(quiet).await;
    UI_FONTS.with(|fonts| {
        *fonts.borrow_mut() = Some((
            load_ttf_font_from_bytes(include_bytes!("../resources/fonts/DejaVuSans.ttf"))
                .expect("regular interface font"),
            load_ttf_font_from_bytes(include_bytes!("../resources/fonts/DejaVuSans-Bold.ttf"))
                .expect("bold interface font"),
        ));
    });
    let _font_guard = UiFontGuard;
    prepare_fonts();
    let visuals = Visuals::new();
    let mut g = Game::new();
    let mut ui = Ui::new(quiet);
    if !quiet {
        a.sound_on = ui.saved.sound;
        if ui.saved.music {
            a.toggle_music().await;
        }
    }
    let target = render_target(
        (ARENA.w * RENDER_SCALE) as u32,
        (ARENA.h * RENDER_SCALE) as u32,
    );
    target.texture.set_filter(FilterMode::Nearest);
    let widescreen = render_target(OUTPUT_WIDTH, OUTPUT_HEIGHT);
    widescreen.texture.set_filter(FilterMode::Linear);
    let mut accumulator = 0.;
    let mut frames = 0;
    let mut last_mouse = vec2(-1., -1.);
    let mut keyboard_x = 320.;
    let mut fullscreen = false;
    let mut pending_click = false;
    let mut probe = verify::Probe::new();
    let mut hud_drawn = false;
    if screenshot.is_some() {
        g.start();
        if let Some(scene) = arg_value("--scene") {
            g.phase = match scene.to_str() {
                Some("menu") => Phase::Menu,
                Some("help") => Phase::Help,
                Some("scores") => Phase::Scores,
                Some("editor") => Phase::Editor,
                _ => Phase::Playing,
            };
        }
    } // Capture the actual first board with the ball ready to launch.
    loop {
        let (scale, origin) = viewport(screen_width(), screen_height());
        let mouse = (vec2(mouse_position().0, mouse_position().1) - origin) / scale
            - vec2(ARENA.x, ARENA.y);
        let mouse_moved = mouse != last_mouse;
        let clicked = is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Space);
        let before = g.phase;
        if !quiet {
            if is_key_pressed(KeyCode::F11) {
                fullscreen = !fullscreen;
                set_fullscreen(fullscreen);
            }
            if g.phase != Phase::Editor && is_key_pressed(KeyCode::S) {
                a.sound_on = !a.sound_on;
            }
            if is_key_pressed(KeyCode::M) {
                a.toggle_music().await;
            }
            if is_key_pressed(KeyCode::N) {
                a.next_music().await;
            }
            if is_key_pressed(KeyCode::Escape) {
                match g.phase {
                    Phase::Menu => break,
                    Phase::Help | Phase::Scores => g.phase = g.return_phase,
                    _ => g.phase = Phase::Menu,
                }
            }
            if is_key_pressed(KeyCode::F1) {
                if g.phase == Phase::Help {
                    g.phase = g.return_phase;
                } else {
                    g.return_phase = g.phase;
                    g.phase = Phase::Help;
                }
            }
            if is_key_pressed(KeyCode::F2) {
                g.phase = Phase::Editor;
            }
            match g.phase {
                Phase::Menu => {
                    if is_key_pressed(KeyCode::Down) {
                        ui.selected = (ui.selected + 1) % 4;
                    }
                    if is_key_pressed(KeyCode::Up) {
                        ui.selected = (ui.selected + 3) % 4;
                    }
                    for i in 0..4 {
                        if mouse_moved
                            && Rect::new(191., 158. + i as f32 * 44., 258., 34.).contains(mouse)
                        {
                            ui.selected = i;
                        }
                    }
                    if clicked || is_key_pressed(KeyCode::Enter) {
                        match ui.selected {
                            0 => g.start(),
                            1 => {
                                g.return_phase = Phase::Menu;
                                g.phase = Phase::Help;
                            }
                            2 => {
                                g.return_phase = Phase::Menu;
                                g.phase = Phase::Scores;
                            }
                            _ => g.phase = Phase::Editor,
                        };
                    }
                }
                Phase::Help | Phase::Scores => {
                    if clicked {
                        g.phase = g.return_phase;
                    }
                }
                Phase::GameOver | Phase::Won => {
                    if clicked || is_key_pressed(KeyCode::Enter) {
                        g.phase = Phase::Menu;
                    }
                }
                Phase::Playing | Phase::Paused => {
                    if is_key_pressed(KeyCode::P) || is_mouse_button_pressed(MouseButton::Right) {
                        g.phase = if g.phase == Phase::Playing {
                            Phase::Paused
                        } else {
                            Phase::Playing
                        };
                    }
                }
                Phase::Editor => {
                    for i in 0..23 {
                        if clicked
                            && Rect::new(32. + i as f32 * 25., 370., 24., 22.).contains(mouse)
                        {
                            ui.editor_kind = i as u8;
                        }
                    }
                    if (20.0..620.).contains(&mouse.x) && (50.0..350.).contains(&mouse.y) {
                        let i =
                            ((mouse.y - 50.) as usize / 15) * 20 + (mouse.x - 20.) as usize / 30;
                        if is_mouse_button_down(MouseButton::Left) {
                            ui.editor[i] = ui.editor_kind;
                        }
                        if is_mouse_button_down(MouseButton::Right) {
                            ui.editor[i] = 0;
                        }
                    }
                    if is_key_pressed(KeyCode::Backspace) {
                        ui.editor.fill(0);
                    }
                    if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::Minus) {
                        if is_key_pressed(KeyCode::Equal) {
                            ui.editor_board = (ui.editor_board + 1) % 50;
                        } else {
                            ui.editor_board = (ui.editor_board + 49) % 50;
                        }
                        ui.editor.copy_from_slice(
                            &game::BOARDS[ui.editor_board * 400..(ui.editor_board + 1) * 400],
                        );
                    }
                    if is_key_pressed(KeyCode::S) {
                        if let Err(e) = save_file(
                            &data_path().join(format!("board-{:02}.bin", ui.editor_board + 1)),
                            &ui.editor,
                        ) {
                            g.notice(&format!("SAVE FAILED: {e}"));
                        } else {
                            g.notice("BOARD SAVED");
                        }
                    }
                    if is_key_pressed(KeyCode::L) {
                        if let Ok(b) = std::fs::read(
                            data_path().join(format!("board-{:02}.bin", ui.editor_board + 1)),
                        ) {
                            if b.len() == 400 && b.iter().all(|&v| v <= 22) {
                                ui.editor.copy_from_slice(&b);
                                g.notice("BOARD LOADED");
                            }
                        }
                    }
                    if is_key_pressed(KeyCode::Enter) && ui.editor.iter().any(|&b| b != 0 && b != 2)
                    {
                        g.start();
                        g.board = ui.editor_board;
                        g.bricks = ui.editor;
                    }
                }
                _ => {}
            }
        }
        if before == Phase::Menu && g.phase == Phase::Playing {
            keyboard_x = g.paddle;
        }
        if mouse != last_mouse && !quiet {
            keyboard_x = mouse.x;
            last_mouse = mouse;
        }
        let direction = (is_key_down(KeyCode::Right) || is_key_down(KeyCode::D)) as i32
            - (is_key_down(KeyCode::Left) || is_key_down(KeyCode::A)) as i32;
        keyboard_x = (keyboard_x + direction as f32 * 480. * get_frame_time().min(0.05))
            .clamp(21. + g.width / 2., 618. - g.width / 2.);
        pending_click |= clicked && before == Phase::Playing;
        accumulator += if quiet {
            1. / 60.
        } else {
            get_frame_time().min(0.1)
        };
        while accumulator >= 1. / 60. {
            let input = if self_test.is_some() {
                probe.drive(&mut g)
            } else {
                Input {
                    x: if quiet { 320. } else { keyboard_x },
                    click: pending_click,
                }
            };
            let previous = g.phase;
            g.step(input);
            pending_click = false;
            if !matches!(previous, Phase::GameOver | Phase::Won)
                && matches!(g.phase, Phase::GameOver | Phase::Won)
            {
                record_score(&g, &mut ui, quiet);
            }
            for s in g.sounds.drain(..) {
                if !quiet {
                    a.sound(s);
                }
            }
            accumulator -= 1. / 60.;
        }
        set_camera(&Camera2D {
            render_target: Some(target.clone()),
            ..Camera2D::from_display_rect(Rect::new(0., 0., 640., 480.))
        });
        clear_background(BLACK);
        match g.phase {
            Phase::Menu => draw_menu(&visuals, &a, &ui),
            Phase::Help => draw_help(&visuals),
            Phase::Scores => draw_scores(&visuals, &ui),
            Phase::Editor => {
                draw_editor(&visuals, &ui);
                if g.message_time > 0 {
                    centered(&g.message, 360., 14., CYAN);
                }
            }
            _ => {
                hud_drawn |= draw_game(&g, &visuals);
            }
        }
        show_mouse(g.phase != Phase::Playing || quiet);
        set_camera(&Camera2D {
            render_target: Some(widescreen.clone()),
            ..Camera2D::from_display_rect(Rect::new(0., 0., VIEW_WIDTH, VIEW_HEIGHT))
        });
        draw_widescreen(&g, &a, &target.texture, &visuals);
        set_default_camera();
        clear_background(Color::new(0.012, 0.014, 0.019, 1.));
        draw_texture_ex(
            &widescreen.texture,
            origin.x,
            origin.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(VIEW_WIDTH * scale, VIEW_HEIGHT * scale)),
                flip_y: true,
                ..Default::default()
            },
        );
        frames += 1;
        if let Some(path) = &screenshot {
            if frames >= 120 {
                if let Some(parent) = path.parent() {
                    if !parent.as_os_str().is_empty() {
                        std::fs::create_dir_all(parent).expect("screenshot directory");
                    }
                }
                // Capture the actual 4K render, even when the desktop fits the window smaller.
                unsafe {
                    macroquad::window::get_internal_gl().flush();
                }
                widescreen
                    .texture
                    .get_texture_data()
                    .export_png(path.to_str().expect("UTF-8 screenshot path"));
                break;
            }
        }
        if let Some(path) = &self_test {
            if probe.done() {
                let result = probe.report(&g, hud_drawn);
                save_file(path, serde_json::to_vec_pretty(&result).unwrap().as_slice())
                    .expect("write self-test report");
                if !result["steps"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|s| s["passed"] == true)
                {
                    std::process::exit(1);
                }
                break;
            }
        }
        next_frame().await;
    }
    if !quiet {
        ui.saved.music = a.music_on;
        ui.saved.sound = a.sound_on;
        let _ = save_file(
            &data_path().join("settings.json"),
            &serde_json::to_vec_pretty(&ui.saved).unwrap(),
        );
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;

    #[test]
    fn widescreen_fit_and_pointer_alignment() {
        for (width, height) in [
            (1280., 720.),
            (1920., 1080.),
            (3840., 2160.),
            (7680., 4320.),
            (1024., 768.),
            (2560., 1080.),
        ] {
            let (scale, origin) = viewport(width, height);
            assert!(origin.x >= 0. && origin.y >= 0.);
            assert!(origin.x + VIEW_WIDTH * scale <= width + 0.01);
            assert!(origin.y + VIEW_HEIGHT * scale <= height + 0.01);
            for local in [vec2(20., 50.), vec2(320., 450.), vec2(620., 350.)] {
                let physical = origin + (vec2(ARENA.x, ARENA.y) + local) * scale;
                let mapped = (physical - origin) / scale - vec2(ARENA.x, ARENA.y);
                assert!(mapped.distance(local) < 0.001);
            }
        }
        assert_eq!(viewport(1280., 720.).1, Vec2::ZERO);
    }
}
