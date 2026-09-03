//! Shared types for GENERATED screen geometry (`gen_screens/`).
//!
//! Each module under `gen_screens/` is emitted by `tools/gen_screen_rs.py`
//! from an exe ground-truth capture (`reports/screen_captures/<slug>.json`,
//! produced by `tools/capture_all_screens.py`). The structs here mirror the
//! capture records field-for-field so no information is lost between the
//! exe capture and the Rust side: what the generator writes is exactly what
//! `to_capture_json()` re-emits, and `tools/diff_screens.py` closes the loop.
//!
//! Hand-written code binds dynamic game state by starting from the generated
//! `GenScreen` and overwriting the fields that depend on state (text, slot
//! values, per-row geometry) — never by re-typing static coordinates.

use crate::widget_pool::{Widget, WidgetDescriptor, KIND_ROOT_HOLDER};

/// One `spawn_area` call (exe FUN_00402B00) — capture `areas[]` record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenArea {
    pub id: u32,
    pub l: i32,
    pub t: i32,
    pub r: i32,
    pub b: i32,
    pub cnt_a: i64,
    pub w_a: i64,
    pub cnt_b: i64,
    pub w_b: i64,
    pub flags: i64,
    pub a10: i64,
    pub parent: i32,
    /// Return address of the spawn_area call (live captures; 0 for
    /// emulation). Attributes chrome: tab-strip backgrounds come from
    /// inside FUN_005d7070, the nav panel from the 0x415acd emitter.
    pub caller: i64,
}

/// One `spawn_widget` call (exe FUN_005D7BD0 / guio ctor 0x549580) —
/// capture `objects[]` record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenObj {
    pub id: u32,
    pub ty: i64,
    pub l: i32,
    pub t: i32,
    pub r: i32,
    pub b: i32,
    pub a6: i64,
    pub a7: i64,
    pub rflags: i64,
    pub col_p: i64,
    pub col_s: i64,
    pub tflags: i64,
    pub font: i64,
    pub tmode: i64,
    /// Raw exe text pointer from the capture. A pointer into the exe's
    /// memory, kept verbatim for provenance; the *string* is dynamic state
    /// bound at render time, not part of the static skeleton.
    pub text_ptr: i64,
    /// Text dereferenced at construction time by the LIVE Frida capture
    /// (tools/frida_live_capture.py). Empty for emulation captures and for
    /// widgets constructed without text. This is a snapshot of one real
    /// game moment — a default/example value for the binding point, not a
    /// static constant.
    pub text: &'static str,
    /// Return address of the constructor call (live captures; 0 for
    /// emulation captures). Attributes each widget to its builder: the
    /// sidebar dispatcher, the tab-strip builder, the nav-bar builder, or
    /// the screen's own draw callback.
    pub caller: i64,
    pub a15: i64,
    pub a16: i64,
    pub a17: i64,
    pub parent: i32,
}

/// One tab-strip builder call (exe FUN_005d7070) — capture `tabs[]` record.
/// Raw builder args; expansion to per-tab rects goes through the ported
/// tab-strip code, not through stored per-tab geometry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenTabStrip {
    pub count: i64,
    pub sel: i64,
    pub top_y_in: i32,
    pub bot_y_in: i32,
    pub split: i64,
    pub p4: i64,
    pub p5: i64,
}

/// One bottom nav-bar builder call (exe FUN_005d75b0) — capture `nav[]`
/// record: raw Back/Next args.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenNav {
    pub back: i64,
    pub next: i64,
}

/// One slot write by the screen's setup fn — capture `slots[]` record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenSlot {
    pub idx: u32,
    pub val: i64,
    pub kind: u32,
}

/// A full captured screen: everything the exe's setup/draw callback created,
/// in creation order within each class.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GenScreen {
    pub name: &'static str,
    pub areas: Vec<GenArea>,
    pub objects: Vec<GenObj>,
    pub tabs: Vec<GenTabStrip>,
    pub nav: Vec<GenNav>,
    pub slots: Vec<GenSlot>,
}

impl GenScreen {
    /// Convert to renderer widgets. Areas become `KIND_ROOT_HOLDER`
    /// widgets, objects keep their exe type code as the widget kind.
    /// Text is left empty — it is a dynamic-state binding point.
    pub fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = Vec::with_capacity(self.areas.len() + self.objects.len());
        for a in &self.areas {
            let mut d = WidgetDescriptor::empty();
            // `kind` = widget flags dword. Areas carry the KIND_ROOT_HOLDER
            // bit OR-ed with any additional flags from the capture.
            d.kind = KIND_ROOT_HOLDER | a.flags as u32;
            let mut w = Widget::default();
            w.descriptor = d;
            w.left = a.l;
            w.top = a.t;
            w.right = a.r;
            w.bottom = a.b;
            w.flags = a.flags as u32;
            out.push(w);
        }
        for o in &self.objects {
            let mut d = WidgetDescriptor::empty();
            d.kind = (o.ty as u32) | (o.rflags as u32);
            d.text_style = o.font as u32;
            d.text = o.text.to_string();
            let mut w = Widget::default();
            w.descriptor = d;
            w.left = o.l;
            w.top = o.t;
            w.right = o.r;
            w.bottom = o.b;
            w.flags = o.rflags as u32;
            out.push(w);
        }
        out
    }

    /// Re-emit the capture JSON shape (`tools/capture_all_screens.py`
    /// field names), full fidelity, for `tools/diff_screens.py`.
    pub fn to_capture_json(&self) -> String {
        let mut s = String::new();
        s.push_str("{\n");
        s.push_str(&format!(" \"screen\": {:?},\n", self.name));

        s.push_str(" \"areas\": [\n");
        for (i, a) in self.areas.iter().enumerate() {
            s.push_str(&format!(
                "  {{ \"id\": {}, \"L\": {}, \"T\": {}, \"R\": {}, \"B\": {}, \
                 \"cntA\": {}, \"wA\": {}, \"cntB\": {}, \"wB\": {}, \
                 \"flags\": {}, \"a10\": {}, \"parent\": {} }}{}\n",
                a.id, a.l, a.t, a.r, a.b, a.cnt_a, a.w_a, a.cnt_b, a.w_b,
                a.flags, a.a10, a.parent,
                if i + 1 == self.areas.len() { "" } else { "," }
            ));
        }
        s.push_str(" ],\n \"objects\": [\n");
        for (i, o) in self.objects.iter().enumerate() {
            s.push_str(&format!(
                "  {{ \"id\": {}, \"type\": {}, \"L\": {}, \"T\": {}, \"R\": {}, \"B\": {}, \
                 \"a6\": {}, \"a7\": {}, \"rflags\": {}, \"colP\": {}, \"colS\": {}, \
                 \"tflags\": {}, \"font\": {}, \"tmode\": {}, \"text\": {}, \
                 \"a15\": {}, \"a16\": {}, \"a17\": {}, \"parent\": {} }}{}\n",
                o.id, o.ty, o.l, o.t, o.r, o.b, o.a6, o.a7, o.rflags,
                o.col_p, o.col_s, o.tflags, o.font, o.tmode, o.text_ptr,
                o.a15, o.a16, o.a17, o.parent,
                if i + 1 == self.objects.len() { "" } else { "," }
            ));
        }
        s.push_str(" ],\n \"tabs\": [\n");
        for (i, t) in self.tabs.iter().enumerate() {
            s.push_str(&format!(
                "  {{ \"count\": {}, \"sel\": {}, \"top_y_in\": {}, \"bot_y_in\": {}, \
                 \"split\": {}, \"p4\": {}, \"p5\": {} }}{}\n",
                t.count, t.sel, t.top_y_in, t.bot_y_in, t.split, t.p4, t.p5,
                if i + 1 == self.tabs.len() { "" } else { "," }
            ));
        }
        s.push_str(" ],\n \"nav\": [],\n \"slots\": [\n");
        for (i, sl) in self.slots.iter().enumerate() {
            s.push_str(&format!(
                "  {{ \"idx\": {}, \"val\": {}, \"kind\": {} }}{}\n",
                sl.idx, sl.val, sl.kind,
                if i + 1 == self.slots.len() { "" } else { "," }
            ));
        }
        s.push_str(" ]\n}\n");
        s
    }
}
