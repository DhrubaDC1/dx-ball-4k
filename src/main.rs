#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod assets;
mod game;
mod verify;
use assets::Assets;
use game::{Game, Input, Phase};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
fn text(s: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_text_ex(
        s,
        x,
        y,
        TextParams {
            font_size: (size * RENDER_SCALE) as u16,
            font_scale: 1. / RENDER_SCALE,
            color,
            ..Default::default()
        },
    );
}
fn centered(s: &str, y: f32, size: f32, color: Color) {
    let w = measure_text(s, None, (size * RENDER_SCALE) as u16, 1. / RENDER_SCALE).width;
    text(s, (640. - w) / 2., y, size, color);
}
const CYAN: Color = Color::new(0.26, 0.92, 0.94, 1.);
fn button(label: &str, r: Rect, selected: bool) {
    draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.025, 0.045, 0.055, 1.));
    draw_rectangle_lines(
        r.x,
        r.y,
        r.w,
        r.h,
        1.,
        if selected {
            CYAN
        } else {
            Color::new(0.16, 0.24, 0.27, 1.)
        },
    );
    text(
        label,
        r.x + 14.,
        r.y + r.h * 0.67,
        16.,
        if selected { WHITE } else { GRAY },
    );
    if selected {
        draw_rectangle(r.x, r.y, 3., r.h, CYAN);
    }
}
fn backdrop(a: &Assets) {
    draw_texture(&a.background, 0., 0., WHITE);
}
fn draw_widescreen(g: &Game, a: &Assets, arena: &Texture2D) {
    let muted = Color::new(0.49, 0.59, 0.64, 1.);
    let edge = Color::new(0.13, 0.23, 0.27, 1.);
    clear_background(Color::new(0.025, 0.04, 0.055, 1.));
    draw_rectangle(12., 18., 664., 504., Color::new(0.04, 0.065, 0.08, 1.));
    draw_rectangle_lines(12., 18., 664., 504., 1., edge);
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
    draw_rectangle(700., 30., 28., 3., CYAN);
    text("BRICK", 700., 75., 34., WHITE);
    text("BREAKER", 700., 109., 34., WHITE);
    text("50 BOARDS. ONE MORE TRY.", 700., 136., 12., muted);
    draw_line(700., 158., 932., 158., 1., edge);

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
        text("SCORE", 700., 190., 12., muted);
        text(&format!("{:06}", g.score), 700., 229., 36., WHITE);
        text("BOARD", 700., 267., 12., muted);
        text(&format!("{:02} / 50", g.board + 1), 700., 294., 24., CYAN);
        text("LIVES", 850., 267., 12., muted);
        text(&g.lives.max(0).to_string(), 850., 294., 24., WHITE);
        text(
            &format!("{} BRICKS LEFT", g.remaining()),
            700.,
            328.,
            13.,
            muted,
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
        text(title, 700., 200., 19., CYAN);
        for (i, line) in lines.iter().enumerate() {
            text(line, 700., 236. + i as f32 * 25., 15., muted);
        }
    }
    draw_line(700., 356., 932., 356., 1., edge);
    text("QUICK CONTROLS", 700., 384., 12., muted);
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
        let y = 413. + i as f32 * 25.;
        text(key, 700., y, 12., WHITE);
        text(action, 810., y, 12., muted);
    }
    text(
        if a.music_on { "MUSIC ON" } else { "MUSIC OFF" },
        700.,
        516.,
        10.,
        muted,
    );
    text("4K", 908., 516., 10., CYAN);
}
fn draw_game(g: &Game, a: &Assets) -> bool {
    backdrop(a);
    for (i, &b) in g.bricks.iter().enumerate() {
        if b == 0 || b == 7 {
            continue;
        }
        let id = match b {
            1 => 3,
            2 => 4,
            3 => 5,
            4 => 6,
            5 => 7,
            6 => 8,
            8 => 9,
            9..=17 => b as u32 + 1,
            18..=22 => b as u32 + 38,
            _ => 3,
        };
        let p = Game::cell_pos(i);
        a.game.sprite(id, p.x, p.y);
    }
    for e in &g.bursts {
        if e.sprite == 145 {
            let id = 145 + (e.age / 2).min(21);
            a.game.sprite(id, e.p.x - 20., e.p.y - 25.);
        } else if e.age < 12 {
            a.game.sprite(e.sprite + (e.age / 4).min(2), e.p.x, e.p.y);
        }
    }
    for b in &g.bonuses {
        let id = if b.kind == 15 {
            50
        } else if b.kind == 16 {
            51
        } else if b.kind == 17 {
            52
        } else if b.kind == 18 {
            53
        } else {
            35 + b.kind as u32
        };
        a.game.sprite(id, b.p.x, b.p.y);
    }
    for b in &g.balls {
        a.game.sprite(b.sprite, b.p.x, b.p.y);
    }
    for p in &g.bullets {
        a.game.sprite(32, p.x, p.y);
    }
    if !matches!(g.phase, Phase::Lost | Phase::GameOver) {
        let n = (g.width / 73.) as u32;
        let frame = (g.ticks / 4 % 4) as u32;
        let base = if g.shooting {
            104
        } else if g.balls.iter().any(|b| b.held) {
            84
        } else {
            64
        };
        let id = base + n * 4 + frame;
        let h = if g.shooting { 29. } else { 14. };
        a.game.sprite(id, g.paddle - g.width / 2., 450. - (h - 14.));
    }
    a.large_text(&g.score.to_string(), 30., 0., 1.);
    for i in 0..(g.lives - 1).clamp(0, 19) {
        a.game
            .sprite(31, 598. - (i % 10) as f32 * 22., 2. + (i / 10) as f32 * 8.);
    }
    // A small board readout and discoverable controls are additions outside the brick field.
    a.small_text(&format!("BOARD {:02} / 50", g.board + 1), 280., 12., 1.);
    a.small_text("F1 HELP   P PAUSE   ESC MENU", 24., 470., 1.);
    if g.message_time > 0 {
        centered(&g.message, 404., 16., CYAN);
    }
    if g.balls.iter().any(|b| b.held) && g.message_time == 0 {
        centered(
            "CLICK TO RELEASE",
            415.,
            13.,
            Color::new(0.5, 0.57, 0.6, 1.),
        );
    }
    if g.phase == Phase::Paused {
        draw_rectangle(180., 207., 280., 62., Color::new(0., 0., 0., 0.88));
        a.large_text("PAUSED", 221., 220., 0.8);
        centered("P OR RIGHT CLICK TO CONTINUE", 292., 13., GRAY);
    }
    if matches!(g.phase, Phase::GameOver | Phase::Won) {
        draw_rectangle(120., 185., 400., 117., Color::new(0., 0., 0., 0.9));
        centered(
            if g.phase == Phase::Won {
                "ALL 50 BOARDS CLEARED"
            } else {
                "GAME OVER"
            },
            226.,
            26.,
            WHITE,
        );
        centered(&format!("FINAL SCORE  {}", g.score), 254., 18., CYAN);
        centered("CLICK TO RETURN TO MENU", 282., 13., GRAY);
    }
    true
}
fn draw_menu(g: &Game, a: &Assets, ui: &Ui, _mouse: Vec2) {
    backdrop(a);
    for y in 0..4 {
        for x in 0..14 {
            let id = [3, 14, 11, 10][y];
            a.game
                .sprite(id, 110. + x as f32 * 30., 38. + y as f32 * 15.);
        }
    }
    draw_rectangle(106., 34., 428., 69., Color::new(0., 0., 0., 0.68));
    a.large_text("BRICK BREAKER", 139., 49., 0.93);
    centered("A CLASSIC PADDLE & BRICK GAME", 126., 13., CYAN);
    let options = [
        "PLAY GAME",
        "BONUSES & CONTROLS",
        "HIGH SCORES",
        "BOARD WORKSHOP",
    ];
    for (i, label) in options.iter().enumerate() {
        let r = Rect::new(191., 158. + i as f32 * 44., 258., 34.);
        button(label, r, ui.selected == i);
    }
    a.game.sprite(68, 283., 366.);
    a.game.sprite(1, 315., 351.);
    centered("MOVE THE MOUSE  /  CLICK TO LAUNCH", 414., 14., WHITE);
    centered("BLUE: GOOD   RED: DANGER   GREY: MIXED", 436., 12., GRAY);
    centered(
        &format!(
            "M  MUSIC {}     S  SOUND {}     F11  FULLSCREEN",
            if a.music_on { "ON" } else { "OFF" },
            if a.sound_on { "ON" } else { "OFF" }
        ),
        462.,
        11.,
        GRAY,
    );
    let _ = g;
}
fn draw_help(a: &Assets) {
    draw_texture(&a.help, 0., 0., WHITE);
    // Finish the opaque reference layer before drawing its text overlays.
    unsafe {
        macroquad::window::get_internal_gl().flush();
    }
    a.large_text("BONUSES", 204., 48., 0.95);
    centered(
        "KEEP EVERY BALL IN PLAY. CLEAR THE BOARD.",
        148.,
        16.,
        WHITE,
    );
    centered("MOUSE: MOVE     LEFT CLICK: LAUNCH / FIRE", 179., 14., CYAN);
    centered("RIGHT CLICK OR P: PAUSE     ESC: BACK", 200., 14., GRAY);
    centered("M: MUSIC     N: NEXT TRACK     S: SOUND", 222., 13., GRAY);
    text("SPACE AND ARROWS ALSO WORK", 34., 247., 11., GRAY);
    text("CLICK ANYWHERE TO RETURN", 400., 472., 12., WHITE);
}
fn draw_scores(a: &Assets, ui: &Ui) {
    backdrop(a);
    a.large_text("HIGH SCORES", 170., 72., 0.85);
    for i in 0..8 {
        let score = ui.saved.scores.get(i).copied().unwrap_or(0);
        a.small_text(
            &format!("{:02}       {:09}", i + 1, score),
            219.,
            145. + i as f32 * 29.,
            2.,
        );
    }
    centered("CLICK TO RETURN", 448., 14., CYAN);
}
fn draw_editor(a: &Assets, ui: &Ui) {
    backdrop(a);
    centered(
        &format!("BOARD WORKSHOP  {:02} / 50", ui.editor_board + 1),
        25.,
        18.,
        CYAN,
    );
    for (i, &b) in ui.editor.iter().enumerate() {
        let p = Game::cell_pos(i);
        if b > 0 {
            let id = match b {
                1..=7 => b as u32 + 2,
                8 => 9,
                9..=17 => b as u32 + 1,
                18..=22 => b as u32 + 38,
                _ => 3,
            };
            a.game.sprite(id, p.x, p.y);
        } else {
            draw_rectangle_lines(p.x, p.y, 30., 15., 0.4, Color::new(0.09, 0.12, 0.13, 1.));
        }
    }
    for i in 0..23 {
        let r = Rect::new(32. + i as f32 * 25., 370., 24., 22.);
        if i == ui.editor_kind as usize {
            draw_rectangle_lines(r.x - 1., r.y - 1., r.w + 2., r.h + 2., 1., CYAN);
        }
        if i > 0 {
            let b = i as u8;
            let id = match b {
                1..=7 => b as u32 + 2,
                8 => 9,
                9..=17 => b as u32 + 1,
                18..=22 => b as u32 + 38,
                _ => 3,
            };
            a.game.sized(id, r.x, r.y, 24., 15.);
        } else {
            text("X", r.x + 7., r.y + 14., 14., GRAY);
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
    let mut g = Game::new();
    let mut ui = Ui::new(quiet);
    if !quiet {
        a.sound_on = ui.saved.sound;
        if ui.saved.music {
            a.toggle_music();
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
                a.toggle_music();
            }
            if is_key_pressed(KeyCode::N) {
                a.next_music();
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
            Phase::Menu => draw_menu(&g, &a, &ui, mouse),
            Phase::Help => draw_help(&a),
            Phase::Scores => draw_scores(&a, &ui),
            Phase::Editor => {
                draw_editor(&a, &ui);
                if g.message_time > 0 {
                    centered(&g.message, 360., 14., CYAN);
                }
            }
            _ => {
                hud_drawn |= draw_game(&g, &a);
            }
        }
        show_mouse(g.phase != Phase::Playing || quiet);
        set_camera(&Camera2D {
            render_target: Some(widescreen.clone()),
            ..Camera2D::from_display_rect(Rect::new(0., 0., VIEW_WIDTH, VIEW_HEIGHT))
        });
        draw_widescreen(&g, &a, &target.texture);
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
