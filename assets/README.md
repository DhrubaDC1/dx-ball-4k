# Local game assets

Original sprites, fonts, boards, sound effects and converted music are deliberately excluded from Git. Import them from your own DX-Ball installation before building:

```sh
python tools/import_assets.py --source "/path/to/Dxball"
python tools/render_music.py
```

Install Pillow and NumPy and put FFmpeg on your PATH first. See the root README for complete build instructions. Keep imported files, screenshots and binaries containing them private.
