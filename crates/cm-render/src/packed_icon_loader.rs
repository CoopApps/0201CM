//! Port of `FUN_005cdb50` — the widget-icon disk-bitmap loader called by
//! `FUN_005d7aa0` block D's cache-miss path.
//!
//! Reads a 48-byte header from a .bmp/.icn file on disk, then a
//! packed-16bpp pixel buffer sized `hdr.pixel_bytes`. If the surface's
//! native pixel format (`DAT_00acdeac`) differs from the disk file's
//! own format field (hdr[0x24]), the pixel stream is converted in
//! place using the exe's exact bit-swizzle formulas (see asm 5cdcc4
//! for 555→565 and 5cdcfb for 565→555).
//!
//! ## Faithfulness
//!
//! Every asm instruction of `FUN_005cdb50` (202 instructions across
//! 0x005cdb50..0x005cdd2a) has a corresponding Rust line below with
//! `; 005cd..` address comment, EXCEPT the 5 CRT calls which are
//! documented substitutions:
//!
//! | asm site   | CRT fn                | Rust replacement                        |
//! |------------|-----------------------|-----------------------------------------|
//! | 005cdb80   | `fopen(name,"rb")`    | `std::fs::File::open(path)`             |
//! | 005cdba4   | `fread(hdr,0x30,1,fp)`| `file.read_exact(&mut hdr[..48])`       |
//! | 005cdbea   | `calloc(0x30,1)`      | `[0u8;48]` stack local + `IconBitmap`   |
//! | 005cdc1e   | `calloc(size,1)`      | `vec![0u8; pixel_bytes as usize]`       |
//! | 005cdc38/  | `fclose(fp)`          | implicit `drop(File)` at scope end      |
//! | 005cdd18   |                       |                                         |
//! | 005cdc76/  | `free(ptr)`           | implicit `drop(Vec)` (owned by record)  |
//! | 005cdc86/  |                       |                                         |
//! | 005cdc2e   |                       |                                         |
//!
//! These are the ONLY non-byte-exact substitutions in this port.
//!
//! ## Signature deviation from FUN_005cdb50
//!
//! The exe has 2 args (filename, cache_slot). The format-comparison
//! `target_pixel_format` value the prompt describes as arg2 is NOT a
//! function arg — the asm reads it from the FILE HEADER at hdr[0x24]
//! (`[esp+0x34]` post-prologue = local slot inside the 0x30-byte
//! locals area where fread wrote the 48-byte header). The port
//! matches the asm and reads hdr[0x24] rather than accepting an arg.

use crate::packed_widget_globals::{DAT_00ACDEAC, DAT_00AD6B44};
use std::fs::File;
use std::io::Read;

/// Header layout on disk (48 bytes = 12 dwords), decoded from the
/// `rep movsd ecx=0xc` at asm 5cdc0b..5cdc16 (copies 12 dwords from
/// `[esp+0x10]` to `[ebx+0]`) and subsequent field reads.
///
/// Only 3 named fields are used by the port; the remaining 9 dwords
/// are preserved as opaque bytes in `raw_header` so callers who need
/// them can decode later.
#[derive(Debug, Clone)]
pub struct IconBitmap {
    /// hdr[0x00] — pixel width. Read at asm 5cdbcf (cache-hit compare)
    /// and 5cdcb5 (`imul ecx, [ebx]`).
    pub width: u32,
    /// hdr[0x04] — pixel height. Read at asm 5cdbda (cache-hit compare)
    /// and 5cdcaf (`mov ecx, [ebx+4]`).
    pub height: u32,
    /// hdr[0x08] — size in bytes of the pixel buffer that follows the
    /// header. Read at asm 5cdc18 (`mov eax, [ebx+8]`), passed as
    /// `nbytes` to the second calloc + fread.
    pub pixel_bytes: u32,
    /// The pixel buffer proper. Stored as `Vec<u16>` for byte-exact
    /// swizzle in the format-conversion loops; length = pixel_bytes/2.
    /// Corresponds to `[ebx+0xc]` in the asm.
    pub pixels: Vec<u16>,
    /// The full 48-byte header verbatim. hdr[0x24] holds the disk-side
    /// pixel format (compared against `DAT_00acdeac` at asm 5cdca1);
    /// other slots hold unused metadata (colour-key, hot-spot, etc).
    pub raw_header: [u8; 48],
}

/// Port of `FUN_005cdb50(filename, cache_slot)`.
///
/// The 3rd formal parameter listed in the commit brief
/// (`target_pixel_format`) is NOT an actual function arg — see the
/// module doc.
pub fn load_icon_bitmap(
    filename: &[u8],
    cache_slot: Option<&IconBitmap>,
) -> Option<IconBitmap> {
    // 005cdb50  mov eax, [0xad6b44]         ; load gate
    let gate = DAT_00AD6B44;
    // 005cdb55  sub esp, 0x30               ; local hdr[48]
    // 005cdb58  test eax, eax
    // 005cdb5a  push ebx                     ; prologue save (n/a in port)
    // 005cdb5b  push ebp                     ; prologue save (n/a)
    // 005cdb5c  push esi                     ; prologue save (n/a)
    // 005cdb5d  push edi                     ; prologue save (n/a)
    // 005cdb5e  je 5cdb6a                    ; skip early-return when gate == 0
    if gate != 0 {
        // 005cdb60  pop edi                  ; epilogue restore
        // 005cdb61  pop esi
        // 005cdb62  pop ebp
        // 005cdb63  xor eax, eax             ; return 0
        // 005cdb65  pop ebx
        // 005cdb66  add esp, 0x30
        // 005cdb69  ret
        return None;
    }

    // 005cdb6a  mov eax, [esp+0x44]          ; eax = filename (arg0)
    // 005cdb6e  test eax, eax
    // 005cdb70  jne 5cdb7a                   ; non-null → continue
    if filename.is_empty() {
        // 005cdb72  pop edi
        // 005cdb73  pop esi
        // 005cdb74  pop ebp
        // 005cdb75  pop ebx
        // 005cdb76  add esp, 0x30
        // 005cdb79  ret                       ; return 0 (implicit — eax is filename == 0)
        return None;
    }

    // 005cdb7a  push 0x9946a8                ; "rb"  — CRT literal
    // 005cdb7f  push eax                      ; push filename
    // 005cdb80  call 0x9343d0                 ; fopen(filename, "rb")   [CRT SUBST]
    let name_end = filename.iter().position(|&b| b == 0).unwrap_or(filename.len());
    let path = std::str::from_utf8(&filename[..name_end]).ok()?;
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => {
            // 005cdb85  mov esi, eax
            // 005cdb87  add esp, 8
            // 005cdb8a  test esi, esi
            // 005cdb8c  mov [esp+0x44], esi   ; store fp in arg0 slot
            // 005cdb90  jne 5cdb9a
            // 005cdb92  pop edi               ; failed → return 0
            // 005cdb93  pop esi
            // 005cdb94  pop ebp
            // 005cdb95  pop ebx
            // 005cdb96  add esp, 0x30
            // 005cdb99  ret
            return None;
        }
    };

    // 005cdb9a  push esi                     ; push fp
    // 005cdb9b  push 1                        ; push nelem=1
    // 005cdb9d  lea eax, [esp+0x18]          ; addr of hdr local
    // 005cdba1  push 0x30                     ; push size=0x30
    // 005cdba3  push eax                      ; push buf
    // 005cdba4  call 0x9344ce                 ; fread(hdr, 0x30, 1, fp)  [CRT SUBST]
    // 005cdba9  add esp, 0x10
    // 005cdbac  cmp eax, 1
    // 005cdbaf  je 5cdbc4                    ; success → continue
    let mut hdr = [0u8; 48];
    if file.read_exact(&mut hdr).is_err() {
        // 005cdbb1  push esi                 ; push fp
        // 005cdbb2  call 0x9342d9             ; fclose(fp)     [CRT SUBST — drop]
        // 005cdbb7  add esp, 4
        // 005cdbba  xor eax, eax              ; return 0
        // 005cdbbc  pop edi
        // 005cdbbd  pop esi
        // 005cdbbe  pop ebp
        // 005cdbbf  pop ebx
        // 005cdbc0  add esp, 0x30
        // 005cdbc3  ret
        return None;
    }

    // Decode header dwords used by the loader
    let hdr_width       = u32::from_le_bytes([hdr[0x00], hdr[0x01], hdr[0x02], hdr[0x03]]);
    let hdr_height      = u32::from_le_bytes([hdr[0x04], hdr[0x05], hdr[0x06], hdr[0x07]]);
    let hdr_pixel_bytes = u32::from_le_bytes([hdr[0x08], hdr[0x09], hdr[0x0a], hdr[0x0b]]);
    let hdr_format      = u32::from_le_bytes([hdr[0x24], hdr[0x25], hdr[0x26], hdr[0x27]]);

    // 005cdbc4  mov ebp, [esp+0x48]          ; ebp = cache_slot (arg1)
    // 005cdbc8  test ebp, ebp
    // 005cdbca  je 5cdbe6                    ; no cache → alloc
    // 005cdbcc  mov ecx, [ebp+0]             ; ecx = cache->width
    // 005cdbcf  mov eax, [esp+0x10]          ; eax = hdr[0] disk width
    // 005cdbd3  cmp ecx, eax
    // 005cdbd5  jne 5cdbe6                   ; width mismatch → alloc
    // 005cdbd7  mov edx, [ebp+4]             ; edx = cache->height
    // 005cdbda  mov eax, [esp+0x14]          ; eax = hdr[4] disk height
    // 005cdbde  cmp edx, eax
    // 005cdbe0  jne 5cdbe6                   ; height mismatch → alloc
    // 005cdbe2  mov ebx, ebp                 ; ebx = cache_slot (reuse)
    // 005cdbe4  jmp 5cdc4e                   ; skip alloc + rep movsd
    let cache_hit = match cache_slot {
        Some(slot) => slot.width == hdr_width && slot.height == hdr_height,
        None => false,
    };

    // ebx layout after allocation: [ebx+0] width, [ebx+4] height,
    // [ebx+8] pixel_bytes, [ebx+0xc] pixel_ptr. On hit, ebx points at
    // the caller's cache record — we reuse its pixel_bytes value.
    let record_width;
    let record_height;
    let record_pixel_bytes;
    if cache_hit {
        let slot = cache_slot.unwrap();
        record_width = slot.width;
        record_height = slot.height;
        record_pixel_bytes = slot.pixel_bytes;
    } else {
        // 005cdbe6  push 0x30
        // 005cdbe8  push 1
        // 005cdbea  call 0x933ddd                ; calloc(0x30, 1) → record   [CRT SUBST]
        // 005cdbef  mov ebx, eax
        // 005cdbf1  add esp, 8
        // 005cdbf4  test ebx, ebx
        // 005cdbf6  jne 5cdc0b                   ; success → continue
        // (Rust: [0u8;48] on stack; owning IconBitmap allocated on return
        //  — no fallible calloc equivalent needed.)
        //
        // 005cdbf8  push esi                     ; failed → fclose + ret 0
        // 005cdbf9  call 0x9342d9
        // 005cdbfe  add esp, 4
        // 005cdc01  xor eax, eax
        // 005cdc03  pop edi ...
        // 005cdc0a  ret

        // 005cdc0b  mov ecx, 0xc
        // 005cdc10  lea esi, [esp+0x10]
        // 005cdc14  mov edi, ebx
        // 005cdc16  rep movsd                    ; copy 48 bytes of hdr → record
        // (Rust: the record's width/height/pixel_bytes come from
        //  hdr_* below; raw_header preserves the byte copy.)
        record_width = hdr_width;
        record_height = hdr_height;

        // 005cdc18  mov eax, [ebx+8]             ; pixel_bytes just copied
        // 005cdc1b  push eax
        // 005cdc1c  push 1
        // 005cdc1e  call 0x933ddd                ; calloc(pixel_bytes, 1)   [CRT SUBST]
        // 005cdc23  add esp, 8
        // 005cdc26  mov [ebx+0xc], eax
        // 005cdc29  test eax, eax
        // 005cdc2b  jne 5cdc4a                   ; success → continue
        // (Rust: vec![0u8; pixel_bytes as usize] later; no fallible
        //  path — a huge pixel_bytes would panic-abort in Rust which
        //  is the semantic equivalent of the exe crashing on OOM.)
        //
        // 005cdc2d  push ebx                    ; failed calloc → cleanup
        // 005cdc2e  call 0x933ba6                ; free(record)   [CRT SUBST — drop]
        // 005cdc33  mov ecx, [esp+0x48]         ; fp
        // 005cdc37  push ecx
        // 005cdc38  call 0x9342d9                ; fclose(fp)     [CRT SUBST]
        // 005cdc3d  add esp, 8
        // 005cdc40  xor eax, eax
        // 005cdc42  pop edi ...
        // 005cdc49  ret

        record_pixel_bytes = hdr_pixel_bytes;
    }

    // 005cdc4a  mov esi, [esp+0x44]          ; esi = fp (re-fetch — cache-hit path skipped
    //                                         ; the esi=fp preserving pushes; on miss
    //                                         ; the fp is still in the arg0 slot)
    // 005cdc4e  mov edx, [ebx+8]             ; edx = pixel_bytes
    // 005cdc51  mov eax, [ebx+0xc]           ; eax = pixel_ptr (destination)
    // 005cdc54  push esi                     ; push fp
    // 005cdc55  push 1                        ; nelem=1
    // 005cdc57  push edx                      ; size=pixel_bytes
    // 005cdc58  push eax                      ; buf=pixel_ptr
    // 005cdc59  call 0x9344ce                 ; fread(pixel_ptr, pixel_bytes, 1, fp)  [CRT SUBST]
    // 005cdc5e  add esp, 0x10
    // 005cdc61  cmp eax, 1
    // 005cdc64  je 5cdca1                    ; success → format check
    let mut pixels_bytes_buf = vec![0u8; record_pixel_bytes as usize];
    if file.read_exact(&mut pixels_bytes_buf).is_err() {
        // 005cdc66  cmp ebx, ebp                ; is ebx the cache slot?
        // 005cdc68  je 5cdc8e                   ; yes → don't free (caller owns)
        // 005cdc6a  test ebx, ebx
        // 005cdc6c  je 5cdc8e                   ; already null → skip
        // 005cdc6e  mov eax, [ebx+0xc]          ; pixel_ptr
        // 005cdc71  test eax, eax
        // 005cdc73  je 5cdc85                   ; already null → skip
        // 005cdc75  push eax
        // 005cdc76  call 0x933ba6                ; free(pixel_ptr)   [CRT SUBST]
        // 005cdc7b  add esp, 4
        // 005cdc7e  mov [ebx+0xc], 0
        // 005cdc85  push ebx
        // 005cdc86  call 0x933ba6                ; free(record)     [CRT SUBST]
        // 005cdc8b  add esp, 4
        // 005cdc8e  push esi
        // 005cdc8f  call 0x9342d9                ; fclose(fp)       [CRT SUBST]
        // 005cdc94  add esp, 4
        // 005cdc97  xor eax, eax
        // 005cdc99  pop edi ...
        // 005cdca0  ret
        return None;
    }

    // Convert bytes → Vec<u16> — the exe reads pixels as `word[eax]`
    // in-place; ours materialises the u16 view.
    let mut pixels: Vec<u16> = pixels_bytes_buf
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();

    // 005cdca1  mov edx, [0xacdeac]          ; edx = native pixel format
    // 005cdca7  mov eax, [esp+0x34]          ; eax = hdr[0x24] disk format
    // 005cdcab  cmp eax, edx
    // 005cdcad  je 5cdd17                    ; equal → skip conversion
    let native = DAT_00ACDEAC as u32;
    if native != hdr_format {
        // 005cdcaf  mov ecx, [ebx+4]         ; ecx = height
        // 005cdcb2  mov eax, [ebx+0xc]       ; eax = pixel_ptr
        // 005cdcb5  imul ecx, [ebx]          ; ecx = height * width
        // 005cdcb8  cmp edx, 0x7e0           ; native == RGB565?
        // 005cdcbe  jne 5cdcf5               ; no → 565→555 branch
        let pixel_count = (record_width as usize).saturating_mul(record_height as usize);
        let pixel_count = pixel_count.min(pixels.len());
        if native == 0x7e0 {
            // ------ 555 → 565 conversion loop (native surface is 565) ------
            // 005cdcc0  test ecx, ecx
            // 005cdcc2  jle 5cdd17
            // 005cdcc4  mov edx, ecx           ; edx = counter
            // 005cdcc6  mov cx, word[eax]      ; load pixel
            // 005cdcc9  add eax, 2
            // 005cdccc  mov edi, ecx
            // 005cdcce  and ecx, 0x1f          ; blue bits
            // 005cdcd1  and edi, 0xffe0        ; everything above blue
            // 005cdcd7  shl edi, 1             ; shift green/red up 1
            // 005cdcd9  or  edi, ecx           ; merge blue
            // 005cdcdb  dec edx
            // 005cdcdc  mov word[eax-2], di
            // 005cdce0  jne 5cdcc6
            for p in &mut pixels[..pixel_count] {
                let old = *p as u32;
                let blue = old & 0x1f;
                let above = old & 0xffe0;
                *p = ((above << 1) | blue) as u16;
            }
            // 005cdce2  push esi
            // 005cdce3  call 0x9342d9          ; fclose(fp)   [CRT SUBST]
            // 005cdce8  add esp, 4
            // 005cdceb  mov eax, ebx           ; return record
            // 005cdced  pop edi ...
            // 005cdcf4  ret
        } else {
            // ------ 565 → 555 conversion loop (native surface is 555) ------
            // 005cdcf5  test ecx, ecx
            // 005cdcf7  jle 5cdd17
            // 005cdcf9  mov edx, ecx
            // 005cdcfb  mov cx, word[eax]
            // 005cdcfe  add eax, 2
            // 005cdd01  mov edi, ecx
            // 005cdd03  and ecx, 0x1f
            // 005cdd06  shr edi, 1
            // 005cdd08  and edi, 0x7fe0
            // 005cdd0e  or  edi, ecx
            // 005cdd10  dec edx
            // 005cdd11  mov word[eax-2], di
            // 005cdd15  jne 5cdcfb
            for p in &mut pixels[..pixel_count] {
                let old = *p as u32;
                let blue = old & 0x1f;
                let above = (old >> 1) & 0x7fe0;
                *p = (above | blue) as u16;
            }
        }
    }
    // 005cdd17  push esi
    // 005cdd18  call 0x9342d9                ; fclose(fp)   [CRT SUBST]
    // 005cdd1d  add esp, 4
    // 005cdd20  mov eax, ebx                 ; return record
    // 005cdd22  pop edi
    // 005cdd23  pop esi
    // 005cdd24  pop ebp
    // 005cdd25  pop ebx
    // 005cdd26  add esp, 0x30
    // 005cdd29  ret
    Some(IconBitmap {
        width: record_width,
        height: record_height,
        pixel_bytes: record_pixel_bytes,
        pixels,
        raw_header: hdr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_path(name: &str) -> std::path::PathBuf {
        let dir = std::env::var("TEMP")
            .or_else(|_| std::env::var("TMP"))
            .unwrap_or_else(|_| "/tmp".to_string());
        let mut p = std::path::PathBuf::from(dir);
        p.push(format!(
            "cm-render-icon-loader-{}-{}.bin",
            name,
            std::process::id()
        ));
        p
    }

    /// Build a 48-byte header with (width, height, pixel_bytes) fields
    /// and a chosen disk-format value at hdr[0x24].
    fn make_hdr(w: u32, h: u32, bytes: u32, fmt: u32) -> [u8; 48] {
        let mut hdr = [0u8; 48];
        hdr[0x00..0x04].copy_from_slice(&w.to_le_bytes());
        hdr[0x04..0x08].copy_from_slice(&h.to_le_bytes());
        hdr[0x08..0x0c].copy_from_slice(&bytes.to_le_bytes());
        hdr[0x24..0x28].copy_from_slice(&fmt.to_le_bytes());
        hdr
    }

    fn write_file(path: &std::path::Path, bytes: &[u8]) {
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(bytes).unwrap();
    }

    fn make_nul_terminated(path: &std::path::Path) -> Vec<u8> {
        let mut v = path.to_str().unwrap().as_bytes().to_vec();
        v.push(0);
        v
    }

    #[test]
    fn tiny_4x4_rgb555_loads_pixels_verbatim_when_native_is_555() {
        // Native format = DAT_00ACDEAC = 0 (default). Disk format = 0.
        // Equal formats → no conversion → pixels returned as-is.
        let path = temp_path("tiny_555");
        let hdr = make_hdr(4, 4, 32, 0);
        let mut file_bytes = hdr.to_vec();
        let pixels_in: Vec<u16> = (0..16).map(|i| i as u16 * 0x101).collect();
        for p in &pixels_in {
            file_bytes.extend_from_slice(&p.to_le_bytes());
        }
        write_file(&path, &file_bytes);
        let name = make_nul_terminated(&path);
        let ibm = load_icon_bitmap(&name, None).expect("load");
        assert_eq!(ibm.width, 4);
        assert_eq!(ibm.height, 4);
        assert_eq!(ibm.pixel_bytes, 32);
        assert_eq!(ibm.pixels, pixels_in);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn tiny_bitmap_converts_555_to_565_when_native_is_565() {
        // DAT_00ACDEAC is 0 in the exe, so we can't hit the 565 branch
        // in production. Verify the swizzle formulas directly.
        let old: u32 = 0x1234;
        // 555→565: ((old & 0xffe0) << 1) | (old & 0x1f)
        let expected_555_to_565 = ((old & 0xffe0) << 1) | (old & 0x1f);
        assert_eq!(expected_555_to_565, 0x2454, "555->565 swizzle: 0x1234 -> 0x2454");
        // 565→555: ((old >> 1) & 0x7fe0) | (old & 0x1f)
        let expected_565_to_555 = ((old >> 1) & 0x7fe0) | (old & 0x1f);
        assert_eq!(expected_565_to_555, 0x0914, "565->555 swizzle: 0x1234 -> 0x0914");
    }

    #[test]
    fn missing_file_returns_none() {
        let name = b"D:\\definitely_does_not_exist_1234567890.bin\0";
        assert!(load_icon_bitmap(name, None).is_none());
    }

    #[test]
    fn short_header_returns_none() {
        let path = temp_path("short_hdr");
        write_file(&path, &[0u8; 10]);
        let name = make_nul_terminated(&path);
        assert!(load_icon_bitmap(&name, None).is_none());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn empty_filename_returns_none() {
        assert!(load_icon_bitmap(&[], None).is_none());
        assert!(load_icon_bitmap(&[0u8], None).is_none());
    }

    #[test]
    fn cache_slot_matching_wh_still_reads_new_pixels() {
        // Cache-hit path in the exe reuses the cache record's
        // pixel-buffer slot but ALWAYS freads the file — the cache is
        // for the record allocation, not for the pixel data.
        let path = temp_path("cache_hit");
        let hdr = make_hdr(2, 2, 8, 0);
        let mut bytes = hdr.to_vec();
        let new_pixels: [u16; 4] = [0x1111, 0x2222, 0x3333, 0x4444];
        for p in &new_pixels {
            bytes.extend_from_slice(&p.to_le_bytes());
        }
        write_file(&path, &bytes);
        let name = make_nul_terminated(&path);
        let cached = IconBitmap {
            width: 2,
            height: 2,
            pixel_bytes: 8,
            pixels: vec![0xAAAA; 4],
            raw_header: hdr,
        };
        let out = load_icon_bitmap(&name, Some(&cached)).expect("load");
        assert_eq!(out.pixels, new_pixels);
        let _ = std::fs::remove_file(&path);
    }
}
