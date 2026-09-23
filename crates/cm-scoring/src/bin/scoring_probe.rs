//! Differential probe: the harness (tools/exe_diff) calls this to get the Rust
//! result for a scoring primitive on a given input, to diff against cm0102.exe.
//! One case per stdin line: `<fn> <args...>`; prints the integer result per line.
//!
//!   repfit <blockhex> <mode0:0|1> <c>            -> i32
//!   closeness <8 flag bits: refnull person_has_club same_club same_club_nation
//!             same_person_nation ref_known_nation ref_known_club_nation regional>  -> u8
//!   basereweight <l> <pos:0|1> <neg:0|1>         -> i32
use std::io::{self, BufRead, Write};
use cm_scoring::*;

fn hexbytes(s: &str) -> Vec<u8> {
    (0..s.len()).step_by(2).filter_map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect()
}

fn main() {
    let stdin = io::stdin();
    let out = io::stdout();
    let mut o = out.lock();
    for line in stdin.lock().lines() {
        let line = match line { Ok(l) => l, Err(_) => break };
        let t: Vec<&str> = line.split_whitespace().collect();
        if t.is_empty() { continue; }
        let r: i64 = match t[0] {
            "repfit" => {
                let block = hexbytes(t[1]);
                let mode0 = t[2] == "1";
                let c: u8 = t[3].parse().unwrap();
                manager_club_repfit(&block, mode0, c) as i64
            }
            "closeness" => {
                let b = |i: usize| t.get(i).map(|x| *x == "1").unwrap_or(false);
                let ci = ClosenessInputs {
                    ref_null: b(1), person_has_club: b(2), same_club: b(3),
                    same_club_nation: b(4), same_person_nation: b(5),
                    ref_known_in_person_nation: b(6), ref_known_in_person_club_nation: b(7),
                    regional_rep_pass: b(8),
                };
                closeness_class(&ci) as i64
            }
            "basereweight" => {
                let l: i32 = t[1].parse().unwrap();
                score_base_reweight(l, t[2] == "1", t[3] == "1") as i64
            }
            "skeleton" => {
                // base status club_rep standing_c standing_4 standing_6 incumbent
                //   incumbent_active ambition aff_a aff_b   (aff: -1 none / 0 neg / 1 pos)
                let p = |i: usize| t[i].parse::<i32>().unwrap();
                let aff = |i: usize| match t[i] { "1" => Some(true), "0" => Some(false), _ => None };
                score_skeleton(p(1), p(2) as u8, p(3), p(4) as i16, p(5) as i16, p(6) as i16,
                    t[7] == "1", t[8] == "1", p(9), aff(10), aff(11)) as i64
            }
            _ => -999999,
        };
        writeln!(o, "{}", r).ok();
    }
}
