fn main() {
    println!("cargo:rerun-if-changed=assets/boards.bin");
    if !std::path::Path::new("assets/boards.bin").is_file() {
        panic!(
            "Original game assets are not distributed with this source repository. \
             Run python tools/import_assets.py --source /path/to/Dxball, then \
             python tools/render_music.py before building. See README.md."
        );
    }
}
