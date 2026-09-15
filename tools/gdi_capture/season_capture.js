// Full-season capture agent for cm0102_GDI.exe.
//
// Loads its hook list from season_capture_manifest.json (passed
// in as `manifest` via the Python driver's `create_script(...)`
// runtime). Runs one JSONL sink; every record is tagged with:
//
//   { seq, ms, rng, group, hook, phase, va, ... }
//
// so the replayer (season_replay.py) can partition offline.
//
// Groups (see manifest):
//   A_year_end        C15.1 year-end chain (GDI VAs confirmed)
//   B_fixtures        weekly fixture pipeline (GDI VAs confirmed)
//   C_match           match resolve (DirectDraw VAs — confirm first)
//   D_news_predicate  weekly news cascade (DirectDraw VAs — confirm first)
//   E_finance         finance rollovers (DirectDraw VAs — confirm first)
//   F_boot_state      one-shot boot dump (DirectDraw VAs — confirm first)
//   G_rng_timeline    RNG state sampler on a timer

'use strict';

const MODULE = 'cm0102_GDI.exe';
let SEQ = 0;
const T0 = Date.now();

// The Python driver injects the manifest object here.
// eslint-disable-next-line no-undef
const MANIFEST = typeof manifest !== 'undefined' ? manifest : { groups: {} };

function baseVA() { return Process.getModuleByName(MODULE).base; }

function ptrFromVA(vaHex) {
    // Absolute image VA; convert to runtime pointer via module base.
    // GDI is loaded at its imagebase but ASLR may rebase — always
    // compute (base + (va - imagebase)).
    const va = parseInt(vaHex, 16);
    // cm0102_GDI.exe's imagebase is 0x00400000 (PE default) — we
    // read imagebase from the module for safety.
    const m = Process.getModuleByName(MODULE);
    // Some Frida builds don't expose module.base_address_imagebase;
    // fall back to 0x00400000.
    const IMAGEBASE = 0x00400000;
    return m.base.add(va - IMAGEBASE);
}

function rngState() {
    const rngVA = (MANIFEST.groups.G_rng_timeline &&
                   MANIFEST.groups.G_rng_timeline.settings &&
                   MANIFEST.groups.G_rng_timeline.settings.rng_va) ||
                  '0x00AC2610';
    try { return ptrFromVA(rngVA).readU32(); }
    catch (e) { return null; }
}

function emit(rec) {
    rec.seq = SEQ++;
    rec.ms = Date.now() - T0;
    rec.rng = rngState();
    send(JSON.stringify(rec));
}

// ---------------- Arg decoders ----------------

function readClub(p) {
    if (p.isNull()) return null;
    try {
        return {
            id:        p.readU32(),
            status_37: p.add(0x37).readU8(),
            primary_57:  p.add(0x57).readS32(),
            previous_5b: p.add(0x5b).readS32(),
            tier_64:   p.add(0x64).readU8(),
            stadium_69: p.add(0x69).readU8(),
            news_cf:   p.add(0xcf).readU8(),
        };
    } catch (e) { return { err: String(e) }; }
}

function readStadium(p) {
    if (p.isNull()) return null;
    try {
        return {
            id:      p.readU32(),
            refuse_20: p.add(0x20).readS8(),
            total_3c: p.add(0x3c).readU32(),
            seated_40: p.add(0x40).readU32(),
            peak_44: p.add(0x44).readU32(),
            alt_48: p.add(0x48).readS32(),
        };
    } catch (e) { return { err: String(e) }; }
}

// Club finance snapshot — reads EVERY candidate cash offset so
// the replayer can see which one actually changes across a
// finance call. Settles the C15.1 §26.1 cash-offset dispute
// empirically.
function readClubFinance(p) {
    if (p.isNull()) return null;
    try {
        return {
            id:          p.readU32(),
            cash_at_00_i64: p.add(0x00).readS64().toString(),
            cash_at_65_i32: p.add(0x65).readS32(),
            expense_ytd_23_i64:    p.add(0x23).readS64().toString(),
            expense_life_4b_i64:   p.add(0x4b).readS64().toString(),
            owner_a_2d_i64:        p.add(0x2d).readS64().toString(),
            owner_b_55_i64:        p.add(0x55).readS64().toString(),
        };
    } catch (e) { return { err: String(e) }; }
}

// C11.3 Phase A — schedule_getter needs richer decoding to prove
// the mid-season regen trigger. Read the arg0 raw + signed +
// plausible-pointer competition record + first N bytes of the comp
// buffer so post-processing can identify the competition, and grab
// the return address so we can chase the caller in Ghidra.
function readCompFromPtr(p) {
    // Only try to dereference plausible user-space pointers.
    // GDI runs at IMAGEBASE 0x00400000; heap addresses are much
    // higher; small integers (0..0x10000) are sentinels not ptrs.
    const asInt = p.toInt32();
    if (asInt >= 0 && asInt < 0x10000) return { sentinel: asInt };
    if (asInt === -1)                  return { sentinel: -1 };
    try {
        // Comp id typically at +0x00 (pool record convention).
        // Also dump the first 128 bytes so a decoder can extract
        // any embedded name/nation/three-letter fields.
        const id = p.readU32();
        const bytes = p.readByteArray(128);
        // Hex-encode the byte dump so it survives JSONL.
        const u8 = new Uint8Array(bytes);
        let hex = '';
        for (let i = 0; i < u8.length; i++) {
            hex += (u8[i] < 0x10 ? '0' : '') + u8[i].toString(16);
        }
        return { id, hex128: hex };
    } catch (e) { return { err: String(e) }; }
}

function decodeArgs(shape, args, ctx) {
    switch (shape) {
        case 'club_first':   return { club_pre: readClub(args[0]) };
        case 'club_promo':   return {
            club_pre: readClub(args[0]),
            new_comp_ptr: args[1].toString(),
            news_enable: args[2].toInt32(),
        };
        case 'stadium_exp':  return {
            club_pre: readClub(args[0]),
            desired_seated: args[1].toInt32(),
            desired_total:  args[2].toInt32(),
            news_ctx:       args[3].toInt32(),
            affordability:  args[4].toInt32(),
            stadium_pre: (function () {
                try { return readStadium(args[0].add(0x69).readPointer()); }
                catch (e) { return null; }
            })(),
        };
        case 'news':         return {
            template_id: args[0].toInt32(),
            comp_id:     args[1].toInt32(),
            news_ctx:    args[2].toInt32(),
            mode:        args[3].toInt32(),
        };
        case 'club_finance': return { club_pre: readClubFinance(args[0]) };
        case 'comp_first':   return { comp_ptr: args[0].toString() };
        // C11.3 Phase A.2 fix: sub_0055F540 (and likely the
        // driver) is __thiscall — `this` is in ECX, not on the
        // stack. args[0] is the first STACK arg (`mode`: -1 or 0
        // for the getter). Read ECX from the Interceptor context
        // to recover `this` (the competition instance), then
        // dereference IT for the comp record.
        case 'schedule_getter':
        case 'comp_and_caller': {
            const stackArg0 = args[0];
            const thisPtr = (ctx && ctx.context && ctx.context.ecx)
                ? ctx.context.ecx : null;
            return {
                stack_arg0_raw: stackArg0.toString(),
                stack_arg0_int: stackArg0.toInt32(),
                this_ptr:  thisPtr ? thisPtr.toString() : null,
                this_record: thisPtr ? readCompFromPtr(thisPtr) : null,
                return_addr: (ctx && ctx.returnAddress)
                                ? ctx.returnAddress.toString() : null,
            };
        }
        case 'generic':      return { arg0: args[0].toString() };
        default:             return { arg0: args[0].toString() };
    }
}

// ---------------- Hook installation ----------------

const HOT_PATH_CTX = new WeakMap();
let HOT_PATH_COUNTER = 0;

function installHook(groupName, def) {
    if (!def.enabled) return { skipped: true, reason: 'disabled' };
    if (!def.va) return { skipped: true, reason: 'no_va_gdi_needed', dd: def.dd_va_hint };
    let target;
    try { target = ptrFromVA(def.va); }
    catch (e) { return { skipped: true, reason: 'resolve_error', err: String(e) }; }
    const sampleRate = def.sample_rate || 1;
    let counter = 0;
    Interceptor.attach(target, {
        onEnter(args) {
            counter++;
            const shouldLog = (counter % sampleRate) === 0;
            if (!shouldLog) { this._skip = true; return; }
            this._ctx = { group: groupName, hook: def.name, va: def.va,
                          call_n: counter };
            const decoded = decodeArgs(def.arg_shape, args, this);
            emit(Object.assign({ phase: 'enter' }, this._ctx, decoded));
        },
        onLeave(retval) {
            if (this._skip) return;
            emit(Object.assign({ phase: 'leave', retval: retval.toInt32() },
                               this._ctx));
        },
    });
    return { hooked: true, va: def.va };
}

// C11.3A Path B — one-shot snapshot of the 34-slot season-roll
// scheduler table at DAT_00b4bc70 (DirectDraw VA) / GDI TBD.
// Called at agent load, AFTER the game has already booted and
// filled the table. Emits one JSONL event per slot with its
// count, trigger_day, last_year, and every comp pointer's
// identity (first 128 bytes of the record, hex-encoded, for
// name/vtable extraction post-hoc).
function snapshotSlotTable() {
    // Try both the DirectDraw VA and a couple of GDI candidates.
    // The DirectDraw VA is documented in the decompile; the GDI
    // delta for the b4____ region isn't in memory but likely
    // shares the ac____ -0xB0 delta. Try all three candidates
    // and log which one produced plausible slot data (count in
    // [0..100], trigger_day in [0..366]).
    const candidates = [
        { name: 'DD_0x00B4BC70',       va: '0x00B4BC70' },
        { name: 'GDI_minus0xB0_0x00B4BBC0', va: '0x00B4BBC0' },
        { name: 'GDI_minus0xB8_0x00B4BBB8', va: '0x00B4BBB8' },
    ];
    const RECORD_SIZE   = 0x48;    // per-slot stride
    const N_SLOTS       = 34;
    const OFF_COUNT     = 0x0c;
    const OFF_POOL_PTR  = 0x10;
    const OFF_TRIGGER   = 0x15;
    const OFF_LAST_YEAR = 0x35;

    for (const c of candidates) {
        let base;
        try { base = ptrFromVA(c.va); }
        catch (e) {
            emit({ hook: '__slot_table_probe__', candidate: c.name,
                   error: String(e) });
            continue;
        }
        // Read one probe slot first — if trigger_day + count look
        // plausible we log the whole table under this candidate.
        let looksReal = false;
        try {
            const s0 = base;
            const count0     = s0.add(OFF_COUNT).readS32();
            const trigger0   = s0.add(OFF_TRIGGER).readU8();
            const lastYear0  = s0.add(OFF_LAST_YEAR).readU16();
            looksReal = (count0 >= 0 && count0 < 100
                         && trigger0 <= 200 && lastYear0 < 3000);
            emit({ hook: '__slot_table_probe__',
                   candidate: c.name, va: c.va,
                   slot0_count: count0, slot0_trigger: trigger0,
                   slot0_last_year: lastYear0,
                   looks_real: looksReal });
        } catch (e) {
            emit({ hook: '__slot_table_probe__', candidate: c.name,
                   error: String(e) });
            continue;
        }
        if (!looksReal) continue;
        // Dump all 34 slots.
        for (let i = 0; i < N_SLOTS; i++) {
            const slotBase = base.add(i * RECORD_SIZE);
            let count = 0, trigger = 0, lastYear = 0, poolPtrStr = '?';
            const comps = [];
            try {
                count      = slotBase.add(OFF_COUNT).readS32();
                trigger    = slotBase.add(OFF_TRIGGER).readU8();
                lastYear   = slotBase.add(OFF_LAST_YEAR).readU16();
                const poolPtr = slotBase.add(OFF_POOL_PTR).readPointer();
                poolPtrStr = poolPtr.toString();
                if (count > 0 && count < 100 && !poolPtr.isNull()) {
                    for (let j = 0; j < count; j++) {
                        try {
                            const compPtr = poolPtr.add(j * 4).readPointer();
                            if (compPtr.isNull()) {
                                comps.push({ idx: j, comp_ptr: null });
                                continue;
                            }
                            // Dump 128 bytes so post-processing
                            // can extract vtable ptr (+0x00),
                            // year (+0x40), name (+0x54).
                            const bytes = compPtr.readByteArray(128);
                            const u8 = new Uint8Array(bytes);
                            let hex = '';
                            for (let k = 0; k < u8.length; k++) {
                                hex += (u8[k] < 0x10 ? '0' : '')
                                       + u8[k].toString(16);
                            }
                            comps.push({
                                idx: j,
                                comp_ptr: compPtr.toString(),
                                hex128: hex,
                            });
                        } catch (e) {
                            comps.push({ idx: j, err: String(e) });
                            break;
                        }
                    }
                }
            } catch (e) {
                emit({ hook: '__slot_table_entry__',
                       candidate: c.name, slot: i,
                       error: String(e) });
                continue;
            }
            emit({
                hook: '__slot_table_entry__',
                candidate: c.name,
                slot: i,
                count, trigger_day: trigger,
                last_processed_year: lastYear,
                pool_ptr: poolPtrStr,
                comps,
            });
        }
        // Only dump for the first-plausible candidate. Break so
        // we don't emit 3x34 = 102 entries for wrong candidates.
        return c.name;
    }
    emit({ hook: '__slot_table_snapshot_failed__',
           reason: 'no candidate VA produced plausible slot 0' });
    return null;
}

function main() {
    const installReport = { groups: {} };
    for (const [gname, gdef] of Object.entries(MANIFEST.groups || {})) {
        if (!gdef.hooks) continue;
        installReport.groups[gname] = [];
        for (const h of gdef.hooks) {
            const r = installHook(gname, h);
            installReport.groups[gname].push({ name: h.name, ...r });
        }
    }
    // Group G — RNG timeline
    const G = MANIFEST.groups.G_rng_timeline;
    if (G && G.settings && G.settings.sample_period_ms > 0) {
        setInterval(() => {
            emit({ group: 'G_rng_timeline', hook: 'rng_sample', phase: 'tick' });
        }, G.settings.sample_period_ms);
    }
    emit({ hook: '__init__', install_report: installReport });

    // C11.3A Path B — one-shot snapshot of DAT_00b4bc70 (34-slot
    // season-roll scheduler table). Runs AFTER the game has
    // booted (Frida attach happens post-boot) so the table is
    // already populated. Emits one JSONL event per slot with
    // count/trigger_day/last_year/pool_ptr + each pooled comp's
    // 128-byte hex header (for post-hoc name / vtable / year
    // extraction). Cheap: 34 slots × ~15 comps avg × 128 bytes.
    const snapshotResult = snapshotSlotTable();
    emit({ hook: '__slot_table_snapshot_done__',
           winning_candidate: snapshotResult });
}

main();
