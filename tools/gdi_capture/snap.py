"""One-shot GDI framebuffer snapshot.

Usage:
    py snap.py <output_name>

Attaches to cm0102_GDI.exe, dumps its 800x600 RGB555 backbuffer via the
project's documented globals (BUF=0x00ad6b1c, PITCH=0x00acdeb8), and
writes:
    scratchpad/prelaunch/<output_name>.png       — decoded RGB image
    scratchpad/prelaunch/<output_name>.rgb555.bin — raw framebuffer bytes

Runs for ~2s, which is enough for the hook to fire and a full frame to
be read. Made for driving mid-conversation captures without keeping a
long-running Frida session open.
"""
import frida, sys, time
from pathlib import Path

def main():
    if len(sys.argv) < 2:
        sys.exit("usage: py snap.py <name>")
    name = sys.argv[1]
    dev = frida.get_local_device()
    pid = None
    for p in dev.enumerate_processes():
        if p.name.lower() == 'cm0102_gdi.exe':
            pid = p.pid; break
    if pid is None:
        sys.exit("cm0102_GDI.exe not running")

    session = frida.attach(pid)
    script = session.create_script(r"""
    const bufPtr = ptr('0x00ad6b1c').readPointer();
    const pitch  = ptr('0x00acdeb8').readU16();
    const w      = ptr('0x00ad6b40').readS32();
    const h      = ptr('0x00ad6b08').readS32();
    const bytes  = bufPtr.readByteArray(pitch * h * 2);
    send({op:'fb', pitch, w, h}, bytes);
    """)
    r = {'bytes': None, 'meta': None}
    def on_msg(m, data):
        if m['type'] == 'send':
            r['bytes'] = data; r['meta'] = m['payload']
    script.on('message', on_msg)
    script.load()
    for _ in range(20):
        if r['bytes']: break
        time.sleep(0.1)
    session.detach()
    if r['bytes'] is None:
        sys.exit("no snapshot in 2s")

    out_dir = Path('scratchpad/prelaunch'); out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / f'{name}.rgb555.bin').write_bytes(r['bytes'])
    from PIL import Image
    m = r['meta']; b = r['bytes']
    img = Image.new('RGB', (m['w'], m['h']))
    data = []
    for y in range(m['h']):
        off = y * m['pitch'] * 2
        for x in range(m['w']):
            v = b[off + x*2] | (b[off + x*2 + 1] << 8)
            r5 = (v >> 10) & 0x1f; g5 = (v >> 5) & 0x1f; b5 = v & 0x1f
            data.append(((r5<<3)|(r5>>2), (g5<<3)|(g5>>2), (b5<<3)|(b5>>2)))
    img.putdata(data)
    out = out_dir / f'{name}.png'
    img.save(out)
    print(f'[+] wrote {out}')

if __name__ == '__main__':
    main()
