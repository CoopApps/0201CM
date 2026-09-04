# ScreenManager (scrman) module inventory

Range: **cm0102.exe** addresses `0x007e4520` .. `0x007e9480` (39 functions actually observed;
task-brief said "30", but the linear-sweep decomp shows every function in the sweep touches
ScreenManager fields at the known byte offsets — `+0x28`, `+0x302a..0x306c`, `+0x3000..0x3018`,
`+0x3036..0x3054` (per-slot depth table), `+0x3056` (current slot), `+0x3058` (target),
`+0x1326d1` (session sub-object), `+0x261fb0..0x261fd4` (secondary-slot far-region)).

The scrman module in this exe extends *below* the `007e4940` pump: the constructor lives at
`FUN_007e4520` and the destructor at `FUN_007e46a0`, plus two helpers (`007e4750`,
`007e47f0`, `007e4920`). Adjacent fns `007e42f0..007e44b0` are **not** scrman (they belong
to the scout/list module — they touch player-record fields, not ScreenManager fields).

## GDI cross-check

GDI address of the constructor: `sub_007e3f20`. Field byte offsets are **byte-identical** to
cm0102.exe (verified against sub_007e3f20's `mov dword ptr [esi + 0x306c], ebp` etc.).
GDI address of the pump: `sub_007e4340` (== cm0102.exe `FUN_007e4940`). GDI relocates
scrman ~0x600 lower but keeps the same struct layout, so cm0102.exe decomps can be trusted
for field offsets.

## The 39 functions

| Address | Lines | Signature | Role |
|--:|--:|---|---|
| 007e4520 | 79 | `undefined2 * (undefined2 *this)` | **Constructor** — allocs 50KB network buffer via sub-obj at `+0x302a`, inits two session sub-objs at `+0x3070` and `+0x1326d1`, zeros 16-slot table and header. **Ported this commit.** |
| 007e46a0 | 33 | `void (int this)` | **Destructor** — calls `007e4750` (drain all slots), `007e47f0` (send-net-goodbye), destroys sub-objs. Structure-clear portion **ported this commit as `reset()`**. |
| 007e4750 | 38 | `void (int this)` | Drain-all-slots helper: for each of 16 slots, pop each entry through its cleanup vptr (fld_0x28+4) then `007e6e00` (pop-top). |
| 007e47f0 | 44 | `void (undefined2 *this)` | Send-net-goodbye packet flush (op 4) to peer via `+0x302a` buffer. |
| 007e4920 | 8 | `void (undefined4)` | Trampoline to `FUN_007628e0` — resolve peer address. |
| 007e4940 | 688 | `bool (short *this)` | **The pump** (dispatch loop) — for each slot: read screen-cleanup vtable, call top-of-stack render/tick, handle codes -1..-13. Foll-up commit. |
| 007e5bd0 | 230 | `void (int this)` | Slot-cleanup-and-teardown per slot; also frees local player list at `+0x1326d1`. |
| 007e6430 | 42 | `undefined4 (this, ..)` | Overlay-active check / push-if-current-slot helper. |
| 007e6570 | 178 | `undefined4 (this, name, ...5 args)` | **push_screen** — allocates new 0x300-byte record; sets `[0]=name,[1]=arg5,[2]=arg3,[3]=arg6,[4]=arg4`; links into slot list at `+10`, updates `+6` head, `+0xe` current, `+0x14` depth, `+0x16=1`, `+0x22=0xffff`. Follow-up commit. |
| 007e6a20 | 16 | `void (this, ...)` | Deferred-push setter: saves next-push args into `[+0x3000..+0x3010]`. |
| 007e6a70 | 25 | `int (int this)` | Getter: peer id for current slot's top screen. |
| 007e6ab0 | 24 | `bool (int this)` | Predicate: current slot has any entries. |
| 007e6b60 | 27 | `undefined4 (this, slot)` | Get named tag of current top for a given slot. |
| 007e6c30 | 24 | `undefined4 (int this)` | Get slot-owner peer id. |
| 007e6cd0 | 24 | `bool (int this)` | Predicate: current top's screen has "modal" flag (`+0x10 != 0`). |
| 007e6d70 | 15 | `char * (int this)` | Get `+0x7f` (screen name string) of current top. |
| 007e6da0 | 43 | `void (this, char *)` | strcpy into `+0x7f` (rename current top). |
| 007e6e00 | 36 | `void (int this)` | **pop_top** — free bag+entry, decrement depth `+0x14`. |
| 007e6ee0 | 32 | `undefined4 (this, level)` | Walk back N entries from current top. |
| 007e7000 | 41 | `void (this)` | Reset current slot to fresh entry (pop-until-empty + push default). |
| 007e7130 | 83 | `void (this, slot, str, own)` | **Set slot-bag string** at `[+0x14 + slot*8]`; ownership flag at `[+0x18 + slot*8]`. Follow-up. |
| 007e73b0 | 45 | `int (this, code, buf)` | Send small net packet: op-code + payload via `+0x302a`. |
| 007e74a0 | 18 | `void (int this)` | Wrapper: net-flush current buffer. |
| 007e74e0 | 79 | `void (undefined4)` | Big net-send: writes header + N bytes to buffer, calls flush if full. |
| 007e76c0 | 43 | `undefined4 (this, s, code)` | Send screen-switch packet to peer. |
| 007e7790 | 18 | `void (this,arg2,arg3,arg4)` | Trampoline to `007e74e0` with 4 args prepared. |
| 007e7820 | 130 | `void (this, s, code)` | **remote_push** — mirror a `push_screen` to network peer + local deferred setter. |
| 007e7b50 | 29 | `void (this, p)` | Free chained record list starting at `p` (called from pump on `+0x1813` cleanup). |
| 007e7bf0 | 22 | `void (this, slot, ptr, val)` | Set slot's aux `+0x1c` pointer + `+0x1e` short. |
| 007e7c40 | 25 | `undefined4 (this, slot)` | Get `+0x1c` aux pointer for slot. |
| 007e7cb0 | 25 | `undefined4 (this, slot)` | Get `+0x24` aux for slot. |
| 007e7d20 | 26 | `void (this, slot, val)` | Set slot's aux `+0x24`. |
| 007e7d90 | 19 | `undefined4 (this, slot)` | Get slot's depth `[+0x3036 + slot*2]`. |
| 007e7de0 | 24 | `int (int this)` | Get current slot's top entry field `+0x1a`. |
| 007e7e10 | 27 | `short (this, slot)` | Get slot-owner-peer-short. |
| 007e7e60 | 41 | `void (int this)` | Net-op: send `+0x302a`-buffer contents as opcode 6 (screen-close). |
| 007e7fc0 | 52 | `void (this, opcode)` | Net-op-dispatch: writes 1-byte opcode + flushes. |
| 007e8190 | 56 | `void (this, opcode, arg)` | Net-op with 1-word arg. |
| 007e83e0 | 81 | `void (this, opcode, arg32)` | Net-op with 1-dword arg. |
| 007e86e0 | 116 | `void (this, ...)` | Net-op with 3 dwords (transfer-offer packet family). |
| 007e8b70 | 81 | `void (this, opcode, arg32)` | Sibling of `007e83e0` — different opcode class. |
| 007e8e70 | 81 | `void (this, opcode, arg16)` | Net-op with 1 word. |
| 007e9180 | 81 | `void (this, opcode, arg32)` | Sibling of `007e83e0`. |
| 007e9480 | 79 | `void (this, opcode, arg8)` | Net-op with 1 byte. |

## Field-offset dependencies (byte-precise, from GDI sub_007e3f20 + cm0102.exe FUN_007e4520)

**Header (0x00 .. 0x27)** — slot-0 root frame:
- `+0x00` word — root frame screen id
- `+0x02..0x0d` five dwords — screen args
- `+0x0e` dword — cur-entry ptr (redundant with slot table)
- `+0x12` word, `+0x14` word — flags/depth
- `+0x16` dword — **initial 1** (running flag)
- `+0x22` word — **initial 0xffff** (peer id sentinel)
- `+0x24` dword — aux
- `+0x28` dword — points to `esi + 0x3070` (initial screen record in first session pool)

**Slot table (0x0000 .. 0x2FFF)** — 16 × 0x300-byte slot records. Constructor's tight-loop skips this
(subsequent `push_screen` allocates entries lazily). Slot inner-record fields via `+slot*0x300+off`:
`+0x02` dtor-fn, `+0x06` head-entry, `+0x0a` prev-entry, `+0x0e` cur-entry,
`+0x10` modal-flag, `+0x14` depth (word), `+0x16` running flag,
`+0x1a..0x24` net-tag / peer state, `+0x14+slot*8` bag values, `+0x18+slot*8` owns-flag,
`+0x7d` prev-list, `+0x7e` next-list, `+0x7f` name (~256B).

**Header (0x3000 .. 0x306f)** — dispatch state:
- `+0x3000` deferred-push name; `+0x3004,+0x3008,+0x300c,+0x3010` args
- `+0x3014` bag-values ptr; `+0x3018` bag-count (word)
- `+0x301a`, `+0x301e`, `+0x3022` — dword aux (init 0)
- `+0x3026` — dword (init 0)
- `+0x302a` — start of the **network-buffer sub-object** (constructed via `sub_007631d0(this, 50000)`
  in GDI == `FUN_00763590(this, 50000)` in cm0102.exe). Interior: `+0x302a` buf-ptr,
  `+0x302e` write-off, `+0x3032` buf-size.
- `+0x3036 + slot*2` (16 words) — per-slot depth counter; slot-0 init to 1
- `+0x3056` — current slot index (word, init 0)
- `+0x3058` — target slot (word, init 0xffff)
- `+0x305c` — flush-in-progress flag (dword, init 0)
- `+0x3060` — dword (init 0)
- `+0x3068` — pump-active flag (init 0)
- `+0x306c` — network mode dword (init 0)

**Far region (0x3070 .. 0x1326D0)** — first session sub-object, `sub_00548d50(this=&scrmgr+0x3070, 0)`
in GDI == `FUN_00548b40(this, 0)` in cm0102.exe. Opaque this commit.

**Far region (0x1326D1 .. 0x261FD3)** — second session sub-object, `sub_00548d50(this=&scrmgr+0x1326d1, 1)`.
Constructor **also directly clears** individual dwords inside this region:
`+0x261d32`, `+0x261fb6`, `+0x261fba` (word), `+0x261fbc`, `+0x261fc0`, `+0x261fc4`,
`+0x261fc8` (word), `+0x261fca` (word), `+0x261fcc`, `+0x261fd0`.
This is a layering quirk — ScreenManager reaches into the sub-object's storage to reset a small
set of fields (probably local-player-list head/tail/count). Documented but treated opaquely.

**Total instance size**: `0x00262000` bytes (~2.4 MB) — the ScreenManager owns the two large session
pools inline. Constructor `sub_007e3f20` writes at most out to `[esi+0x261fd0]+3 = 0x261fd3`.

## Unported deps for follow-up commits

| Dep | GDI | cm0102.exe | Role |
|---|---|---|---|
| `net_buf_ctor` | `sub_007631d0(this, 50000)` | `FUN_00763590(this, 50000)` | Allocate 50000-byte send/recv network buffer at `+0x302a` |
| `session_sub_ctor` | `sub_00548d50(this, {0,1})` | `FUN_00548b40(this, {0,1})` | Init a session sub-object (called twice at `+0x3070` and `+0x1326d1`) |
| `net_buf_dtor` | `sub_007627f0` | `FUN_007627f0` | Free network buffer |
| `session_sub_dtor` | `sub_00548d70` | `FUN_00548bd0` | Session sub-object dtor (called twice) |
| `net_close` | `sub_00762430` | `FUN_007627f0` | Close socket at `+0x306c` |
| `net_reset` | `sub_00763640` | `FUN_00763640` | Reset network module (destructor tail) |

## What THIS commit does

Ports the **structure-clearing** portion of both constructor and destructor:

- `ScreenManager::new()` — heap-allocates a zeroed `0x262000`-byte backing store, then runs the
  exact write sequence from cm0102.exe `FUN_007e4520` (which mirrors GDI `sub_007e3f20` at the
  byte level). External sub-object ctors (`763590`, `548b40 x2`) are documented as TODO and
  the fields those sub-objects would set remain 0 — safe because the pure-state fields the pump
  reads before any push are all zero-initialized.
- `ScreenManager::reset(&mut self)` — re-runs `new()`'s state writes in-place.

Follow-up commits will port: the network buffer sub-object (commit 2), the session sub-objects
(commit 3), the pump `FUN_007e4940` (commit 4), `push_screen`/`pop`/`peek` (commit 5), the
net-op family `007e73b0..007e9480` (commit 6+).
