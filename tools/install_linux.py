"""Install the private release build in the current user's Linux app launcher."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


def install_file(source, destination, mode):
    destination.parent.mkdir(parents=True, exist_ok=True)
    # Replace atomically so updating an installed, running game is safe.
    fd, temporary = tempfile.mkstemp(dir=destination.parent)
    try:
        with os.fdopen(fd, "wb") as output, source.open("rb") as original:
            shutil.copyfileobj(original, output)
        os.chmod(temporary, mode)
        os.replace(temporary, destination)
    finally:
        Path(temporary).unlink(missing_ok=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--data-home", type=Path, help="Override XDG_DATA_HOME")
    args = parser.parse_args()
    if sys.platform != "linux":
        parser.error("This installer is for Linux.")
    here = Path(__file__).resolve().parent
    root = here.parent if here.name == "tools" else here
    binary = args.binary or root / "brick-breaker"
    if not binary.is_file() and args.binary is None:
        binary = root / "target/release/brick-breaker"
    if not binary.is_file():
        parser.error("Build the game in release mode before installing it.")
    icon = root / "resources/brick-breaker.svg"
    if not icon.is_file():
        parser.error("The game icon is missing.")
    data_home = args.data_home or Path(os.environ.get("XDG_DATA_HOME") or Path.home() / ".local/share")
    data_home = data_home.expanduser().resolve()
    executable = data_home / "brick-breaker/bin/brick-breaker"
    install_file(binary, executable, 0o755)
    install_file(icon, data_home / "icons/hicolor/scalable/apps/brick-breaker.svg", 0o644)
    # Exec is parsed by the desktop launcher, not a shell. Escape both its
    # reserved characters and percent field codes (including paths with spaces).
    command = str(executable).replace("\\", "\\\\").replace('"', '\\"')
    for character in ("`", "$"):
        command = command.replace(character, "\\" + character)
    command = command.replace("%", "%%").replace("\\", "\\\\")
    entry = data_home / "applications/brick-breaker.desktop"
    entry.parent.mkdir(parents=True, exist_ok=True)
    entry.write_text(
        "[Desktop Entry]\nType=Application\nName=DX-Ball 4K\n"
        "Comment=Play the brick breaker\n"
        # Some launchers check the program path before expanding %% field codes.
        # env keeps that lookup independent of special characters in the path.
        f'Exec=/usr/bin/env "{command}"\n'
        "Icon=brick-breaker\nTerminal=false\nCategories=Game;ArcadeGame;\n",
        encoding="utf-8",
    )
    updater = shutil.which("update-desktop-database")
    if updater:
        subprocess.run([updater, str(entry.parent)], check=False)
    print(f"Installed DX-Ball 4K: {entry}")
    print("Open your app launcher and choose DX-Ball 4K. No build or terminal needed.")


if __name__ == "__main__":
    try:
        main()
    except OSError as error:
        raise SystemExit(f"Could not install DX-Ball 4K: {error}") from None
