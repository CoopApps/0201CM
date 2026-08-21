// CM0102 Frida harness — hooks the running game to capture GROUND TRUTH.
// Addresses absolute (image base 0x400000, no ASLR on the 2001 build).
// Categories are toggled from the Python runner via rpc.exports.setConfig.
//
//   draw   : the GUI draw stream (panels/text/colors/images) -> pixel-exact replay
//   font   : Win32 CreateFontA -> the traditional (TrueType) fonts
//   view   : screen view-model slot bindings -> maps data to screens
//   match  : match-engine I/O -> ground truth for building the sim (teams -> score)
//   rng    : the match RNG stream -> verify cm-rng is bit-exact

let CFG = { draw: true, font: true, view: true, match: false, rng: false };
const CAP = 20000;
let n = {};
function bump(k) { n[k] = (n[k] || 0) + 1; return n[k] <= CAP; }
function s32(c, i) { return c.esp.add(4 * (i + 1)).readS32(); }
function u32(c, i) { return c.esp.add(4 * (i + 1)).readU32(); }
function cstr(p) { try { const q = ptr(p); return q.isNull() ? '' : q.readCString(); } catch (e) { return ''; } }

function hookDraw() {
  Interceptor.attach(ptr('0x005ce4f0'), { onEnter() { if (bump('color')) // color pack
    send({ t: 'color', r: u32(this.context, 0) & 0xff, g: u32(this.context, 1) & 0xff, b: u32(this.context, 2) & 0xff }); }});
  Interceptor.attach(ptr('0x005cf8e0'), { onEnter() { if (bump('panel')) // panel fill
    send({ t: 'panel', l: s32(this.context, 0), top: s32(this.context, 1), r: s32(this.context, 2),
           b: s32(this.context, 3), flags: u32(this.context, 4), color: u32(this.context, 5) & 0xffff }); }});
  Interceptor.attach(ptr('0x005d0870'), { onEnter() { if (bump('text')) // text box
    send({ t: 'text', l: s32(this.context, 0), top: s32(this.context, 1), r: s32(this.context, 2),
           b: s32(this.context, 3), flags: u32(this.context, 4), font: u32(this.context, 5),
           color: u32(this.context, 6) & 0xffff, s: cstr(u32(this.context, 7)) }); }});
  Interceptor.attach(ptr('0x005cddc0'), { onEnter() { if (bump('imgload')) // load_image_buffer(filename,...)
    send({ t: 'imgload', file: cstr(u32(this.context, 0)) }); }});
  Interceptor.attach(ptr('0x005cdcc0'), { onEnter() { if (bump('imgblt')) // blit background at 0,0
    send({ t: 'imgblt', a0: s32(this.context, 0), a1: s32(this.context, 1), buf: u32(this.context, 2) }); }});
}

function hookFont() {
  let gdi = null;
  try { gdi = Process.getModuleByName('gdi32.dll'); } catch (e) { try { gdi = Process.findModuleByName('gdi32.dll'); } catch (e2) {} }
  if (!gdi) return;
  ['CreateFontA', 'CreateFontW'].forEach(function (fn) {
    let p = null;
    try { p = gdi.findExportByName(fn); } catch (e) {}
    if (!p) return;
    Interceptor.attach(p, { onEnter(a) {
      let face = '';
      try { face = fn === 'CreateFontW' ? a[13].readUtf16String() : a[13].readCString(); } catch (e) {}
      send({ t: 'createfont', height: a[0].toInt32(), weight: a[4].toInt32(), italic: a[5].toInt32() & 0xff, face: face });
    }});
  });
}

function hookView() {
  Interceptor.attach(ptr('0x007e7130'), { onEnter() { if (bump('slotset')) // view-model setter(this,index,str,len)
    send({ t: 'slotset', index: s32(this.context, 0), str: cstr(u32(this.context, 1)) }); }});
}

function hookMatch() {
  // match_phase_final_score_controller: matchstate score bytes +0xf5bc/+0xf5f2, fixture at +0x4792
  Interceptor.attach(ptr('0x006a4020'), { onEnter() {
    if (!bump('matchresult')) return;
    try {
      const ms = this.context.ecx;
      const home = ms.add(0xf5bc).readU8();
      const away = ms.add(0xf5f2).readU8();
      const homeTeam = ms.add(0x1d6).readU16();
      const awayTeam = ms.add(0x1d8).readU16();
      send({ t: 'matchresult', homeTeam: homeTeam, awayTeam: awayTeam, home: home, away: away });
    } catch (e) { send({ t: 'matchresult', err: '' + e }); }
  }});
  Interceptor.attach(ptr('0x0069d950'), { onEnter() { if (bump('matchsetup')) // match_setup(matchstate, fixture,...)
    send({ t: 'matchsetup', fixture: u32(this.context, 0) }); }});
}

function hookRng() {
  Interceptor.attach(ptr('0x008fc4f0'), { onLeave(ret) { if (bump('rng')) // match_random -> value
    send({ t: 'rng', val: ret.toInt32() }); }});
}

function safe(name, fn) { try { fn(); } catch (e) { send({ t: 'hookerr', where: name, err: '' + e }); } }
function install() {
  if (CFG.draw) safe('draw', hookDraw);
  if (CFG.font) safe('font', hookFont);
  if (CFG.view) safe('view', hookView);
  if (CFG.match) safe('match', hookMatch);
  if (CFG.rng) safe('rng', hookRng);
  send({ t: 'ready', cfg: CFG });
}

// Dump the exact DirectDraw framebuffer (Lock the render surface DAT_00ad6bd4).
function grabFramebuffer() {
  const surf = ptr('0x00ad6bd4').readPointer();
  if (surf.isNull()) return { err: 'surface null' };
  const vtbl = surf.readPointer();
  const Lock = new NativeFunction(vtbl.add(0x64).readPointer(), 'int32', ['pointer', 'pointer', 'pointer', 'uint32', 'pointer']);
  const Unlock = new NativeFunction(vtbl.add(0x80).readPointer(), 'int32', ['pointer', 'pointer']);
  const desc = Memory.alloc(0x6c); desc.writeU32(0x6c);
  const hr = Lock(surf, ptr(0), desc, 0x1, ptr(0));
  if (hr !== 0) return { err: 'Lock hr=0x' + (hr >>> 0).toString(16) };
  const pitch = desc.add(0x10).readInt();
  const lp = desc.add(0x24).readPointer();
  const bytes = lp.readByteArray(pitch * 600);
  Unlock(surf, ptr(0));
  send({ t: 'framebuffer', pitch: pitch, w: 800, h: 600 }, bytes);
  return { ok: true };
}

rpc.exports = {
  start: function (cfg) { if (cfg) CFG = Object.assign(CFG, cfg); install(); },
  grab: grabFramebuffer,
};
