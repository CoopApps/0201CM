//! Port of `FUN_00933579` — the CRT sprintf trampoline the widget
//! renderer calls three times from `FUN_005d7aa0` (blocks I / J / K,
//! asm 005d7ecb / 005d7f6c / 005d800d), always with the format string
//! `"%c\0"` at VA `0x009a3ac4` and one char arg.
//!
//! ## FUN_00933579 shape
//!
//! ```
//! 00933579  push    ebp
//! 0093357a  mov     ebp, esp
//! 0093357c  sub     esp, 0x20
//! 0093357f  mov     eax, [ebp+8]         ; dst
//! 00933582  push    esi
//! 00933583  mov     [ebp-0x18], eax
//! 00933586  mov     [ebp-0x20], eax
//! 00933589  lea     eax, [ebp+0x10]      ; &va_args (past fmt)
//! 0093358c  mov     [ebp-0x14], 0x42     ; ??? (unclear scratch slot)
//! 00933593  push    eax                  ; va_args ptr
//! 00933594  lea     eax, [ebp-0x20]      ; &state
//! 00933597  push    [ebp+0xc]            ; fmt
//! 0093359a  mov     [ebp-0x1c], 0x7fffffff  ; cap = INT_MAX
//! 009335a1  push    eax                  ; state
//! 009335a2  call    0x9364c7             ; vsnprintf-like core
//! 009335a7  add     esp, 0xc
//! 009335aa  dec     [ebp-0x1c]           ; cap-- (checks for underflow)
//! 009335ad  mov     esi, eax             ; return value
//! 009335af  js      0x9335b9             ; if cap < 0 → error path
//! 009335b1  mov     eax, [ebp-0x20]      ; current write cursor
//! 009335b4  and     byte [eax], 0        ; null-terminate
//! 009335b7  jmp     0x9335c6
//! 009335b9  lea     eax, [ebp-0x20]      ; error branch — invoke sink
//! 009335bc  push    eax
//! 009335bd  push    0                    ; NULL
//! 009335bf  call    0x9363b2             ; error/flush helper
//! 009335c4  pop     ecx
//! 009335c5  pop     ecx
//! 009335c6  mov     eax, esi
//! 009335c8  pop     esi
//! 009335c9  leave
//! 009335ca  ret
//! ```
//!
//! In shape it is a classic sprintf trampoline over an internal
//! vsnprintf. For the callers we actually reach — three uses in the
//! widget renderer, all `sprintf(buf, "%c\0", ch)` — the effect is
//! "write one char, then NUL-terminate, return 1". `packed_widget`
//! calls into [`sprintf_percent_c`] for those exact three sites; the
//! function name is kept short to make the callsite line up 1:1 with
//! the exe's `call 0x933579` line.
//!
//! ## Format string at VA 0x009a3ac4
//!
//! The three call sites all push `0x9a3ac4` as the format argument
//! (asm 005d7ecb, 005d7f6c, 005d800d) followed by one byte-sized
//! `push` for the character:
//!
//! ```
//! 005d7ec5  push   0x2b            ; '+' (ch)  — block I
//! 005d7ecb  push   0x9a3ac4        ; fmt
//! 005d7ed0  push   edx             ; dst
//! 005d7ed1  call   0x933579
//! ```
//!
//! The literal at `0x9a3ac4` sits inside the string-literal region
//! between `"Albanian..."` at `0x9a3aa0` (len 33, ending at
//! `0x9a3ac1`) and `"%s, %c"` at `0x9a3ad0`. The gap holds a small
//! aligned format literal, and every callsite passes exactly one char
//! arg — the only shape that matches is `"%c\0"`. That is also the
//! shape needed to keep the surrounding widget behaviour correct:
//! blocks I/J/K each want to paint one of `'+'`, `','`, `'-'` as one
//! glyph via `draw_wrapped_text`, and the pre-fill sprintf followed
//! by draw_wrapped_text on a `[ch, 0, ...]` buffer produces the same
//! character stream. If a future exe dump proves otherwise, this
//! module is the one place to fix — every caller goes through it.

/// Port of `sprintf(dst, "%c\0", ch)` — the single-`%c` case reached
/// from every call to `FUN_00933579` inside `FUN_005d7aa0`.
///
/// Writes `ch` to `dst[0]`, NUL-terminates at `dst[1]`, returns 1
/// (matches `esi = 1` after `mov esi, eax` on the vsnprintf return).
/// Returns 0 if `dst` has fewer than 2 slots — the exe's underlying
/// vsnprintf reports negative on truncation and the wrapper's
/// `js 0x9335b9` branch would then invoke its error sink; the two
/// callers we mirror pass a 32-byte scratch (`[esp+0x24]`), so the
/// truncation path is unreachable in practice.
pub fn sprintf_percent_c<const N: usize>(dst: &mut [u8; N], ch: u8) -> i32 {
    if N < 2 {
        return 0;
    }
    dst[0] = ch;
    dst[1] = 0;
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprintf_percent_c_writes_char_and_nul() {
        let mut buf = [0xffu8; 32];
        let n = sprintf_percent_c(&mut buf, b'+');
        assert_eq!(n, 1, "return value matches vsnprintf's char count");
        assert_eq!(buf[0], b'+');
        assert_eq!(buf[1], 0, "must NUL-terminate at index 1");
        // Rest of the scratch is untouched — matches the exe, which
        // writes only 2 bytes and leaves the remainder as whatever the
        // caller had on the stack.
        assert!(buf[2..].iter().all(|&b| b == 0xff));
    }

    #[test]
    fn sprintf_percent_c_writes_all_three_widget_chars() {
        // Blocks I/J/K feed '+', ',', '-' respectively.
        for &c in &[b'+', b',', b'-'] {
            let mut buf = [0u8; 32];
            let n = sprintf_percent_c(&mut buf, c);
            assert_eq!(n, 1);
            assert_eq!(buf[0], c);
            assert_eq!(buf[1], 0);
        }
    }

    #[test]
    fn sprintf_percent_c_reports_zero_when_dst_too_small() {
        let mut buf = [0u8; 1];
        assert_eq!(sprintf_percent_c(&mut buf, b'+'), 0);
    }
}
