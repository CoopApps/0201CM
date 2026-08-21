// Dump the game's exact DirectDraw framebuffer by Lock()ing the render surface.
// The draw primitives write to the surface at global DAT_00ad6bd4 (they Lock it at
// vtable+0x64, Unlock at +0x80). We lock it read-only and read the RGB565 pixels.
//
// DDSURFACEDESC offsets: dwSize@0, lPitch@0x10, lpSurface@0x24. Size = 0x6c.
// IDirectDrawSurface vtable: Lock @ +0x64, Unlock @ +0x80.

const SURF_GLOBAL = ptr('0x00ad6bd4');

rpc.exports = {
  grab: function () {
    const surf = SURF_GLOBAL.readPointer();
    if (surf.isNull()) return { err: 'surface null (no screen rendered yet)' };
    const vtbl = surf.readPointer();
    const Lock = new NativeFunction(vtbl.add(0x64).readPointer(), 'int32',
      ['pointer', 'pointer', 'pointer', 'uint32', 'pointer']);
    const Unlock = new NativeFunction(vtbl.add(0x80).readPointer(), 'int32',
      ['pointer', 'pointer']);

    const desc = Memory.alloc(0x6c);
    desc.writeU32(0x6c);                     // dwSize
    const DDLOCK_WAIT = 0x1;
    const hr = Lock(surf, ptr(0), desc, DDLOCK_WAIT, ptr(0));
    if (hr !== 0) return { err: 'Lock hr=0x' + (hr >>> 0).toString(16) };

    const pitch = desc.add(0x10).readInt();
    const lp = desc.add(0x24).readPointer();
    const H = 600;
    const bytes = lp.readByteArray(pitch * H);   // whole surface (pitch may exceed 800*2)
    Unlock(surf, ptr(0));
    send({ pitch: pitch, w: 800, h: H }, bytes);  // raw pixel bytes as message payload
    return { ok: true, pitch: pitch };
  },
};
