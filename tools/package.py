"""Package an already-built native executable. Does not upload or publish anything."""
from pathlib import Path
import argparse,platform,shutil,tarfile,zipfile,plistlib,stat
p=argparse.ArgumentParser();p.add_argument('--binary');p.add_argument('--os',choices=['linux','windows','macos']);p.add_argument('--arch',default=platform.machine());a=p.parse_args()
root=Path(__file__).resolve().parents[1];system=a.os or {'Linux':'linux','Windows':'windows','Darwin':'macos'}[platform.system()]
ext='.exe' if system=='windows' else '';binary=Path(a.binary)if a.binary else root/'target/release'/('brick-breaker'+ext)
if not binary.is_file():raise SystemExit('Build the release executable on this operating system first.')
stage=root/'dist'/f'brick-breaker-{system}-{a.arch}';stage.mkdir(parents=True,exist_ok=True)
if system=='macos':
 bundle=stage/'Brick Breaker.app'/'Contents';(bundle/'MacOS').mkdir(parents=True,exist_ok=True);(bundle/'Resources').mkdir(exist_ok=True)
 shutil.copy2(binary,bundle/'MacOS/brick-breaker')
 (bundle/'Info.plist').write_bytes(plistlib.dumps({'CFBundleName':'Brick Breaker','CFBundleDisplayName':'Brick Breaker','CFBundleExecutable':'brick-breaker','CFBundleIdentifier':'local.private.brickbreaker','CFBundlePackageType':'APPL','CFBundleVersion':'1','CFBundleShortVersionString':'0.1.0','LSMinimumSystemVersion':'11.0','NSHighResolutionCapable':True}))
else:shutil.copy2(binary,stage/('brick-breaker'+ext))
if system=='linux':
 launch=stage/'Play.sh';launch.write_text('#!/bin/sh\ncd -- "$(dirname -- "$0")"\nexec ./brick-breaker "$@"\n');launch.chmod(0o755)
for name in ['README.md','CREDITS.md','MECHANICS.md','mechanics.json']:shutil.copy2(root/name,stage/name)
license_dir=stage/'licenses';license_dir.mkdir(exist_ok=True)
for name in ['LICENSE-MIT','LICENSE-APACHE']:shutil.copy2(root/'vendor-miniquad'/name,license_dir/('miniquad-'+name))
if system=='linux':
 archive=stage.with_suffix('.tar.gz')
 with tarfile.open(archive,'w:gz')as t:t.add(stage,arcname=stage.name)
else:
 archive=stage.with_suffix('.zip')
 with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED)as z:
  for f in stage.rglob('*'):
   if f.is_file():z.write(f,f.relative_to(stage.parent))
print(archive)
