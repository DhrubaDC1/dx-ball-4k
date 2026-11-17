use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use std::collections::HashMap;

pub struct Assets {
    pub sounds: HashMap<usize, Sound>,
    pub music: Vec<Option<Sound>>,
    pub sound_on: bool,
    pub music_on: bool,
    pub track: usize,
}
const MUSIC: [&[u8]; 6] = [
    include_bytes!("../assets/freebee.ogg"),
    include_bytes!("../assets/12flight.ogg"),
    include_bytes!("../assets/acker-gs.ogg"),
    include_bytes!("../assets/brain.ogg"),
    include_bytes!("../assets/ethno_pa.ogg"),
    include_bytes!("../assets/gmfigaro.ogg"),
];
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
        // Decode a track only when it is played, instead of all six at startup.
        let music = (0..MUSIC.len()).map(|_| None).collect();
        Self {
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
    pub async fn toggle_music(&mut self) {
        self.music_on = !self.music_on;
        self.update_music().await;
    }
    pub async fn next_music(&mut self) {
        self.track = (self.track + 1) % MUSIC.len();
        self.update_music().await;
    }
    async fn update_music(&mut self) {
        for s in self.music.iter().flatten() {
            stop_sound(s);
        }
        if self.music_on {
            if self.music[self.track].is_none() {
                self.music[self.track] = load_sound_from_bytes(MUSIC[self.track]).await.ok();
            }
            if let Some(s) = &self.music[self.track] {
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
}
