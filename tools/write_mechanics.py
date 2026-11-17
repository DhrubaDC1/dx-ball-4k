from pathlib import Path
import json
out=Path(__file__).resolve().parents[1]
v=[]
def add(n,g,x,u='',src=None):
 d={'name':n,'group':g,'value':x}
 if u:d['unit']=u
 if src:d['source']=src
 else:d['estimated']=True
 v.append(d)
add('logical width','arena',640,'px','0x40f7a0')
add('logical height','arena',480,'px','0x40f7a0')
add('simulation rate','timing',60,'Hz','README.TXT Game concept; 0x40f8b0 frame update')
for n,x in [('left wall',20),('right wall',619),('top wall',0),('bottom loss line',479)]:add(n,'arena',x,'px','0x410770')
for n,x in [('columns',20),('rows',20),('brick width',30),('brick height',15),('brick origin x',20),('brick origin y',50)]:add(n,'boards',x,'px' if 'width' in n or 'height' in n or 'origin' in n else '', '0x4119f0; 0x4116c0')
add('board count','boards',50,'boards','DEFAULT.BDS = 20000 bytes; 0x40cc30; 0x415b30')
add('starting lives','lives',3,'paddles','0x40f4c0')
add('maximum lives','lives',20,'paddles','0x4158d0')
add('life loss','lives','Lose one paddle only when all balls are lost; reset bonuses and serve on the same board','','README.TXT; 0x410770; 0x415bb0; 0x415df0')
add('paddle y','paddle',450,'px','0x412df0')
add('paddle width','paddle',73,'px','MBALL2.SBK sprite 68; 0x415c40')
add('paddle rendered height','paddle',14,'px','MBALL2.SBK sprite 68')
add('paddle collision height','paddle',7,'px','0x410770')
add('paddle movement','controls','Mouse x follows immediately; center constrained to [21+width/2, 618-width/2]','','0x412df0')
add('launch and shoot','controls','Left mouse press (not hold)','','README.TXT; 0x40f8b0')
add('pause','controls','P or right mouse',src='0x4158??: inferred from input; estimated after examining 0x40da70 and 0x40f8b0')
# The pause dispatch was not yet established, so label this choice explicitly.
v[-1].pop('source');v[-1]['estimated']=True
add('extra controls','controls','Space launches, arrows/A/D move, Escape menu, F1 help, F2 editor, F11 fullscreen, M music, S sounds')
add('keyboard paddle speed','paddle',480,'px/s')
for n,x in [('normal ball size',9),('small ball size',5)]:add(n,'ball',x,'px','MBALL2.SBK sprites 1 and 55')
for n,x in [('initial speed',5),('slow speed',4),('maximum speed',9),('fast increment',2),('speed increase after bounce counter exceeds',40),('stuck counter threshold',300)]:add(n,'ball',x,'px/frame' if 'speed' in n and 'increase' not in n else '', '0x4105d0; 0x410770; 0x40f8b0')
add('rebound zones','ball','0.07,0.20,0.35,0.50,0.65,0.80,0.93','','0x4200b8 (doubles); 0x411410')
add('rebound angles','ball','150,135,120,105,75,60,45,30','degrees','0x411410')
add('rebound formula','ball','vx=trunc(trunc(cos(angle)*1024)/1024*speed*1.2), vy=trunc(-trunc(sin(angle)*1024)/1024*speed)','','0x402250; 0x402400; 0x402490; 0x411410 instructions; 0x4200f0=1.2')
add('ball normal acceleration','ball',False,src='0x410770; velocity changes only on rebounds and bonuses')
add('ball collision scoring','score','2 * current ball speed per successful brick hit','','0x4116c0')
add('bullet and chain scoring','score',4,'points/hit','0x413170; 0x40f8b0')
add('bonus catch scoring','score',100,'points','0x413e20')
add('score wrap','score',999999999,'points','0x415880')
add('indestructible brick','bricks',2,src='0x411f40; excluded from completion in 0x415a90')
add('durable brick stages','bricks','3 -> 4 -> 5 -> empty (three hits); 7 -> 6 -> empty (invisible first); 21 -> 2 (one counted hit)','','0x411f40')
add('explosive brick','bricks',8,src='0x411f40; 0x412730 spreads to eight neighbours and chains')
add('thru brick','bonuses','Destroys metal and durable bricks without bouncing; fireball turns struck cells into explosions','','0x4116c0; 0x411f40')
add('bonus chance','bonuses',0.2,'per destroyed brick','0x4138d0 random(10)<2, only if no bonus is active')
add('bonus gravity interval','bonuses',21,'frames','0x413e20 increases vy by 1 after counter >20')
add('bonus launch velocity','bonuses','vx = impacting ball vx, vy = impacting ball vy/2; reflected at sides and top','','0x40f8b0; 0x412ba0; 0x4138d0; 0x413e20')
add('bonus selection','bonuses','random(19); extra life and warp retained only on random(5)==1 otherwise kill; index14 becomes expand','','0x4138d0')
add('bonus types','bonuses','Extra life, warp, zap, slow, multiply exploding, thru brick, set off exploding, fireball, shooting, grab, expand, shrink, split, kill, fast, super shrink, falling bricks, shrink ball','','0x413e20; MAINMENU.PCX')
add('paddle expand limit','bonuses',292,'px','0x413e20: base width*4')
add('paddle minimum width','bonuses',36,'px','0x413e20: base width/2 integer')
add('bullet speed','shooting',8,'px/frame','0x413170')
add('bullets per press','shooting',2,'bullets','0x4132c0')
add('maximum bullets in flight','shooting',6,'bullets','0x40f8b0')
add('split','bonuses','Each current ball produces one mirrored horizontal-velocity copy','','0x414df0')
add('falling bricks','bonuses','On each paddle rebound, shift bricks one cell down where the next cell is empty; bottom row stays','','0x415670')
add('last brick lightning','boards','One remaining counted brick is automatically destroyed after 40000+random(20000) milliseconds','','0x415f40')
add('score position','hud','Large original bitmap numbers at upper left (30, approximately 0); reserve paddles at (598,2) stepping left 22 pixels','','0x4158d0; 0x404ee0')
add('board transition pause','timing',1.2,'seconds')
add('life loss pause','timing',1.2,'seconds')
add('explosion chain delay','timing',4,'frames','0x412730 (animation decrements to frame5, interval2); implementation uses equivalent short delayed chain')
add('particle approximation','effects','Original explosion frames; simplified debris paths and palette cycling')
add('music instruments','audio','Original MIDI events with synthesized portable instruments; default music off during play','','README.TXT mentions optional in-game music; original MDS files')
add('modes','modes','Single-player 50-board campaign; bonus reference; high scores; local board editor','','README.TXT; 0x40c740; 0x40e430')
(out/'mechanics.json').write_text(json.dumps({'version':1,'values':v},indent=2)+'\n')
text='# Mechanics and evidence\n\nRecovered from the original DXBALL.EXE by Ghidra through Craft Anything. Addresses refer to that executable; these are static observations, not runtime parity measurements. No original source or decompiled implementation is included.\n\n'
text+='| Value | Group | Setting | Evidence |\n|---|---|---|---|\n'
for x in v:text+=f"| {x['name']} | {x['group']} | {x['value']} {x.get('unit','')} | {x.get('source','estimated')} |\n"
text+='\n## Remaining limits\n\nNo running original or reference gameplay screenshots are available. Exact palette animation, transition timings, pixel sampling in oblique collisions and MIDI instrument timbres remain unverified. All boards and sprites are the readable original data. Keyboard alternatives, neutral title, editor convenience UI and modern window resizing are additions. The optional music preserves note events; instrument rendering differs. Native Windows and Mac builds need their own runners and device checks.\n'
(out/'MECHANICS.md').write_text(text)
