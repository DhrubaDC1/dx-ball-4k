This build contains the original game's own files, for the private use of the person who owns the game, on their own computer. It must not be shared or published.

The notice above applies to the playable local build after importing original assets. This public source repository excludes those assets, executables and screenshots.

# DX-Ball 4K · Brick Breaker

A Rust + macroquad desktop reconstruction with a 4K interface, a 50-board campaign, bonus effects and a local board editor.

This public repository contains the reconstruction source and import tools. Original game assets are **not distributed**: import them from your own DX-Ball installation for private local use. Local builds embed those assets; do not publish those binaries, asset files or gameplay screenshots. No original executable is launched or redistributed. See [CREDITS.md](CREDITS.md) for asset provenance and [MECHANICS.md](MECHANICS.md) for recovered rules and remaining differences.

The game renders at 4K UHD (3840×2160, 16:9), including the playfield, modern dark frame and side panel for score, lives, board progress and controls. Interface text is rasterized at 4K for sharper edges; the original pixel artwork keeps its crisp retro style. It requests a 4K window and fits smaller desktops automatically. Resizing and F11 fullscreen fit the whole layout without cropping or stretching; the original playfield proportions and physics are preserved.

## Play

Move the mouse to steer. Left-click to launch a held ball or fire the shooting paddle. Clear every breakable brick to continue; losing all balls costs one of your three paddles. Blue bonuses help, red bonuses hurt, and grey bonuses have mixed effects.

- P or right-click: pause/resume.
- Escape: menu/back; Escape again on the menu: exit.
- F1: controls and the original bonus reference.
- F2: board workshop. Paint with left-click, erase with right-click, choose a brick along the bottom, +/- changes boards, Backspace clears, S saves, L loads, Enter plays your board.
- M: optional music; N: next track; S outside the editor: sound effects.
- F11: fullscreen. Arrow keys or A/D also steer; Space also launches.

High scores, audio preferences and custom boards are saved in the operating system's application-data directory. Original boards are never overwritten. `BRICK_BREAKER_DATA` can select a different save directory.

## Build for your computer

Install Rust stable, Python 3 and FFmpeg, and have your own original game files available locally. From this repository, run:

```sh
python -m pip install Pillow numpy
python tools/import_assets.py --source "/path/to/Dxball"
python tools/render_music.py
cargo test --locked
cargo build --locked --release
```

Replace `/path/to/Dxball` with the folder containing `DEFAULT.BDS`, the `.PCX`/`.SBK` graphics, `.WAV` sounds and `.MDS` music. FFmpeg must be on your PATH to render music. Imported files remain ignored by Git. On Linux and macOS, use `python3` if your system has no `python` command. Run `target/release/brick-breaker` (Windows: `target\release\brick-breaker.exe`) to play. Assets are embedded in the local executable, which needs no separate game installation after building.

On Linux, ALSA development headers and X11/OpenGL runtime libraries are required. On Debian/Ubuntu the build dependencies are `build-essential pkg-config libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev`. Windows uses Rust with the Visual Studio C++ build tools. macOS uses Rust and Xcode command-line tools.

Run `python tools/package.py` after the release build to create a local portable package. Linux creates a tar.gz with a launcher, Windows a zip containing the .exe, and macOS a zip containing an .app bundle. macOS is unsigned; use the system's Open action for a private local build. The native build recipe at `tools/native-build.yml.example` is a template for a **private** build service with locally imported assets. It is not installed as an active public workflow. Do not upload imported assets or asset-containing packages to a public repository or public Actions artifacts.

The Linux build and rendering are checked here. Windows and macOS source paths and package recipes are provided but require native runners to validate compilation and play. No Windows/Mac release is claimed until that succeeds. Current Windows 11 and both Apple Silicon and Intel Mac targets are included in the native recipe.

## Automated checks

`cargo test` verifies board integrity, rebound numbers, launch input, life loss, speed/paddle bonuses, bullet limits and the scripted core loop.

`brick-breaker --screenshot picture.png` opens a passive, non-focusing window, draws the first board for 120 frames, writes the actual 3840×2160 render as a PNG (even on smaller desktops) and exits. On Windows it also stays behind other windows. `--self-test results.json` drives the same game input and update paths for 600 simulation frames, measures paddle motion, launch, brick hits, scoring and rebounds, renders the HUD, then writes a structured report and exits. These are bounded commands, not servers.

## Fidelity boundaries

Recovered: 640x480 arena; 60 Hz integer-frame ball movement; 73-pixel paddle; 9-pixel ball; 3 lives; original 50 layouts; durable, invisible, explosive and metal brick rules; discrete paddle rebound zones; speed range; all bonus effects; bullet scoring and limits; final-brick lightning timer.

Differences: neutral title and modern menu; keyboard alternatives; resize/fullscreen support; a simpler editor and high-score presentation; approximate transition timing, particle trajectories, palette cycling and MIDI instruments. MIDI note events were converted from the original MDS streams and rendered with a portable synthesizer; effects use original WAV audio. Exact oblique collision pixel behavior and live gameplay feel remain unverified because this session cannot run the original. Static evidence does not establish runtime equivalence.

The vendored miniquad change only prevents automated screenshots/self-tests from activating the window. Its MIT/Apache licenses remain included. `tools/import_assets.py` and `tools/render_music.py` prepare the local assets before the first build. Subsequent builds reuse them. The source repository does not grant redistribution rights to the original game's assets.
