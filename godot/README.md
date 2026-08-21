# CM0102 Rust Godot Front End

This is the first modern delivery shell for the Rust-owned CM0102 backend.

Start the Rust backend first:

```text
cargo run -p cm-app -- serve-rust-db D:/cm0102-rs/rust-db 8770
```

Then open this folder in Godot 4.7:

```text
D:/cm0102-rs/godot
```

The Godot app currently reads the Rust API only. It does not read `.dat` files,
and it must not become a second source of truth. The intended shape is:

```text
Rust DB/save/API -> Godot UI/assets/input/audio
```

Useful endpoints consumed by `scripts/cm_api.gd`:

```text
GET /api/health
GET /api/backend-acceptance
GET /api/promotion-control-room-cached
GET /api/runtime-save
GET /api/tables
GET /api/table/<path>
POST /api/runtime-save/tick
POST /api/headless/run
POST /api/headless/campaign
```

Set `CM0102_RS_API` if the backend is not on `http://127.0.0.1:8770`.
