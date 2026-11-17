"""Portable preview of the original MIDI note events. Timbres are an approximation."""
from pathlib import Path
import struct, numpy as np, subprocess, tempfile, wave
out=Path(__file__).resolve().parents[1]/'assets'
for p in out.glob('*.mid'):
 d=p.read_bytes();end=len(d);pos=22;tick=0;tempo=500000;division=int.from_bytes(d[12:14],'big');seconds=0.;active={};notes=[]
 def vlq():
  global pos
  v=0
  while True:
   b=d[pos];pos+=1;v=v*128+(b&127)
   if b<128:return v
 while pos<end:
  delta=vlq();seconds+=delta*tempo/division/1000000
  status=d[pos];pos+=1
  if status==255:
   typ=d[pos];pos+=1;size=vlq();payload=d[pos:pos+size];pos+=size
   if typ==81:tempo=int.from_bytes(payload,'big')
   if typ==47:break
   continue
  a=d[pos];pos+=1;cmd=status&240;channel=status&15
  b=0
  if cmd not in [192,208]:b=d[pos];pos+=1
  key=(channel,a)
  if cmd==144 and b:
   active[key]=(seconds,b)
  elif cmd==128 or cmd==144:
   if key in active:
    start,vel=active.pop(key);notes.append((start,seconds,a,vel,channel))
 for(ch,n),(st,v)in active.items():notes.append((st,seconds,n,v,ch))
 rate=22050;audio=np.zeros(int((seconds+1)*rate),dtype=np.float32)
 for st,en,n,v,ch in notes:
  begin=int(st*rate);length=max(1,min(int((en-st+.12)*rate),len(audio)-begin));t=np.arange(length)/rate;f=440*2**((n-69)/12)
  if ch==9:
   rng=np.random.default_rng(n);sig=rng.uniform(-1,1,length)*np.exp(-t*22)*.13
  else:
   sig=(np.sin(2*np.pi*f*t)+.25*np.sin(4*np.pi*f*t)+.12*np.sin(6*np.pi*f*t))*.12
   sig*=np.minimum(t/.008,1)*np.exp(-t*.65)*np.minimum((length/rate-t)/.12,1)*(v/127)
  audio[begin:begin+length]+=sig
 peak=max(1.,float(np.max(np.abs(audio)))/.8);audio=np.clip(audio/peak,-1,1)
 wav=out/(p.stem+'.wav')
 with wave.open(str(wav),'wb')as w:w.setparams((1,2,rate,0,'NONE','not compressed'));w.writeframes((audio*32767).astype('<i2').tobytes())
 subprocess.run(['ffmpeg','-v','error','-y','-i',str(wav),'-c:a','libvorbis','-q:a','3',str(out/(p.stem+'.ogg'))],check=True)
 wav.unlink();print(p.name,len(notes),round(seconds))
