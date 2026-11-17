use macroquad::{
    audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound},
    prelude::*,
};
use std::collections::HashMap;

pub struct Atlas {
    pub texture: Texture2D,
    pub rects: HashMap<u32, Vec<f32>>,
}
impl Atlas {
    fn new(png: &[u8], json: &str) -> Self {
        let texture = Texture2D::from_file_with_format(png, Some(ImageFormat::Png));
        texture.set_filter(FilterMode::Nearest);
        Self {
            texture,
            rects: serde_json::from_str(json).unwrap(),
        }
    }
    pub fn sprite(&self, id: u32, x: f32, y: f32) {
        if let Some(r) = self.rects.get(&id) {
            self.sized(id, x, y, r[2], r[3]);
        }
    }
    pub fn sized(&self, id: u32, x: f32, y: f32, w: f32, h: f32) {
        if let Some(r) = self.rects.get(&id) {
            draw_texture_ex(
                &self.texture,
                x.round(),
                y.round(),
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(r[0], r[1], r[2], r[3])),
                    dest_size: Some(vec2(w, h)),
                    ..Default::default()
                },
            );
        }
    }
}
pub struct Assets {
    pub game: Atlas,
    pub font: Atlas,
    pub small: Atlas,
    pub background: Texture2D,
    pub help: Texture2D,
    pub sounds: HashMap<usize, Sound>,
    pub music: Vec<Sound>,
    pub sound_on: bool,
    pub music_on: bool,
    pub track: usize,
}
macro_rules! wav {
    ($p:literal) => {
        include_bytes!(concat!("../assets/", $p, ".wav")) as &[u8]
    };
}
impl Assets {
    pub async fn new(quiet: bool) -> Self {
        let mut sounds = HashMap::new();
        if !quiet {
            let files: [(usize, &[u8]); 26] = [
                (0, wav!("boing")),
                (1, wav!("effect")),
                (2, wav!("bang")),
                (3, wav!("ao-laser")),
                (4, wav!("bassdrum")),
                (5, wav!("byeball")),
                (7, wav!("wowpulse")),
                (8, wav!("saucer")),
                (9, wav!("orchestr")),
                (10, wav!("effect2")),
                (11, wav!("sweepdow")),
                (12, wav!("peow!")),
                (13, wav!("fanfare")),
                (14, wav!("padexplo")),
                (15, wav!("ricochet")),
                (16, wav!("swordswi")),
                (17, wav!("gunfire")),
                (18, wav!("humm")),
                (19, wav!("glass")),
                (20, wav!("orchblas")),
                (21, wav!("voltage")),
                (22, wav!("thudclap")),
                (30, wav!("tank")),
                (31, wav!("xplosht1")),
                (32, wav!("xploshor")),
                (33, wav!("whine")),
            ];
            for (id, bytes) in files {
                if let Ok(s) = load_sound_from_bytes(bytes).await {
                    sounds.insert(id, s);
                }
            }
        }
        let mut music = vec![];
        if !quiet {
            for bytes in [
                include_bytes!("../assets/freebee.ogg") as &[u8],
                include_bytes!("../assets/12flight.ogg"),
                include_bytes!("../assets/acker-gs.ogg"),
                include_bytes!("../assets/brain.ogg"),
                include_bytes!("../assets/ethno_pa.ogg"),
                include_bytes!("../assets/gmfigaro.ogg"),
            ] {
                if let Ok(s) = load_sound_from_bytes(bytes).await {
                    music.push(s);
                }
            }
        }
        let background = Texture2D::from_file_with_format(
            include_bytes!("../assets/mbbkgrnd.png"),
            Some(ImageFormat::Png),
        );
        background.set_filter(FilterMode::Nearest);
        let help = Texture2D::from_file_with_format(
            include_bytes!("../assets/mainmenu.png"),
            Some(ImageFormat::Png),
        );
        help.set_filter(FilterMode::Nearest);
        Self {
            game: Atlas::new(
                include_bytes!("../assets/mball2.png"),
                include_str!("../assets/mball2.json"),
            ),
            font: Atlas::new(
                include_bytes!("../assets/thefont.png"),
                include_str!("../assets/thefont.json"),
            ),
            small: Atlas::new(
                include_bytes!("../assets/sfont.png"),
                include_str!("../assets/sfont.json"),
            ),
            background,
            help,
            sounds,
            music,
            sound_on: !quiet,
            music_on: false,
            track: 0,
        }
    }
    pub fn sound(&self, id: usize) {
        if self.sound_on {
            if let Some(s) = self.sounds.get(&id) {
                play_sound(
                    s,
                    PlaySoundParams {
                        looped: false,
                        volume: 0.55,
                    },
                );
            }
        }
    }
    pub fn toggle_music(&mut self) {
        self.music_on = !self.music_on;
        self.update_music();
    }
    pub fn next_music(&mut self) {
        self.track = (self.track + 1) % self.music.len().max(1);
        self.update_music();
    }
    fn update_music(&self) {
        for s in &self.music {
            stop_sound(s);
        }
        if self.music_on {
            if let Some(s) = self.music.get(self.track) {
                play_sound(
                    s,
                    PlaySoundParams {
                        looped: true,
                        volume: 0.35,
                    },
                );
            }
        }
    }
    pub fn small_text(&self, s: &str, x: f32, y: f32, scale: f32) {
        let mut x = x;
        for c in s.to_uppercase().chars() {
            let id = c as u32 - 31;
            if let Some(r) = self.small.rects.get(&id) {
                let offset = r[5] as u32 as i32;
                self.small.sized(
                    id,
                    x,
                    y + (7. - r[3] - offset as f32) * scale,
                    r[2] * scale,
                    r[3] * scale,
                );
                x += (r[2] + 1.) * scale;
            } else {
                x += 4. * scale;
            }
        }
    }
    pub fn large_text(&self, s: &str, x: f32, y: f32, scale: f32) {
        let mut x = x;
        for c in s.to_uppercase().chars() {
            if let Some((&id, r)) = self
                .font
                .rects
                .iter()
                .find(|(_, r)| r[4] as u32 == c as u32)
            {
                self.font.sized(id, x, y, r[2] * scale, r[3] * scale);
                x += (r[2] + 2.) * scale;
            } else {
                x += 12. * scale;
            }
        }
    }
}
