# CM 01/02 Rust Port — Deviation Register

Every difference between the Rust port and CM 01/02's observed / recovered
behaviour, tracked to closure. Target state: zero deviations.

## Contract terms

Each entry is one of:

- **VERIFIED EXACT** — behaviour established from original executable /
  data evidence. Reproducible byte- or bit-exact.
- **STRONGLY SUPPORTED** — substantial evidence, one or more details
  unverified. Called out.
- **TEMPORARY STUB** — required behaviour not yet recovered. Placeholder
  present; MUST be replaced.

## Entry template

```
## <subsystem>

- **Rust site**: `crates/<crate>/src/<file.rs>[:line]`
- **Executable function/address**: `FUN_XXXXXXXX` / `0x00XXXXXX`
- **Discrepancy**: [what differs from the exe]
- **Evidence**: [what has been recovered so far]
- **Confidence**: VERIFIED EXACT | STRONGLY SUPPORTED | TEMPORARY STUB
- **Work required**: [next concrete step(s)]
- **Player-visible**: yes/no
- **Save-affecting**: yes/no
- **Opened**: YYYY-MM-DD
- **Resolved**: YYYY-MM-DD  (blank until closed)
```

## Live deviations

See `fixture_dates.md`, `cup_draws.md`, etc. in this directory.
