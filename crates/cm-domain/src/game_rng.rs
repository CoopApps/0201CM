//! Byte-exact port of FUN_008fc4f0 — the game's canonical `rand_mod`.
//!
//! Source: cm0102.exe Ghidra decompile at 0x008fc4f0 (79-instr asm from
//! `newcarve/functions/00004-PE_section_.text/06810_sub_008fc4f0.asm`),
//! cross-checked epilogue is `ret` (cdecl, single `int` arg).
//!
//! Seed pool is dumped verbatim from `cm0102_GDI.exe` at VA
//! `0x00a8df38 .. 0x00abfc14` (203_996 bytes / 50_999 int32s), plus 4
//! extra sentinel bytes so a cursor sitting on the last valid int can
//! still be dereferenced without going out of bounds.
//!
//! Wrap-region SHA256 (203_996 bytes, the actual pool):
//!     `e7420bf7d7d0b9f7a29591551c08d28244e9b332b7fd42c40c957c36f10459fc`
//! Dumped file SHA256 (204_000 bytes, pool + 4 sentinel bytes):
//!     `dd03dfee2152974703de72d441b8360a35d45b15de7afa47354ae897e8213809`
//!
//! Reseed jitter comes from FUN_00935a94 — the MSVC LCG rand() variant:
//!     seed = seed * 0x343fd + 0x269ec3; return (seed >> 16) & 0x7fff.
//!
//! In follow-up commits `cm-rng` usage in cm-domain gets replaced by
//! `GameRng` to unlock deterministic replay.

/// The shipped random pool: 203_996 payload bytes + 4 tail bytes so
/// `POOL[POOL_WRAP-4 .. POOL_WRAP]` is always a valid read.
pub const POOL: &[u8] = include_bytes!("../assets/game_rng_pool.bin");

/// Byte offset of the sentinel (`DAT_00abfc14`) relative to the pool
/// base (`DAT_00a8df38`).  Cursor values strictly greater than this
/// trigger the wrap-and-reseed path.  50_999 * 4 = 203_996.
pub const POOL_WRAP: u32 = 203_996;

/// Byte-exact port of FUN_008fc4f0.  Deterministic given the initial
/// state established by `GameRng::new` (which mimics FUN_008fc5d0).
pub struct GameRng {
    /// Byte offset into POOL — `DAT_00dc7238` in the exe, tracked as
    /// (cursor_ptr - base) so it fits in `u32` and the pool is
    /// relocatable.
    cursor: u32,
    /// Reseed jitter — `DAT_00dc7234`, updated on every pool wrap.
    jitter: u32,
    /// MSVC srand state — `DAT_00ac26c0`, used by FUN_00935a94.
    lcg_state: u32,
}

impl GameRng {
    /// Mimics FUN_008fc5d0 (the game's `srand`-equivalent).  Takes a
    /// user seed (equivalent to `param_1` there; if 0 the game calls
    /// `time(0)`, which we don't do — callers pass an explicit seed).
    ///
    /// The setup:
    ///   srand(seed);                                    ; 008fc5d?
    ///   cursor = base + (rand() % 51000) * sizeof(int); ; 008fc5??
    ///   sentinel = &DAT_00abfc14;                       ;
    ///   jitter   = rand() % 0xffff;                     ;
    ///   if (cursor < base || cursor > sentinel)         ; guard
    ///       cursor = base;                              ;
    pub fn new(seed: u32) -> Self {
        let mut rng = GameRng { cursor: 0, jitter: 0, lcg_state: seed };
        let r1 = rng.msvc_rand();
        // `int*` pointer arith: base + N ints = +4*N bytes.
        let offset_ints = (r1 as u32) % 51_000;
        let mut cursor_bytes = offset_ints.wrapping_mul(4);
        let r2 = rng.msvc_rand();
        rng.jitter = (r2 as u32) & 0xffff;
        // The exe's guard is (cursor < base || cursor > sentinel).
        // The '<' can never fire here (offset is a u32 addition), and
        // '>' can only fire if offset_ints == 50_999 -> byte offset ==
        // POOL_WRAP.  Match the exe: reset to base in that case.
        if cursor_bytes > POOL_WRAP {
            cursor_bytes = 0;
        }
        rng.cursor = cursor_bytes;
        rng
    }

    /// FUN_00935a94: `seed = seed*0x343fd + 0x269ec3; return (seed >> 16) & 0x7fff`.
    fn msvc_rand(&mut self) -> u32 {
        self.lcg_state = self.lcg_state.wrapping_mul(0x343fd).wrapping_add(0x269ec3); // 00935a94
        (self.lcg_state >> 16) & 0x7fff                                                // 00935a9c
    }

    /// Byte-exact port of FUN_008fc4f0.  Returns a value in the range
    /// specified by the exe: for `n > 0` in `[0, n)`; for `n == 0`
    /// returns 0; for `n < 0` follows the exe's negated-remainder path.
    pub fn rand_mod(&mut self, n: i32) -> i32 {
        // 008fc4f0  push ecx / push esi / mov esi,[esp+c] / test esi,esi
        if n == 0 {
            return 0; // 008fc4fa xor eax,eax ; ret
        }

        // 008fc4ff  mov  ecx, [DAT_00dc7238]      ; ecx = cursor
        // 008fc505  mov  eax, [DAT_00dc7a70]      ; eax = sentinel
        // 008fc50a  add  ecx, 4                   ; cursor += 4 bytes (1 int)
        // 008fc50d  cmp  ecx, eax
        // 008fc50f  mov  [DAT_00dc7238], ecx      ; store bumped cursor
        // 008fc515  jbe  0x8fc538                 ; if cursor <= sentinel, skip reseed
        self.cursor = self.cursor.wrapping_add(4);
        let jitter_word = if self.cursor > POOL_WRAP {
            // 008fc517  mov  [DAT_00dc7238], 0xa8df38 ; cursor = base
            // 008fc521  call 0x935a94                 ; jitter = rand() & 0xffff
            // 008fc52c  and  eax, 0xffff
            // 008fc531  mov  [DAT_00dc7234], eax
            self.cursor = 0;
            let r = self.msvc_rand();
            self.jitter = r & 0xffff;
            self.jitter
        } else {
            // 008fc538  mov eax, [DAT_00dc7234]
            self.jitter
        };

        // Read *cursor as int32 (little-endian).  The pool base was
        // bumped by 4 above so we're guaranteed cursor >= 4 unless we
        // reset — in which case cursor == 0 and we still read the
        // first int.  The +4 sentinel bytes in POOL ensure the highest
        // possible cursor (POOL_WRAP) can safely load 4 bytes.
        let cursor = self.cursor as usize;
        let pool_int = i32::from_le_bytes([
            POOL[cursor], POOL[cursor + 1], POOL[cursor + 2], POOL[cursor + 3],
        ]);

        // 008fc53d  cmp esi, 0xffff / jg  large   ; signed compare
        // 008fc545  cmp esi, 0xffff0001 / jl large; -0xffff bound
        if n <= 0xffff && n >= -0xffff {
            // 008fc54d  add  eax, [ecx]           ; eax = jitter + *cursor
            // 008fc54f  cdq
            // 008fc550  idiv esi                  ; edx = signed rem, eax = quot
            // 008fc552  test esi, esi
            // 008fc554  jg   0x8fc558
            // 008fc556  neg  edx                  ; if n <= 0, negate rem
            // 008fc558  mov  eax, edx / ret
            let dividend = (jitter_word as i32).wrapping_add(pool_int);
            let mut rem = dividend.wrapping_rem(n); // idiv; may panic if n == i32::MIN * -1, but n != 0 and |n| <= 0xffff
            if n <= 0 {
                rem = rem.wrapping_neg();
            }
            return rem;
        }

        // Large-n path (|n| > 0xffff).
        // 008fc55d  push edi
        // 008fc55e  mov  edi, [ecx]              ; edi = *cursor
        // 008fc560  add  eax, edi                ; eax = jitter + *cursor
        // 008fc562  mov  ecx, 0xffff
        // 008fc567  cdq / idiv ecx               ; edx = (jitter + *cursor) mod 0xffff
        let dividend = (jitter_word as i32).wrapping_add(pool_int);
        let r_mod = dividend.wrapping_rem(0xffff); // signed idiv by 0xffff

        // 008fc56a  mov  [esp+8], edx            ; local = r_mod
        // 008fc56e  fild [esp+8]                 ; ST0 = R
        // 008fc572  fild [esp+0x10]              ; ST0 = n, ST1 = R
        // 008fc576  fmul qword ptr [0x95e718]    ; ST0 = n * (1/65535.0)
        // 008fc57c  fmulp st(1)                  ; ST0 = R * (n * 1/65535)
        // 008fc57e  call __ftol                  ; truncate toward zero to i32
        // The constant at 0x95e718 = 1.5259021896696422e-05 = 1/0xffff exactly.
        let ftol_val = ((r_mod as f64) * ((n as f64) * (1.0f64 / 65535.0f64))) as i32;

        // 008fc583  lea ecx,[esi+esi*4]  / 008fc588 shl ecx,1  ; ecx = n * 10
        // 008fc58a..008fc59b  signed magic-div by 0xffff        ; ecx / 0xffff
        // 008fc59d  push edx / 008fc59e call 0x8fc4f0           ; recurse
        let recurse_arg = (n as i64).wrapping_mul(10) / 0xffff; // signed integer div
        let recurse_val = self.rand_mod(recurse_arg as i32);

        // 008fc5a3..008fc5bd  (recurse_val + 5) signed-div-10
        let adj = recurse_val.wrapping_add(5) / 10;

        // 008fc5bf  add edi, edx                 ; result = ftol + adj
        // 008fc5c1  cmp edi, eax                 ; eax = esi - 1 (n - 1)
        // 008fc5c3  jle 0x8fc5c7 / mov edi, eax  ; clamp to n - 1
        let mut result = ftol_val.wrapping_add(adj);
        let cap = n.wrapping_sub(1);
        if result > cap {
            result = cap;
        }
        result
    }

    /// Test hook — read cursor byte offset.
    #[cfg(test)]
    pub fn cursor_bytes(&self) -> u32 { self.cursor }
    /// Test hook — read jitter word.
    #[cfg(test)]
    pub fn jitter(&self) -> u32 { self.jitter }
    /// Test hook — force cursor for wrap tests.
    #[cfg(test)]
    pub fn set_cursor_bytes(&mut self, c: u32) { self.cursor = c; }
    /// Test hook — force jitter for reference-sequence tests.
    #[cfg(test)]
    pub fn set_jitter(&mut self, j: u32) { self.jitter = j; }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn pool_sha256_matches_documented() {
        // 204_000 dumped bytes = 203_996 pool + 4 sentinel-read bytes.
        assert_eq!(POOL.len(), 204_000);
        let full = format!("{:x}", Sha256::digest(POOL));
        assert_eq!(
            full,
            "dd03dfee2152974703de72d441b8360a35d45b15de7afa47354ae897e8213809"
        );
        let wrap = format!("{:x}", Sha256::digest(&POOL[..POOL_WRAP as usize]));
        assert_eq!(
            wrap,
            "e7420bf7d7d0b9f7a29591551c08d28244e9b332b7fd42c40c957c36f10459fc"
        );
    }

    // Reference sequence: with cursor set to 0 and jitter set to 0
    // before the first call, rand_mod(n) for small n should return
    // ((0 + *POOL[cursor+=4]) mod n) — i.e. pool ints starting at
    // index 1.  First few pool ints (LE i32):
    //   pool[0..4]  = 0x0000186b = 6251
    //   pool[4..8]  = 0x00006df2 = 28146
    //   pool[8..12] = 0x0000bda4 = 48548
    //   pool[12..16]= 0x000032c8 = 13000
    //   pool[16..20]= 0x0000c497 = 50327
    //
    // (Cursor is bumped +4 BEFORE the read, so first call reads pool[4..8].)
    #[test]
    fn rand_mod_matches_hand_computed_reference() {
        let mut rng = GameRng::new(0);
        rng.set_cursor_bytes(0);
        rng.set_jitter(0);
        // First call: cursor becomes 4, reads i32 at pool[4..8] = 28146.
        // rand_mod(100) = (0 + 28146) % 100 = 46.
        assert_eq!(rng.rand_mod(100), 28146 % 100);
        // Cursor = 8, reads pool[8..12] = 48548.
        assert_eq!(rng.rand_mod(100), 48548 % 100);
        // Cursor = 12, reads pool[12..16] = 13000.
        assert_eq!(rng.rand_mod(100), 13000 % 100);
        // Cursor = 16, reads pool[16..20] = 50327.
        assert_eq!(rng.rand_mod(100), 50327 % 100);
    }

    #[test]
    fn rand_mod_zero_returns_zero() {
        let mut rng = GameRng::new(0x12345);
        assert_eq!(rng.rand_mod(0), 0);
        // n == 0 must not advance cursor.
        let before = rng.cursor_bytes();
        rng.rand_mod(0);
        assert_eq!(rng.cursor_bytes(), before);
    }

    #[test]
    fn rand_mod_small_n_stays_in_range() {
        let mut rng = GameRng::new(0xdeadbeef);
        for _ in 0..10_000 {
            let v = rng.rand_mod(100);
            assert!(v >= 0 && v < 100, "out of range: {}", v);
        }
    }

    #[test]
    fn rand_mod_large_n_uses_recursion_path() {
        let mut rng = GameRng::new(0xabcd_1234);
        for _ in 0..1_000 {
            let v = rng.rand_mod(0x20000);
            assert!(v >= 0 && v < 0x20000, "out of range: {}", v);
        }
    }

    #[test]
    fn pool_wrap_reseeds_jitter() {
        let mut rng = GameRng::new(1);
        // Force cursor to sentinel so the very next bump wraps.
        rng.set_cursor_bytes(POOL_WRAP);
        rng.set_jitter(0);
        let jitter_before = rng.jitter();
        // Any call should now trigger the wrap-and-reseed branch.
        let _ = rng.rand_mod(100);
        assert_eq!(rng.cursor_bytes(), 0, "cursor must reset to base on wrap");
        // Jitter changed (LCG output & 0xffff — vanishingly unlikely to
        // be exactly the pre-wrap value of 0).
        assert_ne!(rng.jitter(), jitter_before);
        assert!(rng.jitter() <= 0xffff);
    }

    #[test]
    fn new_with_same_seed_is_deterministic() {
        let mut a = GameRng::new(0x0123_4567);
        let mut b = GameRng::new(0x0123_4567);
        for _ in 0..1_000 {
            assert_eq!(a.rand_mod(1_000), b.rand_mod(1_000));
        }
    }

    #[test]
    fn rand_mod_large_n_boundary_clamped() {
        // n exactly at the small/large boundary + 1 — exercises the
        // recursion & clamp path.
        let mut rng = GameRng::new(42);
        for _ in 0..100 {
            let v = rng.rand_mod(0x10000);
            assert!(v >= 0 && v < 0x10000);
        }
    }
}
