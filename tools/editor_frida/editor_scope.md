# What the editor exposes (and doesn't)

Empirical result of scanning cm0102ed.exe's live memory (v1.1.0, index.dat
loaded) for each shipped `Data/*.dat` file's contents.

## Loaded and directly byte-addressable

15 of 20 shipped .dats. Located in `pool_map.json`.

    city.dat            colour.dat        continent.dat
    club.dat            club_comp.dat     nation.dat
    nation_comp.dat     officials.dat     stadium.dat  [verified]
    first_names.dat     second_names.dat  common_names.dat
    staff_comp.dat      staff_comp_history.dat

## Loaded but unpacked (not raw-contiguous)

- **staff.dat** (30 MB). Section signatures from records 100/1000 of each
  type all land in range `0x086a0000` (33 MB). Editor decomposes each
  Person / type9 / type10 disk record into a Delphi TObject — approximate
  in-memory type6 stride is 179 B (vs 157 B on disk). Byte-diff against
  the raw disk file needs a per-record TObject shape, not a flat stride.
  Deferred to a follow-up.

## Not loaded by the editor at all

Zero memory hits for signatures at 6 different record offsets:

- `nat_club.dat`               (national-team squads; runtime state)
- `club_comp_history.dat`      (season-history; runtime state)
- `nation_comp_history.dat`    (season-history; runtime state)
- `staff_history.dat`          (per-player career records; runtime state)
- `index.dat`                  (a .dat directory, not record data)

The editor v1.1.0 opens what users can edit. Runtime tables generated
by the game don't come along. These files must be validated a different
way — direct byte-diff of shipped `.dat` vs our rust-db loader
round-trip.
