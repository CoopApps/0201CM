import os, sys, struct, random, re
sys.path.insert(0, os.path.join("D:/cm0102-rs","tools","cm-lift"))
from cm_lift.util import load_pe
from cm_lift.emulate import Emulator
import unicorn.x86_const as X
from unicorn import UC_HOOK_CODE
emu=Emulator(pe=load_pe(r"D:\cm0102\cm0102.exe")); uc=emu.uc; emu.setup_seh()
post=[int(m.group(1),16)+5 for m in
      (re.match(r"(0x[0-9a-f]+) call     0x9346d0",l) for l in
       open("D:/temp/claude/D--cm0102-rs/706b93f9-1a45-4378-b3fd-8d95102cc6ea/scratchpad/dis_682420.txt")) if m]
postset=set(post)
traj=[]
uc.hook_add(UC_HOOK_CODE, lambda u,a,s,x: traj.append((lambda v: v-0x100000000 if v>=0x80000000 else v)(u.reg_read(X.UC_X86_REG_EAX))) if a in postset else None)
def fr(e,eax):
    esp=e.uc.reg_read(X.UC_X86_REG_ESP); ret=struct.unpack("<I",e.uc.mem_read(esp,4))[0]
    e.uc.reg_write(X.UC_X86_REG_ESP,esp+4); e.uc.reg_write(X.UC_X86_REG_EAX,eax&0xffffffff); e.uc.reg_write(X.UC_X86_REG_EIP,ret)
BASE={"v":3000}
emu.hook_call(0x0052a330, lambda e,a,s: fr(e,BASE["v"]&0xffff))
for va in (0x00531370,0x005313b0,0x005313f0,0x00531420,0x00533cf0): emu.hook_call(va, lambda e,a,s: fr(e,0))
uc.mem_write(0x00acd56c, struct.pack("<i",0x40000000))

def ftol(x): return int(x) if x>=0 else -int(-x)  # trunc toward zero
def rust_attr_traj(base, rep, sb, pa):
    L=float(base); out=[]
    for off,w in [(0x11,1/1200),(0x10,1/1200),(0x17,1/500),(0x18,1/500),(0x19,1/500),(0x1b,1/500)]:
        L=ftol(L*(1+sb[off]*w)); out.append(L)
    w21 = (1/125) if rep>0x1c52 else ((1/250) if rep>0x1676 else (1/500))
    L=ftol(L*(1+sb[0x21]*w21)); out.append(L)
    for off in (0x59,0x58,0x5a,0x5b):
        L=ftol(L*(1+(pa[off]-10)*(1/200))); out.append(L)
    return out

rng=random.Random(11); mism=0; total=0; first=None
reps=[5000,5750,5751,6000,7250,7251,8000]
for trial in range(240):
    base=rng.choice([1,250,1000,3000,9000,-1,-4000])
    rep=rng.choice(reps)
    sb={o: rng.randint(-20,20) for o in (0x10,0x11,0x17,0x18,0x19,0x1b,0x21)}
    pa={o: rng.choice([0,9,10,11,20]) for o in (0x58,0x59,0x5a,0x5b)}
    club=emu.malloc(0x300)
    for o,t,v in [(0x53,"I",0),(0x80,"h",rep),(0xbf,"I",0)]: emu.write_field(club,o,t,v)
    st=emu.malloc(0x40)
    for o in range(0x40): emu.write_field(st,o,"B",0)
    for o,v in sb.items(): emu.write_field(st,o,"b",v)
    person=emu.malloc(0x100)
    for o,t,v in [(0,"I",5),(0x18,"b",50),(0x3d,"b",5),(0x69,"I",st)]: emu.write_field(person,o,t,v)
    for o,v in pa.items(): emu.write_field(person,o,"b",v)
    BASE["v"]=base&0xffff
    traj.clear()
    emu.call(0x00682420, person, club, 0, 5)
    exe11=traj[:11]
    base_s=struct.unpack("<h",struct.pack("<H",base&0xffff))[0]
    r11=rust_attr_traj(base_s, rep, sb, pa)
    total+=1
    if exe11!=r11:
        mism+=1
        if first is None: first=(base_s,rep,sb,pa,exe11,r11)
print(f"cases={total} attr-trajectory mismatches={mism}")
if first:
    b,rep,sb,pa,e,r=first
    print("first mismatch: base",b,"rep",rep)
    print("  sb",sb); print("  pa",pa)
    print("  exe ",e); print("  rust",r)
    for i,(x,y) in enumerate(zip(e,r)):
        if x!=y: print(f"  first diverge at index {i}: exe={x} rust={y}"); break
