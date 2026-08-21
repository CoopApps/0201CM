# Official Editor Validation Notes

Source: `D:/cm0102/editor`, Championship Manager 01/02 Data Editor.

The official editor is useful as a validation oracle for which data families the
game expected users to edit. It is not treated as source code for the Rust
implementation, but its help and release notes give us a checklist for Rust DB
coverage and editor UI parity.

## Editor View Families

From `cm0102ed.CNT`:

- Staff
- Clubs
- Nations
- Stadiums
- Cities
- Continents
- Staff competitions
- Club competitions
- International competitions
- Officials
- Colours
- Weather configurations
- Names: first, second, common
- Staff configurations

## Validation Leads

- Staff configuration is a separate editor concept used for future transfers,
  loans, injuries, retirement, drug bans, misconduct bans, and false passport bans.
- Staff and name edits must stay synchronized; the official editor changelog
  repeatedly mentions bugs around staff names, common names, and name sorting.
- Club competition history and staff competition history are editable official
  surfaces, so our packed history tables need code-derived field names before they
  can be considered semantically complete.
- Nation staff counts, nation region fields, club cash, club colours, and staff
  classifications are official-editor validation targets for future lifts.

## Current Rust Status

- Rust DB owns the table payloads and can export compatibility `.dat` files.
- The viewer resolves competition IDs for competition history tables where the
  relationship is clear from table IDs.
- Remaining packed history slots are displayed with raw, hex, signed, and
  low/high 16-bit views until code-derived semantics are available.
