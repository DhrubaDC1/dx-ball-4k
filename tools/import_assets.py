"""Read unencrypted sprite banks using the layout established in function 0x404610."""
from pathlib import Path
import argparse
p=argparse.ArgumentParser(description='Import original DX-Ball assets for private local use.')
p.add_argument('--source',type=Path,required=True,help='Folder containing DEFAULT.BDS, PCX, SBK, WAV and MDS files')
a=p.parse_args()
src=a.source.expanduser().resolve()
if not (src/'DEFAULT.BDS').is_file():p.error('Choose the original Dxball game-data folder containing DEFAULT.BDS.')
from PIL import Image, ImageDraw
import struct,json,shutil
out=Path(__file__).resolve().parents[1]/'assets';out.mkdir(exist_ok=True)
credits=[]
for name in ['MBBKGRND.PCX','MAINMENU.PCX','INTRO.PCX','HIGHSCOR.PCX','BIGBOLT.PCX']:
 im=Image.open(src/name); im.save(out/(name.lower().replace('.pcx','.png'))); credits.append(name)
pal=Image.open(src/'MBBKGRND.PCX').getpalette()
for name in ['MBALL2.SBK','THEFONT.SBK','SFONT.SBK','MAINMENU.SBK']:
 d=(src/name).read_bytes();count=struct.unpack_from('<I',d)[0];pos=4; sprites=[]
 atlas=Image.new('RGBA',(1024,2048)); x=y=row=0; meta={}; sheet=Image.new('RGB',(800,((count+9)//10)*65),'#222222');draw=ImageDraw.Draw(sheet)
 for i in range(1,count+1):
  w,h,key,tag=struct.unpack_from('<IIBI',d,pos);pos+=13
  assert 0<w<2000 and 0<h<2000,(name,i,w,h)
  im=Image.frombytes('P',(w,h),d[pos:pos+w*h]);pos+=w*h;im.putpalette(pal);im=im.transpose(Image.Transpose.FLIP_TOP_BOTTOM).convert('RGBA')
  # The engine sets palette index zero as its transparent colour key.
  raw=bytearray(im.tobytes())
  idx=d[pos-w*h:pos]
  for py in range(h):
   for px in range(w):
    if idx[(h-1-py)*w+px]==0:raw[(py*w+px)*4+3]=0
  im=Image.frombytes('RGBA',(w,h),bytes(raw))
  if x+w>1024:x=0;y+=row+2;row=0
  atlas.paste(im,(x,y));meta[str(i)]=[x,y,w,h,key,tag];x+=w+2;row=max(row,h)
  thumb=im.copy();thumb.thumbnail((72,45));tx=((i-1)%10)*80;ty=((i-1)//10)*65;sheet.paste(thumb,(tx,ty),thumb);draw.text((tx,ty+46),str(i),fill='white')
 atlas_name=name.lower().replace('.sbk','-sprites' if name=='MAINMENU.SBK' else '')
 atlas.crop((0,0,1024,y+row+2)).save(out/(atlas_name+'.png'))
 (out/(name.lower().replace('.sbk','.json'))).write_text(json.dumps(meta))
 sheet.save(out/(name.lower().replace('.sbk','-contact.png')));credits.append(name)
 print(name,count,'bytes consumed',pos,len(d))
shutil.copyfile(src/'DEFAULT.BDS',out/'boards.bin');credits.append('DEFAULT.BDS')
for p in src.glob('*.WAV'):shutil.copyfile(p,out/p.name.lower());credits.append(p.name)
# Convert MIDI Stream event blocks to standard MIDI for archival and later playback.
for p in src.glob('*.MDS'):
 d=p.read_bytes(); idx=d.find(b'data'); chunk=d[idx+8:];count=struct.unpack_from('<I',chunk)[0];pos=4;events=bytearray()
 def vlq(n):
  a=[n&127];n>>=7
  while n:a.insert(0,(n&127)|128);n>>=7
  return bytes(a)
 for _ in range(count):
  _,size=struct.unpack_from('<II',chunk,pos);pos+=8;end=pos+size
  while pos+8<=end:
   delta,event=struct.unpack_from('<II',chunk,pos);pos+=8;typ=event>>24
   if typ==0:
    status=event&255;msg=bytes([status,(event>>8)&127]);
    if status&0xf0 not in [0xc0,0xd0]:msg+=bytes([(event>>16)&127])
    events+=vlq(delta)+msg
   elif typ==1:events+=vlq(delta)+b'\xff\x51\x03'+(event&0xffffff).to_bytes(3,'big')
   elif typ&128:pos+=((event&0xffffff)+3)&~3
  pos=end
 events+=b'\x00\xff\x2f\x00';division=struct.unpack_from('<I',d,d.find(b'fmt ')+8)[0]
 (out/(p.stem.lower()+'.mid')).write_bytes(b'MThd'+struct.pack('>IHHH',6,0,1,division)+b'MTrk'+struct.pack('>I',len(events))+events);credits.append(p.name)
root=out.parent
(root/'CREDITS.md').write_text('# Original files used locally\n\nOriginal game by Michael P. Welch; graphics by Seumas McNally; boards credited to Sarah Welch. Original README identifies music and sounds as public domain. Imported assets and executables containing them are for private local use only; they are excluded from the public source repository. This credits list contains filenames, not the original files or personal installation paths. Vendored miniquad retains its MIT/Apache licenses in vendor-miniquad/.\n\n'+'\n'.join('- `Dxball/'+n+'` — '+('converted to PNG sprite atlas' if n.endswith('.SBK') else 'converted to PNG' if n.endswith('.PCX') else 'converted to standard MIDI' if n.endswith('.MDS') else 'copied without modification') for n in credits)+'\n')
