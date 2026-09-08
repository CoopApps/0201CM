// staging file — content replaces lines 13932..14007 in lib.rs
    /// Populate `World.squad_numbers` from every staff-with-employer's
    /// type10 record. Direct port of the exe's boot-time copy loop in
    /// FUN_00842f40 (called from FUN_008120d0 at boot for every
    /// playable club):
    ///
    /// ```text
    ///   b = *(char*)((player->type10) + 4);   // preferred number
    ///   if (b < 0)      contract->+0x3a = 0;
    ///   else if (b < 51) contract->+0x3a = b;
    ///   else            contract->+0x3a = 50;
    /// ```
    ///
    /// The byte at `type10 + 4` (our `flags_byte_04` field) IS the
    /// player's preferred squad number — NOT a CA-anchored ranking
    /// pass as an earlier port guess had it. Cross-checked against
    /// Cheltenham Squad Number capture: Steve Jones 20, Jackson 18,
    /// Duff M 2, Duff S 24 all match the exe verbatim.
    ///
    /// Zero-value slots stay 0 and paint as an empty cell — the exe
    /// only fills them via the interactive "Submit Squad Numbers"
    /// panel (FUN_0047ea60 lines 854-875), not a boot pass.
    pub fn assign_squad_numbers(&mut self) {
        use std::collections::BTreeMap;
        let has_club: BTreeMap<u32, ()> = self.staff.type6.iter()
            .filter(|p| p.current_club_id().is_some())
            .map(|p| (p.id, ()))
            .collect();
        let type10_owner: BTreeMap<u32, u32> = self.staff.type6.iter()
            .filter_map(|p| {
                let pv = crate::typed_records::PlayerView::from_split(p.id, &p.body);
                let link = pv.player_data_id().map(|l| l as u32).unwrap_or(p.id);
                Some((link, p.id))
            })
            .collect();
        for attr in &self.staff.type10 {
            let Some(person_id) = type10_owner.get(&attr.id).copied() else { continue; };
            if !has_club.contains_key(&person_id) { continue; }
            let raw = attr.flags_byte_04 as i8;
            let n = if raw < 0 { 0 } else if raw <= 50 { raw as u8 } else { 50 };
            self.squad_numbers.insert(attr.id, n);
        }
    }
