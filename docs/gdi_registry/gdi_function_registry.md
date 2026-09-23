# GDI function registry (generated — do not hand-edit)

Regenerate: `python tools/gdi_registry/build_registry.py`. Curated rows in `tools/gdi_registry/data/*.csv`; see docs/reverse_engineering_conventions.md.

5031 functions registered. Coverage: docs/gdi_registry/gdi_function_coverage.md. Globals: docs/gdi_registry/gdi_globals.md.


## GUI

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00401000 | african_nations.cpp | FUN_00401000 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00401250 | african_nations.cpp | FUN_00401250 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004019f0 | african_nations.cpp | FUN_004019f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00401bc0 | african_nations.cpp | FUN_00401bc0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00401ee0 | african_nations.cpp | FUN_00401ee0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004020d0 | african_nations.cpp | FUN_004020d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00402300 | african_nations.cpp | FUN_00402300 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x004025e0 | african_nations.cpp | FUN_004025e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00402d50 | african_nations.cpp | FUN_00402d50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00402dc0 | african_nations.cpp | FUN_00402dc0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004031e0 | area.cpp | FUN_004031e0 | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x00403cc0 | area.cpp | FUN_00403cc0 | PORTED_BEHAVIOURAL | crates/cm-widget/src/lib.rs | YES | UNVERIFIED |  |
| 0x00404290 | arg_prm.cpp | FUN_00404290 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004044f0 | arg_prm.cpp | FUN_004044f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00405580 | arg_prm.cpp | FUN_00405580 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00405990 | arg_prm.cpp | FUN_00405990 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00405f30 | arg_prm.cpp | FUN_00405f30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00406910 | arg_prm.cpp | FUN_00406910 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00406c30 | arg_second.cpp | FUN_00406c30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00406e80 | arg_second.cpp | FUN_00406e80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00407d10 | arg_second.cpp | FUN_00407d10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00408210 | arg_second.cpp | FUN_00408210 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00408690 | arg_second.cpp | FUN_00408690 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00408970 | arg_second.cpp | FUN_00408970 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00409450 | arg_second.cpp | FUN_00409450 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040a040 | arg_second.cpp | FUN_0040a040 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040a3b0 | arg_second.cpp | FUN_0040a3b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040a440 | argentina_awards.cpp | FUN_0040a440 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040a5e0 | argentina_awards.cpp | FUN_0040a5e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040a680 | argentina_rules.cpp | FUN_0040a680 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040a830 | argentina_rules.cpp | FUN_0040a830 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040ab40 | asia_club_champ.cpp | FUN_0040ab40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040aef0 | asia_club_champ.cpp | FUN_0040aef0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040b680 | asia_club_champ.cpp | FUN_0040b680 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040b9e0 | asia_club_champ.cpp | FUN_0040b9e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040bec0 | asia_club_champ.cpp | FUN_0040bec0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040c110 | asia_club_champ.cpp | FUN_0040c110 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040c3d0 | asia_club_champ.cpp | FUN_0040c3d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040c720 | asia_club_champ.cpp | FUN_0040c720 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040cbe0 | asia_cup_winner.cpp | FUN_0040cbe0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040cf90 | asia_cup_winner.cpp | FUN_0040cf90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040d4d0 | asia_cup_winner.cpp | FUN_0040d4d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040d800 | asia_cup_winner.cpp | FUN_0040d800 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040dd10 | asia_cup_winner.cpp | FUN_0040dd10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040df50 | asia_cup_winner.cpp | FUN_0040df50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040e2d0 | asia_cup_winner.cpp | FUN_0040e2d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040e780 | asia_nations.cpp | FUN_0040e780 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040e960 | asia_nations.cpp | FUN_0040e960 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040f2f0 | asia_nations.cpp | FUN_0040f2f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040f7c0 | asia_nations.cpp | FUN_0040f7c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040f9b0 | asia_nations.cpp | FUN_0040f9b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040fd10 | asia_nations.cpp | FUN_0040fd10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00410090 | asia_nations.cpp | FUN_00410090 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00410570 | asia_super_cup.cpp | FUN_00410570 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00410830 | asia_super_cup.cpp | FUN_00410830 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00410970 | asia_super_cup.cpp | FUN_00410970 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00410d70 | aus_nsl.cpp | FUN_00410d70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00410fc0 | aus_nsl.cpp | FUN_00410fc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00411e30 | aus_nsl.cpp | FUN_00411e30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004122a0 | aus_nsl.cpp | FUN_004122a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00412550 | aus_nsl.cpp | FUN_00412550 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00412980 | australia_awards.cpp | FUN_00412980 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00412ac0 | australia_awards.cpp | FUN_00412ac0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004130c0 | australia_rules.cpp | FUN_004130c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00413280 | australia_rules.cpp | FUN_00413280 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00413340 | australia_rules.cpp | FUN_00413340 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00413850 | australia_rules.cpp | FUN_00413850 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004139f0 | australia_rules.cpp | FUN_004139f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00415010 | award_screens.cpp | FUN_00415010 | PORTED_BEHAVIOURAL | crates/cm-domain/src/menu.rs;crates/cm-domain/src/screen_batch5.rs | YES | UNVERIFIED |  |
| 0x00415c30 | award_screens.cpp | FUN_00415c30 | PORTED_BEHAVIOURAL | crates/cm-ui-app/src/screens.rs | YES | UNVERIFIED |  |
| 0x004176e0 | award_screens.cpp | FUN_004176e0 | PORTED_BEHAVIOURAL | crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x00417780 | award_screens.cpp | FUN_00417780 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch10.rs | YES | UNVERIFIED |  |
| 0x00418770 | award_shortlist.cpp | FUN_00418770 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x00418ca0 | award_shortlist.cpp | FUN_00418ca0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch10.rs | YES | UNVERIFIED |  |
| 0x0041cb00 | background.cpp | FUN_0041cb00 | PORTED_BEHAVIOURAL | crates/cm-render/src/screen_wire_batch3.rs | YES | UNVERIFIED |  |
| 0x0041cda0 | background.cpp | FUN_0041cda0 | PORTED_BEHAVIOURAL | crates/cm-render/src/screen_wire_batch3.rs | YES | UNVERIFIED |  |
| 0x0041dba0 | bel_fa_cup.cpp | FUN_0041dba0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0041e420 | bel_fa_cup.cpp | FUN_0041e420 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0041e890 | bel_first.cpp | FUN_0041e890 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0041ead0 | bel_first.cpp | FUN_0041ead0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0041f400 | bel_first.cpp | FUN_0041f400 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0041f7e0 | bel_first.cpp | FUN_0041f7e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0041fb80 | bel_second.cpp | FUN_0041fb80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0041fdd0 | bel_second.cpp | FUN_0041fdd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004208c0 | bel_second.cpp | FUN_004208c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00420d80 | bel_second.cpp | FUN_00420d80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00421020 | bel_second.cpp | FUN_00421020 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004217f0 | bel_second.cpp | FUN_004217f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00421a40 | bel_super.cpp | FUN_00421a40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00421d00 | bel_super.cpp | FUN_00421d00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00421e40 | bel_super.cpp | FUN_00421e40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00422240 | bel_third.cpp | FUN_00422240 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00422490 | bel_third.cpp | FUN_00422490 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00423030 | bel_third.cpp | FUN_00423030 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004234c0 | bel_third.cpp | FUN_004234c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00423840 | bel_third.cpp | FUN_00423840 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00423e60 | bel_third.cpp | FUN_00423e60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00424260 | bel_third.cpp | FUN_00424260 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004243d0 | bel_third.cpp | FUN_004243d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004246f0 | bel_third.cpp | FUN_004246f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00424b80 | bel_third.cpp | FUN_00424b80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00424dc0 | bel_third.cpp | FUN_00424dc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00424f60 | bel_third.cpp | FUN_00424f60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00424ff0 | belgium_awards.cpp | FUN_00424ff0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00425670 | belgium_awards.cpp | FUN_00425670 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004256e0 | belgium_rules.cpp | FUN_004256e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00425e70 | belgium_rules.cpp | FUN_00425e70 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch10.rs | YES | UNVERIFIED |  |
| 0x00425ed0 | belgium_rules.cpp | FUN_00425ed0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004265e0 | bra_champ_cup.cpp | FUN_004265e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00426790 | bra_champ_cup.cpp | FUN_00426790 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00426d70 | bra_champ_cup.cpp | FUN_00426d70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004270b0 | bra_champ_cup.cpp | FUN_004270b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00427360 | bra_champ_cup.cpp | FUN_00427360 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00427f60 | bra_champ_cup.cpp | FUN_00427f60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00428070 | bra_cup.cpp | FUN_00428070 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00428340 | bra_cup.cpp | FUN_00428340 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00428860 | bra_cup.cpp | FUN_00428860 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00428b40 | bra_cup.cpp | FUN_00428b40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042a490 | bra_nat_first.cpp | FUN_0042a490 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042a6d0 | bra_nat_first.cpp | FUN_0042a6d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042b0c0 | bra_nat_first.cpp | FUN_0042b0c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042b620 | bra_nat_first.cpp | FUN_0042b620 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042bd70 | bra_nat_first.cpp | FUN_0042bd70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042c160 | bra_nat_first.cpp | FUN_0042c160 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042c3a0 | bra_nat_first.cpp | FUN_0042c3a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042ca80 | bra_nat_second.cpp | FUN_0042ca80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042ccc0 | bra_nat_second.cpp | FUN_0042ccc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042d6b0 | bra_nat_second.cpp | FUN_0042d6b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042daa0 | bra_nat_second.cpp | FUN_0042daa0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042df00 | bra_nat_third.cpp | FUN_0042df00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042e180 | bra_nat_third.cpp | FUN_0042e180 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042e8d0 | bra_nat_third.cpp | FUN_0042e8d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042f070 | bra_nat_third.cpp | FUN_0042f070 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042f340 | bra_nat_third.cpp | FUN_0042f340 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042f5c0 | bra_nat_third.cpp | FUN_0042f5c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042fac0 | bra_nat_third.cpp | FUN_0042fac0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042fce0 | bra_nat_third.cpp | FUN_0042fce0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042ff10 | bra_reg_bahia.cpp | FUN_0042ff10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00430150 | bra_reg_bahia.cpp | FUN_00430150 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00430950 | bra_reg_bahia.cpp | FUN_00430950 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00430c90 | bra_reg_bahia.cpp | FUN_00430c90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00430e60 | bra_reg_bahia.cpp | FUN_00430e60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00431030 | bra_reg_central.cpp | FUN_00431030 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00431270 | bra_reg_central.cpp | FUN_00431270 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00431a70 | bra_reg_central.cpp | FUN_00431a70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00431e50 | bra_reg_central.cpp | FUN_00431e50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00432190 | bra_reg_central.cpp | FUN_00432190 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004323a0 | bra_reg_gaucho.cpp | FUN_004323a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004325e0 | bra_reg_gaucho.cpp | FUN_004325e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00432db0 | bra_reg_gaucho.cpp | FUN_00432db0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004330f0 | bra_reg_gaucho.cpp | FUN_004330f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004333e0 | bra_reg_gaucho.cpp | FUN_004333e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00433550 | bra_reg_goias.cpp | FUN_00433550 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00433790 | bra_reg_goias.cpp | FUN_00433790 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00433f40 | bra_reg_goias.cpp | FUN_00433f40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00434330 | bra_reg_goias.cpp | FUN_00434330 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00434540 | bra_reg_goias.cpp | FUN_00434540 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004346f0 | bra_reg_minas_gerais.cpp | FUN_004346f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00434930 | bra_reg_minas_gerais.cpp | FUN_00434930 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004351b0 | bra_reg_minas_gerais.cpp | FUN_004351b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00435510 | bra_reg_minas_gerais.cpp | FUN_00435510 | PORTED_BEHAVIOURAL | crates/cm-domain/src/year_end_statuses.rs | YES | UNVERIFIED |  |
| 0x00435950 | bra_reg_minas_gerais.cpp | FUN_00435950 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00435b20 | bra_reg_minas_gerais.cpp | FUN_00435b20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00435cb0 | bra_reg_north.cpp | FUN_00435cb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00435ef0 | bra_reg_north.cpp | FUN_00435ef0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004366f0 | bra_reg_north.cpp | FUN_004366f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00436a30 | bra_reg_north.cpp | FUN_00436a30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00436bf0 | bra_reg_north.cpp | FUN_00436bf0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00436d60 | bra_reg_northeast.cpp | FUN_00436d60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00436fa0 | bra_reg_northeast.cpp | FUN_00436fa0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00437700 | bra_reg_northeast.cpp | FUN_00437700 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00437a40 | bra_reg_northeast.cpp | FUN_00437a40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00437c00 | bra_reg_northeast.cpp | FUN_00437c00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00437d70 | bra_reg_parana.cpp | FUN_00437d70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00437fb0 | bra_reg_parana.cpp | FUN_00437fb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00438830 | bra_reg_parana.cpp | FUN_00438830 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00438c30 | bra_reg_parana.cpp | FUN_00438c30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00438f90 | bra_reg_parana.cpp | FUN_00438f90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00439160 | bra_reg_parana.cpp | FUN_00439160 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004392f0 | bra_reg_pern.cpp | FUN_004392f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00439530 | bra_reg_pern.cpp | FUN_00439530 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00439d30 | bra_reg_pern.cpp | FUN_00439d30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043a110 | bra_reg_pern.cpp | FUN_0043a110 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043a4d0 | bra_reg_pern.cpp | FUN_0043a4d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043a640 | bra_reg_rio.cpp | FUN_0043a640 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043a880 | bra_reg_rio.cpp | FUN_0043a880 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043b100 | bra_reg_rio.cpp | FUN_0043b100 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043b500 | bra_reg_rio.cpp | FUN_0043b500 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043b720 | bra_reg_rio.cpp | FUN_0043b720 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043b950 | bra_reg_rio.cpp | FUN_0043b950 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043bae0 | bra_reg_santa.cpp | FUN_0043bae0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043bd40 | bra_reg_santa.cpp | FUN_0043bd40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043c540 | bra_reg_santa.cpp | FUN_0043c540 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043c920 | bra_reg_santa.cpp | FUN_0043c920 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043cae0 | bra_reg_santa.cpp | FUN_0043cae0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043cc50 | bra_reg_sp.cpp | FUN_0043cc50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043ce90 | bra_reg_sp.cpp | FUN_0043ce90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043d5f0 | bra_reg_sp.cpp | FUN_0043d5f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043d6f0 | bra_reg_sp.cpp | FUN_0043d6f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043d9d0 | bra_reg_sp.cpp | FUN_0043d9d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043db90 | bra_reg_sp.cpp | FUN_0043db90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043dd00 | bra_reg_sp.cpp | FUN_0043dd00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043dd90 | brazil_awards.cpp | FUN_0043dd90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043fb30 | brazil_rules.cpp | FUN_0043fb30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0043fc60 | brazil_rules.cpp | FUN_0043fc60 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0043feb0 | brazil_rules.cpp | FUN_0043feb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0043ff10 | brazil_rules.cpp | FUN_0043ff10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0043ff30 | brazil_rules.cpp | FUN_0043ff30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0043ffd0 | brazil_rules.cpp | FUN_0043ffd0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch13.rs | YES | UNVERIFIED |  |
| 0x00440070 | cash.cpp | FUN_00440070 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00440170 | cash.cpp | FUN_00440170 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00440240 | cash.cpp | FUN_00440240 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00441f10 | cash.cpp | FUN_00441f10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00442500 | cash.cpp | FUN_00442500 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00442fd0 | cash.cpp | FUN_00442fd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00443710 | club_history.cpp | FUN_00443710 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00443770 | club_history.cpp | FUN_00443770 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00443c70 | club_history.cpp | FUN_00443c70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00443e10 | club_history.cpp | FUN_00443e10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00444060 | club_history.cpp | FUN_00444060 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00444310 | club_history.cpp | FUN_00444310 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004445c0 | club_history.cpp | FUN_004445c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00444840 | club_history.cpp | FUN_00444840 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00444a80 | club_history.cpp | FUN_00444a80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00448170 | club_records.cpp | FUN_00448170 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x00448d90 | club_records.cpp | FUN_00448d90 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch30.rs | YES | UNVERIFIED |  |
| 0x00449710 | club_records.cpp | FUN_00449710 | PORTED_BEHAVIOURAL | crates/cm-domain/src/club_season_records.rs | YES | UNVERIFIED |  |
| 0x00453360 | club_records.cpp | FUN_00453360 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x00454bb0 | club_screens.cpp | FUN_00454bb0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_nation_dashboard.rs;crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x00457080 | club_screens.cpp | FUN_00457080 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x00457200 | club_screens.cpp | FUN_00457200 | UNKNOWN | crates/cm-render/src/screen_club_squad_faithful.rs;crates/cm-ui-app/src/render_new.rs | INDIRECT | UNVERIFIED |  |
| 0x00460820 | club_screens.cpp | FUN_00460820 | PORTED_BEHAVIOURAL | crates/cm-render/src/screen_club_fixtures_faithful.rs;crates/cm-ui-app/src/main.rs | YES | UNVERIFIED |  |
| 0x0046a6a0 | club_screens.cpp | FUN_0046a6a0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch28.rs | YES | UNVERIFIED |  |
| 0x0046a6f0 | club_screens.cpp | FUN_0046a6f0 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x0046ad30 | club_screens.cpp | FUN_0046ad30 | UNKNOWN | crates/cm-domain/src/screen_batch10.rs | INDIRECT | UNVERIFIED |  |
| 0x0046ba80 | club_screens.cpp | FUN_0046ba80 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch6.rs;crates/cm-render/src/view_render.rs | YES | UNVERIFIED |  |
| 0x00470bf0 | club_screens.cpp | FUN_00470bf0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch10.rs | YES | UNVERIFIED |  |
| 0x00472bf0 | club_screens.cpp | FUN_00472bf0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch10.rs | YES | UNVERIFIED |  |
| 0x00475870 | club_screens.cpp | FUN_00475870 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch11.rs | YES | UNVERIFIED |  |
| 0x00476dc0 | club_screens.cpp | FUN_00476dc0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch28.rs | YES | UNVERIFIED |  |
| 0x00476df0 | club_screens.cpp | FUN_00476df0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch6.rs;crates/cm-render/src/view_render.rs | YES | UNVERIFIED |  |
| 0x004776a0 | club_screens.cpp | FUN_004776a0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch11.rs;crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x00477dd0 | club_screens.cpp | FUN_00477dd0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch11.rs;crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x00478580 | club_screens.cpp | FUN_00478580 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch11.rs | YES | UNVERIFIED |  |
| 0x004787f0 | club_screens.cpp | FUN_004787f0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch11.rs;crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x00478a30 | club_screens.cpp | FUN_00478a30 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch12.rs;crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x00478ca0 | club_screens.cpp | FUN_00478ca0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch11.rs | YES | UNVERIFIED |  |
| 0x00480130 | club_screens.cpp | FUN_00480130 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch12.rs | YES | UNVERIFIED |  |
| 0x00487210 | club_screens.cpp | FUN_00487210 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch9.rs;crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x00487670 | club_screens.cpp | FUN_00487670 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x0048f800 | coach.cpp | FUN_0048f800 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00490220 | coach.cpp | FUN_00490220 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00490410 | coach.cpp | FUN_00490410 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch12.rs | YES | UNVERIFIED |  |
| 0x00490530 | coach.cpp | FUN_00490530 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00490f00 | coach.cpp | FUN_00490f00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00491040 | coach.cpp | FUN_00491040 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00491830 | comp.cpp | FUN_00491830 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00491b70 | comp.cpp | FUN_00491b70 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch28.rs | YES | UNVERIFIED |  |
| 0x00491c20 | comp.cpp | FUN_00491c20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00491d80 | comp.cpp | FUN_00491d80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00491e70 | comp.cpp | FUN_00491e70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004924f0 | comp.cpp | FUN_004924f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00492640 | comp.cpp | FUN_00492640 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x004926e0 | comp.cpp | FUN_004926e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00492850 | comp.cpp | FUN_00492850 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00492b30 | comp.cpp | FUN_00492b30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00492d50 | comp.cpp | FUN_00492d50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00492f00 | comp.cpp | FUN_00492f00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004933b0 | comp.cpp | FUN_004933b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00493470 | comp.cpp | FUN_00493470 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00493650 | comp.cpp | FUN_00493650 | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x004937b0 | comp.cpp | FUN_004937b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00493ac0 | comp.cpp | FUN_00493ac0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00493cb0 | comp.cpp | FUN_00493cb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00493e10 | comp.cpp | FUN_00493e10 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch5.rs;crates/cm-render/src/dispatcher.rs;crates/cm-render/src/view_render.rs | YES | UNVERIFIED |  |
| 0x00494250 | comp.cpp | FUN_00494250 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch6.rs;crates/cm-render/src/view_render.rs | YES | UNVERIFIED |  |
| 0x00495ad0 | comp_screens.cpp | FUN_00495ad0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00497520 | comp_screens.cpp | FUN_00497520 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00498f50 | comp_screens.cpp | FUN_00498f50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00499880 | comp_screens.cpp | FUN_00499880 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00499dd0 | comp_screens.cpp | FUN_00499dd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0049a0d0 | comp_screens.cpp | FUN_0049a0d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0049ae10 | comp_screens.cpp | FUN_0049ae10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0049b3a0 | comp_screens.cpp | FUN_0049b3a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0049c9a0 | comp_screens.cpp | FUN_0049c9a0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch12.rs | YES | UNVERIFIED |  |
| 0x0049d260 | comp_screens.cpp | FUN_0049d260 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0049d280 | comp_screens.cpp | FUN_0049d280 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0049d720 | comp_screens.cpp | FUN_0049d720 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0049dc50 | comp_screens.cpp | FUN_0049dc50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0049de70 | comp_screens.cpp | FUN_0049de70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0049e0d0 | comp_screens.cpp | FUN_0049e0d0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch12.rs | YES | UNVERIFIED |  |
| 0x0049f410 | comp_screens.cpp | FUN_0049f410 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0049fa10 | comp_screens.cpp | FUN_0049fa10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004a0010 | comp_screens.cpp | FUN_004a0010 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch30.rs | YES | UNVERIFIED |  |
| 0x004a0610 | comp_screens.cpp | FUN_004a0610 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch30.rs | YES | UNVERIFIED |  |
| 0x004a2900 | comp_screens.cpp | FUN_004a2900 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch30.rs;crates/cm-render/src/dispatcher.rs;crates/cm-render/src/screen_wire_batch3.rs | YES | UNVERIFIED |  |
| 0x004a3440 | comp_screens.cpp | FUN_004a3440 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004a5550 | comp_screens.cpp | FUN_004a5550 | PORTED_BEHAVIOURAL | crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x004a6030 | comp_stats.cpp | FUN_004a6030 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch31.rs | YES | UNVERIFIED |  |
| 0x004a8710 | comp_stats.cpp | FUN_004a8710 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch32.rs | YES | UNVERIFIED |  |
| 0x004aa9a0 | comp_stats.cpp | FUN_004aa9a0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch32.rs;crates/cm-domain/src/screen_batch33.rs | YES | UNVERIFIED |  |
| 0x004ab310 | comp_stats.cpp | FUN_004ab310 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch29.rs;crates/cm-domain/src/screen_batch30.rs;crates/cm-domain/src/screen_batch32.rs;crates/cm-domain/src/screen_batch33.rs | YES | UNVERIFIED |  |
| 0x004ab4f0 | comp_stats.cpp | FUN_004ab4f0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch32.rs;crates/cm-domain/src/screen_batch33.rs | YES | UNVERIFIED |  |
| 0x004ab5e0 | comp_stats.cpp | FUN_004ab5e0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch32.rs;crates/cm-domain/src/screen_batch33.rs | YES | UNVERIFIED |  |
| 0x004ab880 | comp_stats.cpp | FUN_004ab880 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch32.rs;crates/cm-domain/src/screen_batch33.rs | YES | UNVERIFIED |  |
| 0x004abff0 | comp_stats.cpp | FUN_004abff0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch32.rs | YES | UNVERIFIED |  |
| 0x004ac150 | comp_stats.cpp | FUN_004ac150 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch32.rs;crates/cm-domain/src/screen_batch34.rs;crates/cm-domain/src/screen_batch35.rs | YES | UNVERIFIED |  |
| 0x004ae9e0 | comp_stats.cpp | FUN_004ae9e0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch34.rs | YES | UNVERIFIED |  |
| 0x004aea10 | comp_stats.cpp | FUN_004aea10 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs;crates/cm-domain/src/screen_batch35.rs | YES | UNVERIFIED |  |
| 0x004b3970 | comp_stats.cpp | FUN_004b3970 | PORTED_BEHAVIOURAL | crates/cm-domain/src/african_nations.rs | YES | UNVERIFIED |  |
| 0x004b4140 | comp_stats.cpp | FUN_004b4140 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch30.rs | YES | UNVERIFIED |  |
| 0x004b42b0 | comp_stats.cpp | FUN_004b42b0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch30.rs | YES | UNVERIFIED |  |
| 0x004b5050 | comp_stats.cpp | FUN_004b5050 | PORTED_BEHAVIOURAL | crates/cm-render/src/screen_wire_batch3.rs | YES | UNVERIFIED |  |
| 0x004b5080 | comp_stats.cpp | FUN_004b5080 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch15.rs | YES | UNVERIFIED |  |
| 0x004b5120 | comp_text.cpp | FUN_004b5120 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch34.rs;crates/cm-domain/src/screen_batch35.rs | YES | UNVERIFIED |  |
| 0x004b5360 | comp_text.cpp | FUN_004b5360 | UNKNOWN | crates/cm-domain/src/screen_batch34.rs;crates/cm-domain/src/screen_batch35.rs | INDIRECT | UNVERIFIED |  |
| 0x004b5e70 | comp_text.cpp | FUN_004b5e70 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch35.rs | YES | UNVERIFIED |  |
| 0x004b5eb0 | comp_text.cpp | FUN_004b5eb0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/club_season_records.rs | YES | UNVERIFIED |  |
| 0x004b61b0 | comp_util.cpp | FUN_004b61b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004b6600 | comp_util.cpp | FUN_004b6600 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004b6710 | comp_util.cpp | FUN_004b6710 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004b6b30 | comp_util.cpp | FUN_004b6b30 | PORTED_BEHAVIOURAL | crates/cm-domain/src/club_season_records.rs;crates/cm-domain/src/screen_batch30.rs;crates/cm-domain/src/screen_batch31.rs;crates/cm-domain/src/screen_batch32.rs | YES | UNVERIFIED |  |
| 0x004b6c50 | comp_util.cpp | FUN_004b6c50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004b6c70 | comp_util.cpp | FUN_004b6c70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004b6f80 | comp_util.cpp | FUN_004b6f80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004ba820 | comp_util.cpp | FUN_004ba820 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004ba990 | comp_util.cpp | FUN_004ba990 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004bac50 | comp_util.cpp | FUN_004bac50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004c5d30 | comp_util.cpp | FUN_004c5d30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004c5e30 | comp_util.cpp | FUN_004c5e30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004c6370 | comp_util.cpp | FUN_004c6370 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch35.rs | YES | UNVERIFIED |  |
| 0x004c63c0 | comp_util.cpp | FUN_004c63c0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch28.rs | YES | UNVERIFIED |  |
| 0x004c6540 | comp_util.cpp | FUN_004c6540 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004c6920 | comp_util.cpp | FUN_004c6920 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004c6b20 | comp_util.cpp | FUN_004c6b20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004c6cf0 | comp_util.cpp | FUN_004c6cf0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004c6f50 | comp_util.cpp | FUN_004c6f50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004c7040 | con_champ.cpp | FUN_004c7040 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004c7300 | con_champ.cpp | FUN_004c7300 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004c75a0 | con_champ.cpp | FUN_004c75a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004c7ac0 | con_merc_cup.cpp | FUN_004c7ac0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004c7c70 | con_merc_cup.cpp | FUN_004c7c70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004c81b0 | con_merc_cup.cpp | FUN_004c81b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004c8510 | con_merc_cup.cpp | FUN_004c8510 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004c8700 | con_merc_cup.cpp | FUN_004c8700 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004c8930 | con_merc_cup.cpp | FUN_004c8930 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004c94d0 | conmebol_liber.cpp | FUN_004c94d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004c9b70 | conmebol_liber.cpp | FUN_004c9b70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004c9ee0 | conmebol_liber.cpp | FUN_004c9ee0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004ca0e0 | conmebol_liber.cpp | FUN_004ca0e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004ca310 | conmebol_liber.cpp | FUN_004ca310 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004cac40 | conmebol_merc.cpp | FUN_004cac40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004cb230 | conmebol_merc.cpp | FUN_004cb230 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004cb5b0 | conmebol_merc.cpp | FUN_004cb5b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004cb7b0 | conmebol_merc.cpp | FUN_004cb7b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004cbc30 | conmebol_merc.cpp | FUN_004cbc30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004d08a0 | contract_manager.cpp | FUN_004d08a0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x004d14d0 | contract_manager.cpp | FUN_004d14d0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x004d1680 | contract_manager.cpp | FUN_004d1680 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x004d28c0 | contract_manager.cpp | FUN_004d28c0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x004d2e10 | contract_manager.cpp | FUN_004d2e10 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch24.rs;crates/cm-domain/src/screen_batch26.rs | YES | UNVERIFIED |  |
| 0x004d3300 | contract_manager.cpp | FUN_004d3300 | PORTED_BEHAVIOURAL | crates/cm-domain/src/person_news.rs | YES | UNVERIFIED |  |
| 0x004d3ea0 | contract_manager.cpp | FUN_004d3ea0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x004d4880 | contract_manager.cpp | FUN_004d4880 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x004d59d0 | contract_manager.cpp | FUN_004d59d0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/c14_5_squad_manager.rs;crates/cm-domain/src/c15_1_world_apply.rs;crates/cm-domain/src/screen_batch24.rs;crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x004d5a20 | contract_manager.cpp | FUN_004d5a20 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs;crates/cm-domain/src/screen_batch25.rs;crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x004d5b00 | contract_manager.cpp | FUN_004d5b00 | PORTED_BEHAVIOURAL | crates/cm-domain/src/c14_5_squad_manager.rs;crates/cm-domain/src/c15_1_world_apply.rs | YES | UNVERIFIED |  |
| 0x004d7050 | contract_manager.cpp | FUN_004d7050 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x004dabd0 | contract_manager.cpp | FUN_004dabd0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x004dc980 | contract_manager.cpp | FUN_004dc980 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x004dcf60 | contract_manager.cpp | FUN_004dcf60 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x004e2af0 | contract_manager.cpp | FUN_004e2af0 | PORTED_BEHAVIOURAL | crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x004e2b80 | contract_manager.cpp | FUN_004e2b80 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch13.rs;crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x004e3300 | contract_screens.cpp | FUN_004e3300 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch13.rs;crates/cm-domain/src/screen_batch24.rs | YES | UNVERIFIED |  |
| 0x004e4580 | contract_screens.cpp | FUN_004e4580 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch13.rs | YES | UNVERIFIED |  |
| 0x004e4960 | contract_screens.cpp | FUN_004e4960 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch13.rs | YES | UNVERIFIED |  |
| 0x004e4e50 | contract_screens.cpp | FUN_004e4e50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004e6b80 | contract_screens.cpp | FUN_004e6b80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004e9a10 | contract_screens.cpp | FUN_004e9a10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004ea4d0 | contract_screens.cpp | FUN_004ea4d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004eac70 | contract_screens.cpp | FUN_004eac70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004eb240 | contract_screens.cpp | FUN_004eb240 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch14.rs;crates/cm-render/src/view_render.rs | YES | UNVERIFIED |  |
| 0x004eb330 | contract_screens.cpp | FUN_004eb330 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004ec3a0 | contract_screens.cpp | FUN_004ec3a0 | PORTED_BEHAVIOURAL | crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x004ec550 | contract_screens.cpp | FUN_004ec550 | PORTED_BEHAVIOURAL | crates/cm-domain/src/menu.rs;crates/cm-domain/src/screen_batch14.rs;crates/cm-domain/src/sidebar_dispatcher.rs;crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x004fd1b0 | contract_screens.cpp | FUN_004fd1b0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/menu.rs;crates/cm-domain/src/screen_batch14.rs;crates/cm-domain/src/sidebar_dispatcher.rs | YES | UNVERIFIED |  |
| 0x004fda80 | contract_screens.cpp | FUN_004fda80 | PORTED_BEHAVIOURAL | crates/cm-render/src/screen_wire_batch3.rs | YES | UNVERIFIED |  |
| 0x004fdad0 | cro_a1.cpp | FUN_004fdad0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004fdd10 | cro_a1.cpp | FUN_004fdd10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004fe470 | cro_a1.cpp | FUN_004fe470 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004fe920 | cro_a1.cpp | FUN_004fe920 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004feb90 | cro_a1.cpp | FUN_004feb90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004fee50 | cro_a1.cpp | FUN_004fee50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004feff0 | cro_a2a.cpp | FUN_004feff0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004ff230 | cro_a2a.cpp | FUN_004ff230 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004ff990 | cro_a2a.cpp | FUN_004ff990 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004ffdd0 | cro_a2a.cpp | FUN_004ffdd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00500040 | cro_a2b.cpp | FUN_00500040 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00500280 | cro_a2b.cpp | FUN_00500280 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005009e0 | cro_a2b.cpp | FUN_005009e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00500e20 | cro_a2b.cpp | FUN_00500e20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00501160 | cro_cup.cpp | FUN_00501160 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00501420 | cro_cup.cpp | FUN_00501420 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00501890 | cro_cup.cpp | FUN_00501890 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00501bf0 | cro_cup.cpp | FUN_00501bf0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00501c80 | croatia_awards.cpp | FUN_00501c80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005021e0 | croatia_rules.cpp | FUN_005021e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00502320 | croatia_rules.cpp | FUN_00502320 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00503370 | cup.cpp | FUN_00503370 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00503570 | cup.cpp | FUN_00503570 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00503970 | cup.cpp | FUN_00503970 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00503a70 | cup.cpp | FUN_00503a70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005060f0 | cup.cpp | FUN_005060f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005064e0 | cup.cpp | FUN_005064e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00506870 | cup.cpp | FUN_00506870 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00506fb0 | cup.cpp | FUN_00506fb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005076d0 | cup.cpp | FUN_005076d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00507f00 | cup.cpp | FUN_00507f00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005088c0 | cup.cpp | FUN_005088c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00509570 | cup.cpp | FUN_00509570 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005098e0 | cup.cpp | FUN_005098e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0050b3f0 | cup.cpp | FUN_0050b3f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0050b610 | cup.cpp | FUN_0050b610 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0050bac0 | cup.cpp | FUN_0050bac0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0050bb10 | cup.cpp | FUN_0050bb10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0050bb70 | cup.cpp | FUN_0050bb70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0050bea0 | cup.cpp | FUN_0050bea0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0050c000 | cup.cpp | FUN_0050c000 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0050c5b0 | cup.cpp | FUN_0050c5b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0050c8d0 | cup.cpp | FUN_0050c8d0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x005246e0 | database.cpp | FUN_005246e0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x00525190 | database.cpp | FUN_00525190 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch27.rs;crates/cm-domain/src/tactic_dispatcher.rs;crates/cm-domain/src/typed_records.rs;crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x00525450 | database.cpp | FUN_00525450 | PORTED_BEHAVIOURAL | crates/cm-domain/src/c14_stadium_expansion.rs;crates/cm-domain/src/club_season_records.rs;crates/cm-domain/src/manager_creation.rs;crates/cm-domain/src/screen_batch11.rs;crates/cm-domain/src/screen_batch9.rs;crates/cm-domain/src/screen_club_dashboard.rs;crates/cm-domain/src/transfer.rs;crates/cm-render/src/dispatcher_club_toolbar.rs;crates/cm-render/src/screen_club_squad_faithful.rs | YES | UNVERIFIED |  |
| 0x005265e0 | database.cpp | FUN_005265e0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch9.rs;crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x00527340 | database.cpp | FUN_00527340 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x005276f0 | database.cpp | FUN_005276f0 | UNKNOWN | crates/cm-domain/src/menu.rs;crates/cm-domain/src/screen_batch27.rs;crates/cm-render/src/dispatcher.rs;crates/cm-render/src/screen_wire_batch3.rs | INDIRECT | UNVERIFIED |  |
| 0x0052a330 | database.cpp | FUN_0052a330 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x0052a500 | database.cpp | FUN_0052a500 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs;crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x0052a5a0 | database.cpp | FUN_0052a5a0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/c13_promotion_apply.rs;crates/cm-domain/src/c14_5_squad_manager.rs;crates/cm-domain/src/c15_1_world_apply.rs;crates/cm-domain/src/screen_batch12.rs | YES | UNVERIFIED |  |
| 0x0052dff0 | database.cpp | FUN_0052dff0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch25.rs | YES | UNVERIFIED |  |
| 0x0052e0e0 | database.cpp | FUN_0052e0e0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch25.rs | YES | UNVERIFIED |  |
| 0x00531370 | database.cpp | FUN_00531370 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x005313f0 | database.cpp | FUN_005313f0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x00533cf0 | date.cpp | FUN_00533cf0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00533fd0 | date.cpp | FUN_00533fd0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x00534050 | date.cpp | FUN_00534050 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00534230 | date.cpp | FUN_00534230 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00534530 | date.cpp | FUN_00534530 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00534ed0 | date.cpp | FUN_00534ed0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00535130 | date.cpp | FUN_00535130 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00535ad0 | date.cpp | FUN_00535ad0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00535f90 | date.cpp | FUN_00535f90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00536810 | date.cpp | FUN_00536810 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00536990 | date.cpp | FUN_00536990 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch18.rs;crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x00536b90 | date.cpp | FUN_00536b90 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs;crates/cm-domain/src/screen_batch24.rs;crates/cm-domain/src/screen_batch26.rs;crates/cm-domain/src/screen_transfers.rs | YES | UNVERIFIED |  |
| 0x00536c20 | date.cpp | FUN_00536c20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00536d10 | date.cpp | FUN_00536d10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00536d50 | date.cpp | FUN_00536d50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00537130 | date.cpp | FUN_00537130 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00537190 | date.cpp | FUN_00537190 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005371c0 | db_files.cpp | FUN_005371c0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x00537320 | db_files.cpp | FUN_00537320 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x00537730 | db_files.cpp | FUN_00537730 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x00537870 | db_files.cpp | FUN_00537870 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs;crates/cm-import/src/bin/regen_clubs.rs | YES | UNVERIFIED |  |
| 0x00538210 | db_files.cpp | FUN_00538210 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x00538360 | db_files.cpp | FUN_00538360 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x00538460 | db_files.cpp | FUN_00538460 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x00538c10 | db_files.cpp | FUN_00538c10 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x005397f0 | db_files.cpp | FUN_005397f0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x00539a30 | db_files.cpp | FUN_00539a30 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x00539a90 | db_files.cpp | FUN_00539a90 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x00539f00 | db_files.cpp | FUN_00539f00 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x00539f60 | db_files.cpp | FUN_00539f60 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x0053a1a0 | db_files.cpp | FUN_0053a1a0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x0053a350 | db_files.cpp | FUN_0053a350 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x0053a440 | db_files.cpp | FUN_0053a440 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x0053a590 | den_cup.cpp | FUN_0053a590 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053a850 | den_cup.cpp | FUN_0053a850 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053ae20 | den_cup.cpp | FUN_0053ae20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053b2d0 | den_first.cpp | FUN_0053b2d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053b500 | den_first.cpp | FUN_0053b500 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053be70 | den_first.cpp | FUN_0053be70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053c1c0 | den_prm.cpp | FUN_0053c1c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053c450 | den_prm.cpp | FUN_0053c450 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053d4b0 | den_prm.cpp | FUN_0053d4b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053d890 | den_prm.cpp | FUN_0053d890 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053da60 | den_second.cpp | FUN_0053da60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053dc90 | den_second.cpp | FUN_0053dc90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053e620 | den_second.cpp | FUN_0053e620 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053e9a0 | den_second.cpp | FUN_0053e9a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053ea30 | denmark_awards.cpp | FUN_0053ea30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053f390 | discipline.cpp | FUN_0053f390 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0053f460 | discipline.cpp | FUN_0053f460 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0053f9d0 | discipline.cpp | FUN_0053f9d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0053fe70 | discipline.cpp | FUN_0053fe70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0053fff0 | discipline.cpp | FUN_0053fff0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00540170 | discipline.cpp | FUN_00540170 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005403d0 | discipline.cpp | FUN_005403d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00540430 | discipline.cpp | FUN_00540430 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00540540 | discipline.cpp | FUN_00540540 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00540900 | discipline.cpp | FUN_00540900 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00540a90 | discipline.cpp | FUN_00540a90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00540c00 | discipline.cpp | FUN_00540c00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00540d20 | discipline.cpp | FUN_00540d20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00540e80 | discipline.cpp | FUN_00540e80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005412e0 | discipline.cpp | FUN_005412e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005454b0 | discipline.cpp | FUN_005454b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00546600 | discipline.cpp | FUN_00546600 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005466e0 | discipline.cpp | FUN_005466e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00546720 | discipline.cpp | FUN_00546720 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00546760 | discipline.cpp | FUN_00546760 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00546820 | discipline.cpp | FUN_00546820 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00546ca0 | discipline.cpp | FUN_00546ca0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00546e70 | discipline.cpp | FUN_00546e70 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs;crates/cm-import/src/bin/regen_clubs.rs | YES | UNVERIFIED |  |
| 0x00547020 | discipline.cpp | FUN_00547020 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00547d20 | discipline.cpp | FUN_00547d20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00547e50 | discipline.cpp | FUN_00547e50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00548170 | discipline.cpp | FUN_00548170 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch14.rs;crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x00548bd0 | discipline.cpp | FUN_00548bd0 | UNKNOWN | crates/cm-render/src/scrman.rs | INDIRECT | UNVERIFIED |  |
| 0x00548ef0 | display.cpp | FUN_00548ef0 | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x00549210 | display.cpp | FUN_00549210 | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x005493b0 | display.cpp | FUN_005493b0 | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x00549fd0 | display.cpp | FUN_00549fd0 | PORTED_BEHAVIOURAL | crates/cm-render/src/widget_pool.rs | YES | UNVERIFIED |  |
| 0x0054dc60 | display.cpp | FUN_0054dc60 | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x00554600 | eng_auto_cup.cpp | FUN_00554600 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00554f90 | eng_auto_cup.cpp | FUN_00554f90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00555310 | eng_auto_cup.cpp | FUN_00555310 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00555690 | eng_auto_cup.cpp | FUN_00555690 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005558c0 | eng_auto_cup.cpp | FUN_005558c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00555bf0 | eng_auto_cup.cpp | FUN_00555bf0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00555e80 | eng_cc_cup.cpp | FUN_00555e80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005568a0 | eng_cc_cup.cpp | FUN_005568a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00556f90 | eng_charity.cpp | FUN_00556f90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00557250 | eng_charity.cpp | FUN_00557250 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00557390 | eng_charity.cpp | FUN_00557390 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005577a0 | eng_conf.cpp | FUN_005577a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005579e0 | eng_conf.cpp | FUN_005579e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00558c80 | eng_fa_cup.cpp | FUN_00558c80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005596f0 | eng_fa_cup.cpp | FUN_005596f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055a7d0 | eng_fa_cup.cpp | FUN_0055a7d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055a8f0 | eng_fa_trophy.cpp | FUN_0055a8f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055afa0 | eng_fa_trophy.cpp | FUN_0055afa0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055b340 | eng_first.cpp | FUN_0055b340 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055b580 | eng_first.cpp | FUN_0055b580 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055c8b0 | eng_first.cpp | FUN_0055c8b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055cd40 | eng_first.cpp | FUN_0055cd40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055cf20 | eng_prm.cpp | FUN_0055cf20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055d170 | eng_prm.cpp | FUN_0055d170 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055e5c0 | eng_prm.cpp | FUN_0055e5c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055f280 | eng_second.cpp | FUN_0055f280 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005601d0 | eng_second.cpp | FUN_005601d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00560610 | eng_second.cpp | FUN_00560610 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00560b40 | eng_third.cpp | FUN_00560b40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00560d90 | eng_third.cpp | FUN_00560d90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00561ce0 | eng_third.cpp | FUN_00561ce0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00562130 | eng_third.cpp | FUN_00562130 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00562440 | england_awards.cpp | FUN_00562440 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005637f0 | england_rules.cpp | FUN_005637f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00563da0 | eur_super_cup.cpp | FUN_00563da0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00564060 | eur_super_cup.cpp | FUN_00564060 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005641a0 | eur_super_cup.cpp | FUN_005641a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00564530 | euro_champ.cpp | FUN_00564530 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00564770 | euro_champ.cpp | FUN_00564770 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005651d0 | euro_champ.cpp | FUN_005651d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005655b0 | euro_champ.cpp | FUN_005655b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005657e0 | euro_champ.cpp | FUN_005657e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00565a20 | euro_champ.cpp | FUN_00565a20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00565dc0 | euro_champ.cpp | FUN_00565dc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00565e70 | euro_champ.cpp | FUN_00565e70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00565fc0 | euro_champ.cpp | FUN_00565fc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005663b0 | euro_champ.cpp | FUN_005663b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00566ab0 | euro_champ_qual.cpp | FUN_00566ab0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00566d40 | euro_champ_qual.cpp | FUN_00566d40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00567680 | euro_champ_qual.cpp | FUN_00567680 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00567760 | euro_champ_qual.cpp | FUN_00567760 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00568490 | euro_champ_qual.cpp | FUN_00568490 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005689c0 | euro_champ_qual.cpp | FUN_005689c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00568c00 | euro_champ_qual.cpp | FUN_00568c00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005692b0 | euro_champ_qual.cpp | FUN_005692b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005695d0 | euro_champ_qual.cpp | FUN_005695d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005696c0 | euro_champ_qual.cpp | FUN_005696c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00569a10 | euro_champ_qual.cpp | FUN_00569a10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0056a880 | euro_champ_qual.cpp | FUN_0056a880 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0056cba0 | euro_champ_qual.cpp | FUN_0056cba0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0056d0b0 | euro_champ_qual.cpp | FUN_0056d0b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0056d120 | european_awards.cpp | FUN_0056d120 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00574960 | european_cup.cpp | FUN_00574960 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch15.rs;crates/cm-render/src/view_render.rs | YES | UNVERIFIED |  |
| 0x005750c0 | fifa_confed.cpp | FUN_005750c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00575250 | fifa_confed.cpp | FUN_00575250 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00575a60 | fifa_confed.cpp | FUN_00575a60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00575eb0 | fifa_confed.cpp | FUN_00575eb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00576080 | fifa_confed.cpp | FUN_00576080 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00576260 | fifa_confed.cpp | FUN_00576260 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00576460 | fifa_confed.cpp | FUN_00576460 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00576a60 | fifa_confed.cpp | FUN_00576a60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00576c50 | fifa_confed.cpp | FUN_00576c50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00576d20 | fifa_rankings.cpp | FUN_00576d20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00577550 | fifa_rankings.cpp | FUN_00577550 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00577890 | fifa_rankings.cpp | FUN_00577890 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00577c50 | fifa_rankings.cpp | FUN_00577c50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005792a0 | file_screens.cpp | FUN_005792a0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch15.rs | YES | UNVERIFIED |  |
| 0x0057b9c0 | file_screens.cpp | FUN_0057b9c0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch15.rs;crates/cm-domain/src/screen_batch27.rs;crates/cm-render/src/dispatcher.rs;crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x0057bb30 | file_screens.cpp | FUN_0057bb30 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch15.rs | YES | UNVERIFIED |  |
| 0x0057c470 | fin_cup.cpp | FUN_0057c470 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0057cc40 | fin_cup.cpp | FUN_0057cc40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0057d0c0 | fin_first.cpp | FUN_0057d0c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0057d310 | fin_first.cpp | FUN_0057d310 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0057dc80 | fin_first.cpp | FUN_0057dc80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0057e290 | fin_first.cpp | FUN_0057e290 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0057e820 | fin_first.cpp | FUN_0057e820 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0057eee0 | fin_first.cpp | FUN_0057eee0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0057f200 | fin_first.cpp | FUN_0057f200 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0057f370 | fin_prm.cpp | FUN_0057f370 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0057f620 | fin_prm.cpp | FUN_0057f620 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0057fd70 | fin_prm.cpp | FUN_0057fd70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00580100 | fin_prm.cpp | FUN_00580100 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005829c0 | finance.cpp | FUN_005829c0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch13.rs | YES | UNVERIFIED |  |
| 0x005854d0 | finance.cpp | FUN_005854d0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/c15_1_world_apply.rs | YES | UNVERIFIED |  |
| 0x0058a490 | finance.cpp | FUN_0058a490 | PORTED_BEHAVIOURAL | crates/cm-domain/src/c15_1_world_apply.rs;crates/cm-domain/src/screen_batch25.rs | YES | UNVERIFIED |  |
| 0x0058a550 | find_screens.cpp | FUN_0058a550 | UNKNOWN | crates/cm-domain/src/screen_batch8.rs;crates/cm-domain/src/sidebar_dispatcher.rs;crates/cm-domain/tests/sidebar_dispatcher.rs;crates/cm-render/src/view_render.rs | INDIRECT | UNVERIFIED |  |
| 0x0058b410 | find_screens.cpp | FUN_0058b410 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0058b9b0 | find_screens.cpp | FUN_0058b9b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0058bec0 | find_screens.cpp | FUN_0058bec0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0058c370 | find_screens.cpp | FUN_0058c370 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0058cce0 | find_screens.cpp | FUN_0058cce0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0058cde0 | find_screens.cpp | FUN_0058cde0 | UNKNOWN | crates/cm-domain/src/screen_batch8.rs;crates/cm-domain/src/sidebar_dispatcher.rs;crates/cm-render/src/view_render.rs | INDIRECT | UNVERIFIED |  |
| 0x0058d2c0 | find_screens.cpp | FUN_0058d2c0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x0058d830 | find_screens.cpp | FUN_0058d830 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0058de50 | find_screens.cpp | FUN_0058de50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0058e4d0 | find_screens.cpp | FUN_0058e4d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0058eb30 | find_screens.cpp | FUN_0058eb30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0058eec0 | find_screens.cpp | FUN_0058eec0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0058f170 | find_screens.cpp | FUN_0058f170 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0058f690 | find_screens.cpp | FUN_0058f690 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0058f6f0 | find_screens.cpp | FUN_0058f6f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005928e0 | fine.cpp | FUN_005928e0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch15.rs | YES | UNVERIFIED |  |
| 0x00593620 | finland_awards.cpp | FUN_00593620 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00593760 | finland_awards.cpp | FUN_00593760 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00593fd0 | finland_rules.cpp | FUN_00593fd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00594d00 | fix_man.cpp | FUN_00594d00 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x005962b0 | fix_man.cpp | FUN_005962b0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x00597260 | fix_man.cpp | FUN_00597260 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x0059cdf0 | formation.cpp | FUN_0059cdf0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x0059ceb0 | formation.cpp | FUN_0059ceb0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x0059d240 | formation.cpp | FUN_0059d240 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x0059e2a0 | formation.cpp | FUN_0059e2a0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x0059e5b0 | formation.cpp | FUN_0059e5b0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x0059eab0 | formation.cpp | FUN_0059eab0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x0059f3e0 | formation.cpp | FUN_0059f3e0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_file.rs | YES | UNVERIFIED |  |
| 0x0059f5a0 | formation.cpp | FUN_0059f5a0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x0059fae0 | formation.cpp | FUN_0059fae0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x005a0c10 | formation.cpp | FUN_005a0c10 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x005a16b0 | formation.cpp | FUN_005a16b0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/formation.rs | YES | UNVERIFIED |  |
| 0x005a1890 | formation.cpp | FUN_005a1890 | PORTED_BEHAVIOURAL | crates/cm-domain/src/player_profile.rs | YES | UNVERIFIED |  |
| 0x005a2910 | formation.cpp | FUN_005a2910 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch22.rs | YES | UNVERIFIED |  |
| 0x005a34d0 | fra_cfa.cpp | FUN_005a34d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a4290 | fra_cfa.cpp | FUN_005a4290 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a4390 | fra_cup.cpp | FUN_005a4390 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a4cb0 | fra_cup.cpp | FUN_005a4cb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a5390 | fra_first.cpp | FUN_005a5390 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a5820 | fra_first.cpp | FUN_005a5820 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a64f0 | fra_first.cpp | FUN_005a64f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a69c0 | fra_first.cpp | FUN_005a69c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a6bc0 | fra_lge_cup.cpp | FUN_005a6bc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a73a0 | fra_lge_cup.cpp | FUN_005a73a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a7840 | fra_lower.cpp | FUN_005a7840 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a8470 | fra_lower.cpp | FUN_005a8470 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a8600 | fra_second.cpp | FUN_005a8600 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a8830 | fra_second.cpp | FUN_005a8830 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a91e0 | fra_second.cpp | FUN_005a91e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a95f0 | fra_super.cpp | FUN_005a95f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a98a0 | fra_super.cpp | FUN_005a98a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a9a40 | fra_super.cpp | FUN_005a9a40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a9e10 | fra_third.cpp | FUN_005a9e10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005aa040 | fra_third.cpp | FUN_005aa040 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005aa8a0 | fra_third.cpp | FUN_005aa8a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005aac50 | fra_third.cpp | FUN_005aac50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005aacf0 | france_awards.cpp | FUN_005aacf0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005ab840 | france_awards.cpp | FUN_005ab840 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005ab8b0 | france_rules.cpp | FUN_005ab8b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005af840 | friendly.cpp | FUN_005af840 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x005b09e0 | friendly.cpp | FUN_005b09e0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/club_season_records.rs;crates/cm-domain/src/screen_batch30.rs;crates/cm-domain/src/screen_batch32.rs | YES | UNVERIFIED |  |
| 0x005b0b70 | friendly.cpp | FUN_005b0b70 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch9.rs;crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x005b0be0 | friendly.cpp | FUN_005b0be0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch9.rs;crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x005b0c50 | friendly.cpp | FUN_005b0c50 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x005b0e20 | friendly.cpp | FUN_005b0e20 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x005b11a0 | friendly.cpp | FUN_005b11a0 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x005b1840 | friendly.cpp | FUN_005b1840 | UNKNOWN | crates/cm-render/src/dispatcher_club_toolbar.rs;crates/cm-render/src/screen_club_fixtures_faithful.rs | INDIRECT | UNVERIFIED |  |
| 0x005b2b90 | friendly.cpp | FUN_005b2b90 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch11.rs | YES | UNVERIFIED |  |
| 0x005b2c40 | friendly.cpp | FUN_005b2c40 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch11.rs | YES | UNVERIFIED |  |
| 0x005b6a10 | friendly.cpp | FUN_005b6a10 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher.rs;crates/cm-render/src/msgbox.rs | YES | UNVERIFIED |  |
| 0x005b6a90 | friendly.cpp | FUN_005b6a90 | PORTED_BEHAVIOURAL | crates/cm-domain/src/bin/simulate_season.rs;crates/cm-domain/src/game.rs;crates/cm-domain/src/menu.rs;crates/cm-domain/src/season_roll_scheduler.rs;crates/cm-ui-app/src/main.rs | YES | UNVERIFIED |  |
| 0x005c28d0 | ger_cup.cpp | FUN_005c28d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c3020 | ger_cup.cpp | FUN_005c3020 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c3660 | ger_first.cpp | FUN_005c3660 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c3890 | ger_first.cpp | FUN_005c3890 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c4ee0 | ger_first.cpp | FUN_005c4ee0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c5460 | ger_first.cpp | FUN_005c5460 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c5630 | ger_first.cpp | FUN_005c5630 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c5880 | ger_first.cpp | FUN_005c5880 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c5c00 | ger_first.cpp | FUN_005c5c00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c5d90 | ger_lge_cup.cpp | FUN_005c5d90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c63b0 | ger_lge_cup.cpp | FUN_005c63b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c6880 | ger_regional.cpp | FUN_005c6880 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c6ad0 | ger_regional.cpp | FUN_005c6ad0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c7490 | ger_regional.cpp | FUN_005c7490 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c78f0 | ger_regional.cpp | FUN_005c78f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c7dd0 | ger_regional.cpp | FUN_005c7dd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c7f80 | ger_second.cpp | FUN_005c7f80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c81b0 | ger_second.cpp | FUN_005c81b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c8f50 | ger_second.cpp | FUN_005c8f50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c92f0 | germany_awards.cpp | FUN_005c92f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c9430 | germany_awards.cpp | FUN_005c9430 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c9c60 | germany_awards.cpp | FUN_005c9c60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c9cd0 | germany_rules.cpp | FUN_005c9cd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005cc9b0 | goldcup.cpp | FUN_005cc9b0 | PORTED_BEHAVIOURAL | crates/cm-render/src/window_init.rs | YES | UNVERIFIED |  |
| 0x005ccdd0 | goldcup.cpp | FUN_005ccdd0 | PORTED_BEHAVIOURAL | crates/cm-render/src/fade.rs | YES | UNVERIFIED |  |
| 0x005cd370 | goldcup.cpp | FUN_005cd370 | PORTED_BEHAVIOURAL | crates/cm-render/src/background.rs;crates/cm-render/src/line.rs;crates/cm-render/src/primitives.rs | YES | UNVERIFIED |  |
| 0x005cd9d0 | goldcup.cpp | FUN_005cd9d0 | PORTED_BEHAVIOURAL | crates/cm-widget/src/lib.rs | YES | UNVERIFIED |  |
| 0x005cdfa0 | goldcup.cpp | FUN_005cdfa0 | UNKNOWN | crates/cm-render/src/cursor.rs;crates/cm-render/src/packed_panel.rs;crates/cm-render/src/scrman.rs;crates/cm-render/tests/verify_panel_against_exe.rs | INDIRECT | UNVERIFIED |  |
| 0x005cf610 | goldcup.cpp | FUN_005cf610 | PORTED_BEHAVIOURAL | crates/cm-domain/src/ui_schema.rs;crates/cm-render/src/msgbox.rs;crates/cm-render/src/primitives.rs;crates/cm-render/src/widget_pool.rs | YES | UNVERIFIED |  |
| 0x005d11e0 | goldcup.cpp | FUN_005d11e0 | PORTED_BEHAVIOURAL | crates/cm-render/src/bevel.rs | YES | UNVERIFIED |  |
| 0x005d1b80 | goldcup.cpp | FUN_005d1b80 | PORTED_BEHAVIOURAL | crates/cm-render/src/cursor.rs | YES | UNVERIFIED |  |
| 0x005d2240 | gre_cup.cpp | FUN_005d2240 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d24b0 | gre_cup.cpp | FUN_005d24b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d2c00 | gre_cup.cpp | FUN_005d2c00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d31f0 | gre_cup.cpp | FUN_005d31f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d3420 | gre_cup.cpp | FUN_005d3420 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d3950 | gre_cup.cpp | FUN_005d3950 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d3d00 | gre_prm.cpp | FUN_005d3d00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d3f30 | gre_prm.cpp | FUN_005d3f30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d4650 | gre_prm.cpp | FUN_005d4650 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d4a10 | gre_prm.cpp | FUN_005d4a10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d4ba0 | gre_second.cpp | FUN_005d4ba0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d4dd0 | gre_second.cpp | FUN_005d4dd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d55a0 | gre_second.cpp | FUN_005d55a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d5930 | gre_super.cpp | FUN_005d5930 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d5d20 | gre_super.cpp | FUN_005d5d20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d6140 | gre_super.cpp | FUN_005d6140 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d61d0 | greece_awards.cpp | FUN_005d61d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d67a0 | greece_awards.cpp | FUN_005d67a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d6bf0 | gui_utils.cpp | FUN_005d6bf0 | UNKNOWN | crates/cm-render/src/screens_auto.rs;crates/cm-ui-app/src/screens.rs;crates/cm-widget/src/lib.rs | INDIRECT | UNVERIFIED |  |
| 0x005d7070 | gui_utils.cpp | FUN_005d7070 | PORTED_BEHAVIOURAL | crates/cm-render/src/gen_screen_types.rs;crates/cm-render/src/view_render.rs;crates/cm-ui-app/src/screens.rs;crates/cm-widget/src/bin/render_gen_screen.rs;crates/cm-widget/src/tab_strip.rs | YES | UNVERIFIED |  |
| 0x005d8920 | guio.cpp | FUN_005d8920 | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x005d8b50 | hall_of_fame.cpp | FUN_005d8b50 | PORTED_BEHAVIOURAL | crates/cm-domain/src/hall_of_fame.rs | YES | UNVERIFIED |  |
| 0x005d9ac0 | hall_of_fame.cpp | FUN_005d9ac0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/hall_of_fame.rs | YES | UNVERIFIED |  |
| 0x005dab70 | hall_of_fame.cpp | FUN_005dab70 | PORTED_BEHAVIOURAL | crates/cm-domain/src/history.rs;crates/cm-domain/src/screen_batch16.rs | YES | UNVERIFIED |  |
| 0x005db8b0 | history.cpp | FUN_005db8b0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/history.rs;crates/cm-render/src/hsr.rs | YES | UNVERIFIED |  |
| 0x005dbf60 | history.cpp | FUN_005dbf60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005dbfe0 | history.cpp | FUN_005dbfe0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/history.rs | YES | UNVERIFIED |  |
| 0x005dc160 | history.cpp | FUN_005dc160 | PORTED_BEHAVIOURAL | crates/cm-domain/src/history.rs | YES | UNVERIFIED |  |
| 0x005dc2a0 | history.cpp | FUN_005dc2a0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/history.rs | YES | UNVERIFIED |  |
| 0x005dc5e0 | history.cpp | FUN_005dc5e0 | UNKNOWN | crates/cm-domain/src/screen_batch8.rs;crates/cm-domain/src/sidebar_dispatcher.rs;crates/cm-render/src/dispatcher.rs;crates/cm-render/src/view_render.rs | INDIRECT | UNVERIFIED |  |
| 0x005dc660 | history.cpp | FUN_005dc660 | UNKNOWN | crates/cm-domain/src/history.rs;crates/cm-render/src/hsr.rs | INDIRECT | UNVERIFIED |  |
| 0x005dcee0 | hol_cup.cpp | FUN_005dcee0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005dd090 | hol_cup.cpp | FUN_005dd090 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005dd800 | hol_cup.cpp | FUN_005dd800 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005ddcb0 | hol_cup.cpp | FUN_005ddcb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005ddf10 | hol_cup.cpp | FUN_005ddf10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005de2b0 | hol_cup.cpp | FUN_005de2b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005def20 | hol_first.cpp | FUN_005def20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005df160 | hol_first.cpp | FUN_005df160 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005dfcd0 | hol_first.cpp | FUN_005dfcd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e01b0 | hol_first.cpp | FUN_005e01b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e0560 | hol_first.cpp | FUN_005e0560 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e0dc0 | hol_first.cpp | FUN_005e0dc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e10a0 | hol_prm.cpp | FUN_005e10a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e12d0 | hol_prm.cpp | FUN_005e12d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e1ec0 | hol_prm.cpp | FUN_005e1ec0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e22b0 | hol_super.cpp | FUN_005e22b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e2560 | hol_super.cpp | FUN_005e2560 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e26a0 | hol_super.cpp | FUN_005e26a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e2a70 | hol_super.cpp | FUN_005e2a70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e2b00 | holland_awards.cpp | FUN_005e2b00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e3120 | holland_rules.cpp | FUN_005e3120 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e3490 | host_country.cpp | FUN_005e3490 | PORTED_BEHAVIOURAL | crates/cm-domain/src/host_country.rs | YES | UNVERIFIED |  |
| 0x005e42f0 | host_country.cpp | FUN_005e42f0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/host_country.rs | YES | UNVERIFIED |  |
| 0x005e4690 | host_country.cpp | FUN_005e4690 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005e4840 | host_country.cpp | FUN_005e4840 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005e49e0 | host_country.cpp | FUN_005e49e0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x005e4c80 | host_country.cpp | FUN_005e4c80 | PORTED_BEHAVIOURAL | crates/cm-domain/src/host_country.rs | YES | UNVERIFIED |  |
| 0x005e60d0 | human_manager.cpp | FUN_005e60d0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x005e6280 | human_manager.cpp | FUN_005e6280 | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x005e6d30 | human_manager.cpp | FUN_005e6d30 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs;crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x005e6e30 | human_manager.cpp | FUN_005e6e30 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x005e6fc0 | human_manager.cpp | FUN_005e6fc0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs;crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x005e71f0 | human_manager.cpp | FUN_005e71f0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs;crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x005e7980 | human_manager.cpp | FUN_005e7980 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x005e7b80 | human_manager.cpp | FUN_005e7b80 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs;crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x005e7dd0 | human_manager.cpp | FUN_005e7dd0 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x005e8040 | human_manager.cpp | FUN_005e8040 | PORTED_BEHAVIOURAL | crates/cm-domain/src/human_manager.rs;crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x005e8490 | human_manager.cpp | FUN_005e8490 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_club_dashboard.rs | YES | UNVERIFIED |  |
| 0x005e8990 | human_manager.cpp | FUN_005e8990 | PORTED_BEHAVIOURAL | crates/cm-domain/src/human_manager.rs | YES | UNVERIFIED |  |
| 0x005e9520 | human_manager.cpp | FUN_005e9520 | PORTED_BEHAVIOURAL | crates/cm-domain/src/human_manager.rs | YES | UNVERIFIED |  |
| 0x005ea720 | human_manager.cpp | FUN_005ea720 | PORTED_BEHAVIOURAL | crates/cm-domain/src/human_manager.rs;crates/cm-domain/src/screen_club_dashboard.rs;crates/cm-render/src/dispatcher_club_toolbar.rs;crates/cm-render/src/screen_club_squad_faithful.rs | YES | UNVERIFIED |  |
| 0x005eaa40 | human_manager.cpp | FUN_005eaa40 | PORTED_BEHAVIOURAL | crates/cm-domain/src/human_manager.rs | YES | UNVERIFIED |  |
| 0x005eb250 | human_manager.cpp | FUN_005eb250 | PORTED_BEHAVIOURAL | crates/cm-domain/src/human_manager.rs | YES | UNVERIFIED |  |
| 0x005eb410 | human_manager.cpp | FUN_005eb410 | PORTED_BEHAVIOURAL | crates/cm-domain/src/index.rs | YES | UNVERIFIED |  |
| 0x005f7140 | index.cpp | FUN_005f7140 | PORTED_BEHAVIOURAL | crates/cm-domain/src/index.rs | YES | UNVERIFIED |  |
| 0x00600680 | index.cpp | FUN_00600680 | PORTED_BEHAVIOURAL | crates/cm-domain/src/index.rs | YES | UNVERIFIED |  |
| 0x00600f50 | index.cpp | FUN_00600f50 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x00615c70 | index.cpp | FUN_00615c70 | UNKNOWN | crates/cm-domain/src/player_profile.rs | INDIRECT | UNVERIFIED |  |
| 0x006176f0 | index.cpp | FUN_006176f0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x006180c0 | index.cpp | FUN_006180c0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x00618450 | index.cpp | FUN_00618450 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x0061b760 | inter_amer_cup.cpp | FUN_0061b760 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061c1a0 | inter_amer_cup.cpp | FUN_0061c1a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061c210 | international_awards.cpp | FUN_0061c210 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061c690 | intertoto_cup.cpp | FUN_0061c690 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061c980 | intertoto_cup.cpp | FUN_0061c980 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061d290 | intertoto_cup.cpp | FUN_0061d290 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher.rs;crates/cm-render/src/msgbox.rs | YES | UNVERIFIED |  |
| 0x0061d730 | ire_chal_cup.cpp | FUN_0061d730 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061de60 | ire_chal_cup.cpp | FUN_0061de60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061e2a0 | ire_first.cpp | FUN_0061e2a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061e4e0 | ire_first.cpp | FUN_0061e4e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061ef70 | ire_first.cpp | FUN_0061ef70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061f420 | ire_first.cpp | FUN_0061f420 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061fb00 | ire_leinster_cup.cpp | FUN_0061fb00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006200e0 | ire_leinster_cup.cpp | FUN_006200e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00620430 | ire_lge_cup.cpp | FUN_00620430 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006206a0 | ire_lge_cup.cpp | FUN_006206a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00620c20 | ire_lge_cup.cpp | FUN_00620c20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00621020 | ire_lge_cup.cpp | FUN_00621020 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00621260 | ire_lge_cup.cpp | FUN_00621260 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00621430 | ire_lge_cup.cpp | FUN_00621430 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00621a80 | ire_lge_cup.cpp | FUN_00621a80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00622490 | ire_munster_cup.cpp | FUN_00622490 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00622900 | ire_pres_cup.cpp | FUN_00622900 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00622cf0 | ire_pres_cup.cpp | FUN_00622cf0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00623240 | ire_prm.cpp | FUN_00623240 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00623490 | ire_prm.cpp | FUN_00623490 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00623dd0 | ire_prm.cpp | FUN_00623dd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00624360 | ire_super_cup.cpp | FUN_00624360 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00624530 | ire_super_cup.cpp | FUN_00624530 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00624750 | ire_super_cup.cpp | FUN_00624750 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00624ce0 | ireland_awards.cpp | FUN_00624ce0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00624e30 | ireland_awards.cpp | FUN_00624e30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006258b0 | ireland_rules.cpp | FUN_006258b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00625c60 | ita_c1_super.cpp | FUN_00625c60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00625f10 | ita_c1_super.cpp | FUN_00625f10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00626050 | ita_c1_super.cpp | FUN_00626050 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006262c0 | ita_c_cup.cpp | FUN_006262c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00626530 | ita_c_cup.cpp | FUN_00626530 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00626c60 | ita_c_cup.cpp | FUN_00626c60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00627290 | ita_c_cup.cpp | FUN_00627290 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00627660 | ita_c_cup.cpp | FUN_00627660 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00627c00 | ita_c_cup.cpp | FUN_00627c00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006281f0 | ita_cup.cpp | FUN_006281f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006288d0 | ita_cup.cpp | FUN_006288d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00628cc0 | ita_cup.cpp | FUN_00628cc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00628ef0 | ita_cup.cpp | FUN_00628ef0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00629170 | ita_cup.cpp | FUN_00629170 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00629890 | ita_cup.cpp | FUN_00629890 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00629d50 | ita_ser_a.cpp | FUN_00629d50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00629fa0 | ita_ser_a.cpp | FUN_00629fa0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0062ab90 | ita_ser_a.cpp | FUN_0062ab90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0062b0f0 | ita_ser_a.cpp | FUN_0062b0f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0062b640 | ita_ser_a.cpp | FUN_0062b640 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0062b840 | ita_ser_a.cpp | FUN_0062b840 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0062bcb0 | ita_ser_a.cpp | FUN_0062bcb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0062c0f0 | ita_ser_a.cpp | FUN_0062c0f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0062c180 | ita_ser_a.cpp | FUN_0062c180 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0062ef10 | ita_ser_b.cpp | FUN_0062ef10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0062f160 | ita_ser_b.cpp | FUN_0062f160 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0062fe80 | ita_ser_b.cpp | FUN_0062fe80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006302b0 | ita_ser_b.cpp | FUN_006302b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006308a0 | ita_ser_b.cpp | FUN_006308a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006340e0 | ita_ser_c1a.cpp | FUN_006340e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00634330 | ita_ser_c1a.cpp | FUN_00634330 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00634ce0 | ita_ser_c1a.cpp | FUN_00634ce0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006350e0 | ita_ser_c1a.cpp | FUN_006350e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00635500 | ita_ser_c1a.cpp | FUN_00635500 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00638290 | ita_ser_c1b.cpp | FUN_00638290 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006384e0 | ita_ser_c1b.cpp | FUN_006384e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00638e90 | ita_ser_c1b.cpp | FUN_00638e90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006392a0 | ita_ser_c1b.cpp | FUN_006392a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00639890 | ita_ser_c1b.cpp | FUN_00639890 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0063c620 | ita_ser_c2a.cpp | FUN_0063c620 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0063c870 | ita_ser_c2a.cpp | FUN_0063c870 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0063d220 | ita_ser_c2a.cpp | FUN_0063d220 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0063d620 | ita_ser_c2a.cpp | FUN_0063d620 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0063da20 | ita_ser_c2a.cpp | FUN_0063da20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006407b0 | ita_ser_c2b.cpp | FUN_006407b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00640a00 | ita_ser_c2b.cpp | FUN_00640a00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006413b0 | ita_ser_c2b.cpp | FUN_006413b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006417b0 | ita_ser_c2b.cpp | FUN_006417b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00641bb0 | ita_ser_c2b.cpp | FUN_00641bb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00644940 | ita_ser_c2c.cpp | FUN_00644940 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00644b90 | ita_ser_c2c.cpp | FUN_00644b90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00645540 | ita_ser_c2c.cpp | FUN_00645540 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00645940 | ita_ser_c2c.cpp | FUN_00645940 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00645f50 | ita_ser_c2c.cpp | FUN_00645f50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00648ce0 | ita_super.cpp | FUN_00648ce0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00648f90 | ita_super.cpp | FUN_00648f90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006490d0 | ita_super.cpp | FUN_006490d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006494d0 | ita_super.cpp | FUN_006494d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00649560 | italy_awards.cpp | FUN_00649560 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064aa20 | italy_awards.cpp | FUN_0064aa20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064aa90 | italy_rules.cpp | FUN_0064aa90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064acd0 | italy_rules.cpp | FUN_0064acd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064af90 | italy_rules.cpp | FUN_0064af90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064b3d0 | jap_emp_cup.cpp | FUN_0064b3d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064b690 | jap_emp_cup.cpp | FUN_0064b690 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064bb90 | jap_emp_cup.cpp | FUN_0064bb90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064c200 | jap_j1.cpp | FUN_0064c200 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064c440 | jap_j1.cpp | FUN_0064c440 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064cbb0 | jap_j1.cpp | FUN_0064cbb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064d120 | jap_j1.cpp | FUN_0064d120 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064d2f0 | jap_j1.cpp | FUN_0064d2f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064db90 | jap_j2.cpp | FUN_0064db90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064ddc0 | jap_j2.cpp | FUN_0064ddc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064e700 | jap_j2.cpp | FUN_0064e700 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064eb00 | jap_j_cup.cpp | FUN_0064eb00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064edc0 | jap_j_cup.cpp | FUN_0064edc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064f180 | jap_j_cup.cpp | FUN_0064f180 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064f520 | jap_super.cpp | FUN_0064f520 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064f7d0 | jap_super.cpp | FUN_0064f7d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064f910 | jap_super.cpp | FUN_0064f910 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064fe40 | jap_super.cpp | FUN_0064fe40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064fed0 | japan_awards.cpp | FUN_0064fed0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00650380 | japan_awards.cpp | FUN_00650380 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006503f0 | japan_rules.cpp | FUN_006503f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00652e60 | key_nation.cpp | FUN_00652e60 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch28.rs | YES | UNVERIFIED |  |
| 0x00652ff0 | key_nation.cpp | FUN_00652ff0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/club_history.rs | YES | UNVERIFIED |  |
| 0x00653270 | key_nation.cpp | FUN_00653270 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch22.rs | YES | UNVERIFIED |  |
| 0x006535f0 | key_nation.cpp |  | PORTED_BEHAVIOURAL | crates/cm-ui-app/src/game_state.rs | YES | UNVERIFIED |  |
| 0x006547b0 | langlib.cpp | FUN_006547b0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/index.rs | YES | UNVERIFIED |  |
| 0x006679a0 | league.cpp | FUN_006679a0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs;crates/cm-domain/src/screen_batch28.rs;crates/cm-domain/src/screen_batch29.rs | YES | UNVERIFIED |  |
| 0x00668890 | league.cpp | FUN_00668890 | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs;crates/cm-domain/src/exe_date.rs;crates/cm-domain/src/lib.rs | YES | UNVERIFIED |  |
| 0x00669fa0 | league.cpp | FUN_00669fa0 | UNKNOWN | crates/cm-domain/src/c15_1g_snapshot.rs;crates/cm-domain/src/year_end_statuses.rs | INDIRECT | UNVERIFIED |  |
| 0x0066a910 | league.cpp | FUN_0066a910 | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x00672400 | league_stage.cpp | FUN_00672400 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs;crates/cm-domain/src/screen_batch7.rs | YES | UNVERIFIED |  |
| 0x00672420 | league_stage.cpp | FUN_00672420 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x006808a0 | manager_manager.cpp | FUN_006808a0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs;crates/cm-domain/src/human_manager.rs | YES | UNVERIFIED |  |
| 0x00680cc0 | manager_manager.cpp | FUN_00680cc0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs;crates/cm-domain/src/c13_promotion_apply.rs | YES | UNVERIFIED |  |
| 0x006822d0 | manager_manager.cpp | FUN_006822d0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x00688070 | manager_manager.cpp | FUN_00688070 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x00693410 | manager_manager.cpp | FUN_00693410 | UNKNOWN | crates/cm-domain/src/screen_manager_batch.rs;crates/cm-domain/src/sidebar_dispatcher.rs;crates/cm-render/src/dispatcher.rs | INDIRECT | UNVERIFIED |  |
| 0x00695e60 | manager_manager.cpp | FUN_00695e60 | UNKNOWN | crates/cm-domain/src/screen_manager_batch.rs;crates/cm-domain/src/sidebar_dispatcher.rs;crates/cm-render/src/dispatcher.rs;crates/cm-render/src/view_render.rs | INDIRECT | UNVERIFIED |  |
| 0x00696fa0 | manager_screens.cpp | FUN_00696fa0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch16.rs;crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x00697390 | manager_screens.cpp | FUN_00697390 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch16.rs | YES | UNVERIFIED |  |
| 0x006976a0 | manager_screens.cpp | FUN_006976a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006977b0 | manager_screens.cpp | FUN_006977b0 | UNKNOWN | crates/cm-domain/src/screen_manager_batch.rs;crates/cm-domain/src/sidebar_dispatcher.rs;crates/cm-render/src/dispatcher.rs | INDIRECT | UNVERIFIED |  |
| 0x00697c30 | manager_screens.cpp | FUN_00697c30 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch16.rs;crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x00698020 | manager_screens.cpp | FUN_00698020 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006986c0 | manager_screens.cpp | FUN_006986c0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch3.rs;crates/cm-render/src/screen_wire_batch3.rs | YES | UNVERIFIED |  |
| 0x0069aa70 | match_day.cpp | FUN_0069aa70 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x0069ac90 | match_day.cpp | FUN_0069ac90 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch17.rs | YES | UNVERIFIED |  |
| 0x0069b4a0 | match_day.cpp | FUN_0069b4a0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs;crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x0069b930 | match_day.cpp | FUN_0069b930 | PORTED_BEHAVIOURAL | crates/cm-domain/src/club_season_records.rs | YES | UNVERIFIED |  |
| 0x0069bc10 | match_day.cpp | FUN_0069bc10 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x0069d880 | match_day.cpp | FUN_0069d880 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x006a1470 | match_eng.cpp | FUN_006a1470 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x006a84f0 | match_eng.cpp | FUN_006a84f0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x006a91d0 | match_eng.cpp | FUN_006a91d0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs;crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x006a9200 | match_eng.cpp | FUN_006a9200 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x006a9a40 | match_eng.cpp | FUN_006a9a40 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x006a9a90 | match_eng.cpp | FUN_006a9a90 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x006ac3b0 | match_eng.cpp | FUN_006ac3b0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs;crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x006af650 | match_eng.cpp | FUN_006af650 | PORTED_BEHAVIOURAL | crates/cm-domain/src/card_model.rs | YES | UNVERIFIED |  |
| 0x006b12e0 | match_eng.cpp | FUN_006b12e0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x006b2f70 | match_eng.cpp | FUN_006b2f70 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x006b3a90 | match_eng.cpp | FUN_006b3a90 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x006b57d0 | match_eng.cpp | FUN_006b57d0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x006b5800 | match_eng.cpp | FUN_006b5800 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x006bba10 | match_eng.cpp | FUN_006bba10 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x006be690 | match_events.cpp | FUN_006be690 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006be730 | match_events.cpp | FUN_006be730 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006bf2d0 | match_events.cpp | FUN_006bf2d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006bf660 | match_events.cpp | FUN_006bf660 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006c0ce0 | match_events.cpp | FUN_006c0ce0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006c0f10 | match_man.cpp | FUN_006c0f10 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x006d08b0 | match_official.cpp | FUN_006d08b0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x006d1780 | match_official.cpp | FUN_006d1780 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x006d46c0 | match_official.cpp | FUN_006d46c0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x006d9ea0 | match_pl.cpp | FUN_006d9ea0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x006db210 | match_pl.cpp | FUN_006db210 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x006db520 | match_pl.cpp | FUN_006db520 | PORTED_BEHAVIOURAL | crates/cm-domain/src/card_model.rs;crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x006db580 | match_pl.cpp | FUN_006db580 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x006db630 | match_pl.cpp | FUN_006db630 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x006dfc50 | match_pl.cpp | FUN_006dfc50 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x006e0740 | match_pl.cpp | FUN_006e0740 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x006f0320 | match_pl.cpp | FUN_006f0320 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x006fa730 | match_pl.cpp | FUN_006fa730 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x006fa740 | match_pl.cpp | FUN_006fa740 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x006fce10 | match_pl.cpp | FUN_006fce10 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x006fce70 | match_pl.cpp | FUN_006fce70 | PORTED_BEHAVIOURAL | crates/cm-events/src/lib.rs | YES | UNVERIFIED |  |
| 0x006fd6c0 | match_screens.cpp | FUN_006fd6c0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch16.rs | YES | UNVERIFIED |  |
| 0x006fe590 | match_screens.cpp | FUN_006fe590 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch17.rs | YES | UNVERIFIED |  |
| 0x00701240 | match_screens.cpp | FUN_00701240 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch7.rs;crates/cm-render/src/dispatcher.rs;crates/cm-render/src/view_render.rs | YES | UNVERIFIED |  |
| 0x0070d350 | match_screens.cpp | FUN_0070d350 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch17.rs | YES | UNVERIFIED |  |
| 0x0070f550 | match_screens.cpp | FUN_0070f550 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch17.rs | YES | UNVERIFIED |  |
| 0x00710c70 | match_stats.cpp | FUN_00710c70 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x007116b0 | match_stats.cpp | FUN_007116b0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch7.rs;crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x007297b0 | media.cpp | FUN_007297b0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch17.rs;crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x007328a0 | media.cpp | FUN_007328a0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x0074d010 | media.cpp | FUN_0074d010 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs;crates/cm-domain/src/screen_batch17.rs;crates/cm-domain/src/screen_nation_dashboard.rs | YES | UNVERIFIED |  |
| 0x0074d830 | mini_cup.cpp | FUN_0074d830 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074da70 | mini_cup.cpp | FUN_0074da70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074dbb0 | mini_cup.cpp | FUN_0074dbb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074ded0 | mini_cup.cpp | FUN_0074ded0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074eb90 | month_award.cpp | FUN_0074eb90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074ee00 | month_award.cpp | FUN_0074ee00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074ef20 | month_award.cpp | FUN_0074ef20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074ef70 | month_award.cpp | FUN_0074ef70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074f730 | month_award.cpp | FUN_0074f730 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074f760 | month_ratings.cpp | FUN_0074f760 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074f9a0 | month_ratings.cpp | FUN_0074f9a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074fbe0 | month_ratings.cpp | FUN_0074fbe0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074fed0 | month_ratings.cpp | FUN_0074fed0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074fef0 | month_ratings.cpp | FUN_0074fef0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074ff50 | month_ratings.cpp | FUN_0074ff50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00750000 | nation_awards.cpp | FUN_00750000 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00750360 | nation_awards.cpp | FUN_00750360 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00750580 | nation_awards.cpp | FUN_00750580 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00750760 | nation_awards.cpp | FUN_00750760 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00750930 | nation_awards.cpp | FUN_00750930 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00750d00 | nation_awards.cpp | FUN_00750d00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00750fc0 | nation_awards.cpp | FUN_00750fc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00751530 | nation_awards.cpp | FUN_00751530 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00751730 | nation_awards.cpp | FUN_00751730 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007518b0 | nation_awards.cpp | FUN_007518b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00751ec0 | nation_awards.cpp | FUN_00751ec0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00751fc0 | nation_awards.cpp | FUN_00751fc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00752830 | national_teams.cpp | FUN_00752830 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x007536f0 | national_teams.cpp | FUN_007536f0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x007553c0 | national_teams.cpp | FUN_007553c0 | PORTED_BEHAVIOURAL | crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x00760b60 | national_teams.cpp | FUN_00760b60 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch17.rs | YES | UNVERIFIED |  |
| 0x00761620 | national_teams_screens.cpp | FUN_00761620 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch18.rs;crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x007627f0 | national_teams_screens.cpp | FUN_007627f0 | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x00762b90 | network.cpp | FUN_00762b90 | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x00762e80 | network.cpp | FUN_00762e80 | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x00763660 | new_transfer_rule_screens.cpp | FUN_00763660 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch18.rs;crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x007637b0 | new_transfer_rule_screens.cpp | FUN_007637b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00763b90 | new_transfer_rule_screens.cpp | FUN_00763b90 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs;crates/cm-domain/src/c14_stadium_expansion.rs;crates/cm-domain/src/person_news.rs | YES | UNVERIFIED |  |
| 0x0076d610 | news.cpp | FUN_0076d610 | PORTED_BEHAVIOURAL | crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x0076d7d0 | news.cpp | FUN_0076d7d0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs;crates/cm-domain/src/screen_batch12.rs;crates/cm-domain/src/screen_batch13.rs;crates/cm-domain/src/screen_batch14.rs;crates/cm-domain/src/screen_batch16.rs;crates/cm-domain/src/screen_batch17.rs;crates/cm-domain/src/screen_batch18.rs;crates/cm-domain/src/screen_batch23.rs;crates/cm-domain/src/screen_batch24.rs;crates/cm-domain/src/screen_batch25.rs;crates/cm-domain/src/screen_batch27.rs;crates/cm-domain/src/screen_batch35.rs | YES | UNVERIFIED |  |
| 0x0076eb10 | news.cpp | FUN_0076eb10 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch11.rs;crates/cm-domain/src/screen_batch12.rs;crates/cm-domain/src/screen_batch14.rs;crates/cm-domain/src/screen_batch16.rs;crates/cm-domain/src/screen_batch17.rs;crates/cm-domain/src/screen_batch18.rs;crates/cm-domain/src/screen_batch27.rs;crates/cm-domain/src/screen_batch29.rs | YES | UNVERIFIED |  |
| 0x0076ecd0 | news.cpp | FUN_0076ecd0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch29.rs;crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x0076ef80 | news.cpp | FUN_0076ef80 | UNKNOWN | crates/cm-render/src/dispatcher.rs;crates/cm-render/src/screen_news.rs | INDIRECT | UNVERIFIED |  |
| 0x00770170 | news_screens.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/screen_news.rs | YES | UNVERIFIED |  |
| 0x00771810 | news_screens.cpp | FUN_00771810 | UNKNOWN | crates/cm-domain/src/screen_batch4.rs;crates/cm-domain/src/sidebar_dispatcher.rs;crates/cm-render/src/view_render.rs | INDIRECT | UNVERIFIED |  |
| 0x00771880 | news_screens.cpp | FUN_00771880 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch18.rs;crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x00771c80 | news_screens.cpp | FUN_00771c80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00771ec0 | nir_charity.cpp | FUN_00771ec0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00772170 | nir_charity.cpp | FUN_00772170 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007722b0 | nir_charity.cpp | FUN_007722b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00772680 | nir_cup.cpp | FUN_00772680 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00772940 | nir_cup.cpp | FUN_00772940 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00772d60 | nir_cup.cpp | FUN_00772d60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007730e0 | nir_first.cpp | FUN_007730e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00773320 | nir_first.cpp | FUN_00773320 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00773d20 | nir_first.cpp | FUN_00773d20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00774170 | nir_first.cpp | FUN_00774170 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00774850 | nir_lge_cup.cpp | FUN_00774850 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00774ac0 | nir_lge_cup.cpp | FUN_00774ac0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007750a0 | nir_lge_cup.cpp | FUN_007750a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00775490 | nir_lge_cup.cpp | FUN_00775490 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007756d0 | nir_lge_cup.cpp | FUN_007756d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007758d0 | nir_lge_cup.cpp | FUN_007758d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00775da0 | nir_lge_cup.cpp | FUN_00775da0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00776140 | nir_prm.cpp | FUN_00776140 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00776370 | nir_prm.cpp | FUN_00776370 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00776d30 | nir_prm.cpp | FUN_00776d30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007770f0 | nor_cup.cpp | FUN_007770f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007773b0 | nor_cup.cpp | FUN_007773b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00777960 | nor_cup.cpp | FUN_00777960 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00777e10 | nor_first.cpp | FUN_00777e10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00778050 | nor_first.cpp | FUN_00778050 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00778a70 | nor_first.cpp | FUN_00778a70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00778ef0 | nor_first.cpp | FUN_00778ef0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007795d0 | nor_prm.cpp | FUN_007795d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00779800 | nor_prm.cpp | FUN_00779800 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077a6b0 | nor_prm.cpp | FUN_0077a6b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077aa30 | nor_prm.cpp | FUN_0077aa30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077ae50 | northern_ireland_awards.cpp | FUN_0077ae50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077af90 | northern_ireland_awards.cpp | FUN_0077af90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077b7a0 | northern_ireland_rules.cpp | FUN_0077b7a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077b9d0 | norway_awards.cpp | FUN_0077b9d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077bb10 | norway_awards.cpp | FUN_0077bb10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077c580 | norway_rules.cpp | FUN_0077c580 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077caf0 | norway_rules.cpp | FUN_0077caf0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077cbc0 | norway_rules.cpp | FUN_0077cbc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077d770 | notes.cpp | FUN_0077d770 | PORTED_BEHAVIOURAL | crates/cm-domain/src/scouting.rs;crates/cm-domain/src/screen_batch18.rs | YES | UNVERIFIED |  |
| 0x0077e1d0 | notes.cpp | FUN_0077e1d0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch18.rs | YES | UNVERIFIED |  |
| 0x0077f150 | oceania_club_champ.cpp | FUN_0077f150 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077f2f0 | oceania_club_champ.cpp | FUN_0077f2f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077fa40 | oceania_club_champ.cpp | FUN_0077fa40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077fcc0 | oceania_club_champ.cpp | FUN_0077fcc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00780000 | oceania_club_champ.cpp | FUN_00780000 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007801d0 | oceania_club_champ.cpp | FUN_007801d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007803a0 | oceania_club_champ.cpp | FUN_007803a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00780a30 | oceania_nations.cpp | FUN_00780a30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00780c20 | oceania_nations.cpp | FUN_00780c20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00781720 | oceania_nations.cpp | FUN_00781720 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00781be0 | oceania_nations.cpp | FUN_00781be0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00781dd0 | oceania_nations.cpp | FUN_00781dd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00781fb0 | oceania_nations.cpp | FUN_00781fb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00782280 | oceania_nations.cpp | FUN_00782280 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007826b0 | oceania_nations.cpp | FUN_007826b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00782ef0 | oceania_nations.cpp | FUN_00782ef0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00785890 | old_finland_awards.cpp | FUN_00785890 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007859d0 | old_finland_awards.cpp | FUN_007859d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00786040 | old_finland_awards.cpp | FUN_00786040 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007860d0 | old_france_awards.cpp | FUN_007860d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00786680 | old_france_awards.cpp | FUN_00786680 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007868b0 | old_france_awards.cpp | FUN_007868b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00786920 | old_international_awards.cpp | FUN_00786920 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00786d30 | old_ireland_awards.cpp | FUN_00786d30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00786e70 | old_ireland_awards.cpp | FUN_00786e70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00787860 | olympics.cpp | FUN_00787860 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00787a40 | olympics.cpp | FUN_00787a40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00788470 | olympics.cpp | FUN_00788470 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00788900 | olympics.cpp | FUN_00788900 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00788af0 | olympics.cpp | FUN_00788af0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00788d10 | olympics.cpp | FUN_00788d10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007894f0 | olympics.cpp | FUN_007894f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00789dc0 | os.cpp | FUN_00789dc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0078a000 | os.cpp | FUN_0078a000 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0078a980 | os.cpp | FUN_0078a980 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0078ad80 | physio.cpp | FUN_0078ad80 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch18.rs | YES | UNVERIFIED |  |
| 0x0079af40 | player_search.cpp | FUN_0079af40 | UNKNOWN | crates/cm-domain/src/scouting.rs | INDIRECT | UNVERIFIED |  |
| 0x007a9040 | player_stats.cpp | FUN_007a9040 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a9690 | player_stats.cpp | FUN_007a9690 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x007a97f0 | player_stats.cpp | FUN_007a97f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a99a0 | player_stats.cpp | FUN_007a99a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a9c90 | player_stats.cpp | FUN_007a9c90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a9da0 | player_stats.cpp | FUN_007a9da0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a9e60 | player_stats.cpp | FUN_007a9e60 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007aa170 | player_stats.cpp | FUN_007aa170 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007aa490 | player_stats.cpp | FUN_007aa490 | PORTED_BEHAVIOURAL | crates/cm-domain/src/player_rating.rs | YES | UNVERIFIED |  |
| 0x007aa8f0 | player_stats.cpp | FUN_007aa8f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007aaa90 | player_stats.cpp | FUN_007aaa90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007ab3a0 | player_stats.cpp | FUN_007ab3a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007abc60 | player_stats.cpp | FUN_007abc60 | PORTED_BEHAVIOURAL | crates/cm-domain/src/player_rating.rs | YES | UNVERIFIED |  |
| 0x007ac180 | player_stats.cpp | FUN_007ac180 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007acaf0 | player_stats.cpp | FUN_007acaf0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007acbf0 | player_stats.cpp | FUN_007acbf0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007ad8e0 | player_stats.cpp | FUN_007ad8e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007ae0c0 | player_stats.cpp | FUN_007ae0c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007ae320 | player_stats.cpp | FUN_007ae320 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007ae9b0 | player_stats.cpp | FUN_007ae9b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007aeb90 | player_stats.cpp | FUN_007aeb90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007aed90 | player_stats.cpp | FUN_007aed90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007aef50 | player_stats.cpp | FUN_007aef50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007af000 | player_stats.cpp | FUN_007af000 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007af190 | player_stats.cpp | FUN_007af190 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007af320 | plot.cpp | FUN_007af320 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007af460 | plot.cpp | FUN_007af460 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007af5a0 | plot.cpp | FUN_007af5a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007af6a0 | plot.cpp | FUN_007af6a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007af8e0 | plot.cpp | FUN_007af8e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b0190 | pol_cup.cpp | FUN_007b0190 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b0450 | pol_cup.cpp | FUN_007b0450 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b0960 | pol_cup.cpp | FUN_007b0960 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b0e90 | pol_first.cpp | FUN_007b0e90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b1160 | pol_first.cpp | FUN_007b1160 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b1860 | pol_first.cpp | FUN_007b1860 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b1d10 | pol_first.cpp | FUN_007b1d10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b1f70 | pol_first.cpp | FUN_007b1f70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b2440 | pol_first.cpp | FUN_007b2440 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b28f0 | pol_lge_cup.cpp | FUN_007b28f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b2bb0 | pol_lge_cup.cpp | FUN_007b2bb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b3140 | pol_lge_cup.cpp | FUN_007b3140 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b35e0 | pol_second.cpp | FUN_007b35e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b3820 | pol_second.cpp | FUN_007b3820 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b4200 | pol_second.cpp | FUN_007b4200 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b46a0 | pol_second.cpp | FUN_007b46a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b4d30 | pol_super.cpp | FUN_007b4d30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b4fe0 | pol_super.cpp | FUN_007b4fe0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b5120 | pol_super.cpp | FUN_007b5120 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b5540 | pol_super.cpp | FUN_007b5540 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b55d0 | poland_awards.cpp | FUN_007b55d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b5c40 | poland_rules.cpp | FUN_007b5c40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b5d90 | por_cup.cpp | FUN_007b5d90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b6050 | por_cup.cpp | FUN_007b6050 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b6630 | por_cup.cpp | FUN_007b6630 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b6b00 | por_prm.cpp | FUN_007b6b00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b6d30 | por_prm.cpp | FUN_007b6d30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b7be0 | por_prm.cpp | FUN_007b7be0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b8110 | por_prm.cpp | FUN_007b8110 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b8350 | por_prm.cpp | FUN_007b8350 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b8700 | por_prm.cpp | FUN_007b8700 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b8d30 | por_second.cpp | FUN_007b8d30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b8f60 | por_second.cpp | FUN_007b8f60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b9900 | por_second.cpp | FUN_007b9900 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b9d00 | por_second_b.cpp | FUN_007b9d00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b9f60 | por_second_b.cpp | FUN_007b9f60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ba7e0 | por_second_b.cpp | FUN_007ba7e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007bac50 | por_second_b.cpp | FUN_007bac50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007bb0b0 | por_second_b.cpp | FUN_007bb0b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007bb290 | por_super.cpp | FUN_007bb290 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007bb540 | por_super.cpp | FUN_007bb540 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007bb6b0 | por_super.cpp | FUN_007bb6b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007bba80 | por_super.cpp | FUN_007bba80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007bbb10 | portugal_awards.cpp | FUN_007bbb10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007bc1d0 | portugal_awards.cpp | FUN_007bc1d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007bc240 | portugal_rules.cpp | FUN_007bc240 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007bc8b0 | printouts.cpp | FUN_007bc8b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007bd6c0 | printouts.cpp | FUN_007bd6c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007be500 | printouts.cpp | FUN_007be500 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007bee30 | printouts.cpp | FUN_007bee30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c00c0 | printouts.cpp | FUN_007c00c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c0560 | printouts.cpp | FUN_007c0560 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c1b60 | printouts.cpp | FUN_007c1b60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c24b0 | printouts.cpp | FUN_007c24b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c25d0 | printouts.cpp | FUN_007c25d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c2ac0 | printouts.cpp | FUN_007c2ac0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c2b10 | printouts.cpp | FUN_007c2b10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c2fc0 | printouts.cpp | FUN_007c2fc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c3150 | printouts.cpp | FUN_007c3150 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c3380 | printouts.cpp | FUN_007c3380 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c3580 | printouts.cpp | FUN_007c3580 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c37e0 | rb_argentina.cpp | FUN_007c37e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c3a30 | rb_asia.cpp | FUN_007c3a30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c3d70 | rb_australia.cpp | FUN_007c3d70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c4120 | rb_belgium_cup.cpp | FUN_007c4120 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c4bc0 | rb_brazil_regional.cpp | FUN_007c4bc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c4fa0 | rb_croatia.cpp | FUN_007c4fa0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c5400 | rb_denmark.cpp | FUN_007c5400 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c5480 | rb_denmark.cpp | FUN_007c5480 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c58c0 | rb_england.cpp | FUN_007c58c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c5b20 | rb_europe.cpp | FUN_007c5b20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c5e50 | rb_finland_cup.cpp | FUN_007c5e50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c6180 | rb_finland_league.cpp | FUN_007c6180 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c67a0 | rb_france.cpp | FUN_007c67a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c6ae0 | rb_germany_cup.cpp | FUN_007c6ae0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c6e20 | rb_germany_league.cpp | FUN_007c6e20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c7210 | rb_greece.cpp | FUN_007c7210 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c75f0 | rb_holland.cpp | FUN_007c75f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c7900 | rb_international.cpp | FUN_007c7900 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c7de0 | rb_ireland.cpp | FUN_007c7de0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c81b0 | rb_italy_cup.cpp | FUN_007c81b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c85f0 | rb_italy_league.cpp | FUN_007c85f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c88d0 | rb_japan_cup.cpp | FUN_007c88d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c8b70 | rb_japan_league.cpp | FUN_007c8b70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c9450 | rb_norway_league.cpp | FUN_007c9450 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c9690 | rb_oceania.cpp | FUN_007c9690 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c9ae0 | rb_poland.cpp | FUN_007c9ae0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c9e80 | rb_portugal.cpp | FUN_007c9e80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ca220 | rb_russia.cpp | FUN_007ca220 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ca4f0 | rb_scotland_cup.cpp | FUN_007ca4f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ca750 | rb_scotland_cup.cpp | FUN_007ca750 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007caa60 | rb_scotland_league.cpp | FUN_007caa60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007cacb0 | rb_south_america.cpp | FUN_007cacb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007cb040 | rb_spain_cup.cpp | FUN_007cb040 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007cb3d0 | rb_spain_league.cpp | FUN_007cb3d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007cb7a0 | rb_sweden_cup.cpp | FUN_007cb7a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007cbb10 | rb_sweden_league.cpp | FUN_007cbb10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007cbea0 | rb_turkey_cup.cpp | FUN_007cbea0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007cc1f0 | rb_turkey_league.cpp | FUN_007cc1f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007cc4c0 | rb_usa.cpp | FUN_007cc4c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ccd10 | record_utils.cpp | FUN_007ccd10 | PORTED_BEHAVIOURAL | crates/cm-domain/src/club_season_records.rs | YES | UNVERIFIED |  |
| 0x007cd740 | record_utils.cpp | FUN_007cd740 | PORTED_BEHAVIOURAL | crates/cm-domain/src/club_season_records.rs | YES | UNVERIFIED |  |
| 0x007cd7f0 | record_utils.cpp | FUN_007cd7f0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/injury_table.rs | YES | UNVERIFIED |  |
| 0x007cdb80 | record_utils.cpp | FUN_007cdb80 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch19.rs | YES | UNVERIFIED |  |
| 0x007ce4a0 | record_utils.cpp | FUN_007ce4a0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x007cf040 | record_utils.cpp | FUN_007cf040 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch29.rs;crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x007cf880 | ruling_body.cpp | FUN_007cf880 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007cf980 | ruling_body.cpp | FUN_007cf980 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007cf9c0 | ruling_body.cpp | FUN_007cf9c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007cfca0 | ruling_body.cpp | FUN_007cfca0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d0110 | ruling_body.cpp | FUN_007d0110 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d0400 | ruling_body.cpp | FUN_007d0400 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d0630 | ruling_body.cpp | FUN_007d0630 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d06d0 | ruling_body.cpp | FUN_007d06d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d09f0 | ruling_body.cpp | FUN_007d09f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d0f30 | ruling_body.cpp | FUN_007d0f30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d1170 | ruling_body.cpp | FUN_007d1170 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d12a0 | ruling_body.cpp | FUN_007d12a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d12f0 | ruling_body.cpp | FUN_007d12f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d1330 | rus_cup.cpp | FUN_007d1330 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d1650 | rus_cup.cpp | FUN_007d1650 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d1ac0 | rus_cup.cpp | FUN_007d1ac0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d2080 | rus_first.cpp | FUN_007d2080 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d22b0 | rus_first.cpp | FUN_007d22b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d2810 | rus_first.cpp | FUN_007d2810 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d2bd0 | rus_prm.cpp | FUN_007d2bd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d2e10 | rus_prm.cpp | FUN_007d2e10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d3760 | rus_prm.cpp | FUN_007d3760 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d3be0 | rus_prm.cpp | FUN_007d3be0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d3f70 | rus_prm.cpp | FUN_007d3f70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d4340 | rus_prm.cpp | FUN_007d4340 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d43d0 | russia_awards.cpp | FUN_007d43d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d48a0 | russia_awards.cpp | FUN_007d48a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d4910 | russia_rules.cpp | FUN_007d4910 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d4b20 | sco_chal_cup.cpp | FUN_007d4b20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d4de0 | sco_chal_cup.cpp | FUN_007d4de0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d51b0 | sco_chal_cup.cpp | FUN_007d51b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d54b0 | sco_fa_cup.cpp | FUN_007d54b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d5770 | sco_fa_cup.cpp | FUN_007d5770 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d5c90 | sco_fa_cup.cpp | FUN_007d5c90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d6160 | sco_first.cpp | FUN_007d6160 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d63c0 | sco_first.cpp | FUN_007d63c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d6ed0 | sco_first.cpp | FUN_007d6ed0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d74e0 | sco_first.cpp | FUN_007d74e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d7b80 | sco_lge_cup.cpp | FUN_007d7b80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d7e50 | sco_lge_cup.cpp | FUN_007d7e50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d85a0 | sco_lge_cup.cpp | FUN_007d85a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d8d20 | sco_prm.cpp | FUN_007d8d20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d8f90 | sco_prm.cpp | FUN_007d8f90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007da9c0 | sco_prm.cpp | FUN_007da9c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007daf30 | sco_prm.cpp | FUN_007daf30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007db3f0 | sco_second.cpp | FUN_007db3f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007db620 | sco_second.cpp | FUN_007db620 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007dc020 | sco_second.cpp | FUN_007dc020 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007dc390 | sco_third.cpp | FUN_007dc390 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007dc5c0 | sco_third.cpp | FUN_007dc5c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007dcfc0 | sco_third.cpp | FUN_007dcfc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007dd310 | scotland_awards.cpp | FUN_007dd310 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007dd450 | scotland_awards.cpp | FUN_007dd450 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007de530 | scotland_rules.cpp | FUN_007de530 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007dfac0 | scout_manager.cpp | FUN_007dfac0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/menu.rs;crates/cm-domain/src/screen_batch21.rs;crates/cm-domain/src/screen_batch4.rs;crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x007e0490 | scout_manager.cpp | FUN_007e0490 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x007e05c0 | scout_manager.cpp | FUN_007e05c0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch9.rs;crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x007e06a0 | scout_manager.cpp | FUN_007e06a0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch9.rs;crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x007e6430 | scrman.cpp | FUN_007e6430 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch16.rs;crates/cm-domain/src/screen_batch17.rs;crates/cm-domain/src/screen_batch19.rs;crates/cm-render/src/screen_wire_batch3.rs | YES | UNVERIFIED |  |
| 0x007e6ab0 | scrman.cpp | FUN_007e6ab0 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher.rs;crates/cm-render/src/screen_menu_bar.rs | YES | UNVERIFIED |  |
| 0x007e6b60 | scrman.cpp | FUN_007e6b60 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher.rs;crates/cm-render/src/screen_menu_bar.rs | YES | UNVERIFIED |  |
| 0x007e6e00 | scrman.cpp | FUN_007e6e00 | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x007e6ee0 | scrman.cpp | FUN_007e6ee0 | UNKNOWN | crates/cm-domain/src/screen_batch10.rs;crates/cm-domain/src/screen_batch12.rs;crates/cm-domain/src/screen_batch18.rs;crates/cm-domain/src/screen_batch21.rs;crates/cm-domain/src/screen_batch3.rs;crates/cm-domain/src/screen_batch4.rs;crates/cm-domain/src/screen_batch6.rs;crates/cm-domain/src/screen_batch8.rs;crates/cm-domain/src/tactic_dispatcher.rs;crates/cm-render/src/dispatcher_club_toolbar.rs;crates/cm-render/src/screen_club_squad_faithful.rs | INDIRECT | UNVERIFIED |  |
| 0x007e7000 | scrman.cpp | FUN_007e7000 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch16.rs;crates/cm-domain/src/screen_batch3.rs;crates/cm-domain/src/screen_manager_batch.rs;crates/cm-domain/src/screen_nation_dashboard.rs;crates/cm-render/src/screen_wire_batch3.rs | YES | UNVERIFIED |  |
| 0x007e7790 | scrman.cpp | FUN_007e7790 | PORTED_BEHAVIOURAL | crates/cm-domain/src/menu.rs;crates/cm-domain/src/screen_batch4.rs;crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x007e7820 | scrman.cpp | FUN_007e7820 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch19.rs | YES | UNVERIFIED |  |
| 0x007e7b50 | scrman.cpp | FUN_007e7b50 | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x007e7c40 | scrman.cpp | FUN_007e7c40 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x007e7cb0 | scrman.cpp | FUN_007e7cb0 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x007e83e0 | scrman.cpp | FUN_007e83e0 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher.rs;crates/cm-render/src/screen_menu_bar.rs | YES | UNVERIFIED |  |
| 0x007e8b70 | scrman.cpp | FUN_007e8b70 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x007e9180 | scrman.cpp | FUN_007e9180 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher.rs;crates/cm-render/src/screen_menu_bar.rs | YES | UNVERIFIED |  |
| 0x007e9a80 | scrman.cpp | FUN_007e9a80 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x007e9dc0 | scrman.cpp | FUN_007e9dc0 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x007eb990 | scrman.cpp | FUN_007eb990 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x007ebca0 | scrman.cpp | FUN_007ebca0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_manager_batch.rs | YES | UNVERIFIED |  |
| 0x007ebf80 | search_edit_session.cpp | FUN_007ebf80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ec020 | search_edit_session.cpp | FUN_007ec020 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ec070 | search_edit_session.cpp | FUN_007ec070 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ec150 | search_edit_session.cpp | FUN_007ec150 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ec2f0 | search_edit_session.cpp | FUN_007ec2f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ec3c0 | search_edit_session.cpp | FUN_007ec3c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ec480 | search_edit_session.cpp | FUN_007ec480 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ec590 | search_edit_session.cpp | FUN_007ec590 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ec7b0 | search_edit_session.cpp | FUN_007ec7b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ec8f0 | search_edit_session.cpp | FUN_007ec8f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007f0020 | search_filters.cpp | FUN_007f0020 | UNKNOWN | crates/cm-domain/src/screen_manager_batch.rs;crates/cm-domain/src/sidebar_dispatcher.rs;crates/cm-render/src/dispatcher.rs | INDIRECT | UNVERIFIED |  |
| 0x007f0170 | search_screens.cpp | FUN_007f0170 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007f6720 | search_screens.cpp | FUN_007f6720 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007faec0 | search_screens.cpp | FUN_007faec0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008018b0 | search_screens.cpp | FUN_008018b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008055e0 | setup.cpp |  | UNKNOWN | crates/cm-render/src/screen_pre_boot.rs;crates/cm-ui-app/src/game_state.rs;crates/cm-ui-app/src/screens.rs;crates/cm-widget/src/lib.rs | INDIRECT | UNVERIFIED |  |
| 0x0080a450 | setup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/screen_name_faithful.rs;crates/cm-render/src/screen_pre_boot.rs;crates/cm-ui-app/src/game_state.rs | YES | UNVERIFIED |  |
| 0x0080bbd0 | setup.cpp | FUN_0080bbd0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/human_manager.rs;crates/cm-domain/src/screen_batch20.rs;crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x0080cc20 | setup.cpp | FUN_0080cc20 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch20.rs | YES | UNVERIFIED |  |
| 0x0080fac0 | setup.cpp | FUN_0080fac0 | UNKNOWN | crates/cm-domain/src/menu.rs;crates/cm-domain/src/screen_batch8.rs;crates/cm-domain/src/sidebar_dispatcher.rs;crates/cm-render/src/dispatcher.rs;crates/cm-render/src/view_render.rs | INDIRECT | UNVERIFIED |  |
| 0x008109c0 | setup.cpp | FUN_008109c0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch20.rs | YES | UNVERIFIED |  |
| 0x00810ca0 | setup.cpp | FUN_00810ca0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch20.rs;crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x00811d80 | setup.cpp | FUN_00811d80 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch8.rs | YES | UNVERIFIED |  |
| 0x008224b0 | setup.cpp | FUN_008224b0 | UNKNOWN | crates/cm-render/src/dispatcher.rs | INDIRECT | UNVERIFIED |  |
| 0x00822580 | setup.cpp | FUN_00822580 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch9.rs;crates/cm-render/src/dispatcher.rs;crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x00822920 | setup.cpp | FUN_00822920 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x00822940 | setup.cpp | FUN_00822940 | PORTED_BEHAVIOURAL | crates/cm-domain/src/menu.rs;crates/cm-render/src/dispatcher.rs;crates/cm-render/src/screen_wire_batch3.rs | YES | UNVERIFIED |  |
| 0x0082a0b0 | shortlist_manager.cpp | FUN_0082a0b0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x0082fc50 | shortlist_manager.cpp | FUN_0082fc50 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x00833210 | shortlist_manager.cpp | FUN_00833210 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x008360f0 | spa_cup.cpp | FUN_008360f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008363c0 | spa_cup.cpp | FUN_008363c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00836ab0 | spa_cup.cpp | FUN_00836ab0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008372d0 | spa_first.cpp | FUN_008372d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00837550 | spa_first.cpp | FUN_00837550 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00838e10 | spa_first.cpp | FUN_00838e10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00839260 | spa_first.cpp | FUN_00839260 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008394a0 | spa_first.cpp | FUN_008394a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008396d0 | spa_first.cpp | FUN_008396d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00839b10 | spa_first.cpp | FUN_00839b10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0083a1c0 | spa_lower.cpp | FUN_0083a1c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0083c570 | spa_lower.cpp | FUN_0083c570 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0083cb40 | spa_second.cpp | FUN_0083cb40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0083cd70 | spa_second.cpp | FUN_0083cd70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0083f0e0 | spa_second_b.cpp | FUN_0083f0e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0083f340 | spa_second_b.cpp | FUN_0083f340 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0083fe70 | spa_second_b.cpp | FUN_0083fe70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008403b0 | spa_second_b.cpp | FUN_008403b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00840710 | spa_second_b.cpp | FUN_00840710 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00840a00 | spa_second_b.cpp | FUN_00840a00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00840ba0 | spa_second_b.cpp | FUN_00840ba0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00841490 | spa_second_b.cpp | FUN_00841490 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00841850 | spa_super.cpp | FUN_00841850 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00841b00 | spa_super.cpp | FUN_00841b00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00841c40 | spa_super.cpp | FUN_00841c40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00842010 | spa_super.cpp | FUN_00842010 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008420a0 | spain_awards.cpp | FUN_008420a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00842430 | spain_awards.cpp | FUN_00842430 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00842570 | spain_awards.cpp | FUN_00842570 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008425e0 | spain_rules.cpp | FUN_008425e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00843100 | squad_manager.cpp | FUN_00843100 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x00843880 | squad_manager.cpp | FUN_00843880 | PORTED_BEHAVIOURAL | crates/cm-domain/src/contract_init.rs;crates/cm-domain/src/screen_batch11.rs;crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x00843ef0 | squad_manager.cpp | FUN_00843ef0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/c13_promotion_apply.rs;crates/cm-domain/src/c14_5_squad_manager.rs;crates/cm-domain/src/screen_batch11.rs | YES | UNVERIFIED |  |
| 0x00844b60 | stadium.cpp | FUN_00844b60 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00844fc0 | stadium.cpp | FUN_00844fc0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00845380 | stadium.cpp | FUN_00845380 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00845920 | stadium.cpp | FUN_00845920 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00846b20 | stadium.cpp | FUN_00846b20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00846ed0 | stadium.cpp | FUN_00846ed0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00846f40 | stadium.cpp | FUN_00846f40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008470e0 | stadium.cpp | FUN_008470e0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x00847530 | stadium.cpp | FUN_00847530 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008475a0 | stadium.cpp | FUN_008475a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00848980 | staff_contracts.cpp | FUN_00848980 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x00851770 | staff_records.cpp | FUN_00851770 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008517e0 | staff_records.cpp | FUN_008517e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00851c00 | staff_records.cpp | FUN_00851c00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00852430 | staff_records.cpp | FUN_00852430 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00852a50 | staff_records.cpp | FUN_00852a50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00853020 | staff_records.cpp | FUN_00853020 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00853170 | staff_records.cpp | FUN_00853170 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008537b0 | staff_records.cpp | FUN_008537b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00854060 | staff_records.cpp | FUN_00854060 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008546c0 | staff_records.cpp | FUN_008546c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00854d80 | staff_records.cpp | FUN_00854d80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00854fb0 | staff_records.cpp | FUN_00854fb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00855180 | staff_records.cpp | FUN_00855180 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00855340 | staff_records.cpp | FUN_00855340 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008556e0 | staff_records.cpp | FUN_008556e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00855870 | staff_records.cpp | FUN_00855870 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00855d20 | staff_records.cpp | FUN_00855d20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008567b0 | staff_records.cpp | FUN_008567b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008569d0 | staff_records.cpp | FUN_008569d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00856be0 | staff_records.cpp | FUN_00856be0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00856e20 | staff_records.cpp | FUN_00856e20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00856f40 | staff_records.cpp | FUN_00856f40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00857060 | staff_records.cpp | FUN_00857060 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008582b0 | staff_records.cpp | FUN_008582b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00858560 | staff_records.cpp | FUN_00858560 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00858900 | staff_records.cpp | FUN_00858900 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00858a60 | staff_records.cpp | FUN_00858a60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00858b30 | staff_records.cpp | FUN_00858b30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00858c90 | staff_records.cpp | FUN_00858c90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00858f80 | staff_records.cpp | FUN_00858f80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00859250 | staff_screens.cpp | FUN_00859250 | UNKNOWN | crates/cm-domain/src/menu.rs;crates/cm-domain/src/screen_batch3.rs;crates/cm-domain/src/sidebar_dispatcher.rs;crates/cm-render/src/screen_wire_batch3.rs | INDIRECT | UNVERIFIED |  |
| 0x00859600 | staff_screens.cpp | FUN_00859600 | UNKNOWN | crates/cm-domain/src/screen_batch21.rs;crates/cm-domain/src/sidebar_dispatcher.rs | INDIRECT | UNVERIFIED |  |
| 0x00873040 | staff_screens.cpp | FUN_00873040 | UNKNOWN | crates/cm-domain/src/screen_batch21.rs | INDIRECT | UNVERIFIED |  |
| 0x00874a10 | staff_screens.cpp | FUN_00874a10 | UNKNOWN | crates/cm-domain/src/screen_batch21.rs | INDIRECT | UNVERIFIED |  |
| 0x00877450 | sub_league.cpp | FUN_00877450 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00877770 | sub_league.cpp | FUN_00877770 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00877930 | swe_cup.cpp | FUN_00877930 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00877cc0 | swe_cup.cpp | FUN_00877cc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00878320 | swe_cup.cpp | FUN_00878320 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008787a0 | swe_cup.cpp | FUN_008787a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00878b20 | swe_cup.cpp | FUN_00878b20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00878de0 | swe_cup.cpp | FUN_00878de0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008791a0 | swe_cup.cpp | FUN_008791a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00879360 | swe_first.cpp | FUN_00879360 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008795a0 | swe_first.cpp | FUN_008795a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00879d10 | swe_first.cpp | FUN_00879d10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087a160 | swe_first.cpp | FUN_0087a160 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087a890 | swe_prm.cpp | FUN_0087a890 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087aba0 | swe_prm.cpp | FUN_0087aba0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087b1b0 | swe_prm.cpp | FUN_0087b1b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087b700 | swe_prm.cpp | FUN_0087b700 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087b8c0 | swe_prm.cpp | FUN_0087b8c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087b9f0 | swe_prm.cpp | FUN_0087b9f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087c3a0 | swe_second.cpp | FUN_0087c3a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087c650 | swe_second.cpp | FUN_0087c650 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087c710 | swe_second.cpp | FUN_0087c710 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087d320 | swe_second.cpp | FUN_0087d320 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087d6c0 | swe_second.cpp | FUN_0087d6c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087d860 | swe_second.cpp | FUN_0087d860 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087e120 | swe_second.cpp | FUN_0087e120 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087e3d0 | swe_second.cpp | FUN_0087e3d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087e460 | sweden_awards.cpp | FUN_0087e460 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00881220 | tactics.cpp | FUN_00881220 | PORTED_BEHAVIOURAL | crates/cm-domain/src/human_manager.rs | YES | UNVERIFIED |  |
| 0x008815a0 | tactics.cpp | FUN_008815a0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/human_manager.rs;crates/cm-domain/src/screen_batch11.rs | YES | UNVERIFIED |  |
| 0x00881910 | tactics.cpp | FUN_00881910 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x00882640 | tactics.cpp | FUN_00882640 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x00882f60 | tactics.cpp | FUN_00882f60 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x008830a0 | tactics.cpp | FUN_008830a0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x00890360 | tactics_screens.cpp | FUN_00890360 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x00894380 | tactics_screens.cpp | FUN_00894380 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x00894650 | tactics_screens.cpp | FUN_00894650 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x00894a20 | tactics_screens.cpp | FUN_00894a20 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x00894bb0 | tactics_screens.cpp | FUN_00894bb0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x008956a0 | tactics_screens.cpp | FUN_008956a0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x008957f0 | tactics_screens.cpp | FUN_008957f0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x00895d40 | tactics_screens.cpp | FUN_00895d40 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch22.rs;crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x00895f90 | tactics_screens.cpp | FUN_00895f90 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x008962f0 | tactics_screens.cpp | FUN_008962f0 | UNKNOWN | crates/cm-domain/src/screen_batch22.rs;crates/cm-domain/src/tactic_dispatcher.rs | INDIRECT | UNVERIFIED |  |
| 0x00897010 | tactics_screens.cpp | FUN_00897010 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x00897e20 | tactics_screens.cpp | FUN_00897e20 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x0089bc20 | tcpip.cpp | FUN_0089bc20 | PORTED_BEHAVIOURAL | crates/cm-domain/src/honours.rs | YES | UNVERIFIED |  |
| 0x0089d100 | team_award.cpp | FUN_0089d100 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch22.rs | YES | UNVERIFIED |  |
| 0x0089d2f0 | training_edit_session.cpp | FUN_0089d2f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0089d500 | training_edit_session.cpp | FUN_0089d500 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0089d570 | training_edit_session.cpp | FUN_0089d570 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008a1c90 | training_schedule.cpp | FUN_008a1c90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008a1e00 | training_schedule.cpp | FUN_008a1e00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008a1f90 | training_schedule.cpp | FUN_008a1f90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008a20a0 | training_schedule.cpp | FUN_008a20a0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch22.rs | YES | UNVERIFIED |  |
| 0x008a6380 | training_schedule.cpp | FUN_008a6380 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008a6420 | training_screens.cpp | FUN_008a6420 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch22.rs | YES | UNVERIFIED |  |
| 0x008a7890 | training_screens.cpp | FUN_008a7890 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ac0c0 | transfer_manager.cpp | FUN_008ac0c0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x008acd10 | transfer_manager.cpp | FUN_008acd10 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch26.rs | YES | UNVERIFIED |  |
| 0x008ad0e0 | transfer_manager.cpp | FUN_008ad0e0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x008b3240 | transfer_manager.cpp | FUN_008b3240 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch23.rs;crates/cm-domain/src/screen_batch24.rs;crates/cm-domain/src/screen_batch25.rs;crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x008b3260 | transfer_manager.cpp | FUN_008b3260 | PORTED_BEHAVIOURAL | crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x008b8870 | transfer_manager.cpp | FUN_008b8870 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x008ba4b0 | transfer_manager.cpp | FUN_008ba4b0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x008bbda0 | transfer_manager.cpp | FUN_008bbda0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch22.rs | YES | UNVERIFIED |  |
| 0x008bc140 | transfer_manager.cpp | FUN_008bc140 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x008bd0a0 | transfer_manager.cpp | FUN_008bd0a0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch26.rs;crates/cm-domain/src/transfer_eligibility.rs | YES | UNVERIFIED |  |
| 0x008bd590 | transfer_manager.cpp | FUN_008bd590 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch27.rs | YES | UNVERIFIED |  |
| 0x008bd5f0 | transfer_manager.cpp | FUN_008bd5f0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch26.rs | YES | UNVERIFIED |  |
| 0x008d2d20 | transfer_offer.cpp | FUN_008d2d20 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x008d48b0 | transfer_offer.cpp | FUN_008d48b0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x008d4b10 | transfer_offer.cpp | FUN_008d4b10 | PORTED_BEHAVIOURAL | crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x008d6510 | transfer_offer.cpp | FUN_008d6510 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch22.rs;crates/cm-domain/src/screen_batch23.rs | YES | UNVERIFIED |  |
| 0x008d6720 | transfer_screens.cpp | FUN_008d6720 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch22.rs | YES | UNVERIFIED |  |
| 0x008da210 | transfer_screens.cpp | FUN_008da210 | UNKNOWN | crates/cm-domain/src/screen_batch22.rs;crates/cm-domain/src/screen_batch23.rs | INDIRECT | UNVERIFIED |  |
| 0x008dcd50 | transfer_screens.cpp | FUN_008dcd50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008dcfd0 | transfer_screens.cpp | FUN_008dcfd0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch23.rs | YES | UNVERIFIED |  |
| 0x008dd030 | transfer_screens.cpp | FUN_008dd030 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch23.rs | YES | UNVERIFIED |  |
| 0x008ddc90 | transfer_screens.cpp | FUN_008ddc90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008def70 | transfer_screens.cpp | FUN_008def70 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch23.rs | YES | UNVERIFIED |  |
| 0x008df8f0 | transfer_screens.cpp | FUN_008df8f0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch23.rs;crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x008df9f0 | transfer_screens.cpp | FUN_008df9f0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch23.rs;crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x008dfd00 | transfer_screens.cpp | FUN_008dfd00 | PORTED_BEHAVIOURAL | crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x008e0560 | transfer_screens.cpp | FUN_008e0560 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch24.rs | YES | UNVERIFIED |  |
| 0x008e0760 | transfer_screens.cpp | FUN_008e0760 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch23.rs;crates/cm-domain/src/screen_batch24.rs;crates/cm-domain/src/screen_batch26.rs | YES | UNVERIFIED |  |
| 0x008e0bb0 | transfer_screens.cpp | FUN_008e0bb0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch24.rs | YES | UNVERIFIED |  |
| 0x008e10c0 | transfer_screens.cpp | FUN_008e10c0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch25.rs;crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x008e1250 | transfer_screens.cpp | FUN_008e1250 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008e26a0 | transfer_screens.cpp | FUN_008e26a0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch25.rs | YES | UNVERIFIED |  |
| 0x008e26f0 | transfer_screens.cpp | FUN_008e26f0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch25.rs | YES | UNVERIFIED |  |
| 0x008e3320 | transfer_screens.cpp | FUN_008e3320 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch25.rs | YES | UNVERIFIED |  |
| 0x008e3700 | transfer_screens.cpp | FUN_008e3700 | UNKNOWN | crates/cm-domain/src/menu.rs;crates/cm-domain/src/screen_transfers.rs;crates/cm-domain/src/sidebar_dispatcher.rs;crates/cm-render/src/dispatcher.rs;crates/cm-render/src/view_render.rs | INDIRECT | UNVERIFIED |  |
| 0x008e37e0 | transfer_screens.cpp | FUN_008e37e0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_transfers.rs | YES | UNVERIFIED |  |
| 0x008e5040 | transfer_screens.cpp | FUN_008e5040 | PORTED_BEHAVIOURAL | crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x008e50e0 | transfer_screens.cpp | FUN_008e50e0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch25.rs | YES | UNVERIFIED |  |
| 0x008e5380 | transfer_screens.cpp | FUN_008e5380 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008e61e0 | transfer_screens.cpp | FUN_008e61e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008e6cc0 | transfer_screens.cpp | FUN_008e6cc0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch26.rs | YES | UNVERIFIED |  |
| 0x008e7270 | transfer_screens.cpp | FUN_008e7270 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch26.rs | YES | UNVERIFIED |  |
| 0x008e72b0 | transfer_screens.cpp | FUN_008e72b0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch26.rs | YES | UNVERIFIED |  |
| 0x008e78d0 | transfer_screens.cpp | FUN_008e78d0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch26.rs | YES | UNVERIFIED |  |
| 0x008e82b0 | transfer_screens.cpp | FUN_008e82b0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch26.rs;crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x008e8590 | transfer_screens.cpp | FUN_008e8590 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch26.rs | YES | UNVERIFIED |  |
| 0x008e8920 | transfer_screens.cpp | FUN_008e8920 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch24.rs;crates/cm-domain/src/screen_batch26.rs | YES | UNVERIFIED |  |
| 0x008ea760 | transfer_screens.cpp | FUN_008ea760 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch27.rs | YES | UNVERIFIED |  |
| 0x008eaef0 | transfer_screens.cpp | FUN_008eaef0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch27.rs | YES | UNVERIFIED |  |
| 0x008eb690 | tur_cup.cpp | FUN_008eb690 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008eb950 | tur_cup.cpp | FUN_008eb950 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ebe70 | tur_cup.cpp | FUN_008ebe70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ec240 | tur_first.cpp | FUN_008ec240 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ec470 | tur_first.cpp | FUN_008ec470 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ed0a0 | tur_first.cpp | FUN_008ed0a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ed680 | tur_first.cpp | FUN_008ed680 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ed9d0 | tur_second.cpp | FUN_008ed9d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008edc00 | tur_second.cpp | FUN_008edc00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ee430 | tur_second.cpp | FUN_008ee430 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ee7a0 | tur_second_b.cpp | FUN_008ee7a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008eea00 | tur_second_b.cpp | FUN_008eea00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ef5a0 | tur_second_b.cpp | FUN_008ef5a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008efa60 | tur_second_b.cpp | FUN_008efa60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008efd90 | tur_second_b.cpp | FUN_008efd90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008effe0 | tur_second_b.cpp | FUN_008effe0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f0320 | tur_second_b.cpp | FUN_008f0320 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f0c30 | tur_second_b.cpp | FUN_008f0c30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f0f40 | tur_second_b.cpp | FUN_008f0f40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f0fd0 | turkey_awards.cpp | FUN_008f0fd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f1370 | turkey_rules.cpp | FUN_008f1370 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f5800 | ultimatum.cpp | FUN_008f5800 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch27.rs | YES | UNVERIFIED |  |
| 0x008f5d90 | usa_awards.cpp | FUN_008f5d90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f5ed0 | usa_awards.cpp | FUN_008f5ed0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f6390 | usa_mls.cpp | FUN_008f6390 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f6620 | usa_mls.cpp | FUN_008f6620 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f7970 | usa_mls.cpp | FUN_008f7970 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f7af0 | usa_mls.cpp | FUN_008f7af0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f83d0 | usa_mls.cpp | FUN_008f83d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f85f0 | usa_mls.cpp | FUN_008f85f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f88d0 | usa_mls.cpp | FUN_008f88d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f8be0 | usa_mls.cpp | FUN_008f8be0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f8fc0 | usa_mls.cpp | FUN_008f8fc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f92d0 | usa_mls.cpp | FUN_008f92d0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch28.rs | YES | UNVERIFIED |  |
| 0x008f9410 | usa_mls.cpp | FUN_008f9410 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f9560 | usa_mls_all_stars.cpp | FUN_008f9560 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f9810 | usa_mls_all_stars.cpp | FUN_008f9810 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f9950 | usa_mls_all_stars.cpp | FUN_008f9950 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f9d10 | usa_mls_all_stars.cpp | FUN_008f9d10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f9d90 | usa_mls_all_stars.cpp | FUN_008f9d90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008fa2c0 | usa_open_cup.cpp | FUN_008fa2c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008fa580 | usa_open_cup.cpp | FUN_008fa580 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008fa940 | usa_open_cup.cpp | FUN_008fa940 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008fad50 | usa_rules.cpp | FUN_008fad50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008faef0 | utils.cpp | FUN_008faef0 | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x008fb0b0 | utils.cpp | FUN_008fb0b0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch19.rs | YES | UNVERIFIED |  |
| 0x008fb240 | utils.cpp | FUN_008fb240 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch8.rs;crates/cm-render/src/screen_menu_bar.rs;crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x008fb3f0 | utils.cpp | FUN_008fb3f0 | UNKNOWN | crates/cm-domain/src/screen_batch19.rs;crates/cm-render/src/scrman.rs | INDIRECT | UNVERIFIED |  |
| 0x008fc660 | utils.cpp | FUN_008fc660 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch24.rs;crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x008fe000 | wales_awards.cpp | FUN_008fe000 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008fe140 | wales_awards.cpp | FUN_008fe140 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008fe5b0 | wales_rules.cpp | FUN_008fe5b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008fe710 | wc_african_cup.cpp | FUN_008fe710 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008feb20 | wc_african_cup.cpp | FUN_008feb20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ff630 | wc_african_cup.cpp | FUN_008ff630 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ffbb0 | wc_african_cup.cpp | FUN_008ffbb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ffef0 | wc_african_cup.cpp | FUN_008ffef0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00900230 | wc_african_cup.cpp | FUN_00900230 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009006e0 | wc_african_cup.cpp | FUN_009006e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009010e0 | wc_asia_league.cpp | FUN_009010e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009012f0 | wc_asia_league.cpp | FUN_009012f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00901fb0 | wc_asia_league.cpp | FUN_00901fb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009020b0 | wc_asia_league.cpp | FUN_009020b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00902290 | wc_asia_league.cpp | FUN_00902290 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00902620 | wc_asia_league.cpp | FUN_00902620 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00902a20 | wc_asia_league.cpp | FUN_00902a20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00902c20 | wc_asia_league.cpp | FUN_00902c20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00903040 | wc_asia_league.cpp | FUN_00903040 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00903230 | wc_asia_league.cpp | FUN_00903230 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00903440 | wc_asia_league.cpp | FUN_00903440 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009042c0 | wc_concacaf_cup.cpp | FUN_009042c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00905220 | wc_concacaf_cup.cpp | FUN_00905220 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00905320 | wc_concacaf_cup.cpp | FUN_00905320 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00905730 | wc_concacaf_cup.cpp | FUN_00905730 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009061b0 | wc_concacaf_cup.cpp | FUN_009061b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009065f0 | wc_concacaf_cup.cpp | FUN_009065f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00906800 | wc_concacaf_cup.cpp | FUN_00906800 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00906d10 | wc_concacaf_cup.cpp | FUN_00906d10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00907290 | wc_concacaf_cup.cpp | FUN_00907290 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00907700 | wc_europe_league.cpp | FUN_00907700 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009079b0 | wc_europe_league.cpp | FUN_009079b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009082c0 | wc_europe_league.cpp | FUN_009082c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009083b0 | wc_europe_league.cpp | FUN_009083b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009085c0 | wc_europe_league.cpp | FUN_009085c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00908bc0 | wc_europe_league.cpp | FUN_00908bc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00908e00 | wc_europe_league.cpp | FUN_00908e00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00909320 | wc_europe_league.cpp | FUN_00909320 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00909630 | wc_europe_league.cpp | FUN_00909630 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00909730 | wc_europe_league.cpp | FUN_00909730 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009099f0 | wc_europe_league.cpp | FUN_009099f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00909cd0 | wc_europe_league.cpp | FUN_00909cd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0090ac50 | wc_europe_league.cpp | FUN_0090ac50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0090d050 | wc_europe_league.cpp | FUN_0090d050 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0090d480 | wc_oceania_league.cpp | FUN_0090d480 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0090d6d0 | wc_oceania_league.cpp | FUN_0090d6d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0090e0d0 | wc_oceania_league.cpp | FUN_0090e0d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0090e670 | wc_oceania_league.cpp | FUN_0090e670 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0090e8a0 | wc_oceania_league.cpp | FUN_0090e8a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0090eb10 | wc_oceania_league.cpp | FUN_0090eb10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0090ecd0 | wc_oceania_league.cpp | FUN_0090ecd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0090ef50 | wc_oceania_league.cpp | FUN_0090ef50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0090f400 | wc_oceania_league.cpp | FUN_0090f400 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0090f780 | wc_south_american_league.cpp | FUN_0090f780 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0090f9c0 | wc_south_american_league.cpp | FUN_0090f9c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009101d0 | wc_south_american_league.cpp | FUN_009101d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00910370 | wc_south_american_league.cpp | FUN_00910370 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00910620 | wc_south_american_league.cpp | FUN_00910620 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00911810 | wc_south_american_league.cpp | FUN_00911810 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009138e0 | wel_cup.cpp | FUN_009138e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00913ba0 | wel_cup.cpp | FUN_00913ba0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00913f70 | wel_cup.cpp | FUN_00913f70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009142f0 | wel_first.cpp | FUN_009142f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00914520 | wel_first.cpp | FUN_00914520 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00915350 | wel_lge_cup.cpp | FUN_00915350 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009155c0 | wel_lge_cup.cpp | FUN_009155c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00915bc0 | wel_lge_cup.cpp | FUN_00915bc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00915fc0 | wel_lge_cup.cpp | FUN_00915fc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009161c0 | wel_lge_cup.cpp | FUN_009161c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009164d0 | wel_lge_cup.cpp | FUN_009164d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009169b0 | wel_lge_cup.cpp | FUN_009169b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00916cb0 | wel_prm_cup.cpp | FUN_00916cb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00916f10 | wel_prm_cup.cpp | FUN_00916f10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00917520 | wel_prm_cup.cpp | FUN_00917520 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00917910 | wel_prm_cup.cpp | FUN_00917910 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00917b10 | wel_prm_cup.cpp | FUN_00917b10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00917d20 | wel_prm_cup.cpp | FUN_00917d20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00918370 | wel_prm_cup.cpp | FUN_00918370 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009186b0 | world_club_champ.cpp | FUN_009186b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00918840 | world_club_champ.cpp | FUN_00918840 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00919010 | world_club_champ.cpp | FUN_00919010 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009192b0 | world_club_champ.cpp | FUN_009192b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009193d0 | world_club_champ.cpp | FUN_009193d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009196b0 | world_club_champ.cpp | FUN_009196b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00919c20 | world_club_champ.cpp | FUN_00919c20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00919e20 | world_club_champ.cpp | FUN_00919e20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091aa20 | world_club_cup.cpp | FUN_0091aa20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091ace0 | world_club_cup.cpp | FUN_0091ace0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091ae20 | world_club_cup.cpp | FUN_0091ae20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091b4c0 | world_cup.cpp | FUN_0091b4c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091b740 | world_cup.cpp | FUN_0091b740 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091c660 | world_cup.cpp | FUN_0091c660 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091cab0 | world_cup.cpp | FUN_0091cab0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091cce0 | world_cup.cpp | FUN_0091cce0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091cfb0 | world_cup.cpp | FUN_0091cfb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091dcb0 | world_cup.cpp | FUN_0091dcb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091dd60 | world_cup.cpp | FUN_0091dd60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091deb0 | world_cup.cpp | FUN_0091deb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091e250 | world_cup.cpp | FUN_0091e250 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091e2e0 | world_cup.cpp | FUN_0091e2e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091eca0 | world_cup.cpp | FUN_0091eca0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091f080 | year_award.cpp | FUN_0091f080 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091f380 | year_award.cpp | FUN_0091f380 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091f580 | year_award.cpp | FUN_0091f580 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00921770 | zipdir.cpp | FUN_00921770 | PORTED_BEHAVIOURAL | crates/cm-domain/src/human_manager.rs | YES | UNVERIFIED |  |
| 0x00933d2f | zipdir.cpp | FUN_00933d2f | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch32.rs;crates/cm-domain/src/screen_manager_batch.rs | YES | UNVERIFIED |  |
| 0x00934c8e | zipdir.cpp | FUN_00934c8e | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x0093543f | zipdir.cpp | FUN_0093543f | PORTED_BEHAVIOURAL | crates/cm-domain/src/c15_1_world_apply.rs;crates/cm-domain/src/finance.rs | YES | UNVERIFIED |  |
| 0x009354f4 | zipdir.cpp | FUN_009354f4 | PORTED_BEHAVIOURAL | crates/cm-render/src/lang_bank.rs | YES | UNVERIFIED |  |

## REGISTRY

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x006725c0 | main.cpp | FUN_006725c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |

## RESOURCE

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x004135c0 | australia_rules.cpp | FUN_004135c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004433a0 | club_history.cpp | FUN_004433a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004c6ea0 | comp_util.cpp | FUN_004c6ea0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004c7010 | comp_util.cpp | FUN_004c7010 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004cc3c0 | conmebol_merc.cpp | FUN_004cc3c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053f0a0 | discipline.cpp | FUN_0053f0a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00584530 | finance.cpp | FUN_00584530 | PORTED_BEHAVIOURAL | crates/cm-domain/src/c15_1_world_apply.rs | YES | UNVERIFIED |  |
| 0x0058f530 | find_screens.cpp | FUN_0058f530 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e32e0 | host_country.cpp | FUN_005e32e0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/host_country.rs | YES | UNVERIFIED |  |
| 0x005e4f50 | human_manager.cpp | FUN_005e4f50 | PORTED_BEHAVIOURAL | crates/cm-domain/src/human_manager.rs | YES | UNVERIFIED |  |
| 0x007838d0 | oceania_nations.cpp | FUN_007838d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00789a00 | olympics.cpp | FUN_00789a00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00789a40 | olympics.cpp | FUN_00789a40 | UNKNOWN | crates/cm-domain/src/screen_batch8.rs;crates/cm-render/src/dispatcher.rs | INDIRECT | UNVERIFIED |  |
| 0x00789a80 | olympics.cpp | FUN_00789a80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00789b50 | os.cpp | FUN_00789b50 | UNKNOWN | crates/cm-domain/src/sidebar_dispatcher.rs;crates/cm-render/src/dispatcher.rs | INDIRECT | UNVERIFIED |  |
| 0x00842bc0 | spain_rules.cpp | FUN_00842bc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00851340 | staff_records.cpp | FUN_00851340 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0089d5c0 | training_edit_session.cpp | FUN_0089d5c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008fc280 | utils.cpp | FUN_008fc280 | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x00911b80 | wc_south_american_league.cpp | FUN_00911b80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009354ca | zipdir.cpp | FUN_009354ca | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x0093cc01 | zipdir.cpp | FUN_0093cc01 | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |

## SOUND

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x005c2180 | game_config.cpp | FUN_005c2180 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch8.rs | YES | UNVERIFIED |  |
| 0x006bd530 | match_events.cpp | FUN_006bd530 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006bf870 | match_events.cpp | FUN_006bf870 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |

## UNKNOWN

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00401230 | african_nations.cpp | FUN_00401230 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00401ae0 | african_nations.cpp | FUN_00401ae0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00402b00 | african_nations.cpp | FUN_00402b00 | PORTED_BEHAVIOURAL | crates/cm-render/src/gen_screen_types.rs;crates/cm-render/src/widget_pool.rs | YES | UNVERIFIED |  |
| 0x00403640 | area.cpp | FUN_00403640 | PORTED_BEHAVIOURAL | crates/cm-widget/src/lib.rs | YES | UNVERIFIED |  |
| 0x004044d0 | arg_prm.cpp | FUN_004044d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004056b0 | arg_prm.cpp | FUN_004056b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00406e60 | arg_second.cpp | FUN_00406e60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00407e90 | arg_second.cpp | FUN_00407e90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040a770 | argentina_rules.cpp | FUN_0040a770 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040adb0 | asia_club_champ.cpp | FUN_0040adb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040add0 | asia_club_champ.cpp | FUN_0040add0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040b9b0 | asia_club_champ.cpp | FUN_0040b9b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040cba0 | asia_club_champ.cpp | FUN_0040cba0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040ce50 | asia_cup_winner.cpp | FUN_0040ce50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040ce70 | asia_cup_winner.cpp | FUN_0040ce70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040e6d0 | asia_cup_winner.cpp | FUN_0040e6d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040e940 | asia_nations.cpp | FUN_0040e940 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0040f3e0 | asia_nations.cpp | FUN_0040f3e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00410740 | asia_super_cup.cpp | FUN_00410740 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00410760 | asia_super_cup.cpp | FUN_00410760 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00410fa0 | aus_nsl.cpp | FUN_00410fa0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00411f30 | aus_nsl.cpp | FUN_00411f30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00413330 | australia_rules.cpp | FUN_00413330 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00413900 | australia_rules.cpp | FUN_00413900 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00414e50 | award_manager.cpp | FUN_00414e50 | UNKNOWN | crates/cm-domain/src/screen_batch5.rs | INDIRECT | UNVERIFIED |  |
| 0x0041dd70 | bel_fa_cup.cpp | FUN_0041dd70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0041dd90 | bel_fa_cup.cpp | FUN_0041dd90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0041eab0 | bel_first.cpp | FUN_0041eab0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0041f530 | bel_first.cpp | FUN_0041f530 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0041fac0 | bel_first.cpp | FUN_0041fac0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0041fdb0 | bel_second.cpp | FUN_0041fdb0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004209f0 | bel_second.cpp | FUN_004209f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00420c60 | bel_second.cpp | FUN_00420c60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00421c10 | bel_super.cpp | FUN_00421c10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00421c30 | bel_super.cpp | FUN_00421c30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00422470 | bel_third.cpp | FUN_00422470 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00423240 | bel_third.cpp | FUN_00423240 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00425990 | belgium_rules.cpp | FUN_00425990 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00426770 | bra_champ_cup.cpp | FUN_00426770 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00426e70 | bra_champ_cup.cpp | FUN_00426e70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00428250 | bra_cup.cpp | FUN_00428250 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00428270 | bra_cup.cpp | FUN_00428270 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00428b00 | bra_cup.cpp | FUN_00428b00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042a6b0 | bra_nat_first.cpp | FUN_0042a6b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042b1e0 | bra_nat_first.cpp | FUN_0042b1e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042cca0 | bra_nat_second.cpp | FUN_0042cca0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042d7b0 | bra_nat_second.cpp | FUN_0042d7b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042e160 | bra_nat_third.cpp | FUN_0042e160 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0042ea90 | bra_nat_third.cpp | FUN_0042ea90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00430130 | bra_reg_bahia.cpp | FUN_00430130 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00431250 | bra_reg_central.cpp | FUN_00431250 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00431b70 | bra_reg_central.cpp | FUN_00431b70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004325c0 | bra_reg_gaucho.cpp | FUN_004325c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00433770 | bra_reg_goias.cpp | FUN_00433770 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00434040 | bra_reg_goias.cpp | FUN_00434040 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00434910 | bra_reg_minas_gerais.cpp | FUN_00434910 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00435aa0 | bra_reg_minas_gerais.cpp | FUN_00435aa0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00435ed0 | bra_reg_north.cpp | FUN_00435ed0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00436f80 | bra_reg_northeast.cpp | FUN_00436f80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00437f90 | bra_reg_parana.cpp | FUN_00437f90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00438930 | bra_reg_parana.cpp | FUN_00438930 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004390e0 | bra_reg_parana.cpp | FUN_004390e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00439510 | bra_reg_pern.cpp | FUN_00439510 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00439e30 | bra_reg_pern.cpp | FUN_00439e30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043a860 | bra_reg_rio.cpp | FUN_0043a860 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043b200 | bra_reg_rio.cpp | FUN_0043b200 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043b8d0 | bra_reg_rio.cpp | FUN_0043b8d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043bd20 | bra_reg_santa.cpp | FUN_0043bd20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043c640 | bra_reg_santa.cpp | FUN_0043c640 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043ce70 | bra_reg_sp.cpp | FUN_0043ce70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0043fd20 | brazil_rules.cpp | FUN_0043fd20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0043fd50 | brazil_rules.cpp | FUN_0043fd50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0043fda0 | brazil_rules.cpp | FUN_0043fda0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0043fe90 | brazil_rules.cpp | FUN_0043fe90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00490f80 | coach.cpp | FUN_00490f80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00490fd0 | coach.cpp | FUN_00490fd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004915e0 | comp.cpp | FUN_004915e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00491780 | comp.cpp | FUN_00491780 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004928b0 | comp.cpp | FUN_004928b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00492b00 | comp.cpp | FUN_00492b00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004b4fb0 | comp_stats.cpp | FUN_004b4fb0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch33.rs | YES | UNVERIFIED |  |
| 0x004b6110 | comp_util.cpp | FUN_004b6110 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004b62e0 | comp_util.cpp | FUN_004b62e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004b6440 | comp_util.cpp | FUN_004b6440 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004b6c90 | comp_util.cpp | FUN_004b6c90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004c7210 | con_champ.cpp | FUN_004c7210 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004c7230 | con_champ.cpp | FUN_004c7230 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004c7c50 | con_merc_cup.cpp | FUN_004c7c50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004c82a0 | con_merc_cup.cpp | FUN_004c82a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004c94b0 | conmebol_liber.cpp | FUN_004c94b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004c9c60 | conmebol_liber.cpp | FUN_004c9c60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004cac20 | conmebol_merc.cpp | FUN_004cac20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004cb320 | conmebol_merc.cpp | FUN_004cb320 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004e6620 | contract_screens.cpp | FUN_004e6620 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004fdcf0 | cro_a1.cpp | FUN_004fdcf0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004fe5a0 | cro_a1.cpp | FUN_004fe5a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004ff210 | cro_a2a.cpp | FUN_004ff210 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004ffac0 | cro_a2a.cpp | FUN_004ffac0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00500260 | cro_a2b.cpp | FUN_00500260 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00500b10 | cro_a2b.cpp | FUN_00500b10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00501330 | cro_cup.cpp | FUN_00501330 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00501350 | cro_cup.cpp | FUN_00501350 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00502350 | croatia_rules.cpp | FUN_00502350 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00502370 | croatia_rules.cpp | FUN_00502370 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00503ca0 | cup.cpp | FUN_00503ca0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00508550 | cup.cpp | FUN_00508550 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0050b790 | cup.cpp | FUN_0050b790 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0050b9c0 | cup.cpp | FUN_0050b9c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0050ba60 | cup.cpp | FUN_0050ba60 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0050c270 | cup.cpp | FUN_0050c270 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0050e9b0 | cup_stage.cpp | FUN_0050e9b0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch8.rs | YES | UNVERIFIED |  |
| 0x00529fe0 | database.cpp | FUN_00529fe0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x00537160 | date.cpp | FUN_00537160 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053a760 | den_cup.cpp | FUN_0053a760 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053a780 | den_cup.cpp | FUN_0053a780 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053b290 | den_cup.cpp | FUN_0053b290 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053b4e0 | den_first.cpp | FUN_0053b4e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053bf90 | den_first.cpp | FUN_0053bf90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053c430 | den_prm.cpp | FUN_0053c430 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053d5e0 | den_prm.cpp | FUN_0053d5e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053dc70 | den_second.cpp | FUN_0053dc70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053e740 | den_second.cpp | FUN_0053e740 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053f040 | denmark_awards.cpp | FUN_0053f040 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0053fe10 | discipline.cpp | FUN_0053fe10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00540870 | discipline.cpp | FUN_00540870 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005467c0 | discipline.cpp | FUN_005467c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00546f60 | discipline.cpp | FUN_00546f60 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00546fc0 | discipline.cpp | FUN_00546fc0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00554870 | eng_auto_cup.cpp | FUN_00554870 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00554890 | eng_auto_cup.cpp | FUN_00554890 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005552d0 | eng_auto_cup.cpp | FUN_005552d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00556060 | eng_cc_cup.cpp | FUN_00556060 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00556080 | eng_cc_cup.cpp | FUN_00556080 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00556ec0 | eng_cc_cup.cpp | FUN_00556ec0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00557160 | eng_charity.cpp | FUN_00557160 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00557180 | eng_charity.cpp | FUN_00557180 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00557770 | eng_charity.cpp | FUN_00557770 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005579c0 | eng_conf.cpp | FUN_005579c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00558a20 | eng_conf.cpp | FUN_00558a20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00558e70 | eng_fa_cup.cpp | FUN_00558e70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00558e90 | eng_fa_cup.cpp | FUN_00558e90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055aac0 | eng_fa_trophy.cpp | FUN_0055aac0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055aae0 | eng_fa_trophy.cpp | FUN_0055aae0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055b560 | eng_first.cpp | FUN_0055b560 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055ca50 | eng_first.cpp | FUN_0055ca50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055d150 | eng_prm.cpp | FUN_0055d150 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055e710 | eng_prm.cpp | FUN_0055e710 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055f260 | eng_second.cpp | FUN_0055f260 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00560320 | eng_second.cpp | FUN_00560320 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00560d70 | eng_third.cpp | FUN_00560d70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00561e30 | eng_third.cpp | FUN_00561e30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00563910 | england_rules.cpp | FUN_00563910 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00563d30 | england_rules.cpp | FUN_00563d30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00563f70 | eur_super_cup.cpp | FUN_00563f70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00563f90 | eur_super_cup.cpp | FUN_00563f90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00564750 | euro_champ.cpp | FUN_00564750 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005652d0 | euro_champ.cpp | FUN_005652d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005653d0 | euro_champ.cpp | FUN_005653d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00566a20 | euro_champ.cpp | FUN_00566a20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00566d20 | euro_champ_qual.cpp | FUN_00566d20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005685a0 | euro_champ_qual.cpp | FUN_005685a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00569cc0 | euro_champ_qual.cpp | FUN_00569cc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00569cf0 | euro_champ_qual.cpp | FUN_00569cf0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0056a830 | euro_champ_qual.cpp | FUN_0056a830 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0056cf30 | euro_champ_qual.cpp | FUN_0056cf30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0056cf70 | euro_champ_qual.cpp | FUN_0056cf70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00575230 | fifa_confed.cpp | FUN_00575230 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00575b60 | fifa_confed.cpp | FUN_00575b60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00575da0 | fifa_confed.cpp | FUN_00575da0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00576cd0 | fifa_confed.cpp | FUN_00576cd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005774c0 | fifa_rankings.cpp | FUN_005774c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00577f40 | fifa_rankings.cpp | FUN_00577f40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0057c640 | fin_cup.cpp | FUN_0057c640 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0057c660 | fin_cup.cpp | FUN_0057c660 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0057d2f0 | fin_first.cpp | FUN_0057d2f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0057dd90 | fin_first.cpp | FUN_0057dd90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0057f600 | fin_prm.cpp | FUN_0057f600 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0057fe90 | fin_prm.cpp | FUN_0057fe90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00580250 | fin_prm.cpp | FUN_00580250 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0059bed0 | fog_of_war.cpp | FUN_0059bed0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/ui_schema.rs;crates/cm-render/src/primitives.rs | YES | UNVERIFIED |  |
| 0x005a3730 | fra_cfa.cpp | FUN_005a3730 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a3750 | fra_cfa.cpp | FUN_005a3750 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a4560 | fra_cup.cpp | FUN_005a4560 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a4580 | fra_cup.cpp | FUN_005a4580 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a5770 | fra_first.cpp | FUN_005a5770 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a5790 | fra_first.cpp | FUN_005a5790 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a57d0 | fra_first.cpp | FUN_005a57d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a57f0 | fra_first.cpp | FUN_005a57f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a5800 | fra_first.cpp | FUN_005a5800 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a6650 | fra_first.cpp | FUN_005a6650 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a6d90 | fra_lge_cup.cpp | FUN_005a6d90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a6db0 | fra_lge_cup.cpp | FUN_005a6db0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a7810 | fra_lge_cup.cpp | FUN_005a7810 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a7a90 | fra_lower.cpp | FUN_005a7a90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a7ab0 | fra_lower.cpp | FUN_005a7ab0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a8570 | fra_lower.cpp | FUN_005a8570 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a8810 | fra_second.cpp | FUN_005a8810 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a9370 | fra_second.cpp | FUN_005a9370 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a97b0 | fra_super.cpp | FUN_005a97b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005a97d0 | fra_super.cpp | FUN_005a97d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005aa020 | fra_third.cpp | FUN_005aa020 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005aa9d0 | fra_third.cpp | FUN_005aa9d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005ab9e0 | france_rules.cpp | FUN_005ab9e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005afaa0 | friendly.cpp | FUN_005afaa0 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x005b0b00 | friendly.cpp | FUN_005b0b00 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch9.rs;crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x005c1970 | game_config.cpp | FUN_005c1970 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch20.rs | YES | UNVERIFIED |  |
| 0x005c2aa0 | ger_cup.cpp | FUN_005c2aa0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c2ac0 | ger_cup.cpp | FUN_005c2ac0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c3870 | ger_first.cpp | FUN_005c3870 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c50b0 | ger_first.cpp | FUN_005c50b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c5f60 | ger_lge_cup.cpp | FUN_005c5f60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c5f80 | ger_lge_cup.cpp | FUN_005c5f80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c6840 | ger_lge_cup.cpp | FUN_005c6840 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c6ab0 | ger_regional.cpp | FUN_005c6ab0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c7690 | ger_regional.cpp | FUN_005c7690 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c8190 | ger_second.cpp | FUN_005c8190 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c9080 | ger_second.cpp | FUN_005c9080 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005c9c10 | germany_awards.cpp | FUN_005c9c10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005ce6e0 | goldcup.cpp | FUN_005ce6e0 | PORTED_BEHAVIOURAL | crates/cm-render/src/area.rs | YES | UNVERIFIED |  |
| 0x005cf840 | goldcup.cpp | FUN_005cf840 | PORTED_BEHAVIOURAL | crates/cm-domain/src/ui_schema.rs;crates/cm-render/src/font.rs;crates/cm-render/src/primitives.rs | YES | UNVERIFIED |  |
| 0x005d1900 | goldcup.cpp | FUN_005d1900 | PORTED_BEHAVIOURAL | crates/cm-render/src/blit.rs | YES | UNVERIFIED |  |
| 0x005d2490 | gre_cup.cpp | FUN_005d2490 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d2cf0 | gre_cup.cpp | FUN_005d2cf0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d3f10 | gre_prm.cpp | FUN_005d3f10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d4780 | gre_prm.cpp | FUN_005d4780 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d4db0 | gre_second.cpp | FUN_005d4db0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d56d0 | gre_second.cpp | FUN_005d56d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d5af0 | gre_super.cpp | FUN_005d5af0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005d5b10 | gre_super.cpp | FUN_005d5b10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005dd070 | hol_cup.cpp | FUN_005dd070 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005dd900 | hol_cup.cpp | FUN_005dd900 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005deb00 | hol_cup.cpp | FUN_005deb00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005df140 | hol_first.cpp | FUN_005df140 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005dfdd0 | hol_first.cpp | FUN_005dfdd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e0040 | hol_first.cpp | FUN_005e0040 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e0e60 | hol_first.cpp | FUN_005e0e60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e12b0 | hol_prm.cpp | FUN_005e12b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e1ff0 | hol_prm.cpp | FUN_005e1ff0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e2470 | hol_super.cpp | FUN_005e2470 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e2490 | hol_super.cpp | FUN_005e2490 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e32c0 | holland_rules.cpp | FUN_005e32c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x005e4800 | host_country.cpp | FUN_005e4800 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005f71d0 | index.cpp | FUN_005f71d0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/finance.rs | YES | UNVERIFIED |  |
| 0x0061b930 | inter_amer_cup.cpp | FUN_0061b930 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061b950 | inter_amer_cup.cpp | FUN_0061b950 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061c890 | intertoto_cup.cpp | FUN_0061c890 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061c8b0 | intertoto_cup.cpp | FUN_0061c8b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061d900 | ire_chal_cup.cpp | FUN_0061d900 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061d920 | ire_chal_cup.cpp | FUN_0061d920 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061e4c0 | ire_first.cpp | FUN_0061e4c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061f100 | ire_first.cpp | FUN_0061f100 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061fcd0 | ire_leinster_cup.cpp | FUN_0061fcd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0061fcf0 | ire_leinster_cup.cpp | FUN_0061fcf0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00620680 | ire_lge_cup.cpp | FUN_00620680 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00620d20 | ire_lge_cup.cpp | FUN_00620d20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00621fd0 | ire_munster_cup.cpp | FUN_00621fd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00621ff0 | ire_munster_cup.cpp | FUN_00621ff0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006228d0 | ire_munster_cup.cpp | FUN_006228d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00622ac0 | ire_pres_cup.cpp | FUN_00622ac0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00622ae0 | ire_pres_cup.cpp | FUN_00622ae0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00623040 | ire_pres_cup.cpp | FUN_00623040 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00623470 | ire_prm.cpp | FUN_00623470 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00623f90 | ire_prm.cpp | FUN_00623f90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00624510 | ire_super_cup.cpp | FUN_00624510 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00624b50 | ire_super_cup.cpp | FUN_00624b50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00625800 | ireland_awards.cpp | FUN_00625800 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006259d0 | ireland_rules.cpp | FUN_006259d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00625a50 | ireland_rules.cpp | FUN_00625a50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00625e20 | ita_c1_super.cpp | FUN_00625e20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00625e40 | ita_c1_super.cpp | FUN_00625e40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00626510 | ita_c_cup.cpp | FUN_00626510 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00626d60 | ita_c_cup.cpp | FUN_00626d60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006281d0 | ita_cup.cpp | FUN_006281d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006289d0 | ita_cup.cpp | FUN_006289d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00629f80 | ita_ser_a.cpp | FUN_00629f80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0062acc0 | ita_ser_a.cpp | FUN_0062acc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0062be10 | ita_ser_a.cpp | FUN_0062be10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0062f140 | ita_ser_b.cpp | FUN_0062f140 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0062ffa0 | ita_ser_b.cpp | FUN_0062ffa0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00634310 | ita_ser_c1a.cpp | FUN_00634310 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00634de0 | ita_ser_c1a.cpp | FUN_00634de0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006384c0 | ita_ser_c1b.cpp | FUN_006384c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00638f90 | ita_ser_c1b.cpp | FUN_00638f90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0063c850 | ita_ser_c2a.cpp | FUN_0063c850 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0063d320 | ita_ser_c2a.cpp | FUN_0063d320 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006409e0 | ita_ser_c2b.cpp | FUN_006409e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006414b0 | ita_ser_c2b.cpp | FUN_006414b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00644b70 | ita_ser_c2c.cpp | FUN_00644b70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00645640 | ita_ser_c2c.cpp | FUN_00645640 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00648ea0 | ita_super.cpp | FUN_00648ea0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00648ec0 | ita_super.cpp | FUN_00648ec0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006494a0 | ita_super.cpp | FUN_006494a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064b5a0 | jap_emp_cup.cpp | FUN_0064b5a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064b5c0 | jap_emp_cup.cpp | FUN_0064b5c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064c1d0 | jap_emp_cup.cpp | FUN_0064c1d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064c420 | jap_j1.cpp | FUN_0064c420 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064ccf0 | jap_j1.cpp | FUN_0064ccf0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064d010 | jap_j1.cpp | FUN_0064d010 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064da60 | jap_j1.cpp | FUN_0064da60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064dad0 | jap_j1.cpp | FUN_0064dad0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064dda0 | jap_j2.cpp | FUN_0064dda0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064e810 | jap_j2.cpp | FUN_0064e810 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064ecd0 | jap_j_cup.cpp | FUN_0064ecd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064ecf0 | jap_j_cup.cpp | FUN_0064ecf0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064f4f0 | jap_j_cup.cpp | FUN_0064f4f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064f6e0 | jap_super.cpp | FUN_0064f6e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0064f700 | jap_super.cpp | FUN_0064f700 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00650890 | japan_rules.cpp | FUN_00650890 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006537a0 | key_nation.cpp |  | PORTED_BEHAVIOURAL | crates/cm-ui-app/src/game_state.rs | YES | UNVERIFIED |  |
| 0x006809d0 | manager_manager.cpp | FUN_006809d0 | UNKNOWN | crates/cm-domain/src/sidebar_dispatcher.rs;crates/cm-render/src/dispatcher.rs | INDIRECT | UNVERIFIED |  |
| 0x00698140 | manager_screens.cpp | FUN_00698140 | UNKNOWN | crates/cm-domain/src/screen_batch4.rs;crates/cm-domain/src/sidebar_dispatcher.rs | INDIRECT | UNVERIFIED |  |
| 0x006986a0 | manager_screens.cpp | FUN_006986a0 | UNKNOWN | crates/cm-domain/src/screen_batch3.rs;crates/cm-domain/src/sidebar_dispatcher.rs;crates/cm-render/src/screen_wire_batch3.rs | INDIRECT | UNVERIFIED |  |
| 0x006a2730 | match_eng.cpp | FUN_006a2730 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x006a88f0 | match_eng.cpp | FUN_006a88f0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x006bb6e0 | match_eng.cpp | FUN_006bb6e0 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x006bd2e0 | match_events.cpp | FUN_006bd2e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x006fa700 | match_pl.cpp | FUN_006fa700 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x006fbdf0 | match_pl.cpp | FUN_006fbdf0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x0074da50 | mini_cup.cpp | FUN_0074da50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074dae0 | mini_cup.cpp | FUN_0074dae0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074ed90 | month_award.cpp | FUN_0074ed90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074fb20 | month_ratings.cpp | FUN_0074fb20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074fba0 | month_ratings.cpp | FUN_0074fba0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074fd70 | month_ratings.cpp | FUN_0074fd70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074fe40 | month_ratings.cpp | FUN_0074fe40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0074ffe0 | month_ratings.cpp | FUN_0074ffe0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007553f0 | national_teams.cpp | FUN_007553f0 | PORTED_BEHAVIOURAL | crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x00762730 | national_teams_screens.cpp | FUN_00762730 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007627d0 | national_teams_screens.cpp | FUN_007627d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007628e0 | national_teams_screens.cpp | FUN_007628e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00762950 | national_teams_screens.cpp | FUN_00762950 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00762b60 | national_teams_screens.cpp | FUN_00762b60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00771300 | news_screens.cpp | FUN_00771300 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00772080 | nir_charity.cpp | FUN_00772080 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007720a0 | nir_charity.cpp | FUN_007720a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00772850 | nir_cup.cpp | FUN_00772850 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00772870 | nir_cup.cpp | FUN_00772870 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007730b0 | nir_cup.cpp | FUN_007730b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00773300 | nir_first.cpp | FUN_00773300 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00773e40 | nir_first.cpp | FUN_00773e40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00774aa0 | nir_lge_cup.cpp | FUN_00774aa0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007751a0 | nir_lge_cup.cpp | FUN_007751a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00776350 | nir_prm.cpp | FUN_00776350 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00776e50 | nir_prm.cpp | FUN_00776e50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007772c0 | nor_cup.cpp | FUN_007772c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007772e0 | nor_cup.cpp | FUN_007772e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00778030 | nor_first.cpp | FUN_00778030 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00778bd0 | nor_first.cpp | FUN_00778bd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007797e0 | nor_prm.cpp | FUN_007797e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077a7d0 | nor_prm.cpp | FUN_0077a7d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077ac10 | nor_prm.cpp | FUN_0077ac10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077b8c0 | northern_ireland_rules.cpp | FUN_0077b8c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077b940 | northern_ireland_rules.cpp | FUN_0077b940 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077da00 | notes.cpp | FUN_0077da00 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch18.rs | YES | UNVERIFIED |  |
| 0x0077f2d0 | oceania_club_champ.cpp | FUN_0077f2d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077fb30 | oceania_club_champ.cpp | FUN_0077fb30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0077fee0 | oceania_club_champ.cpp | FUN_0077fee0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00780c00 | oceania_nations.cpp | FUN_00780c00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00781810 | oceania_nations.cpp | FUN_00781810 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00783b50 | oceania_nations.cpp | FUN_00783b50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00785160 | officials_manager.cpp | FUN_00785160 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x00787a20 | olympics.cpp | FUN_00787a20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00788560 | olympics.cpp | FUN_00788560 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0078a8f0 | os.cpp | FUN_0078a8f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007aec90 | player_stats.cpp | FUN_007aec90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007af0f0 | player_stats.cpp | FUN_007af0f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007af140 | player_stats.cpp | FUN_007af140 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007af400 | plot.cpp | FUN_007af400 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b0360 | pol_cup.cpp | FUN_007b0360 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b0380 | pol_cup.cpp | FUN_007b0380 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b0e60 | pol_cup.cpp | FUN_007b0e60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b1140 | pol_first.cpp | FUN_007b1140 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b1aa0 | pol_first.cpp | FUN_007b1aa0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b28a0 | pol_first.cpp | FUN_007b28a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b2ac0 | pol_lge_cup.cpp | FUN_007b2ac0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b2ae0 | pol_lge_cup.cpp | FUN_007b2ae0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b35a0 | pol_lge_cup.cpp | FUN_007b35a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b3800 | pol_second.cpp | FUN_007b3800 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b4350 | pol_second.cpp | FUN_007b4350 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b4ef0 | pol_super.cpp | FUN_007b4ef0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b4f10 | pol_super.cpp | FUN_007b4f10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b5f60 | por_cup.cpp | FUN_007b5f60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b5f80 | por_cup.cpp | FUN_007b5f80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b6d10 | por_prm.cpp | FUN_007b6d10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b7d10 | por_prm.cpp | FUN_007b7d10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b85f0 | por_prm.cpp | FUN_007b85f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b8f40 | por_second.cpp | FUN_007b8f40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b9a80 | por_second.cpp | FUN_007b9a80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007b9f40 | por_second_b.cpp | FUN_007b9f40 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ba9f0 | por_second_b.cpp | FUN_007ba9f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007bb450 | por_super.cpp | FUN_007bb450 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007bb470 | por_super.cpp | FUN_007bb470 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c2ea0 | printouts.cpp | FUN_007c2ea0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c2f60 | printouts.cpp | FUN_007c2f60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c3530 | printouts.cpp | FUN_007c3530 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c3790 | rb_argentina.cpp | FUN_007c3790 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c39e0 | rb_asia.cpp | FUN_007c39e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c3d20 | rb_australia.cpp | FUN_007c3d20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c40d0 | rb_belgium_cup.cpp | FUN_007c40d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c44b0 | rb_belgium_league.cpp | FUN_007c44b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c4830 | rb_brazil_national.cpp | FUN_007c4830 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c4b70 | rb_brazil_regional.cpp | FUN_007c4b70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c4f50 | rb_croatia.cpp | FUN_007c4f50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c5870 | rb_england.cpp | FUN_007c5870 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c5ad0 | rb_europe.cpp | FUN_007c5ad0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c5e00 | rb_finland_cup.cpp | FUN_007c5e00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c6130 | rb_finland_league.cpp | FUN_007c6130 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c6750 | rb_france.cpp | FUN_007c6750 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c6a90 | rb_germany_cup.cpp | FUN_007c6a90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c6dd0 | rb_germany_league.cpp | FUN_007c6dd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c71c0 | rb_greece.cpp | FUN_007c71c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c75b0 | rb_holland.cpp | FUN_007c75b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c78b0 | rb_international.cpp | FUN_007c78b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c7d90 | rb_ireland.cpp | FUN_007c7d90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c8160 | rb_italy_cup.cpp | FUN_007c8160 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c85a0 | rb_italy_league.cpp | FUN_007c85a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c8880 | rb_japan_cup.cpp | FUN_007c8880 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c8b20 | rb_japan_league.cpp | FUN_007c8b20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c8f20 | rb_northern_ireland.cpp | FUN_007c8f20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c9180 | rb_norway_cup.cpp | FUN_007c9180 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c9400 | rb_norway_league.cpp | FUN_007c9400 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c9640 | rb_oceania.cpp | FUN_007c9640 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c9a90 | rb_poland.cpp | FUN_007c9a90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007c9e30 | rb_portugal.cpp | FUN_007c9e30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ca1d0 | rb_russia.cpp | FUN_007ca1d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ca4a0 | rb_scotland_cup.cpp | FUN_007ca4a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ca960 | rb_scotland_league.cpp | FUN_007ca960 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007caa10 | rb_scotland_league.cpp | FUN_007caa10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007cac60 | rb_south_america.cpp | FUN_007cac60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007caff0 | rb_spain_cup.cpp | FUN_007caff0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007cb380 | rb_spain_league.cpp | FUN_007cb380 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007cb750 | rb_sweden_cup.cpp | FUN_007cb750 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007cbac0 | rb_sweden_league.cpp | FUN_007cbac0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007cbe50 | rb_turkey_cup.cpp | FUN_007cbe50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007cc1a0 | rb_turkey_league.cpp | FUN_007cc1a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007cc470 | rb_usa.cpp | FUN_007cc470 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d06b0 | ruling_body.cpp | FUN_007d06b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d0e80 | ruling_body.cpp | FUN_007d0e80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d1520 | rus_cup.cpp | FUN_007d1520 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d1540 | rus_cup.cpp | FUN_007d1540 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d1fe0 | rus_cup.cpp | FUN_007d1fe0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d2290 | rus_first.cpp | FUN_007d2290 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d2960 | rus_first.cpp | FUN_007d2960 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d2df0 | rus_prm.cpp | FUN_007d2df0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d3890 | rus_prm.cpp | FUN_007d3890 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d47f0 | russia_awards.cpp | FUN_007d47f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d4cf0 | sco_chal_cup.cpp | FUN_007d4cf0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d4d10 | sco_chal_cup.cpp | FUN_007d4d10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d5480 | sco_chal_cup.cpp | FUN_007d5480 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d5680 | sco_fa_cup.cpp | FUN_007d5680 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d56a0 | sco_fa_cup.cpp | FUN_007d56a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d63a0 | sco_first.cpp | FUN_007d63a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d6ff0 | sco_first.cpp | FUN_007d6ff0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d7d60 | sco_lge_cup.cpp | FUN_007d7d60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d7d80 | sco_lge_cup.cpp | FUN_007d7d80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007d8f70 | sco_prm.cpp | FUN_007d8f70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007dab10 | sco_prm.cpp | FUN_007dab10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007db600 | sco_second.cpp | FUN_007db600 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007dc120 | sco_second.cpp | FUN_007dc120 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007dc5a0 | sco_third.cpp | FUN_007dc5a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007dd0c0 | sco_third.cpp | FUN_007dd0c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007e6a20 | scrman.cpp | FUN_007e6a20 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch19.rs;crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x007e74e0 | scrman.cpp | FUN_007e74e0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch4.rs | YES | UNVERIFIED |  |
| 0x007ebaf0 | scrman.cpp | FUN_007ebaf0 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x007ec230 | search_edit_session.cpp | FUN_007ec230 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ec520 | search_edit_session.cpp | FUN_007ec520 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007ec640 | search_edit_session.cpp | FUN_007ec640 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007f6710 | search_screens.cpp | FUN_007f6710 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007f6730 | search_screens.cpp | FUN_007f6730 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007f6760 | search_screens.cpp | FUN_007f6760 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x007f6790 | search_screens.cpp | FUN_007f6790 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00800600 | search_screens.cpp | FUN_00800600 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00800660 | search_screens.cpp | FUN_00800660 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00803650 | search_screens.cpp | FUN_00803650 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00803d70 | search_screens.cpp | FUN_00803d70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0081a020 | setup.cpp | FUN_0081a020 | PORTED_BEHAVIOURAL | crates/cm-domain/src/league_calendar.rs | YES | UNVERIFIED |  |
| 0x008362d0 | spa_cup.cpp | FUN_008362d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008362f0 | spa_cup.cpp | FUN_008362f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00837530 | spa_first.cpp | FUN_00837530 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00838f90 | spa_first.cpp | FUN_00838f90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00839a60 | spa_first.cpp | FUN_00839a60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0083a330 | spa_lower.cpp | FUN_0083a330 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0083a350 | spa_lower.cpp | FUN_0083a350 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0083c930 | spa_lower.cpp | FUN_0083c930 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0083cd50 | spa_second.cpp | FUN_0083cd50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0083ee60 | spa_second.cpp | FUN_0083ee60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0083f320 | spa_second_b.cpp | FUN_0083f320 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00840090 | spa_second_b.cpp | FUN_00840090 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00841300 | spa_second_b.cpp | FUN_00841300 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00841a10 | spa_super.cpp | FUN_00841a10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00841a30 | spa_super.cpp | FUN_00841a30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008469c0 | stadium.cpp | FUN_008469c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00847410 | stadium.cpp | FUN_00847410 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008474a0 | stadium.cpp | FUN_008474a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00856690 | staff_records.cpp | FUN_00856690 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x00858bd0 | staff_records.cpp | FUN_00858bd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00858c20 | staff_records.cpp | FUN_00858c20 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00877170 | staff_screens.cpp | FUN_00877170 | UNKNOWN | crates/cm-domain/src/screen_batch21.rs;crates/cm-domain/src/sidebar_dispatcher.rs | INDIRECT | UNVERIFIED |  |
| 0x008773e0 | staff_screens.cpp | FUN_008773e0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch21.rs | YES | UNVERIFIED |  |
| 0x008776f0 | sub_league.cpp | FUN_008776f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00877710 | sub_league.cpp | FUN_00877710 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00877b80 | swe_cup.cpp | FUN_00877b80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00877ba0 | swe_cup.cpp | FUN_00877ba0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00879580 | swe_first.cpp | FUN_00879580 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00879e10 | swe_first.cpp | FUN_00879e10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087ab80 | swe_prm.cpp | FUN_0087ab80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087b2d0 | swe_prm.cpp | FUN_0087b2d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087c020 | swe_prm.cpp | FUN_0087c020 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087c630 | swe_second.cpp | FUN_0087c630 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087d030 | swe_second.cpp | FUN_0087d030 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087ea30 | sweden_awards.cpp | FUN_0087ea30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00884b00 | tactics_screens.cpp | FUN_00884b00 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x0089a580 | tactics_screens.cpp | FUN_0089a580 | PORTED_BEHAVIOURAL | crates/cm-domain/src/tactic_dispatcher.rs | YES | UNVERIFIED |  |
| 0x0089afd0 | tcpip.cpp | FUN_0089afd0 | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x0089d410 | training_edit_session.cpp | FUN_0089d410 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0089d460 | training_edit_session.cpp | FUN_0089d460 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0089d4c0 | training_edit_session.cpp | FUN_0089d4c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0089d770 | training_edit_session.cpp | FUN_0089d770 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008a8a70 | training_screens.cpp | FUN_008a8a70 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008df880 | transfer_screens.cpp | FUN_008df880 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch23.rs | YES | UNVERIFIED |  |
| 0x008e0af0 | transfer_screens.cpp | FUN_008e0af0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch24.rs;crates/cm-domain/src/screen_batch26.rs | YES | UNVERIFIED |  |
| 0x008eb860 | tur_cup.cpp | FUN_008eb860 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008eb880 | tur_cup.cpp | FUN_008eb880 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ec450 | tur_first.cpp | FUN_008ec450 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ed270 | tur_first.cpp | FUN_008ed270 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ed530 | tur_first.cpp | FUN_008ed530 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ed860 | tur_first.cpp | FUN_008ed860 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008edbe0 | tur_second.cpp | FUN_008edbe0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ee560 | tur_second.cpp | FUN_008ee560 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ee9e0 | tur_second_b.cpp | FUN_008ee9e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008ef740 | tur_second_b.cpp | FUN_008ef740 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f6600 | usa_mls.cpp | FUN_008f6600 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f9720 | usa_mls_all_stars.cpp | FUN_008f9720 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008f9740 | usa_mls_all_stars.cpp | FUN_008f9740 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008fa490 | usa_open_cup.cpp | FUN_008fa490 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008fa4b0 | usa_open_cup.cpp | FUN_008fa4b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008fe9f0 | wc_african_cup.cpp | FUN_008fe9f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008fea10 | wc_african_cup.cpp | FUN_008fea10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00900620 | wc_african_cup.cpp | FUN_00900620 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00901090 | wc_african_cup.cpp | FUN_00901090 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009012d0 | wc_asia_league.cpp | FUN_009012d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00902390 | wc_asia_league.cpp | FUN_00902390 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009038f0 | wc_asia_league.cpp | FUN_009038f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00904180 | wc_concacaf_cup.cpp | FUN_00904180 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009041a0 | wc_concacaf_cup.cpp | FUN_009041a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009071b0 | wc_concacaf_cup.cpp | FUN_009071b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00907990 | wc_europe_league.cpp | FUN_00907990 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009086d0 | wc_europe_league.cpp | FUN_009086d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00909ca0 | wc_europe_league.cpp | FUN_00909ca0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0090ac00 | wc_europe_league.cpp | FUN_0090ac00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0090d3e0 | wc_europe_league.cpp | FUN_0090d3e0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0090d6b0 | wc_oceania_league.cpp | FUN_0090d6b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0090e1c0 | wc_oceania_league.cpp | FUN_0090e1c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0090f9a0 | wc_south_american_league.cpp | FUN_0090f9a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00910500 | wc_south_american_league.cpp | FUN_00910500 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00910b30 | wc_south_american_league.cpp | FUN_00910b30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00910b90 | wc_south_american_league.cpp | FUN_00910b90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00911c90 | wc_south_american_league.cpp | FUN_00911c90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00912e30 | weather.cpp | FUN_00912e30 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x00913ab0 | wel_cup.cpp | FUN_00913ab0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00913ad0 | wel_cup.cpp | FUN_00913ad0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009142c0 | wel_cup.cpp | FUN_009142c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00914500 | wel_first.cpp | FUN_00914500 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00915110 | wel_first.cpp | FUN_00915110 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009155a0 | wel_lge_cup.cpp | FUN_009155a0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00915cc0 | wel_lge_cup.cpp | FUN_00915cc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00916ef0 | wel_prm_cup.cpp | FUN_00916ef0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00917620 | wel_prm_cup.cpp | FUN_00917620 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009182c0 | wel_prm_cup.cpp | FUN_009182c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00918820 | world_club_champ.cpp | FUN_00918820 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00919120 | world_club_champ.cpp | FUN_00919120 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091abf0 | world_club_cup.cpp | FUN_0091abf0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091ac10 | world_club_cup.cpp | FUN_0091ac10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091b720 | world_cup.cpp | FUN_0091b720 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091c770 | world_cup.cpp | FUN_0091c770 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091c880 | world_cup.cpp | FUN_0091c880 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091ebd0 | world_cup.cpp | FUN_0091ebd0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091f560 | year_award.cpp | FUN_0091f560 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0093bc41 | zipdir.cpp | FUN_0093bc41 | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |

## awards

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x004143d0 | award_manager.cpp | FUN_004143d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00414590 | award_manager.cpp | FUN_00414590 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004146b0 | award_manager.cpp | FUN_004146b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004146d0 | award_manager.cpp | FUN_004146d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004146f0 | award_manager.cpp | FUN_004146f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00414710 | award_manager.cpp | FUN_00414710 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00414730 | award_manager.cpp | FUN_00414730 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004147e0 | award_manager.cpp | FUN_004147e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00414810 | award_manager.cpp | FUN_00414810 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00414be0 | award_manager.cpp | FUN_00414be0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00414cd0 | award_manager.cpp | FUN_00414cd0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00414d90 | award_manager.cpp | FUN_00414d90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00414eb0 | award_manager.cpp | FUN_00414eb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00417f90 | award_shortlist.cpp | FUN_00417f90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004180e0 | award_shortlist.cpp | FUN_004180e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004180f0 | award_shortlist.cpp | FUN_004180f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00418280 | award_shortlist.cpp | FUN_00418280 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00418400 | award_shortlist.cpp | FUN_00418400 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00418580 | award_shortlist.cpp | FUN_00418580 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004187b0 | award_shortlist.cpp | FUN_004187b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00418b40 | award_shortlist.cpp | FUN_00418b40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00418e80 | award_shortlist.cpp | FUN_00418e80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00418fa0 | award_shortlist.cpp | FUN_00418fa0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00419150 | award_shortlist.cpp | FUN_00419150 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004191d0 | award_shortlist.cpp | FUN_004191d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089bd60 | team_award.cpp | FUN_0089bd60 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089c140 | team_award.cpp | FUN_0089c140 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089c490 | team_award.cpp | FUN_0089c490 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089c4c0 | team_award.cpp | FUN_0089c4c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089c600 | team_award.cpp | FUN_0089c600 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089c650 | team_award.cpp | FUN_0089c650 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089c6b0 | team_award.cpp | FUN_0089c6b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089c7a0 | team_award.cpp | FUN_0089c7a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089c840 | team_award.cpp | FUN_0089c840 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089ce30 | team_award.cpp | FUN_0089ce30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089d220 | team_award.cpp | FUN_0089d220 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089d240 | team_award.cpp | FUN_0089d240 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0091ed10 | world_cup_awards.cpp | FUN_0091ed10 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0091ef10 | world_cup_awards.cpp | FUN_0091ef10 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0091eff0 | world_cup_awards.cpp | FUN_0091eff0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0091f010 | world_cup_awards.cpp | FUN_0091f010 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00920420 | year_ratings.cpp | FUN_00920420 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00920570 | year_ratings.cpp | FUN_00920570 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00920660 | year_ratings.cpp | FUN_00920660 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00920720 | year_ratings.cpp | FUN_00920720 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00920b30 | year_ratings.cpp | FUN_00920b30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00920b70 | year_ratings.cpp | FUN_00920b70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00920de0 | year_ratings.cpp | FUN_00920de0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00921590 | year_ratings.cpp | FUN_00921590 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |

## comp-stats

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x004a5900 | comp_stats.cpp | FUN_004a5900 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004a5bf0 | comp_stats.cpp | comp_stats (no decompile) | UNKNOWN |  | BLOCKED | UNVERIFIED |  |
| 0x004a5d30 | comp_stats.cpp | FUN_004a5d30 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004a5d80 | comp_stats.cpp | FUN_004a5d80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004a5dd0 | comp_stats.cpp | comp_stats (no decompile) | UNKNOWN |  | BLOCKED | UNVERIFIED |  |
| 0x004a5de0 | comp_stats.cpp | FUN_004a5de0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004a5e30 | comp_stats.cpp | comp_stats (no decompile) | UNKNOWN |  | BLOCKED | UNVERIFIED |  |
| 0x004a5e40 | comp_stats.cpp | comp_stats (no decompile) | UNKNOWN |  | BLOCKED | UNVERIFIED |  |
| 0x004a5e50 | comp_stats.cpp | comp_stats (no decompile) | UNKNOWN |  | BLOCKED | UNVERIFIED |  |
| 0x004a5eb0 | comp_stats.cpp | comp_stats (no decompile) | UNKNOWN |  | BLOCKED | UNVERIFIED |  |
| 0x004a5f10 | comp_stats.cpp | FUN_004a5f10 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004a8a00 | comp_stats.cpp | comp_stats (no decompile) | UNKNOWN |  | BLOCKED | UNVERIFIED |  |
| 0x004a8a30 | comp_stats.cpp | comp_stats (no decompile) | UNKNOWN |  | BLOCKED | UNVERIFIED |  |
| 0x004a8a70 | comp_stats.cpp | comp_stats (no decompile) | UNKNOWN |  | BLOCKED | UNVERIFIED |  |
| 0x004a8b10 | comp_stats.cpp | comp_stats (no decompile) | UNKNOWN |  | BLOCKED | UNVERIFIED |  |
| 0x004a8b50 | comp_stats.cpp | FUN_004a8b50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004a8db0 | comp_stats.cpp | comp_stats (no decompile) | UNKNOWN |  | BLOCKED | UNVERIFIED |  |
| 0x004a9c60 | comp_stats.cpp | comp_stats (no decompile) | UNKNOWN |  | BLOCKED | UNVERIFIED |  |
| 0x004b1b80 | comp_stats.cpp | FUN_004b1b80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004b2070 | comp_stats.cpp | FUN_004b2070 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004b2090 | comp_stats.cpp | FUN_004b2090 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004b20b0 | comp_stats.cpp | FUN_004b20b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004b2350 | comp_stats.cpp | FUN_004b2350 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004b2fa0 | comp_stats.cpp | FUN_004b2fa0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004b3af0 | comp_stats.cpp | FUN_004b3af0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004b4380 | comp_stats.cpp | FUN_004b4380 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004b4480 | comp_stats.cpp | FUN_004b4480 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004b4dc0 | comp_stats.cpp | FUN_004b4dc0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004b50f0 | comp_stats.cpp | FUN_004b50f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |

## competition

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x004cc590 | conmebol_seeding.cpp | FUN_004cc590 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x004cc660 | conmebol_seeding.cpp | FUN_004cc660 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x004cc810 | conmebol_seeding.cpp | FUN_004cc810 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x004ccb10 | conmebol_seeding.cpp | FUN_004ccb10 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x004ccc70 | conmebol_seeding.cpp | FUN_004ccc70 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x004ccd40 | conmebol_seeding.cpp | FUN_004ccd40 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x004ccf40 | conmebol_seeding.cpp | FUN_004ccf40 | FOREIGN_BREADTH |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004cd410 | conmebol_seeding.cpp | FUN_004cd410 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x004cd8c0 | conmebol_seeding.cpp | FUN_004cd8c0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00525040 | database.cpp | linked +0x53 record resolver | UNKNOWN | crates/cm-domain/src/lib.rs | NO | UNVERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x0056d460 | european_cup.cpp | FUN_0056d460 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0056d670 | european_cup.cpp | FUN_0056d670 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0056d690 | european_cup.cpp | FUN_0056d690 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0056d8e0 | european_cup.cpp | FUN_0056d8e0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0056e620 | european_cup.cpp | FUN_0056e620 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0056ec30 | european_cup.cpp | FUN_0056ec30 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0056ef20 | european_cup.cpp | FUN_0056ef20 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0056f740 | european_cup.cpp | FUN_0056f740 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0056ffc0 | european_cup.cpp | FUN_0056ffc0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00570590 | european_cup.cpp | FUN_00570590 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00570eb0 | european_cup.cpp | FUN_00570eb0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00571000 | european_cup.cpp | FUN_00571000 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00571290 | european_cup.cpp | FUN_00571290 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00571490 | european_cup.cpp | FUN_00571490 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005718f0 | european_cup.cpp | FUN_005718f0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00571b20 | european_cup.cpp | FUN_00571b20 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00571f60 | european_cup.cpp | FUN_00571f60 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00572080 | european_cup.cpp | FUN_00572080 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00572250 | european_cup.cpp | FUN_00572250 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00572360 | european_cup.cpp | FUN_00572360 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00572530 | european_cup.cpp | FUN_00572530 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00572670 | european_cup.cpp | FUN_00572670 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00572960 | european_cup.cpp | FUN_00572960 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00572b90 | european_cup.cpp | FUN_00572b90 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00572f90 | european_cup.cpp | FUN_00572f90 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00573110 | european_cup.cpp | FUN_00573110 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00573360 | european_cup.cpp | FUN_00573360 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00573510 | european_cup.cpp | FUN_00573510 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00573850 | european_cup.cpp | FUN_00573850 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005739f0 | european_cup.cpp | FUN_005739f0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00573d00 | european_cup.cpp | FUN_00573d00 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00573ea0 | european_cup.cpp | FUN_00573ea0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00574160 | european_cup.cpp | FUN_00574160 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005742f0 | european_cup.cpp | FUN_005742f0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00574520 | european_cup.cpp | FUN_00574520 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00574590 | european_cup.cpp | FUN_00574590 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00574610 | european_cup.cpp | FUN_00574610 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005746f0 | european_cup.cpp | FUN_005746f0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005749c0 | european_cup.cpp | FUN_005749c0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x00596590 | fix_man.cpp | participant maintenance branch | UNKNOWN | crates/cm-app/src/main.rs;crates/cm-domain/src/lib.rs | NO | UNVERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x005ca350 | goldcup.cpp | FUN_005ca350 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005ca510 | goldcup.cpp | FUN_005ca510 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005ca530 | goldcup.cpp | FUN_005ca530 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005caed0 | goldcup.cpp | FUN_005caed0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005cafc0 | goldcup.cpp | FUN_005cafc0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005cb3a0 | goldcup.cpp | FUN_005cb3a0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005cb590 | goldcup.cpp | FUN_005cb590 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005cb9c0 | goldcup.cpp | FUN_005cb9c0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005cbe10 | goldcup.cpp | FUN_005cbe10 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005ccb40 | goldcup.cpp | FUN_005ccb40 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005ccb90 | goldcup.cpp | FUN_005ccb90 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005cddc0 | goldcup.cpp | FUN_005cddc0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005ce580 | goldcup.cpp | FUN_005ce580 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005ce750 | goldcup.cpp | FUN_005ce750 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005d1090 | goldcup.cpp | FUN_005d1090 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005d1440 | goldcup.cpp | FUN_005d1440 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005d18a0 | goldcup.cpp | FUN_005d18a0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005d1b00 | goldcup.cpp | FUN_005d1b00 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005d1c20 | goldcup.cpp | FUN_005d1c20 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005e4250 | host_country.cpp | host lookup by (sub year) | PORTED_PARTIAL | HostCountry::lookup | INDIRECT | PARTIAL |  |
| 0x005e4e40 | host_country.cpp | HostCountry.tmp writer | PORTED_BEHAVIOURAL | HostCountry::save_tmp | YES | STRUCTURALLY_VERIFIED |  |
| 0x00667e00 | league.cpp | FUN_00667e00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006680e0 | league.cpp | FUN_006680e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00668240 | league.cpp | FUN_00668240 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006682e0 | league.cpp | FUN_006682e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006687c0 | league.cpp | FUN_006687c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006691b0 | league.cpp | FUN_006691b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00669500 | league.cpp | FUN_00669500 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00669910 | league.cpp | FUN_00669910 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00669a90 | league.cpp | FUN_00669a90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0066a770 | league.cpp | FUN_0066a770 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0066b000 | league.cpp | FUN_0066b000 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0066b230 | league.cpp | FUN_0066b230 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0066b940 | league.cpp | FUN_0066b940 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0066bc10 | league.cpp | FUN_0066bc10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0066bc70 | league.cpp | FUN_0066bc70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0066c700 | league.cpp | FUN_0066c700 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0066c9e0 | league.cpp | FUN_0066c9e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0066cc40 | league.cpp | FUN_0066cc40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0066cde0 | league.cpp | FUN_0066cde0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0066ed20 | league.cpp | FUN_0066ed20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0066f630 | league.cpp | FUN_0066f630 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0066f720 | league.cpp | FUN_0066f720 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0066faa0 | league.cpp | FUN_0066faa0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0066fbd0 | league.cpp | FUN_0066fbd0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0066fcd0 | league.cpp | FUN_0066fcd0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0066fd20 | league.cpp | FUN_0066fd20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0066fdc0 | league.cpp | FUN_0066fdc0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00670240 | league.cpp | FUN_00670240 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00670350 | league_stage.cpp | FUN_00670350 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00670720 | league_stage.cpp | FUN_00670720 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00670740 | league_stage.cpp | FUN_00670740 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x006708f0 | league_stage.cpp | FUN_006708f0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00670970 | league_stage.cpp | FUN_00670970 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00671c90 | league_stage.cpp | FUN_00671c90 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00672210 | league_stage.cpp | FUN_00672210 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00672290 | league_stage.cpp | FUN_00672290 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x006722d0 | league_stage.cpp | FUN_006722d0 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00672440 | league_stage.cpp | FUN_00672440 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00672530 | league_stage.cpp | FUN_00672530 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00672540 | league_stage.cpp | FUN_00672540 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0075ee00 | national_teams.cpp | participant-notification helper | UNKNOWN | crates/cm-app/src/main.rs;crates/cm-domain/src/lib.rs | NO | UNVERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x0075f0f0 | national_teams.cpp | notification cleanup helper (day%0x46) | UNKNOWN | crates/cm-domain/src/lib.rs | NO | UNVERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x008f15d0 | uefa_cup.cpp | FUN_008f15d0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f17c0 | uefa_cup.cpp | FUN_008f17c0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f17e0 | uefa_cup.cpp | FUN_008f17e0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f18b0 | uefa_cup.cpp | FUN_008f18b0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f1920 | uefa_cup.cpp | FUN_008f1920 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f1ef0 | uefa_cup.cpp | FUN_008f1ef0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f2410 | uefa_cup.cpp | FUN_008f2410 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f26c0 | uefa_cup.cpp | FUN_008f26c0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f28d0 | uefa_cup.cpp | FUN_008f28d0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f28f0 | uefa_cup.cpp | FUN_008f28f0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f2b10 | uefa_seeding.cpp | FUN_008f2b10 | FOREIGN_BREADTH |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008f2bf0 | uefa_seeding.cpp | FUN_008f2bf0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f2eb0 | uefa_seeding.cpp | FUN_008f2eb0 | FOREIGN_BREADTH |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008f3020 | uefa_seeding.cpp | FUN_008f3020 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f33a0 | uefa_seeding.cpp | FUN_008f33a0 | FOREIGN_BREADTH |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008f3570 | uefa_seeding.cpp | FUN_008f3570 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f38e0 | uefa_seeding.cpp | FUN_008f38e0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f4190 | uefa_seeding.cpp | FUN_008f4190 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f4430 | uefa_seeding.cpp | FUN_008f4430 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f4590 | uefa_seeding.cpp | FUN_008f4590 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f4870 | uefa_seeding.cpp | FUN_008f4870 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f4af0 | uefa_seeding.cpp | FUN_008f4af0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f4c10 | uefa_seeding.cpp | FUN_008f4c10 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f4c40 | uefa_seeding.cpp | FUN_008f4c40 | FOREIGN_BREADTH |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008f4d40 | uefa_seeding.cpp | FUN_008f4d40 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f4fc0 | uefa_seeding.cpp | FUN_008f4fc0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f50a0 | uefa_seeding.cpp | FUN_008f50a0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f5260 | uefa_seeding.cpp | FUN_008f5260 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f5490 | uefa_seeding.cpp | FUN_008f5490 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x008f5770 | uefa_seeding.cpp | FUN_008f5770 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |

## condition-fitness

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x005f8970 | index.cpp | FUN_005f8970 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005fae70 | index.cpp | FUN_005fae70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005fde10 | index.cpp | FUN_005fde10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00600ee0 | index.cpp | FUN_00600ee0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00601930 | index.cpp | FUN_00601930 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00613de0 | index.cpp | FUN_00613de0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00614000 | index.cpp | FUN_00614000 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00614080 | index.cpp | FUN_00614080 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x006141e0 | index.cpp | FUN_006141e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006149b0 | index.cpp | FUN_006149b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006153f0 | index.cpp | FUN_006153f0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x006154e0 | index.cpp | FUN_006154e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00615640 | index.cpp | FUN_00615640 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00615680 | index.cpp | FUN_00615680 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00615760 | index.cpp | FUN_00615760 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006159d0 | index.cpp | FUN_006159d0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00615ab0 | index.cpp | FUN_00615ab0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00615ae0 | index.cpp | FUN_00615ae0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00615bf0 | index.cpp | FUN_00615bf0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00615d40 | index.cpp | FUN_00615d40 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00615d60 | index.cpp | FUN_00615d60 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00616510 | index.cpp | FUN_00616510 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00616650 | index.cpp | FUN_00616650 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00617380 | index.cpp | FUN_00617380 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00617be0 | index.cpp | FUN_00617be0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00617de0 | index.cpp | FUN_00617de0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00617f00 | index.cpp | FUN_00617f00 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x006182e0 | index.cpp | FUN_006182e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00618300 | index.cpp | FUN_00618300 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006183c0 | index.cpp | FUN_006183c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00618510 | index.cpp | FUN_00618510 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00619dc0 | index.cpp | FUN_00619dc0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0061a1d0 | index.cpp | FUN_0061a1d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0061a330 | index.cpp | FUN_0061a330 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0061a5c0 | index.cpp | FUN_0061a5c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0061ac30 | index.cpp | FUN_0061ac30 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0061ac60 | index.cpp | FUN_0061ac60 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0061b4a0 | index.cpp | FUN_0061b4a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0061b670 | index.cpp | FUN_0061b670 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0078ac30 | physio.cpp | FUN_0078ac30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0078ade0 | physio.cpp | FUN_0078ade0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0078b310 | physio.cpp | FUN_0078b310 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0078b4c0 | physio.cpp | FUN_0078b4c0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0078b540 | physio.cpp | FUN_0078b540 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0078b6e0 | physio.cpp | FUN_0078b6e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0078be10 | physio.cpp | FUN_0078be10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0078d430 | physio.cpp | FUN_0078d430 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0078d490 | physio.cpp | FUN_0078d490 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |

## config

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x005c1540 | game_config.cpp | FUN_005c1540 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x005c1a10 | game_config.cpp | FUN_005c1a10 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x005c1a60 | game_config.cpp | FUN_005c1a60 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x005c1b00 | game_config.cpp | FUN_005c1b00 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x005c1ec0 | game_config.cpp | FUN_005c1ec0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x005c2030 | game_config.cpp | FUN_005c2030 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x005c2860 | game_config.cpp | FUN_005c2860 | NON_USEFUL |  | NO | UNVERIFIED |  |

## containers

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00672320 | league_stage.cpp | queue enqueue helper | UNKNOWN | crates/cm-app/src/main.rs;crates/cm-domain/src/lib.rs;crates/cm-domain/src/match_engine_exe.rs | NO | UNVERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x006724d0 | league_stage.cpp | queue helper companion | UNKNOWN | crates/cm-domain/src/lib.rs | NO | UNVERIFIED | rust:crates/cm-domain/src/lib.rs |

## contracts

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x004cd930 | contract_manager.cpp | CONTRACT_MANAGER::initialise_all | PORTED_BEHAVIOURAL | initialise_all | YES | STRUCTURALLY_VERIFIED | memory:contract-clauses-generated-at-boot.md |
| 0x004cdd80 | contract_manager.cpp | FUN_004cdd80 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004cddd0 | contract_manager.cpp | FUN_004cddd0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004cdef0 | contract_manager.cpp | FUN_004cdef0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d0390 | contract_manager.cpp | FUN_004d0390 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d05f0 | contract_manager.cpp | FUN_004d05f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d06d0 | contract_manager.cpp | FUN_004d06d0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004d0730 | contract_manager.cpp | FUN_004d0730 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004d1d90 | contract_manager.cpp | FUN_004d1d90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d1dc0 | contract_manager.cpp | FUN_004d1dc0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004d1f50 | contract_manager.cpp | FUN_004d1f50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d1f90 | contract_manager.cpp | FUN_004d1f90 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004d2150 | contract_manager.cpp | FUN_004d2150 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d24e0 | contract_manager.cpp | FUN_004d24e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d2520 | contract_manager.cpp | FUN_004d2520 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d2560 | contract_manager.cpp | FUN_004d2560 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004d2700 | contract_manager.cpp | FUN_004d2700 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d27d0 | contract_manager.cpp | FUN_004d27d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d2c40 | contract_manager.cpp | FUN_004d2c40 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004d2d60 | contract_manager.cpp | FUN_004d2d60 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d2e60 | contract_manager.cpp | FUN_004d2e60 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004d2ea0 | contract_manager.cpp | FUN_004d2ea0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004d2f60 | contract_manager.cpp | FUN_004d2f60 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004d3020 | contract_manager.cpp | FUN_004d3020 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004d30d0 | contract_manager.cpp | FUN_004d30d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d3660 | contract_manager.cpp | FUN_004d3660 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d3b00 | contract_manager.cpp | FUN_004d3b00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d3df0 | contract_manager.cpp | FUN_004d3df0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d4910 | contract_manager.cpp | FUN_004d4910 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d58d0 | contract_manager.cpp | FUN_004d58d0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004d5990 | contract_manager.cpp | FUN_004d5990 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d5b50 | contract_manager.cpp | FUN_004d5b50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d6610 | contract_manager.cpp | FUN_004d6610 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d6c10 | contract_manager.cpp | FUN_004d6c10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d6f80 | contract_manager.cpp | FUN_004d6f80 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004d7000 | contract_manager.cpp | FUN_004d7000 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004d7090 | contract_manager.cpp | exact staff wage formula | NOT_YET_PORTED | compute_wage | NO | UNVERIFIED | memory:contract-clauses-generated-at-boot.md |
| 0x004da500 | contract_manager.cpp | FUN_004da500 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004da710 | contract_manager.cpp | FUN_004da710 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004da820 | contract_manager.cpp | FUN_004da820 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004da890 | contract_manager.cpp | FUN_004da890 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004daa60 | contract_manager.cpp | FUN_004daa60 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004db1e0 | contract_manager.cpp | FUN_004db1e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004dc6d0 | contract_manager.cpp | FUN_004dc6d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004dc7b0 | contract_manager.cpp | FUN_004dc7b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004dcbf0 | contract_manager.cpp | FUN_004dcbf0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004dd0d0 | contract_manager.cpp | FUN_004dd0d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004dd3a0 | contract_manager.cpp | FUN_004dd3a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004dd960 | contract_manager.cpp | FUN_004dd960 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004de7f0 | contract_manager.cpp | FUN_004de7f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004deb70 | contract_manager.cpp | FUN_004deb70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004df5a0 | contract_manager.cpp | FUN_004df5a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004df720 | contract_manager.cpp | FUN_004df720 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004df980 | contract_manager.cpp | FUN_004df980 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004e00c0 | contract_manager.cpp | FUN_004e00c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004e02d0 | contract_manager.cpp | FUN_004e02d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004e06d0 | contract_manager.cpp | FUN_004e06d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004e0b40 | contract_manager.cpp | FUN_004e0b40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004e0f20 | contract_manager.cpp | FUN_004e0f20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004e1000 | contract_manager.cpp | FUN_004e1000 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004e11f0 | contract_manager.cpp | FUN_004e11f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004e1420 | contract_manager.cpp | FUN_004e1420 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004e1700 | contract_manager.cpp | FUN_004e1700 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004e24e0 | contract_manager.cpp | FUN_004e24e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004e2a50 | contract_manager.cpp | FUN_004e2a50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008476b0 | staff_contracts.cpp | FUN_008476b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00847870 | staff_contracts.cpp | FUN_00847870 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00847a10 | staff_contracts.cpp | FUN_00847a10 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00847a80 | staff_contracts.cpp | per-person release-clause roll | PORTED_PARTIAL | roll_clauses | YES | BEHAVIOURALLY_EXACT | memory:contract-clauses-generated-at-boot.md |
| 0x008488f0 | staff_contracts.cpp | FUN_008488f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00848920 | staff_contracts.cpp | FUN_00848920 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00848930 | staff_contracts.cpp | FUN_00848930 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00848940 | staff_contracts.cpp | FUN_00848940 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00848950 | staff_contracts.cpp | FUN_00848950 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00848a90 | staff_contracts.cpp | FUN_00848a90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00848b40 | staff_contracts.cpp | FUN_00848b40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00848b50 | staff_contracts.cpp | FUN_00848b50 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00848b70 | staff_contracts.cpp | FUN_00848b70 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00848b80 | staff_contracts.cpp | FUN_00848b80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0084b830 | staff_contracts.cpp | FUN_0084b830 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0084f5d0 | staff_contracts.cpp | FUN_0084f5d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0084f7e0 | staff_contracts.cpp | FUN_0084f7e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0084fe90 | staff_contracts.cpp | FUN_0084fe90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0084ff10 | staff_contracts.cpp | FUN_0084ff10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008501f0 | staff_contracts.cpp | FUN_008501f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00850490 | staff_contracts.cpp | FUN_00850490 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008504d0 | staff_contracts.cpp | FUN_008504d0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00850510 | staff_contracts.cpp | FUN_00850510 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00850680 | staff_contracts.cpp | FUN_00850680 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008506a0 | staff_contracts.cpp | FUN_008506a0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008506b0 | staff_contracts.cpp | FUN_008506b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00850950 | staff_contracts.cpp | FUN_00850950 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008509b0 | staff_contracts.cpp | FUN_008509b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00850a50 | staff_contracts.cpp | FUN_00850a50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00850a80 | staff_contracts.cpp | FUN_00850a80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00850b00 | staff_contracts.cpp | FUN_00850b00 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00850b20 | staff_contracts.cpp | FUN_00850b20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00850c00 | staff_contracts.cpp | FUN_00850c00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00850ce0 | staff_contracts.cpp | FUN_00850ce0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00850ea0 | staff_contracts.cpp | FUN_00850ea0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008511c0 | staff_contracts.cpp | FUN_008511c0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |

## crt

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00921cc0 | zipdir.cpp | container store helper | NON_USEFUL | crates/cm-domain/src/lib.rs;crates/cm-domain/src/typed_records.rs | NO | UNVERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x009343c3 | zipdir.cpp | CRT memcpy/plumbing thunk | NON_USEFUL | crates/cm-app/src/main.rs;crates/cm-domain/src/history.rs | NO | UNVERIFIED | rust:crates/cm-app/src/main.rs |
| 0x00935080 | zipdir.cpp | CRT float/plumbing thunk | NON_USEFUL | crates/cm-app/src/main.rs;crates/cm-domain/src/player_regen.rs | NO | UNVERIFIED | rust:crates/cm-app/src/main.rs |
| 0x009350a2 | zipdir.cpp | CRT x87 double helper | NON_USEFUL | crates/cm-app/src/main.rs | NO | UNVERIFIED | rust:crates/cm-app/src/main.rs |
| 0x0095a080 | zipdir.cpp | ire_super_cup vtable pointer | NON_USEFUL |  | NO | UNVERIFIED | rust:crates/cm-domain/src/lib.rs |

## cups

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x0041de60 | bel_fa_cup.cpp | Belgian FA Cup round-date helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x00502470 | cup.cpp | cup fixture-tree mapper | PORTED_BEHAVIOURAL | CupState::progress | YES | BEHAVIOURALLY_EXACT | memory/cup-engine-progressive.md |
| 0x00503e30 | cup.cpp | cup fixture processor (leg/replay) | PORTED_BEHAVIOURAL | CupState::tie_outcome;draw_round | YES | BEHAVIOURALLY_EXACT | memory/cup-engine-progressive.md |
| 0x0050ca60 | cup_stage.cpp | FUN_0050ca60 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0050cd20 | cup_stage.cpp | FUN_0050cd20 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0050cd40 | cup_stage.cpp | FUN_0050cd40 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0050cf60 | cup_stage.cpp | FUN_0050cf60 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0050cfe0 | cup_stage.cpp | FUN_0050cfe0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0050db10 | cup_stage.cpp | FUN_0050db10 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0050de40 | cup_stage.cpp | FUN_0050de40 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0050e970 | cup_stage.cpp | FUN_0050e970 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0050e990 | cup_stage.cpp | FUN_0050e990 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x005549b0 | eng_auto_cup.cpp | English Vans/AWS Trophy main-draw date helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x00556150 | eng_cc_cup.cpp | English League Cup round-date helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x00558f60 | eng_fa_cup.cpp | English FA Cup round-date helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x0055abb0 | eng_fa_trophy.cpp | English FA Trophy round-date helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x0057c730 | fin_cup.cpp | Finnish Cup round-date helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x005a4650 | fra_cup.cpp | French Cup round-date helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x005a6e80 | fra_lge_cup.cpp | French League Cup round-date helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x005c2b90 | ger_cup.cpp | DFB-Pokal round-date helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x005c6050 | ger_lge_cup.cpp | German League Cup round-date helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x005d2ff0 | gre_cup.cpp | Greek Cup sub-comp round helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | INDIRECT | PARTIAL | rust:crates/cm-domain/src/lib.rs |
| 0x005d5be0 | gre_super.cpp | Greek Cup main round-date helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x005dd170 | hol_cup.cpp | Dutch Cup round-date helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x0061ba20 | inter_amer_cup.cpp | CONCACAF single-round setup helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | INDIRECT | PARTIAL | rust:crates/cm-domain/src/lib.rs |
| 0x0061bb60 | inter_amer_cup.cpp | CONCACAF club filter helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | INDIRECT | PARTIAL | rust:crates/cm-domain/src/lib.rs |
| 0x0061cd40 | intertoto_cup.cpp | Intertoto Cup participant/round helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | INDIRECT | PARTIAL | rust:crates/cm-domain/src/lib.rs |
| 0x0061d9f0 | ire_chal_cup.cpp | Ireland Challenge Cup round-date helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x0061fdc0 | ire_leinster_cup.cpp | Ireland Leinster Cup round-date helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x006220c0 | ire_munster_cup.cpp | Ireland League Cup KO round-date helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x00622bb0 | ire_pres_cup.cpp | Ireland President's Cup round-date helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x006245f0 | ire_super_cup.cpp | Italian League Cup KO round-date helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x00627060 | ita_c_cup.cpp | Italian League Cup group-phase helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | INDIRECT | PARTIAL | rust:crates/cm-domain/src/lib.rs |
| 0x006282d0 | ita_cup.cpp | Italian Cup round-date helper | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |

## date

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00533b50 | date.cpp | pack_date (pre-snap) | PORTED_EXACT | pack_date | YES | BYTE_EXACT | memory/fixtures-runtime-capture-findings.md |
| 0x00533d10 | date.cpp | pack-date-with-snap entry | PORTED_BEHAVIOURAL | pack_date;apply_flag_snap | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/exe_date.rs |
| 0x00533eb0 | date.cpp | weekday flag-snap | PORTED_EXACT | apply_flag_snap | YES | BYTE_EXACT | memory/fixtures-runtime-capture-findings.md |
| 0x00536190 | date.cpp | date-window builder | PORTED_BEHAVIOURAL | CmPackedDate::add_days | INDIRECT | PARTIAL | rust:crates/cm-domain/src/lib.rs |
| 0x00536350 | date.cpp | add N weeks | PORTED_BEHAVIOURAL | CmPackedDate::add_days | INDIRECT | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x005364c0 | date.cpp | add/subtract N days | PORTED_BEHAVIOURAL | CmPackedDate::add_days;CmPackedDate::add_negative_days | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x00536690 | date.cpp | subtract N weeks | PORTED_BEHAVIOURAL | CmPackedDate::add_negative_days | INDIRECT | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x00536b50 | date.cpp | doy -> day-of-month unpack | PORTED_BEHAVIOURAL | CmPackedDate::to_game_date | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x00536bc0 | date.cpp | is-leap-year predicate | PORTED_BEHAVIOURAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x0066f3b0 | league.cpp | schedule round-record writer | PORTED_EXACT | write_round_record | YES | BYTE_EXACT | memory/c11-2-fixture-subsystem-frozen.md |
| 0x0066f410 | league.cpp | schedule fixture sub-slot writer | PORTED_PARTIAL | write_slot | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/exe_date.rs |

## db-load

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x005121a0 | database.cpp | start_new_game_section_loader | REPLACED_BY_RUST | World::read_rust_db_dir | YES | STRUCTURALLY_VERIFIED | memory:start-new-game-flow.md |
| 0x0051b110 | database.cpp | pool_link_id_to_pointer_swizzle | REPLACED_BY_RUST | id_opt | YES | STRUCTURALLY_VERIFIED | rust:typed_records.rs |
| 0x00537580 | db_files.cpp | FUN_00537580 | REPLACED_BY_RUST |  | INDIRECT | STRUCTURALLY_VERIFIED |  |
| 0x005376d0 | db_files.cpp | FUN_005376d0 | REPLACED_BY_RUST |  | INDIRECT | STRUCTURALLY_VERIFIED |  |
| 0x00538580 | db_files.cpp | FUN_00538580 | REPLACED_BY_RUST |  | INDIRECT | UNVERIFIED |  |
| 0x00538bb0 | db_files.cpp | FUN_00538bb0 | REPLACED_BY_RUST |  | INDIRECT | STRUCTURALLY_VERIFIED |  |
| 0x00538d30 | db_files.cpp | FUN_00538d30 | REPLACED_BY_RUST |  | INDIRECT | UNVERIFIED |  |
| 0x00539790 | db_files.cpp | FUN_00539790 | REPLACED_BY_RUST |  | INDIRECT | STRUCTURALLY_VERIFIED |  |
| 0x005398e0 | db_files.cpp | FUN_005398e0 | REPLACED_BY_RUST |  | INDIRECT | STRUCTURALLY_VERIFIED |  |
| 0x00539db0 | db_files.cpp | FUN_00539db0 | REPLACED_BY_RUST |  | INDIRECT | STRUCTURALLY_VERIFIED |  |
| 0x0053a050 | db_files.cpp | FUN_0053a050 | REPLACED_BY_RUST |  | INDIRECT | STRUCTURALLY_VERIFIED |  |
| 0x0053a2f0 | db_files.cpp | FUN_0053a2f0 | REPLACED_BY_RUST |  | INDIRECT | STRUCTURALLY_VERIFIED |  |
| 0x0053a530 | db_files.cpp | FUN_0053a530 | REPLACED_BY_RUST |  | INDIRECT | STRUCTURALLY_VERIFIED |  |
| 0x00842f40 | squad_manager.cpp | squad-number assignment pass | PORTED_BEHAVIOURAL | World::assign_squad_numbers | YES | STRUCTURALLY_VERIFIED | memory:start-new-game-flow.md |
| 0x00843970 | squad_manager.cpp | squad_position_code_write | PORTED_BEHAVIOURAL | write_squad_position | YES | STRUCTURALLY_VERIFIED | rust:c15_1_world_apply.rs |

## discipline

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x004194e0 | awol.cpp | FUN_004194e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00419ac0 | awol.cpp | FUN_00419ac0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00419b50 | awol.cpp | FUN_00419b50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00419bd0 | awol.cpp | FUN_00419bd0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0041a200 | awol.cpp | FUN_0041a200 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0041a270 | awol.cpp | FUN_0041a270 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0041a320 | awol.cpp | FUN_0041a320 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0041a3a0 | awol.cpp | FUN_0041a3a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0041a630 | awol.cpp | FUN_0041a630 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0041a7c0 | awol.cpp | FUN_0041a7c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0041aec0 | awol.cpp | FUN_0041aec0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0041b190 | awol.cpp | FUN_0041b190 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0041b440 | awol.cpp | FUN_0041b440 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0041b930 | awol.cpp | FUN_0041b930 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0041bb90 | awol.cpp | FUN_0041bb90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0058f930 | fine.cpp | FUN_0058f930 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0058fe90 | fine.cpp | FUN_0058fe90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0058fec0 | fine.cpp | FUN_0058fec0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0058fee0 | fine.cpp | FUN_0058fee0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00590f10 | fine.cpp | FUN_00590f10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00591900 | fine.cpp | FUN_00591900 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00591a10 | fine.cpp | FUN_00591a10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00591c40 | fine.cpp | FUN_00591c40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00592430 | fine.cpp | FUN_00592430 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00592480 | fine.cpp | FUN_00592480 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005925f0 | fine.cpp | FUN_005925f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00592940 | fine.cpp | FUN_00592940 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006cf040 | match_man.cpp | card applier verdict (red never live) | PORTED_EXACT | card_verdict;disc_map | INDIRECT | BEHAVIOURALLY_EXACT | memory:history-panels-noheuristic-ports.md |
| 0x006cf230 | match_man.cpp | foul severity accumulation (algo exact; live red blocked) | PORTED_EXACT | foul_severity | INDIRECT | BEHAVIOURALLY_EXACT | memory:history-panels-noheuristic-ports.md |

## editor

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00414d5c | award_manager.cpp | editor Set-Position popup / load code | UNKNOWN | crates/cm-data/src/lib.rs;crates/cm-domain/src/lib.rs;crates/cm-ui-app/src/render_new.rs | NO | UNVERIFIED | memory:editor-is-ground-truth.md |
| 0x0044773c | club_records.cpp | editor record load code (ground truth) | UNKNOWN | crates/cm-data/src/lib.rs;crates/cm-domain/src/lib.rs | NO | UNVERIFIED | memory:editor-is-ground-truth.md |
| 0x0048bd00 | club_screens.cpp | editor attribute-display code | UNKNOWN | crates/cm-domain/src/lib.rs;crates/cm-ui-app/src/render_new.rs | NO | UNVERIFIED | memory:editor-is-ground-truth.md |

## fifa-rankings

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00577210 | fifa_rankings.cpp | comp_fifa_monthly_maintenance | NOT_YET_PORTED | NationRanking::history | NO | PARTIAL | memory:fifa-rankings-mechanism.md |
| 0x005773c0 | fifa_rankings.cpp | comp_fifa_rank_array_build_sort | PORTED_BEHAVIOURAL | fifa_rankings::compute | YES | STATE_EXACT | rust:fifa_rankings.rs |
| 0x00577b00 | fifa_rankings.cpp | comp_fifa_build_runtime_table | PORTED_EXACT | fifa_rankings::compute | YES | BYTE_EXACT | memory:fifa-rankings-mechanism.md |
| 0x00577d70 | fifa_rankings.cpp | comp_fifa_recompute_points | NOT_YET_PORTED | crates/cm-domain/src/fifa_rankings.rs | NO | PARTIAL | memory:fifa-rankings-mechanism.md |
| 0x00577ff0 | fifa_rankings.cpp | comp_fifa_sort_comparator | PORTED_EXACT | fifa_rankings::compute | YES | BEHAVIOURALLY_EXACT | rust:fifa_rankings.rs |
| 0x005c01d0 | game.cpp | game_recompute_fifa_rankings | PORTED_EXACT | fifa_rankings::fifa_score | TEST_ONLY | BYTE_EXACT | memory:fifa-rankings-mechanism.md |

## finance

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x005803d0 | finance.cpp | initial club-finance seed | PORTED_EXACT | ClubFinance::seed_from;START_CASH | YES | STATE_EXACT | memory:finance-mechanisms-decoded.md |
| 0x00581f70 | finance.cpp | FUN_00581f70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00582500 | finance.cpp | FUN_00582500 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00582530 | finance.cpp | FUN_00582530 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00582870 | finance.cpp | finance-status classifier | PORTED_EXACT | ClubFinance::status;FinanceStatus::to_signed_byte | YES | STATE_EXACT | rust:finance.rs |
| 0x00582980 | finance.cpp | FUN_00582980 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005839c0 | finance.cpp | FUN_005839c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00583a10 | finance.cpp | FUN_00583a10 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00583a50 | finance.cpp | FUN_00583a50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00583aa0 | finance.cpp | FUN_00583aa0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00583ae0 | finance.cpp | FUN_00583ae0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00583b20 | finance.cpp | FUN_00583b20 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00583b60 | finance.cpp | FUN_00583b60 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00583be0 | finance.cpp | FUN_00583be0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00584760 | finance.cpp | FUN_00584760 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00584790 | finance.cpp | post-match gate receipts + TV/prize | PORTED_PARTIAL | FinanceBook::record_match_income | YES | PARTIAL | rust:finance.rs |
| 0x005853c0 | finance.cpp | FUN_005853c0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005855e0 | finance.cpp | FUN_005855e0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00585750 | finance.cpp | FUN_00585750 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00585900 | finance.cpp | FUN_00585900 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00586cf0 | finance.cpp | end-of-month rollover | PORTED_BEHAVIOURAL | FinanceBook::end_of_month | YES | BEHAVIOURALLY_EXACT | rust:finance.rs |
| 0x00586e70 | finance.cpp | FUN_00586e70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00586ec0 | finance.cpp | weekly finance tick (stadium-share GBP20M + wage cascade) | PORTED_BEHAVIOURAL | FinanceBook::stadium_share_transfers;FinanceBook::pay_weekly_wages | YES | STATE_EXACT | memory:finance-mechanisms-decoded.md |
| 0x00587c40 | finance.cpp | board debt payment (news case 4) | PORTED_EXACT | FinanceBook::board_debt_payment | YES | STATE_EXACT | memory:finance-mechanisms-decoded.md |
| 0x00587f50 | finance.cpp | chairman cash injection | PORTED_EXACT | chairman_cash_inject_cap | YES | STATE_EXACT | rust:finance.rs |
| 0x005884a0 | finance.cpp | new-board takeover (news case 3) | PORTED_BEHAVIOURAL | FinanceBook::takeover_check | YES | STATE_EXACT | memory:finance-mechanisms-decoded.md |
| 0x00588840 | finance.cpp | silent-takeover trigger + chairman reroll | PORTED_PARTIAL | chairman_takeover_fires;reroll_chairman_stats | YES | STATE_EXACT | memory:finance-mechanisms-decoded.md |
| 0x00588c70 | finance.cpp | monthly board dispatch | PORTED_BEHAVIOURAL | FinanceBook::tick_month_board | YES | STATE_EXACT | rust:finance.rs |
| 0x00589200 | finance.cpp | FUN_00589200 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005892e0 | finance.cpp | FUN_005892e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00589800 | finance.cpp | FUN_00589800 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005898c0 | finance.cpp | FUN_005898c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005899e0 | finance.cpp | FUN_005899e0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00589a90 | finance.cpp | FUN_00589a90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00589b60 | finance.cpp | FUN_00589b60 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00589bf0 | finance.cpp | FUN_00589bf0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00589d10 | finance.cpp | FUN_00589d10 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00589db0 | finance.cpp | FUN_00589db0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00589eb0 | finance.cpp | FUN_00589eb0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00589f50 | finance.cpp | FUN_00589f50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0058a130 | finance.cpp | FUN_0058a130 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0058a1f0 | finance.cpp | FUN_0058a1f0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0058a280 | finance.cpp | FUN_0058a280 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0058a3c0 | finance.cpp | FUN_0058a3c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00618410 | index.cpp | club-status byte lookup | PORTED_EXACT | club_status_byte | TEST_ONLY | STATE_EXACT | rust:finance.rs |
| 0x0067fdf0 | manager_manager.cpp | chairman manager-sack decision | PORTED_EXACT | chairman_will_sack | YES | STATE_EXACT | rust:finance.rs |
| 0x0084b870 | staff_contracts.cpp | staff-contract wage floor/cap | PORTED_EXACT | STAFF_BASE_FLOOR;age_wage_cap;star_floor | TEST_ONLY | STATE_EXACT | rust:finance.rs |

## fixtures

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x004b6000 | comp_util.cpp | sort_and_shuffle | PORTED_EXACT | sort_and_shuffle | YES | BEHAVIOURALLY_EXACT | memory:english-pyramid-final-graph.md |
| 0x00533ad0 | database.cpp | undecoded schedule function | UNKNOWN | crates/cm-domain/src/african_nations.rs;crates/cm-domain/src/lib.rs;crates/cm-domain/src/screen_batch11.rs | NO | UNVERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x00557970 | eng_conf.cpp | eng_conf_schedule | PORTED_EXACT | ENGLISH_CONFERENCE_RUNTIME | YES | STATE_EXACT | memory:c11-2-fixture-subsystem-frozen.md |
| 0x0055b540 | eng_first.cpp | eng_first_schedule | PORTED_EXACT | ENGLISH_FIRST_RUNTIME | YES | STATE_EXACT | memory:c11-2-fixture-subsystem-frozen.md |
| 0x0055d120 | eng_prm.cpp | eng_prem_schedule | PORTED_EXACT | ENGLISH_PREMIER_RUNTIME | YES | STATE_EXACT | memory:c11-2-fixture-subsystem-frozen.md |
| 0x0055f040 | eng_second.cpp | eng_second_ctor | PORTED_BEHAVIOURAL | generate_english_traditional_league | YES | STATE_EXACT | memory:eng-second-ctor-identity-confirmed.md |
| 0x0055f340 | eng_second.cpp | eng_second_schedule_getter | PORTED_EXACT | build_eng_second_schedule | YES | BYTE_EXACT | memory:fixtures-runtime-capture-findings.md |
| 0x00560d40 | eng_third.cpp | eng_third_schedule | PORTED_EXACT | ENGLISH_THIRD_RUNTIME | YES | STATE_EXACT | memory:c11-2-fixture-subsystem-frozen.md |
| 0x00594370 | fix_man.cpp | FUN_00594370 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00594750 | fix_man.cpp | FUN_00594750 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005952f0 | fix_man.cpp | FUN_005952f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005958b0 | fix_man.cpp | FUN_005958b0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00595a40 | fix_man.cpp | FUN_00595a40 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00595b90 | fix_man.cpp | FUN_00595b90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00596190 | fix_man.cpp | FUN_00596190 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00596410 | fix_man.cpp | FUN_00596410 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005966e0 | fix_man.cpp | FUN_005966e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00596a30 | fix_man.cpp | FUN_00596a30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00596e20 | fix_man.cpp | FUN_00596e20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00597560 | fix_man.cpp | FUN_00597560 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00597e00 | fix_man.cpp | FUN_00597e00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00598220 | fix_man.cpp | FUN_00598220 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00598a00 | fix_man.cpp | FUN_00598a00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00598d50 | fix_man.cpp | FUN_00598d50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00598ef0 | fix_man.cpp | FUN_00598ef0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00599050 | fix_man.cpp | FUN_00599050 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005994c0 | fix_man.cpp | FUN_005994c0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005994e0 | fix_man.cpp | FUN_005994e0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00599510 | fix_man.cpp | FUN_00599510 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005996c0 | fix_man.cpp | FUN_005996c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00599800 | fix_man.cpp | FUN_00599800 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00599970 | fix_man.cpp | FUN_00599970 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00599a10 | fix_man.cpp | FUN_00599a10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00599b20 | fix_man.cpp | FUN_00599b20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00599c30 | fix_man.cpp | FUN_00599c30 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00599cb0 | fix_man.cpp | FUN_00599cb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00599d20 | fix_man.cpp | FUN_00599d20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00667aa0 | league.cpp | league final-table sort | PORTED_BEHAVIOURAL | season_rows_for_club | INDIRECT | PARTIAL | reports/club_history_screen_decode.md |
| 0x006684e0 | league.cpp | league table comparator | PORTED_PARTIAL | season_rows_for_club | INDIRECT | PARTIAL | reports/club_history_screen_decode.md |
| 0x00669780 | league.cpp | matrix_seed_base | PORTED_EXACT | matrix_seed_base | YES | BYTE_EXACT | rust:eng_second_fixtures.rs |
| 0x0066bd40 | league.cpp | matrix_perturb | PORTED_EXACT | matrix_perturb | YES | BYTE_EXACT | memory:perturb-resolver-slot-vs-club-bug.md |
| 0x0066f280 | league.cpp | walker_step | PORTED_EXACT | walker_step | YES | BYTE_EXACT | memory:c11-2-fixture-subsystem-frozen.md |

## friendly

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x005ac250 | friendly.cpp | FUN_005ac250 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005ac570 | friendly.cpp | FUN_005ac570 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005ac590 | friendly.cpp | FUN_005ac590 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005ac5a0 | friendly.cpp | FUN_005ac5a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005aca70 | friendly.cpp | FUN_005aca70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005acc60 | friendly.cpp | FUN_005acc60 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005acc80 | friendly.cpp | FUN_005acc80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005acdb0 | friendly.cpp | FUN_005acdb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005ad010 | friendly.cpp | FUN_005ad010 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005ad310 | friendly.cpp | FUN_005ad310 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005adeb0 | friendly.cpp | FUN_005adeb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005ae310 | friendly.cpp | FUN_005ae310 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005ae650 | friendly.cpp | FUN_005ae650 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005ae870 | friendly.cpp | FUN_005ae870 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005ae950 | friendly.cpp | FUN_005ae950 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005aea40 | friendly.cpp | FUN_005aea40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005aee30 | friendly.cpp | FUN_005aee30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005aef80 | friendly.cpp | FUN_005aef80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005af310 | friendly.cpp | FUN_005af310 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005af5a0 | friendly.cpp | FUN_005af5a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005af5e0 | friendly.cpp | FUN_005af5e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005afaf0 | friendly.cpp | FUN_005afaf0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b0210 | friendly.cpp | FUN_005b0210 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b0620 | friendly.cpp | FUN_005b0620 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b07b0 | friendly.cpp | FUN_005b07b0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005b0840 | friendly.cpp | FUN_005b0840 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005b0a10 | friendly.cpp | FUN_005b0a10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b10b0 | friendly.cpp | FUN_005b10b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b1490 | friendly.cpp | FUN_005b1490 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b1770 | friendly.cpp | FUN_005b1770 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b2500 | friendly.cpp | FUN_005b2500 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b2870 | friendly.cpp | FUN_005b2870 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005b2a50 | friendly.cpp | FUN_005b2a50 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005b2c90 | friendly.cpp | FUN_005b2c90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b3860 | friendly.cpp | FUN_005b3860 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b4260 | friendly.cpp | FUN_005b4260 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005b42c0 | friendly.cpp | FUN_005b42c0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005b44c0 | friendly.cpp | FUN_005b44c0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005b4610 | friendly.cpp | FUN_005b4610 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b4720 | friendly.cpp | FUN_005b4720 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b5be0 | friendly.cpp | FUN_005b5be0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b5c10 | friendly.cpp | FUN_005b5c10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b5d30 | friendly.cpp | FUN_005b5d30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b6000 | friendly.cpp | FUN_005b6000 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b6020 | friendly.cpp | FUN_005b6020 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b6500 | friendly.cpp | FUN_005b6500 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b6610 | friendly.cpp | FUN_005b6610 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b6740 | friendly.cpp | FUN_005b6740 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b6820 | friendly.cpp | FUN_005b6820 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b6920 | friendly.cpp | FUN_005b6920 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005b6a70 | friendly.cpp | FUN_005b6a70 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0074e140 | mini_league.cpp | FUN_0074e140 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0074e420 | mini_league.cpp | FUN_0074e420 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0074e440 | mini_league.cpp | FUN_0074e440 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0074e530 | mini_league.cpp | FUN_0074e530 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0074e5b0 | mini_league.cpp | FUN_0074e5b0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0074e830 | mini_league.cpp | FUN_0074e830 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0074e9f0 | mini_league.cpp | FUN_0074e9f0 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0074ea50 | mini_league.cpp | FUN_0074ea50 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0074eb60 | mini_league.cpp | FUN_0074eb60 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |
| 0x0074eb80 | mini_league.cpp | FUN_0074eb80 | FOREIGN_BREADTH |  | NO | UNVERIFIED |  |

## game-loop

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x005bfcd0 | game.cpp | FUN_005bfcd0 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005bfce0 | game.cpp | FUN_005bfce0 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005bfcf0 | game.cpp | FUN_005bfcf0 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005bfd00 | game.cpp | FUN_005bfd00 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005bfd10 | game.cpp | FUN_005bfd10 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005bfd60 | game.cpp | FUN_005bfd60 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005bfd70 | game.cpp | FUN_005bfd70 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005c0480 | game.cpp | FUN_005c0480 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005c0500 | game.cpp | FUN_005c0500 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x005c1410 | game.cpp | FUN_005c1410 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |

## geography

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00402eb0 | area.cpp | FUN_00402eb0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00403320 | area.cpp | FUN_00403320 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00403360 | area.cpp | FUN_00403360 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004037a0 | area.cpp | FUN_004037a0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00403ab0 | area.cpp | FUN_00403ab0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00403f20 | area.cpp | FUN_00403f20 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00404210 | area.cpp | FUN_00404210 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |

## i18n

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00653d30 | langlib.cpp | FUN_00653d30 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00653e20 | langlib.cpp | FUN_00653e20 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00667370 | langlib.cpp | FUN_00667370 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00667420 | langlib.cpp | FUN_00667420 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x006674d0 | langlib.cpp | FUN_006674d0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x006675b0 | langlib.cpp | FUN_006675b0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x006675d0 | langlib.cpp | FUN_006675d0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00667650 | langlib.cpp | FUN_00667650 | NON_USEFUL |  | NO | UNVERIFIED |  |

## injuries

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x0052df60 | database.cpp | physio rating (x87) | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | memory:history-panels-noheuristic-ports.md |
| 0x00616820 | index.cpp | pick injury id within body region | PORTED_EXACT | pick_injury | INDIRECT | BYTE_EXACT | memory:history-panels-noheuristic-ports.md |
| 0x00616930 | index.cpp | injury recovery-days formula | PORTED_EXACT | compute_injury | INDIRECT | BYTE_EXACT | memory:history-panels-noheuristic-ports.md |
| 0x00618610 | index.cpp | injury-name switch + duration table | PORTED_EXACT | INJURY_TYPES | INDIRECT | BYTE_EXACT | memory:history-panels-noheuristic-ports.md |

## io

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x005536b0 | dispute.cpp | FUN_005536b0 | REPLACED_BY_RUST |  | INDIRECT | UNVERIFIED |  |
| 0x00553a00 | dispute.cpp | FUN_00553a00 | REPLACED_BY_RUST |  | INDIRECT | STRUCTURALLY_VERIFIED |  |
| 0x00553c00 | dispute.cpp | FUN_00553c00 | REPLACED_BY_RUST |  | INDIRECT | STRUCTURALLY_VERIFIED |  |
| 0x00553c60 | dispute.cpp | FUN_00553c60 | REPLACED_BY_RUST |  | INDIRECT | STRUCTURALLY_VERIFIED |  |
| 0x00553cd0 | dispute.cpp | FUN_00553cd0 | REPLACED_BY_RUST |  | INDIRECT | UNVERIFIED |  |
| 0x00553f80 | dispute.cpp | FUN_00553f80 | REPLACED_BY_RUST |  | INDIRECT | STRUCTURALLY_VERIFIED |  |
| 0x00554130 | dispute.cpp | FUN_00554130 | REPLACED_BY_RUST |  | INDIRECT | UNVERIFIED |  |
| 0x00578030 | file_llist.cpp | FUN_00578030 | REPLACED_BY_RUST |  | INDIRECT | STRUCTURALLY_VERIFIED |  |
| 0x005780e0 | file_llist.cpp | FUN_005780e0 | REPLACED_BY_RUST |  | INDIRECT | UNVERIFIED |  |
| 0x00578750 | file_llist.cpp | FUN_00578750 | REPLACED_BY_RUST |  | INDIRECT | UNVERIFIED |  |
| 0x00578eb0 | file_llist.cpp | FUN_00578eb0 | REPLACED_BY_RUST |  | INDIRECT | UNVERIFIED |  |
| 0x00578ec0 | file_llist.cpp | FUN_00578ec0 | REPLACED_BY_RUST |  | INDIRECT | UNVERIFIED |  |
| 0x00921b90 | zipdir.cpp | FUN_00921b90 | REPLACED_BY_RUST |  | INDIRECT | STRUCTURALLY_VERIFIED |  |
| 0x00921ea0 | zipdir.cpp | FUN_00921ea0 | REPLACED_BY_RUST |  | INDIRECT | UNVERIFIED |  |
| 0x00922110 | zipdir.cpp | FUN_00922110 | REPLACED_BY_RUST |  | INDIRECT | STRUCTURALLY_VERIFIED |  |
| 0x00922200 | zipdir.cpp | FUN_00922200 | REPLACED_BY_RUST |  | INDIRECT | STRUCTURALLY_VERIFIED |  |
| 0x00922250 | zipdir.cpp | FUN_00922250 | REPLACED_BY_RUST |  | INDIRECT | UNVERIFIED |  |
| 0x009223b0 | zipdir.cpp | FUN_009223b0 | REPLACED_BY_RUST |  | INDIRECT | UNVERIFIED |  |
| 0x009226a0 | zipdir.cpp | FUN_009226a0 | REPLACED_BY_RUST |  | INDIRECT | UNVERIFIED |  |
| 0x009228f0 | zipdir.cpp | FUN_009228f0 | REPLACED_BY_RUST |  | INDIRECT | UNVERIFIED |  |

## manager

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x0052e370 | database.cpp | club-pickable/league filter gate | PORTED_BEHAVIOURAL | club_is_pickable | YES | BEHAVIOURALLY_EXACT | memory:league-dates-and-comp-wiring.md |
| 0x005e5330 | human_manager.cpp | add-human index allocator | PORTED_BEHAVIOURAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | memory:manager-creation-flow.md |
| 0x006809e0 | manager_manager.cpp | human-manager resign / clear-club link | PORTED_BEHAVIOURAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | memory:dashboard-manager-model.md |
| 0x00809ad0 | setup.cpp | create manager person+seat | PORTED_BEHAVIOURAL | create_manager | YES | BEHAVIOURALLY_EXACT | memory:manager-creation-flow.md |
| 0x00809cc0 | setup.cpp | Enter Name screen | PORTED_BEHAVIOURAL | ManagerNameEntryView | YES | BEHAVIOURALLY_EXACT | memory:manager-creation-flow.md |
| 0x0080a880 | setup.cpp | Select Nationality screen | PORTED_BEHAVIOURAL | ManagerNationalitySelectView | YES | BEHAVIOURALLY_EXACT | memory:manager-creation-flow.md |
| 0x0080b2b0 | setup.cpp | Select Team screen | PORTED_BEHAVIOURAL | ManagerClubSelectView | YES | BEHAVIOURALLY_EXACT | memory:manager-creation-flow.md |
| 0x00810f50 | setup.cpp | install human manager at club | PORTED_BEHAVIOURAL | create_manager | YES | BEHAVIOURALLY_EXACT | memory:manager-creation-flow.md |
| 0x00811140 | setup.cpp | start-game manageable gate + club-picker loop | PORTED_BEHAVIOURAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | memory:manager-creation-flow.md |

## manager-ai

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00672e40 | manager_manager.cpp | FUN_00672e40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00673330 | manager_manager.cpp | FUN_00673330 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006733d0 | manager_manager.cpp | FUN_006733d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00673b70 | manager_manager.cpp | FUN_00673b70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00674380 | manager_manager.cpp | FUN_00674380 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00675980 | manager_manager.cpp | FUN_00675980 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00675ae0 | manager_manager.cpp | FUN_00675ae0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00675dc0 | manager_manager.cpp | FUN_00675dc0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00675e20 | manager_manager.cpp | FUN_00675e20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00675e80 | manager_manager.cpp | FUN_00675e80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00675fd0 | manager_manager.cpp | FUN_00675fd0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00678aa0 | manager_manager.cpp | FUN_00678aa0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00679690 | manager_manager.cpp | FUN_00679690 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00679ed0 | manager_manager.cpp | FUN_00679ed0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0067a430 | manager_manager.cpp | FUN_0067a430 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0067a490 | manager_manager.cpp | FUN_0067a490 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0067b7c0 | manager_manager.cpp | FUN_0067b7c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0067bb90 | manager_manager.cpp | FUN_0067bb90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0067bd60 | manager_manager.cpp | FUN_0067bd60 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0067c490 | manager_manager.cpp | FUN_0067c490 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0067c680 | manager_manager.cpp | FUN_0067c680 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0067cb40 | manager_manager.cpp | FUN_0067cb40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006805a0 | manager_manager.cpp | FUN_006805a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006817b0 | manager_manager.cpp | FUN_006817b0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00681920 | manager_manager.cpp | FUN_00681920 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00681a50 | manager_manager.cpp | FUN_00681a50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00681b80 | manager_manager.cpp | FUN_00681b80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00681c70 | manager_manager.cpp | FUN_00681c70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00682420 | manager_manager.cpp | FUN_00682420 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00683dc0 | manager_manager.cpp | FUN_00683dc0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00684a30 | manager_manager.cpp | FUN_00684a30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00684a80 | manager_manager.cpp | FUN_00684a80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00684ed0 | manager_manager.cpp | FUN_00684ed0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00684fc0 | manager_manager.cpp | FUN_00684fc0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00685160 | manager_manager.cpp | FUN_00685160 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006857b0 | manager_manager.cpp | FUN_006857b0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00685900 | manager_manager.cpp | FUN_00685900 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00685c00 | manager_manager.cpp | FUN_00685c00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00685d40 | manager_manager.cpp | FUN_00685d40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006860e0 | manager_manager.cpp | FUN_006860e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00686400 | manager_manager.cpp | FUN_00686400 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00686d70 | manager_manager.cpp | FUN_00686d70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00686fd0 | manager_manager.cpp | FUN_00686fd0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006882d0 | manager_manager.cpp | FUN_006882d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006889e0 | manager_manager.cpp | FUN_006889e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00689710 | manager_manager.cpp | FUN_00689710 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006897c0 | manager_manager.cpp | FUN_006897c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00689c90 | manager_manager.cpp | FUN_00689c90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00689d50 | manager_manager.cpp | FUN_00689d50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00689eb0 | manager_manager.cpp | FUN_00689eb0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00689fb0 | manager_manager.cpp | FUN_00689fb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0068a1f0 | manager_manager.cpp | FUN_0068a1f0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0068a390 | manager_manager.cpp | FUN_0068a390 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0068a880 | manager_manager.cpp | FUN_0068a880 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0068acd0 | manager_manager.cpp | FUN_0068acd0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0068adb0 | manager_manager.cpp | FUN_0068adb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0068af80 | manager_manager.cpp | FUN_0068af80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0068b0a0 | manager_manager.cpp | FUN_0068b0a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0068b210 | manager_manager.cpp | FUN_0068b210 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0068b580 | manager_manager.cpp | FUN_0068b580 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0068bc30 | manager_manager.cpp | FUN_0068bc30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0068d9e0 | manager_manager.cpp | FUN_0068d9e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0068e5c0 | manager_manager.cpp | FUN_0068e5c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0068e6e0 | manager_manager.cpp | FUN_0068e6e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0068f090 | manager_manager.cpp | FUN_0068f090 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0068f0d0 | manager_manager.cpp | FUN_0068f0d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0068fa40 | manager_manager.cpp | FUN_0068fa40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0068fb80 | manager_manager.cpp | FUN_0068fb80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0068fd00 | manager_manager.cpp | FUN_0068fd00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0068fde0 | manager_manager.cpp | FUN_0068fde0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00690020 | manager_manager.cpp | FUN_00690020 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00690150 | manager_manager.cpp | FUN_00690150 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00690540 | manager_manager.cpp | FUN_00690540 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006908e0 | manager_manager.cpp | FUN_006908e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00690a70 | manager_manager.cpp | FUN_00690a70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00691160 | manager_manager.cpp | FUN_00691160 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x006912d0 | manager_manager.cpp | FUN_006912d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00691400 | manager_manager.cpp | FUN_00691400 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006916e0 | manager_manager.cpp | FUN_006916e0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x006917d0 | manager_manager.cpp | FUN_006917d0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00691970 | manager_manager.cpp | FUN_00691970 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00691af0 | manager_manager.cpp | FUN_00691af0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00691cc0 | manager_manager.cpp | FUN_00691cc0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00692880 | manager_manager.cpp | FUN_00692880 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00692ba0 | manager_manager.cpp | FUN_00692ba0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00692de0 | manager_manager.cpp | FUN_00692de0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006933f0 | manager_manager.cpp | FUN_006933f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00693510 | manager_manager.cpp | FUN_00693510 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006959d0 | manager_manager.cpp | FUN_006959d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |

## manager-model

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x005e51b0 | human_manager.cpp | FUN_005e51b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005e5240 | human_manager.cpp | FUN_005e5240 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005e5250 | human_manager.cpp | FUN_005e5250 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005e5820 | human_manager.cpp | FUN_005e5820 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e5940 | human_manager.cpp | FUN_005e5940 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e5b30 | human_manager.cpp | FUN_005e5b30 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e5c50 | human_manager.cpp | FUN_005e5c50 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e5d70 | human_manager.cpp | FUN_005e5d70 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e5e90 | human_manager.cpp | FUN_005e5e90 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e5fb0 | human_manager.cpp | FUN_005e5fb0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e6680 | human_manager.cpp | FUN_005e6680 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005e6b10 | human_manager.cpp | FUN_005e6b10 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e6c20 | human_manager.cpp | FUN_005e6c20 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e70d0 | human_manager.cpp | FUN_005e70d0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e7350 | human_manager.cpp | FUN_005e7350 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e7470 | human_manager.cpp | FUN_005e7470 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005e7570 | human_manager.cpp | FUN_005e7570 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e7670 | human_manager.cpp | FUN_005e7670 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005e7770 | human_manager.cpp | FUN_005e7770 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e7870 | human_manager.cpp | FUN_005e7870 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e7a80 | human_manager.cpp | FUN_005e7a80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005e7c80 | human_manager.cpp | FUN_005e7c80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005e7f60 | human_manager.cpp | FUN_005e7f60 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e8240 | human_manager.cpp | FUN_005e8240 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e8410 | human_manager.cpp | FUN_005e8410 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e8590 | human_manager.cpp | FUN_005e8590 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e86a0 | human_manager.cpp | FUN_005e86a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005e8890 | human_manager.cpp | FUN_005e8890 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005e9a70 | human_manager.cpp | FUN_005e9a70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005ea120 | human_manager.cpp | FUN_005ea120 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005ea280 | human_manager.cpp | FUN_005ea280 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005ea860 | human_manager.cpp | FUN_005ea860 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005ea8b0 | human_manager.cpp | FUN_005ea8b0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005eabd0 | human_manager.cpp | FUN_005eabd0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005ead90 | human_manager.cpp | FUN_005ead90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005eaf30 | human_manager.cpp | FUN_005eaf30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005eb0b0 | human_manager.cpp | FUN_005eb0b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005f5440 | human_manager.cpp | FUN_005f5440 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |

## match

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00699bc0 | match_day.cpp | FUN_00699bc0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0069aff0 | match_day.cpp | FUN_0069aff0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0069b3d0 | match_day.cpp | FUN_0069b3d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0069b710 | match_day.cpp | FUN_0069b710 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0069bfa0 | match_day.cpp | FUN_0069bfa0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0069c3a0 | match_day.cpp | FUN_0069c3a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0069ced0 | match_day.cpp | FUN_0069ced0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0069d3d0 | match_day.cpp | FUN_0069d3d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0069d720 | match_day.cpp | FUN_0069d720 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006c1520 | match_man.cpp | FUN_006c1520 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006c1590 | match_man.cpp | FUN_006c1590 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006c1660 | match_man.cpp | FUN_006c1660 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006c4e70 | match_man.cpp | FUN_006c4e70 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006c6710 | match_man.cpp | FUN_006c6710 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006c6aa0 | match_man.cpp | FUN_006c6aa0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006c7350 | match_man.cpp | FUN_006c7350 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006c7450 | match_man.cpp | FUN_006c7450 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006c74b0 | match_man.cpp | FUN_006c74b0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006c76e0 | match_man.cpp | FUN_006c76e0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006c7700 | match_man.cpp | FUN_006c7700 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006c7a10 | match_man.cpp | FUN_006c7a10 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006cbd50 | match_man.cpp | FUN_006cbd50 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006ce320 | match_man.cpp | FUN_006ce320 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006cee70 | match_man.cpp | FUN_006cee70 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006cee80 | match_man.cpp | match atmosphere/attendance factor | PORTED_EXACT | compute_atmosphere | YES | BYTE_EXACT |  |
| 0x006cef50 | match_man.cpp | FUN_006cef50 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x00710880 | match_stats.cpp | FUN_00710880 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00710c40 | match_stats.cpp | FUN_00710c40 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00710d80 | match_stats.cpp | FUN_00710d80 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x00710e80 | match_stats.cpp | FUN_00710e80 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x007110a0 | match_stats.cpp | FUN_007110a0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00711450 | match_stats.cpp | FUN_00711450 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00711870 | match_stats.cpp | FUN_00711870 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00712300 | match_stats.cpp | FUN_00712300 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00712670 | match_stats.cpp | FUN_00712670 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00712a50 | match_stats.cpp | FUN_00712a50 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00712a80 | match_stats.cpp | FUN_00712a80 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00911cd0 | weather.cpp | FUN_00911cd0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00912100 | weather.cpp | FUN_00912100 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x009121e0 | weather.cpp | FUN_009121e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x009124d0 | weather.cpp | FUN_009124d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00912780 | weather.cpp | FUN_00912780 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00912860 | weather.cpp | FUN_00912860 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00912af0 | weather.cpp | FUN_00912af0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00913020 | weather.cpp | FUN_00913020 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00913180 | weather.cpp | FUN_00913180 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00913290 | weather.cpp | FUN_00913290 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00913510 | weather.cpp | FUN_00913510 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00913620 | weather.cpp | FUN_00913620 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |

## match-engine

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x005a2c70 | formation.cpp | formation primary mask classifier | OUT_OF_SCOPE | crates/cm-domain/src/match_engine_exe.rs | NO | PARTIAL | memory:match-engine-gap-audit.md |
| 0x00699640 | match_day.cpp | match-day build | PORTED_BEHAVIOURAL | match_day_build | YES | STRUCTURALLY_VERIFIED | reports:match_day.md |
| 0x00699cd0 | match_day.cpp | match pre-play pass | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x00699d90 | match_day.cpp | match-day play driver | PORTED_BEHAVIOURAL | simulate_one_fixture_token_model | YES | PARTIAL | memory:match-engine-gap-audit.md |
| 0x0069d950 | match_eng.cpp | match-engine setup/pre-match pass | PORTED_PARTIAL | run_pre_match_pass | YES | PARTIAL | memory:history-panels-noheuristic-ports.md |
| 0x0069f2f0 | match_eng.cpp | match tick/step controller (subset) | PORTED_PARTIAL | match_tick | YES | PARTIAL | memory:match-engine-gap-audit.md |
| 0x006a0550 | match_eng.cpp | stored-action event resolver | NOT_YET_PORTED | crates/cm-domain/src/match_engine_exe.rs | NO | PARTIAL | memory:match-engine-gap-audit.md |
| 0x006a1940 | match_eng.cpp | pass-target picker | PORTED_EXACT | pass_target_picker | YES | BEHAVIOURALLY_EXACT | reports:match_action_path_decode.md |
| 0x006a2790 | match_eng.cpp | shot/pass target-lane picker | PORTED_EXACT | target_picker | YES | BEHAVIOURALLY_EXACT | reports:match_action_path_decode.md |
| 0x006a3240 | match_eng.cpp | period-transition/score snapshot | OUT_OF_SCOPE | crates/cm-domain/src/match_engine_exe.rs | NO | PARTIAL | memory:match-engine-gap-audit.md |
| 0x006a4020 | match_eng.cpp | phase/possession controller | OUT_OF_SCOPE | crates/cm-domain/src/match_engine_exe.rs | NO | PARTIAL | memory:match-engine-gap-audit.md |
| 0x006aae20 | match_eng.cpp | per-tick tactical/commentary updater | NOT_YET_PORTED | crates/cm-domain/src/match_engine_exe.rs | NO | PARTIAL | memory:match-engine-gap-audit.md |
| 0x006ae160 | match_eng.cpp | shot/pass/tackle outcome classifier | PORTED_EXACT | classify_shot_outcome | YES | BEHAVIOURALLY_EXACT | reports:match_action_path_decode.md |
| 0x006b3de0 | match_eng.cpp | per-match rating finalize | PORTED_EXACT | finalize_rating | YES | BEHAVIOURALLY_EXACT | rust:match_engine_exe.rs |
| 0x006b4510 | match_eng.cpp | positional candidate selector | OUT_OF_SCOPE | crates/cm-domain/src/match_engine_exe.rs | NO | PARTIAL | memory:match-engine-gap-audit.md |
| 0x006b69e0 | match_eng.cpp | man-of-the-match selector | PORTED_EXACT | select_motm | YES | BEHAVIOURALLY_EXACT | memory:history-panels-noheuristic-ports.md |
| 0x006b6c10 | match_eng.cpp | resolve queued shots | PORTED_BEHAVIOURAL | resolve_queued_shots | YES | BEHAVIOURALLY_EXACT | reports:match_action_path_decode.md |
| 0x006ba1e0 | match_eng.cpp | derby/grudge score | PORTED_EXACT | derby_score | YES | BEHAVIOURALLY_EXACT | rust:match_engine_exe.rs |
| 0x006ba380 | match_eng.cpp | post-match rating writeback | PORTED_EXACT | crates/cm-domain/src/match_engine_exe.rs | YES | STRUCTURALLY_VERIFIED | rust:match_engine_exe.rs |
| 0x006bc8d0 | match_events.cpp | match event-queue writer (subset) | PORTED_PARTIAL | match_events_generate | YES | PARTIAL | memory:match-engine-gap-audit.md |
| 0x006cfef0 | match_man.cpp | shot outcome dice resolver | PORTED_EXACT | shot_outcome_resolver | YES | BEHAVIOURALLY_EXACT | reports:match_action_path_decode.md |
| 0x006d1a20 | match_official.cpp | player evaluation float fields | OUT_OF_SCOPE | crates/cm-domain/src/match_engine_exe.rs | NO | PARTIAL | memory:match-engine-gap-audit.md |
| 0x006d63b0 | match_official.cpp | distance-quality LUT lookup | PORTED_EXACT | distance_quality | YES | BYTE_EXACT | rust:match_engine_exe.rs |
| 0x006d63f0 | match_pl.cpp | positional move/action resolution | OUT_OF_SCOPE | crates/cm-domain/src/match_engine_exe.rs | NO | PARTIAL | memory:match-engine-gap-audit.md |
| 0x006da0b0 | match_pl.cpp | token cell move | PORTED_EXACT | cell_move | INDIRECT | BYTE_EXACT | rust:match_engine_exe.rs |
| 0x006e65e0 | match_pl.cpp | shot/action score (positional) | OUT_OF_SCOPE | crates/cm-domain/src/match_engine_exe.rs | NO | PARTIAL | memory:match-engine-gap-audit.md |
| 0x006f5de0 | match_pl.cpp | physics/pressure continuation tick (subset) | PORTED_PARTIAL | physics_tick | YES | PARTIAL | memory:match-engine-gap-audit.md |
| 0x006f63f0 | match_pl.cpp | event-resolution dispatcher (positional) | OUT_OF_SCOPE | crates/cm-domain/src/match_engine_exe.rs | NO | PARTIAL | memory:match-engine-gap-audit.md |
| 0x006f99c0 | match_pl.cpp | per-tick action selector (subset; full positional out-of-scope) | PORTED_PARTIAL | decide_action;shot_attempt_gate | YES | PARTIAL | memory:match-engine-gap-audit.md |
| 0x007a90b0 | player_stats.cpp | per-match rating accumulator writer | PORTED_BEHAVIOURAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x00834fc0 | simulated_stats.cpp | non-engine competition advance | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | memory:league-tier-model.md |
| 0x00845cc0 | stadium.cpp | attendance + pitch modifiers | PORTED_PARTIAL | combine_attendance | YES | PARTIAL | rust:match_engine_exe.rs |

## match-physics

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x0069f1d0 | match_eng.cpp | FUN_0069f1d0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006a04a0 | match_eng.cpp | FUN_006a04a0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006a11e0 | match_eng.cpp | FUN_006a11e0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006a1260 | match_eng.cpp | FUN_006a1260 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006a12b0 | match_eng.cpp | FUN_006a12b0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006a1320 | match_eng.cpp | FUN_006a1320 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a1360 | match_eng.cpp | FUN_006a1360 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006a1390 | match_eng.cpp | FUN_006a1390 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006a1750 | match_eng.cpp | FUN_006a1750 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a1870 | match_eng.cpp | FUN_006a1870 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006a24e0 | match_eng.cpp | FUN_006a24e0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a2ec0 | match_eng.cpp | FUN_006a2ec0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006a2fc0 | match_eng.cpp | FUN_006a2fc0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a3070 | match_eng.cpp | FUN_006a3070 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a3a30 | match_eng.cpp | FUN_006a3a30 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a3e30 | match_eng.cpp | FUN_006a3e30 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006a47c0 | match_eng.cpp | FUN_006a47c0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a5fc0 | match_eng.cpp | FUN_006a5fc0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a60b0 | match_eng.cpp | FUN_006a60b0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a61a0 | match_eng.cpp | FUN_006a61a0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a6290 | match_eng.cpp | FUN_006a6290 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a6380 | match_eng.cpp | FUN_006a6380 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a6480 | match_eng.cpp | FUN_006a6480 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a6570 | match_eng.cpp | FUN_006a6570 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a7960 | match_eng.cpp | FUN_006a7960 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a7c20 | match_eng.cpp | FUN_006a7c20 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a7d00 | match_eng.cpp | FUN_006a7d00 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a8060 | match_eng.cpp | FUN_006a8060 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a8190 | match_eng.cpp | FUN_006a8190 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a8350 | match_eng.cpp | FUN_006a8350 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006a8920 | match_eng.cpp | FUN_006a8920 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a89f0 | match_eng.cpp | FUN_006a89f0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a9230 | match_eng.cpp | FUN_006a9230 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a9260 | match_eng.cpp | FUN_006a9260 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006a9310 | match_eng.cpp | FUN_006a9310 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006a93b0 | match_eng.cpp | FUN_006a93b0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a9ac0 | match_eng.cpp | FUN_006a9ac0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a9ba0 | match_eng.cpp | FUN_006a9ba0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006a9d80 | match_eng.cpp | FUN_006a9d80 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006aa130 | match_eng.cpp | FUN_006aa130 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006aa7d0 | match_eng.cpp | FUN_006aa7d0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006aa9e0 | match_eng.cpp | FUN_006aa9e0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006aaaf0 | match_eng.cpp | FUN_006aaaf0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b0fd0 | match_eng.cpp | FUN_006b0fd0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b1040 | match_eng.cpp | match bearing LUT read | PORTED_EXACT | bearing_atan2 | YES | BYTE_EXACT |  |
| 0x006b1080 | match_eng.cpp | FUN_006b1080 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b1100 | match_eng.cpp | FUN_006b1100 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b1290 | match_eng.cpp | FUN_006b1290 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b12b0 | match_eng.cpp | FUN_006b12b0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b1330 | match_eng.cpp | FUN_006b1330 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b1450 | match_eng.cpp | FUN_006b1450 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b1530 | match_eng.cpp | FUN_006b1530 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b1580 | match_eng.cpp | FUN_006b1580 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b19f0 | match_eng.cpp | FUN_006b19f0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b1e60 | match_eng.cpp | match distance LUT read | PORTED_EXACT | distance_quality | YES | BYTE_EXACT |  |
| 0x006b1ec0 | match_eng.cpp | FUN_006b1ec0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b24e0 | match_eng.cpp | FUN_006b24e0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b2cb0 | match_eng.cpp | FUN_006b2cb0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b2d50 | match_eng.cpp | FUN_006b2d50 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b2e10 | match_eng.cpp | FUN_006b2e10 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b2fe0 | match_eng.cpp | FUN_006b2fe0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b30f0 | match_eng.cpp | FUN_006b30f0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b33a0 | match_eng.cpp | FUN_006b33a0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b3680 | match_eng.cpp | FUN_006b3680 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b36d0 | match_eng.cpp | FUN_006b36d0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b3a70 | match_eng.cpp | FUN_006b3a70 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b3c90 | match_eng.cpp | FUN_006b3c90 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b4340 | match_eng.cpp | FUN_006b4340 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b5170 | match_eng.cpp | FUN_006b5170 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b51b0 | match_eng.cpp | FUN_006b51b0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b56c0 | match_eng.cpp | FUN_006b56c0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b5700 | match_eng.cpp | FUN_006b5700 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b5730 | match_eng.cpp | FUN_006b5730 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b5770 | match_eng.cpp | FUN_006b5770 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b57a0 | match_eng.cpp | FUN_006b57a0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b5870 | match_eng.cpp | FUN_006b5870 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b63e0 | match_eng.cpp | FUN_006b63e0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b65c0 | match_eng.cpp | FUN_006b65c0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b6aa0 | match_eng.cpp | FUN_006b6aa0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b7170 | match_eng.cpp | FUN_006b7170 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b76c0 | match_eng.cpp | FUN_006b76c0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b80b0 | match_eng.cpp | FUN_006b80b0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b9010 | match_eng.cpp | FUN_006b9010 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b97e0 | match_eng.cpp | FUN_006b97e0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006b9c50 | match_eng.cpp | FUN_006b9c50 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b9c80 | match_eng.cpp | FUN_006b9c80 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b9ca0 | match_eng.cpp | FUN_006b9ca0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006b9d30 | match_eng.cpp | FUN_006b9d30 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006ba4d0 | match_eng.cpp | FUN_006ba4d0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006bb250 | match_eng.cpp | FUN_006bb250 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006bb290 | match_eng.cpp | FUN_006bb290 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006bb2b0 | match_eng.cpp | FUN_006bb2b0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006bb310 | match_eng.cpp | FUN_006bb310 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006bb370 | match_eng.cpp | FUN_006bb370 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006bb3d0 | match_eng.cpp | FUN_006bb3d0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006bb450 | match_eng.cpp | FUN_006bb450 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006bb480 | match_eng.cpp | FUN_006bb480 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006bb4b0 | match_eng.cpp | FUN_006bb4b0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006bb550 | match_eng.cpp | FUN_006bb550 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006bb630 | match_eng.cpp | FUN_006bb630 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006bb660 | match_eng.cpp | FUN_006bb660 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006da000 | match_pl.cpp | FUN_006da000 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006da050 | match_pl.cpp | FUN_006da050 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006da320 | match_pl.cpp | FUN_006da320 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006da430 | match_pl.cpp | FUN_006da430 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006da440 | match_pl.cpp | FUN_006da440 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006da470 | match_pl.cpp | FUN_006da470 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006da4a0 | match_pl.cpp | FUN_006da4a0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006dafc0 | match_pl.cpp | FUN_006dafc0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006db0c0 | match_pl.cpp | FUN_006db0c0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006dc600 | match_pl.cpp | FUN_006dc600 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006df990 | match_pl.cpp | FUN_006df990 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006dfb40 | match_pl.cpp | match reachability die | PORTED_EXACT | reachability_check | YES | BEHAVIOURALLY_EXACT |  |
| 0x006dfbe0 | match_pl.cpp | FUN_006dfbe0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006dfe90 | match_pl.cpp | FUN_006dfe90 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006e5d00 | match_pl.cpp | FUN_006e5d00 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006e5d30 | match_pl.cpp | FUN_006e5d30 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006e5d40 | match_pl.cpp | FUN_006e5d40 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006e5d50 | match_pl.cpp | FUN_006e5d50 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006e5d60 | match_pl.cpp | FUN_006e5d60 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006e5d70 | match_pl.cpp | FUN_006e5d70 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006e5d80 | match_pl.cpp | FUN_006e5d80 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006e5d90 | match_pl.cpp | FUN_006e5d90 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006e5da0 | match_pl.cpp | FUN_006e5da0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006e5db0 | match_pl.cpp | FUN_006e5db0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006e5dc0 | match_pl.cpp | FUN_006e5dc0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006e5e10 | match_pl.cpp | FUN_006e5e10 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006e5e20 | match_pl.cpp | FUN_006e5e20 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006e5e30 | match_pl.cpp | FUN_006e5e30 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006f0270 | match_pl.cpp | FUN_006f0270 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006f02a0 | match_pl.cpp | FUN_006f02a0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006f02d0 | match_pl.cpp | FUN_006f02d0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006f2e40 | match_pl.cpp | FUN_006f2e40 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006f3040 | match_pl.cpp | FUN_006f3040 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006f30a0 | match_pl.cpp | FUN_006f30a0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006f3900 | match_pl.cpp | FUN_006f3900 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006f3930 | match_pl.cpp | FUN_006f3930 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006f3950 | match_pl.cpp | FUN_006f3950 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006f3980 | match_pl.cpp | FUN_006f3980 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006f39b0 | match_pl.cpp | FUN_006f39b0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006f39e0 | match_pl.cpp | FUN_006f39e0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006f4680 | match_pl.cpp | FUN_006f4680 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006f5dc0 | match_pl.cpp | FUN_006f5dc0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006f9790 | match_pl.cpp | FUN_006f9790 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006f9890 | match_pl.cpp | FUN_006f9890 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006fa660 | match_pl.cpp | FUN_006fa660 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006fa6a0 | match_pl.cpp | FUN_006fa6a0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006fb930 | match_pl.cpp | FUN_006fb930 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006fb950 | match_pl.cpp | FUN_006fb950 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006fbad0 | match_pl.cpp | FUN_006fbad0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006fbae0 | match_pl.cpp | FUN_006fbae0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006fbaf0 | match_pl.cpp | FUN_006fbaf0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006fbc70 | match_pl.cpp | FUN_006fbc70 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006fbd10 | match_pl.cpp | FUN_006fbd10 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006fbdc0 | match_pl.cpp | FUN_006fbdc0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006fbe50 | match_pl.cpp | FUN_006fbe50 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006fbeb0 | match_pl.cpp | FUN_006fbeb0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006fbfb0 | match_pl.cpp | FUN_006fbfb0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006fc030 | match_pl.cpp | FUN_006fc030 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006fc690 | match_pl.cpp | FUN_006fc690 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006fc810 | match_pl.cpp | FUN_006fc810 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006fccd0 | match_pl.cpp | FUN_006fccd0 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006fce00 | match_pl.cpp | FUN_006fce00 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006fce30 | match_pl.cpp | FUN_006fce30 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x006fce50 | match_pl.cpp | FUN_006fce50 | OUT_OF_SCOPE |  | NO | PARTIAL |  |
| 0x006fd670 | match_pl.cpp | FUN_006fd670 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |

## media

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00712ac0 | media.cpp | FUN_00712ac0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00712e70 | media.cpp | FUN_00712e70 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00713090 | media.cpp | FUN_00713090 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007131c0 | media.cpp | FUN_007131c0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00713470 | media.cpp | FUN_00713470 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x007135d0 | media.cpp | FUN_007135d0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00713a70 | media.cpp | FUN_00713a70 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00714ae0 | media.cpp | FUN_00714ae0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0071a4f0 | media.cpp | FUN_0071a4f0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0071a860 | media.cpp | FUN_0071a860 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0071b2c0 | media.cpp | FUN_0071b2c0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0071b820 | media.cpp | FUN_0071b820 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0071c9c0 | media.cpp | FUN_0071c9c0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0071cfe0 | media.cpp | FUN_0071cfe0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0071d020 | media.cpp | FUN_0071d020 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0071d0a0 | media.cpp | FUN_0071d0a0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0071d0d0 | media.cpp | FUN_0071d0d0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0071df80 | media.cpp | FUN_0071df80 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0071e460 | media.cpp | FUN_0071e460 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0071fba0 | media.cpp | FUN_0071fba0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0071ff50 | media.cpp | FUN_0071ff50 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x007205b0 | media.cpp | FUN_007205b0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x007260d0 | media.cpp | FUN_007260d0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00728880 | media.cpp | FUN_00728880 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00729c20 | media.cpp | FUN_00729c20 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0072a760 | media.cpp | FUN_0072a760 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0072acf0 | media.cpp | FUN_0072acf0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0072af10 | media.cpp | FUN_0072af10 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0072b3c0 | media.cpp | FUN_0072b3c0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0072b6a0 | media.cpp | FUN_0072b6a0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0072bbb0 | media.cpp | FUN_0072bbb0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0072c8e0 | media.cpp | FUN_0072c8e0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x007329a0 | media.cpp | FUN_007329a0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00732aa0 | media.cpp | FUN_00732aa0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0073c180 | media.cpp | FUN_0073c180 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0073c3f0 | media.cpp | FUN_0073c3f0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0073c470 | media.cpp | FUN_0073c470 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0073cd80 | media.cpp | FUN_0073cd80 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0073d2a0 | media.cpp | FUN_0073d2a0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0073eaf0 | media.cpp | FUN_0073eaf0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0073ece0 | media.cpp | FUN_0073ece0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0073f490 | media.cpp | FUN_0073f490 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0073f6b0 | media.cpp | FUN_0073f6b0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0073f7e0 | media.cpp | FUN_0073f7e0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0073f970 | media.cpp | FUN_0073f970 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0073f9e0 | media.cpp | FUN_0073f9e0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0073fa50 | media.cpp | FUN_0073fa50 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0073fc40 | media.cpp | FUN_0073fc40 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00740820 | media.cpp | FUN_00740820 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00740a20 | media.cpp | FUN_00740a20 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00741050 | media.cpp | FUN_00741050 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00741450 | media.cpp | FUN_00741450 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00741d80 | media.cpp | FUN_00741d80 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00742080 | media.cpp | FUN_00742080 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00742380 | media.cpp | FUN_00742380 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00742e80 | media.cpp | FUN_00742e80 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00743120 | media.cpp | FUN_00743120 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00743400 | media.cpp | FUN_00743400 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00743dd0 | media.cpp | FUN_00743dd0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00743fe0 | media.cpp | FUN_00743fe0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00744a00 | media.cpp | FUN_00744a00 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00744c50 | media.cpp | FUN_00744c50 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00748e70 | media.cpp | FUN_00748e70 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00748ed0 | media.cpp | FUN_00748ed0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0074cba0 | media.cpp | FUN_0074cba0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0074cd70 | media.cpp | FUN_0074cd70 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |

## nation

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x006508e0 | key_nation.cpp | per-nation season-table builder | PORTED_BEHAVIOURAL | season_start | YES | STRUCTURALLY_VERIFIED | memory/league-dates-and-comp-wiring.md |
| 0x006527e0 | key_nation.cpp | tier-1 same-nation routing | PORTED_BEHAVIOURAL | MatchDetailMode::NotSimulated | YES | BEHAVIOURALLY_EXACT | memory:league-tier-model.md |
| 0x00652820 | key_nation.cpp | FUN_00652820 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00652930 | key_nation.cpp | FUN_00652930 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00652990 | key_nation.cpp | FUN_00652990 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x006529e0 | key_nation.cpp | FUN_006529e0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00652a00 | key_nation.cpp | nation -> season-boundary index resolver | NOT_YET_PORTED | crates/cm-domain/src/club_history.rs;crates/cm-domain/src/league_calendar.rs | BLOCKED | HYPOTHESIS | reports/club_history_screen_decode.md |
| 0x00652b10 | key_nation.cpp | FUN_00652b10 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00652c60 | key_nation.cpp | FUN_00652c60 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00652ca0 | key_nation.cpp | FUN_00652ca0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00652cd0 | key_nation.cpp | season-year -> "YYYY/YY" label | PORTED_PARTIAL | season_label | INDIRECT | PARTIAL | reports/club_history_screen_decode.md |
| 0x00653070 | key_nation.cpp | FUN_00653070 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x006530b0 | key_nation.cpp | FUN_006530b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00653180 | key_nation.cpp | FUN_00653180 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00653760 | key_nation.cpp | FUN_00653760 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00653920 | key_nation.cpp | FUN_00653920 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x006539f0 | key_nation.cpp | FUN_006539f0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00653a00 | key_nation.cpp | FUN_00653a00 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00653ad0 | key_nation.cpp | FUN_00653ad0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00653b00 | key_nation.cpp | FUN_00653b00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00653ca0 | key_nation.cpp | FUN_00653ca0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00653cf0 | key_nation.cpp | FUN_00653cf0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00683e30 | manager_manager.cpp | nation-tier selected-bit set/clear + job-take | PORTED_BEHAVIOURAL | crates/cm-domain/src/lib.rs | YES | STRUCTURALLY_VERIFIED | memory:league-tier-model.md |
| 0x0069c0d0 | match_day.cpp | tier-2 detailed-match routing | PORTED_BEHAVIOURAL | MatchDetailMode | YES | BEHAVIOURALLY_EXACT | memory:league-tier-model.md |
| 0x00806640 | setup.cpp | league picker tier bitfield | PORTED_BEHAVIOURAL | LeagueTier;build_nation_tiers | YES | BEHAVIOURALLY_EXACT | memory:league-tier-model.md |
| 0x00821e90 | setup.cpp | compid->loaded-league table | REPLACED_BY_RUST | competition_ids_for_nations | YES | STRUCTURALLY_VERIFIED | memory:league-dates-and-comp-wiring.md |

## national-caps

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00753240 | national_teams.cpp | match_finalize_credit_caps_goals | PORTED_BEHAVIOURAL | World::credit_caps | INDIRECT | BEHAVIOURALLY_EXACT | memory:fifa-rankings-mechanism.md |
| 0x00855c00 | staff_records.cpp | person_history_intl_debut | PORTED_BEHAVIOURAL | World::credit_caps | INDIRECT | BEHAVIOURALLY_EXACT | rust:national_match.rs |

## national-teams

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00752120 | national_teams.cpp | FUN_00752120 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00752350 | national_teams.cpp | FUN_00752350 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00752480 | national_teams.cpp | FUN_00752480 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00753ca0 | national_teams.cpp | FUN_00753ca0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00753d30 | national_teams.cpp | FUN_00753d30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00753d60 | national_teams.cpp | FUN_00753d60 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00755070 | national_teams.cpp | FUN_00755070 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007551a0 | national_teams.cpp | FUN_007551a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00755420 | national_teams.cpp | FUN_00755420 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00755450 | national_teams.cpp | FUN_00755450 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007554c0 | national_teams.cpp | FUN_007554c0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00755580 | national_teams.cpp | FUN_00755580 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007555f0 | national_teams.cpp | FUN_007555f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007556c0 | national_teams.cpp | FUN_007556c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00755740 | national_teams.cpp | FUN_00755740 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00755c50 | national_teams.cpp | FUN_00755c50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00755c80 | national_teams.cpp | FUN_00755c80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00755cb0 | national_teams.cpp | FUN_00755cb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00755cf0 | national_teams.cpp | FUN_00755cf0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00755ff0 | national_teams.cpp | FUN_00755ff0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00756de0 | national_teams.cpp | FUN_00756de0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00756fb0 | national_teams.cpp | FUN_00756fb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00757410 | national_teams.cpp | FUN_00757410 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00757680 | national_teams.cpp | FUN_00757680 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00757950 | national_teams.cpp | FUN_00757950 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00757b80 | national_teams.cpp | FUN_00757b80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00757e30 | national_teams.cpp | FUN_00757e30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00758030 | national_teams.cpp | FUN_00758030 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00758300 | national_teams.cpp | FUN_00758300 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00758960 | national_teams.cpp | FUN_00758960 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075cb70 | national_teams.cpp | FUN_0075cb70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075cbd0 | national_teams.cpp | FUN_0075cbd0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075cc40 | national_teams.cpp | FUN_0075cc40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075ccb0 | national_teams.cpp | FUN_0075ccb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075ce20 | national_teams.cpp | FUN_0075ce20 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0075ce60 | national_teams.cpp | FUN_0075ce60 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075d060 | national_teams.cpp | FUN_0075d060 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075d100 | national_teams.cpp | FUN_0075d100 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0075d160 | national_teams.cpp | FUN_0075d160 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075d1c0 | national_teams.cpp | FUN_0075d1c0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0075d250 | national_teams.cpp | FUN_0075d250 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075d3d0 | national_teams.cpp | FUN_0075d3d0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0075d410 | national_teams.cpp | FUN_0075d410 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075d7e0 | national_teams.cpp | FUN_0075d7e0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0075d910 | national_teams.cpp | FUN_0075d910 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075dbd0 | national_teams.cpp | FUN_0075dbd0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075de90 | national_teams.cpp | FUN_0075de90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075df30 | national_teams.cpp | FUN_0075df30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075dfc0 | national_teams.cpp | FUN_0075dfc0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075dff0 | national_teams.cpp | FUN_0075dff0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075e660 | national_teams.cpp | FUN_0075e660 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075ec00 | national_teams.cpp | FUN_0075ec00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075f1d0 | national_teams.cpp | FUN_0075f1d0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0075f200 | national_teams.cpp | FUN_0075f200 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075f420 | national_teams.cpp | FUN_0075f420 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0075f5f0 | national_teams.cpp | FUN_0075f5f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075f620 | national_teams.cpp | FUN_0075f620 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0075f8e0 | national_teams.cpp | FUN_0075f8e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |

## netcode

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00763640 | network.cpp | FUN_00763640 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0089a880 | tcpip.cpp | FUN_0089a880 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0089a960 | tcpip.cpp | FUN_0089a960 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0089a9a0 | tcpip.cpp | FUN_0089a9a0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0089aa40 | tcpip.cpp | FUN_0089aa40 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0089aa70 | tcpip.cpp | FUN_0089aa70 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0089ad40 | tcpip.cpp | FUN_0089ad40 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0089af40 | tcpip.cpp | FUN_0089af40 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0089b0c0 | tcpip.cpp | FUN_0089b0c0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0089b180 | tcpip.cpp | FUN_0089b180 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0089b360 | tcpip.cpp | FUN_0089b360 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0089b390 | tcpip.cpp | FUN_0089b390 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0089b480 | tcpip.cpp | FUN_0089b480 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0089b500 | tcpip.cpp | FUN_0089b500 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0089b520 | tcpip.cpp | FUN_0089b520 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0089b990 | tcpip.cpp | FUN_0089b990 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0089bd30 | tcpip.cpp | FUN_0089bd30 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x0089bd50 | tcpip.cpp | FUN_0089bd50 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |

## news

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x0067ce90 | manager_manager.cpp | news_weekly_predicate_cascade | NOT_YET_PORTED | crates/cm-domain/src/news.rs | BLOCKED | HYPOTHESIS | memory:news-generation-logic.md |
| 0x00733610 | media.cpp | news_formatter_code_to_template | NOT_YET_PORTED | news::format | BLOCKED | HYPOTHESIS | memory:news-generation-logic.md |
| 0x00763c30 | news.cpp | FUN_00763c30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0076ab10 | news.cpp | news-item action classifier | PORTED_BEHAVIOURAL | classify_news_action | INDIRECT | STRUCTURALLY_VERIFIED | memory:news-generation-logic.md |
| 0x0076c0a0 | news.cpp | FUN_0076c0a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0076ce50 | news.cpp | person_news_severity_overwrite | NOT_YET_PORTED | PersonNewsItem::severity | NO | PARTIAL | rust:person_news.rs |
| 0x0076d730 | news.cpp | person_news_set_param_slot | PORTED_PARTIAL | PersonNewsItem::params | INDIRECT | STRUCTURALLY_VERIFIED | rust:person_news.rs |
| 0x0076d860 | news.cpp | FUN_0076d860 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0076d9c0 | news.cpp | FUN_0076d9c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0076dac0 | news.cpp | FUN_0076dac0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0076dca0 | news.cpp | FUN_0076dca0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0076dce0 | news.cpp | person_news_persist_and_id | PORTED_BEHAVIOURAL | PersonNewsMailboxPool::next_news_id | INDIRECT | BEHAVIOURALLY_EXACT | rust:person_news.rs |
| 0x0076dfb0 | news.cpp | FUN_0076dfb0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0076e030 | news.cpp | FUN_0076e030 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0076e270 | news.cpp | FUN_0076e270 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0076e4c0 | news.cpp | FUN_0076e4c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0076e5e0 | news.cpp | FUN_0076e5e0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0076e720 | news.cpp | FUN_0076e720 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0076e800 | news.cpp | FUN_0076e800 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0076e900 | news.cpp | FUN_0076e900 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0076f0e0 | news.cpp | FUN_0076f0e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0076f450 | news.cpp | FUN_0076f450 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0076f580 | news.cpp | FUN_0076f580 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0076f5d0 | news.cpp | FUN_0076f5d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0076f720 | news.cpp | FUN_0076f720 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0076fbd0 | news.cpp | FUN_0076fbd0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d0d90 | transfer_manager.cpp | person_news_item_build | PORTED_PARTIAL | PersonNewsItem;PersonNewsMailboxPool::push | INDIRECT | STRUCTURALLY_VERIFIED | memory:news-generation-logic.md |
| 0x009346f7 | zipdir.cpp | person_news_pool_realloc | REPLACED_BY_RUST | PersonNewsMailboxPool::by_person | INDIRECT | STRUCTURALLY_VERIFIED | rust:person_news.rs |

## notes

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x0077cc00 | notes.cpp | FUN_0077cc00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0077cf20 | notes.cpp | FUN_0077cf20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0077d380 | notes.cpp | FUN_0077d380 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0077d3f0 | notes.cpp | FUN_0077d3f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0077d570 | notes.cpp | FUN_0077d570 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0077d680 | notes.cpp | FUN_0077d680 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0077d6f0 | notes.cpp | FUN_0077d6f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0077d730 | notes.cpp | FUN_0077d730 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0077d7d0 | notes.cpp | FUN_0077d7d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0077da30 | notes.cpp | FUN_0077da30 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0077da80 | notes.cpp | FUN_0077da80 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0077dab0 | notes.cpp | FUN_0077dab0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0077dbb0 | notes.cpp | FUN_0077dbb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0077de40 | notes.cpp | FUN_0077de40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0077df70 | notes.cpp | FUN_0077df70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0077e120 | notes.cpp | FUN_0077e120 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0077e360 | notes.cpp | FUN_0077e360 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0077ecb0 | notes.cpp | FUN_0077ecb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |

## officials

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x006d0170 | match_official.cpp | FUN_006d0170 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006d0570 | match_official.cpp | FUN_006d0570 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006d6040 | match_official.cpp | FUN_006d6040 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006d6370 | match_official.cpp | FUN_006d6370 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00783ba0 | officials_manager.cpp | FUN_00783ba0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00783ed0 | officials_manager.cpp | FUN_00783ed0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00784290 | officials_manager.cpp | FUN_00784290 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007842b0 | officials_manager.cpp | FUN_007842b0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00784880 | officials_manager.cpp | FUN_00784880 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00784910 | officials_manager.cpp | FUN_00784910 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00784b00 | officials_manager.cpp | FUN_00784b00 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00784df0 | officials_manager.cpp | FUN_00784df0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00785230 | officials_manager.cpp | FUN_00785230 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x007854e0 | officials_manager.cpp | FUN_007854e0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00785540 | officials_manager.cpp | FUN_00785540 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00785650 | officials_manager.cpp | FUN_00785650 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00785700 | officials_manager.cpp | FUN_00785700 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00785730 | officials_manager.cpp | FUN_00785730 | NON_USEFUL |  | NO | UNVERIFIED |  |

## player

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x0052e260 | database.cpp | form/morale byte setter | UNKNOWN | crates/cm-domain/src/lib.rs | NO | UNVERIFIED | memory:squad-number-not-at-45.md |
| 0x0069c6f0 | match_day.cpp | form/morale post-match writer | UNKNOWN | crates/cm-domain/src/lib.rs | NO | UNVERIFIED | memory:squad-number-not-at-45.md |

## player-init

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x0051f5d0 | database.cpp | player_init_seed_person | PORTED_PARTIAL | PlayerInitState::seed;World::initialise_players | YES | STATE_EXACT | memory:player-init-decode.md |
| 0x00524160 | database.cpp | player_attr_generate_mode_a | PORTED_PARTIAL | PlayerInitState::generate_attributes_core | YES | PARTIAL | memory:player-init-decode.md |
| 0x00524560 | database.cpp | player attr generate mode-A slot order | PORTED_PARTIAL | PlayerInitState::generate_attributes_core | YES | PARTIAL | memory:player-init-decode.md |
| 0x008120d0 | setup.cpp | init_game_data_boot_pass | PORTED_BEHAVIOURAL | World::boot_player_init_states;World::run_start_game_init;World::assign_squad_numbers | YES | STRUCTURALLY_VERIFIED | memory:start-new-game-flow.md |

## player-relationships

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x0050e9d0 | database.cpp | FUN_0050e9d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00510420 | database.cpp | FUN_00510420 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00510ea0 | database.cpp | FUN_00510ea0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00511b40 | database.cpp | FUN_00511b40 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00511b80 | database.cpp | FUN_00511b80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00511bc0 | database.cpp | FUN_00511bc0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00511ce0 | database.cpp | FUN_00511ce0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0051c970 | database.cpp | FUN_0051c970 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0051cbf0 | database.cpp | FUN_0051cbf0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0051d160 | database.cpp | FUN_0051d160 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0051d280 | database.cpp | FUN_0051d280 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0051d510 | database.cpp | FUN_0051d510 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0051f480 | database.cpp | FUN_0051f480 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0051f490 | database.cpp | FUN_0051f490 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00522710 | database.cpp | FUN_00522710 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00522a10 | database.cpp | FUN_00522a10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00522fe0 | database.cpp | FUN_00522fe0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005232a0 | database.cpp | FUN_005232a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00523630 | database.cpp | FUN_00523630 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005236e0 | database.cpp | FUN_005236e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005237b0 | database.cpp | FUN_005237b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00523c30 | database.cpp | FUN_00523c30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00523d00 | database.cpp | FUN_00523d00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00524100 | database.cpp | FUN_00524100 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00524400 | database.cpp | FUN_00524400 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005244d0 | database.cpp | FUN_005244d0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00524790 | database.cpp | FUN_00524790 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00525160 | database.cpp | FUN_00525160 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00525170 | database.cpp | FUN_00525170 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00525310 | database.cpp | FUN_00525310 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005254f0 | database.cpp | FUN_005254f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00525860 | database.cpp | FUN_00525860 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00525ce0 | database.cpp | FUN_00525ce0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005260e0 | database.cpp | FUN_005260e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00526420 | database.cpp | FUN_00526420 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00526500 | database.cpp | FUN_00526500 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005266d0 | database.cpp | FUN_005266d0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00526780 | database.cpp | FUN_00526780 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005268b0 | database.cpp | FUN_005268b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005269e0 | database.cpp | FUN_005269e0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00526a30 | database.cpp | FUN_00526a30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00527290 | database.cpp | FUN_00527290 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00527320 | database.cpp | FUN_00527320 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00527450 | database.cpp | FUN_00527450 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00527490 | database.cpp | FUN_00527490 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005274d0 | database.cpp | FUN_005274d0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00527690 | database.cpp | FUN_00527690 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00527c90 | database.cpp | FUN_00527c90 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00527e30 | database.cpp | FUN_00527e30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005284e0 | database.cpp | FUN_005284e0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00528600 | database.cpp | FUN_00528600 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00529bd0 | database.cpp | FUN_00529bd0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00529c80 | database.cpp | FUN_00529c80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00529dc0 | database.cpp | FUN_00529dc0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00529e90 | database.cpp | FUN_00529e90 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0052a070 | database.cpp | FUN_0052a070 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052a2d0 | database.cpp | FUN_0052a2d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052a410 | database.cpp | FUN_0052a410 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0052a4d0 | database.cpp | FUN_0052a4d0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0052b9b0 | database.cpp | FUN_0052b9b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052bae0 | database.cpp | FUN_0052bae0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0052bc50 | database.cpp | FUN_0052bc50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052c290 | database.cpp | FUN_0052c290 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0052cd30 | database.cpp | FUN_0052cd30 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0052cdd0 | database.cpp | FUN_0052cdd0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0052cf90 | database.cpp | FUN_0052cf90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052d660 | database.cpp | FUN_0052d660 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052da80 | database.cpp | FUN_0052da80 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0052db20 | database.cpp | FUN_0052db20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052de40 | database.cpp | FUN_0052de40 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0052ded0 | database.cpp | FUN_0052ded0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0052e070 | database.cpp | FUN_0052e070 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0052e150 | database.cpp | FUN_0052e150 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0052e1c0 | database.cpp | FUN_0052e1c0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0052e220 | database.cpp | FUN_0052e220 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052e410 | database.cpp | FUN_0052e410 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052e4c0 | database.cpp | FUN_0052e4c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052e7c0 | database.cpp | FUN_0052e7c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052ea80 | database.cpp | FUN_0052ea80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052eb10 | database.cpp | FUN_0052eb10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052ef40 | database.cpp | FUN_0052ef40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052f010 | database.cpp | FUN_0052f010 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0052f0f0 | database.cpp | FUN_0052f0f0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0052f1f0 | database.cpp | FUN_0052f1f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052f2c0 | database.cpp | FUN_0052f2c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052f3d0 | database.cpp | FUN_0052f3d0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0052f4e0 | database.cpp | FUN_0052f4e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052f5b0 | database.cpp | FUN_0052f5b0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0052f910 | database.cpp | FUN_0052f910 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052f9e0 | database.cpp | FUN_0052f9e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0052fd90 | database.cpp | FUN_0052fd90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00530180 | database.cpp | FUN_00530180 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00530690 | database.cpp | FUN_00530690 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00530cb0 | database.cpp | FUN_00530cb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00531350 | database.cpp | FUN_00531350 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005313b0 | database.cpp | FUN_005313b0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00531420 | database.cpp | FUN_00531420 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00531450 | database.cpp | FUN_00531450 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00531520 | database.cpp | FUN_00531520 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005315f0 | database.cpp | FUN_005315f0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005316d0 | database.cpp | FUN_005316d0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005317b0 | database.cpp | FUN_005317b0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005317f0 | database.cpp | FUN_005317f0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00531830 | database.cpp | FUN_00531830 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00531870 | database.cpp | FUN_00531870 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005318c0 | database.cpp | FUN_005318c0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00531910 | database.cpp | FUN_00531910 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00531940 | database.cpp | FUN_00531940 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00531970 | database.cpp | FUN_00531970 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00531a50 | database.cpp | FUN_00531a50 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00531b30 | database.cpp | FUN_00531b30 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00531b70 | database.cpp | FUN_00531b70 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00531c70 | database.cpp | FUN_00531c70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00531cd0 | database.cpp | FUN_00531cd0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00531d30 | database.cpp | FUN_00531d30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00531d80 | database.cpp | FUN_00531d80 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00531ed0 | database.cpp | FUN_00531ed0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00532390 | database.cpp | FUN_00532390 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005323b0 | database.cpp | FUN_005323b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005323f0 | database.cpp | FUN_005323f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00532980 | database.cpp | FUN_00532980 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00532ce0 | database.cpp | FUN_00532ce0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00532de0 | database.cpp | FUN_00532de0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00532ea0 | database.cpp | FUN_00532ea0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00532fa0 | database.cpp | FUN_00532fa0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00533050 | database.cpp | FUN_00533050 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005330b0 | database.cpp | FUN_005330b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00533190 | database.cpp | FUN_00533190 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00533270 | database.cpp | FUN_00533270 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00533350 | database.cpp | FUN_00533350 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00533430 | database.cpp | FUN_00533430 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00533520 | database.cpp | FUN_00533520 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00533610 | database.cpp | FUN_00533610 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00533870 | database.cpp | FUN_00533870 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005338d0 | database.cpp | FUN_005338d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00533940 | database.cpp | FUN_00533940 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005339b0 | database.cpp | FUN_005339b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00533a30 | database.cpp | FUN_00533a30 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00533a80 | database.cpp | FUN_00533a80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |

## player-stats

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x007abd80 | player_stats.cpp | career apps per competition | PORTED_BEHAVIOURAL | World::club_history_view | YES | STRUCTURALLY_VERIFIED | reports/club_history_screen_decode.md |
| 0x007abef0 | player_stats.cpp | career goals per competition | PORTED_BEHAVIOURAL | World::club_history_view | YES | STRUCTURALLY_VERIFIED | reports/club_history_screen_decode.md |
| 0x007ac060 | player_stats.cpp | career total apps | PORTED_BEHAVIOURAL | total_apps | YES | STRUCTURALLY_VERIFIED | reports/club_history_screen_decode.md |
| 0x00834ae0 | simulated_stats.cpp | FUN_00834ae0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00834fb0 | simulated_stats.cpp | FUN_00834fb0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008356b0 | simulated_stats.cpp | FUN_008356b0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00835820 | simulated_stats.cpp | FUN_00835820 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00835990 | simulated_stats.cpp | FUN_00835990 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008359d0 | simulated_stats.cpp | FUN_008359d0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00835c20 | simulated_stats.cpp | FUN_00835c20 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00835c90 | simulated_stats.cpp | FUN_00835c90 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x00835cb0 | simulated_stats.cpp | FUN_00835cb0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00835d10 | simulated_stats.cpp | FUN_00835d10 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00835d60 | simulated_stats.cpp | FUN_00835d60 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00835e20 | simulated_stats.cpp | FUN_00835e20 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00835e60 | simulated_stats.cpp | FUN_00835e60 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00835eb0 | simulated_stats.cpp | FUN_00835eb0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008360a0 | simulated_stats.cpp | FUN_008360a0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |

## plumbing

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00933d24 | zipdir.cpp | CRT/container plumbing thunk | NON_USEFUL | crates/cm-render/src/scrman.rs | NO | STRUCTURALLY_VERIFIED | rust:cm-render/scrman.rs |
| 0x00933d8f | zipdir.cpp | CRT/container plumbing thunk | NON_USEFUL | crates/cm-render/src/dispatcher.rs | NO | STRUCTURALLY_VERIFIED | rust:cm-render/dispatcher.rs |
| 0x0093435a | zipdir.cpp | CRT/container plumbing thunk | NON_USEFUL | crates/cm-render/src/scrman.rs | NO | STRUCTURALLY_VERIFIED | rust:cm-render/scrman.rs |
| 0x009349c4 | zipdir.cpp | CRT/container plumbing thunk | NON_USEFUL | crates/cm-render/src/scrman.rs | NO | STRUCTURALLY_VERIFIED | rust:cm-render |
| 0x00934ba3 | zipdir.cpp | CRT/container plumbing thunk | NON_USEFUL | crates/cm-render/src/dispatcher.rs | NO | STRUCTURALLY_VERIFIED | rust:cm-render/dispatcher.rs |
| 0x0093534b | zipdir.cpp | CRT/container plumbing thunk | NON_USEFUL | crates/cm-render/src/scrman.rs | NO | STRUCTURALLY_VERIFIED | rust:cm-render/scrman.rs |
| 0x0093578b | zipdir.cpp | CRT/container plumbing thunk | NON_USEFUL | crates/cm-render/src/scrman.rs | NO | STRUCTURALLY_VERIFIED | rust:cm-render/scrman.rs |
| 0x00935f4b | zipdir.cpp | CRT/container plumbing thunk | NON_USEFUL | crates/cm-render/src/scrman.rs | NO | STRUCTURALLY_VERIFIED | rust:cm-render |
| 0x0093acf0 | zipdir.cpp | CRT/container plumbing thunk | NON_USEFUL | crates/cm-render/src/scrman.rs | NO | STRUCTURALLY_VERIFIED | rust:cm-render/scrman.rs |
| 0x0093b4b0 | zipdir.cpp | CRT/container plumbing thunk | NON_USEFUL | crates/cm-render/src/scrman.rs | NO | STRUCTURALLY_VERIFIED | rust:cm-render/scrman.rs |

## promotion

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x004d3460 | contract_manager.cpp | relegation per-person walk | PORTED_BEHAVIOURAL | walk_relegation_persons | YES | STRUCTURALLY_VERIFIED |  |
| 0x004d3550 | contract_manager.cpp | promotion_person_walk | PORTED_BEHAVIOURAL | walk_promotion_persons | TEST_ONLY | STRUCTURALLY_VERIFIED | rust:c13_promotion_apply.rs |
| 0x005026a0 | cup.cpp | playoff_bracket_driver | NOT_YET_PORTED | crates/cm-domain/src/eng_second_fixtures.rs | NO | UNVERIFIED | rust:eng_second_fixtures.rs |
| 0x0055cf40 | eng_prm.cpp | english_playoff_builder | PORTED_PARTIAL | build_english_playoff | TEST_ONLY | STRUCTURALLY_VERIFIED | rust:eng_second_fixtures.rs |
| 0x0055e7b0 | eng_prm.cpp | english_conference_dispatch | PORTED_PARTIAL | english_conference_dispatch | TEST_ONLY | STRUCTURALLY_VERIFIED | rust:eng_second_fixtures.rs |
| 0x0055ea00 | eng_prm.cpp | conference_fallback_promotion | PORTED_BEHAVIOURAL | conference_fallback_promotion | TEST_ONLY | STRUCTURALLY_VERIFIED | rust:eng_second_fixtures.rs |
| 0x0055ec40 | eng_prm.cpp | conference_feeder_swap | PORTED_BEHAVIOURAL | conference_feeder_swap | TEST_ONLY | STRUCTURALLY_VERIFIED | memory:english-pyramid-final-graph.md |
| 0x0055ee90 | eng_prm.cpp | english_pyramid_annual_rollover | PORTED_BEHAVIOURAL | english_pyramid_annual_rollover | TEST_ONLY | STRUCTURALLY_VERIFIED | memory:english-pyramid-final-graph.md |
| 0x00668380 | league.cpp | promotion_install | PORTED_BEHAVIOURAL | apply_promotion_install | TEST_ONLY | STRUCTURALLY_VERIFIED | rust:c13_promotion_apply.rs |
| 0x00668470 | league.cpp | relegation_install | PORTED_BEHAVIOURAL | apply_relegation_install;apply_report_to_world | TEST_ONLY | STRUCTURALLY_VERIFIED | rust:c13_promotion_apply.rs;rust:c15_1_world_apply.rs |
| 0x0066eed0 | league.cpp | promote_relegate_swap | PORTED_BEHAVIOURAL | promote_relegate_swap | TEST_ONLY | STRUCTURALLY_VERIFIED | rust:eng_second_fixtures.rs |

## records

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00444e10 | club_records.cpp | FUN_00444e10 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00445190 | club_records.cpp | FUN_00445190 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00445220 | club_records.cpp | club-record recompute engine | PORTED_BEHAVIOURAL | ClubSeasonRecords::update_with_match | YES | BEHAVIOURALLY_EXACT | reports/club_history_screen_decode.md |
| 0x00447d20 | club_records.cpp | highest/lowest league position writer | PORTED_PARTIAL | season_rows_for_club | INDIRECT | PARTIAL | reports/club_history_screen_decode.md |
| 0x00448a00 | club_records.cpp | FUN_00448a00 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00448a60 | club_records.cpp | FUN_00448a60 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00448aa0 | club_records.cpp | FUN_00448aa0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00448b20 | club_records.cpp | FUN_00448b20 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00448b50 | club_records.cpp | FUN_00448b50 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00448d80 | club_records.cpp | FUN_00448d80 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00448dd0 | club_records.cpp | FUN_00448dd0 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00448e00 | club_records.cpp | FUN_00448e00 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00448e20 | club_records.cpp | FUN_00448e20 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00448e80 | club_records.cpp | FUN_00448e80 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00448ea0 | club_records.cpp | FUN_00448ea0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00448ef0 | club_records.cpp | FUN_00448ef0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00448f40 | club_records.cpp | FUN_00448f40 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00448f90 | club_records.cpp | FUN_00448f90 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00448fa0 | club_records.cpp | FUN_00448fa0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00449040 | club_records.cpp | FUN_00449040 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x004493b0 | club_records.cpp | FUN_004493b0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00449590 | club_records.cpp | per-club apps/goals updater | PORTED_BEHAVIOURAL | World::club_history_view | YES | BEHAVIOURALLY_EXACT | memory/club-history-screen.md |
| 0x00449880 | club_records.cpp | FUN_00449880 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00449990 | club_records.cpp | FUN_00449990 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00449fb0 | club_records.cpp | FUN_00449fb0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x0044a0f0 | club_records.cpp | FUN_0044a0f0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x0044cf30 | club_records.cpp | FUN_0044cf30 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00451350 | club_records.cpp | FUN_00451350 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00451360 | club_records.cpp | FUN_00451360 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00451370 | club_records.cpp | FUN_00451370 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004513a0 | club_records.cpp | FUN_004513a0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x004513f0 | club_records.cpp | FUN_004513f0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00451440 | club_records.cpp | FUN_00451440 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00451490 | club_records.cpp | FUN_00451490 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x004514e0 | club_records.cpp | FUN_004514e0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00451530 | club_records.cpp | FUN_00451530 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00451590 | club_records.cpp | FUN_00451590 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x004515f0 | club_records.cpp | FUN_004515f0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00451aa0 | club_records.cpp | FUN_00451aa0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00451e10 | club_records.cpp | FUN_00451e10 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00451e40 | club_records.cpp | FUN_00451e40 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00451e70 | club_records.cpp | FUN_00451e70 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x004528c0 | club_records.cpp | FUN_004528c0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00452fe0 | club_records.cpp | this-season record body accessor | PORTED_BEHAVIOURAL | ClubSeasonRecords | YES | STRUCTURALLY_VERIFIED | reports/club_history_screen_decode.md |
| 0x004531a0 | club_records.cpp | all-time record body accessor | PORTED_BEHAVIOURAL | ClubSeasonRecords | YES | STRUCTURALLY_VERIFIED | reports/club_history_screen_decode.md |
| 0x00453580 | club_records.cpp | FUN_00453580 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00453670 | club_records.cpp | FUN_00453670 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x004538b0 | club_records.cpp | record-body initializer | PORTED_BEHAVIOURAL | ClubSeasonRecords::new | YES | STRUCTURALLY_VERIFIED | reports/club_history_screen_decode.md |
| 0x00454490 | club_records.cpp | FUN_00454490 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x004544b0 | club_records.cpp | FUN_004544b0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00454500 | club_records.cpp | FUN_00454500 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x00454550 | club_records.cpp | FUN_00454550 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x004545b0 | club_records.cpp | FUN_004545b0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x004a84b0 | comp_stats.cpp | per-match min-goals/max-attr tracker | PORTED_BEHAVIOURAL | update_match_min_goals_tracker | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004a90b0 | comp_stats.cpp | aggregate team match stats | PORTED_BEHAVIOURAL | aggregate_team_match_stats | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x005372c0 | db_files.cpp | continent_record_serializer | REPLACED_BY_RUST | ContinentView | YES | STRUCTURALLY_VERIFIED | rust:typed_records.rs |
| 0x00537420 | db_files.cpp | nation_record_serializer | REPLACED_BY_RUST | NationView | YES | STRUCTURALLY_VERIFIED | memory:record-layouts-decoded.md |
| 0x00537480 | db_files.cpp | city_record_serializer | REPLACED_BY_RUST | CityView | YES | STRUCTURALLY_VERIFIED | rust:typed_records.rs |
| 0x005375e0 | db_files.cpp | stadium_record_serializer | REPLACED_BY_RUST | StadiumView | YES | STRUCTURALLY_VERIFIED | rust:typed_records.rs |
| 0x005381b0 | db_files.cpp | club_record_serializer | REPLACED_BY_RUST | ClubView | YES | STRUCTURALLY_VERIFIED | memory:club-record-decoded.md |
| 0x00538300 | db_files.cpp | staff_type6_person_serializer | REPLACED_BY_RUST | PlayerView | YES | STRUCTURALLY_VERIFIED | memory:editor-is-ground-truth.md |
| 0x00539940 | db_files.cpp | staff_history_record_serializer | REPLACED_BY_RUST | StaffHistoryView | YES | STRUCTURALLY_VERIFIED | memory:club-history-screen.md |
| 0x00539bb0 | db_files.cpp | competition_record_serializer | REPLACED_BY_RUST | CompetitionView | YES | STRUCTURALLY_VERIFIED | rust:typed_records.rs |
| 0x00539e10 | db_files.cpp | comp_history_record_serializer | REPLACED_BY_RUST | ClubCompHistoryView | YES | STRUCTURALLY_VERIFIED | rust:typed_records.rs |
| 0x0053a0b0 | db_files.cpp | colour_record_serializer | REPLACED_BY_RUST | ColourView | YES | BEHAVIOURALLY_EXACT | rust:typed_records.rs |
| 0x0053a200 | db_files.cpp | official_record_serializer | REPLACED_BY_RUST | OfficialView | YES | PARTIAL | rust:typed_records.rs |
| 0x005d8c20 | hall_of_fame.cpp | FUN_005d8c20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005d8c90 | hall_of_fame.cpp | FUN_005d8c90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005d8d50 | hall_of_fame.cpp | FUN_005d8d50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005d8fe0 | hall_of_fame.cpp | FUN_005d8fe0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005d9370 | hall_of_fame.cpp | FUN_005d9370 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005d94a0 | hall_of_fame.cpp | FUN_005d94a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005d9750 | hall_of_fame.cpp | FUN_005d9750 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005d9e30 | hall_of_fame.cpp | FUN_005d9e30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005da020 | hall_of_fame.cpp | FUN_005da020 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005da080 | hall_of_fame.cpp | FUN_005da080 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005da1c0 | hall_of_fame.cpp | FUN_005da1c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005daa70 | hall_of_fame.cpp | FUN_005daa70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a8090 | player_stats.cpp | records apps/goals boot seed | PORTED_BEHAVIOURAL | World::club_history_view;World::player_history_view_for | YES | BEHAVIOURALLY_EXACT | memory/club-history-screen.md;rust:player_profile.rs |
| 0x007cc7c0 | record_utils.cpp | fixture->record stamp builder | PORTED_BEHAVIOURAL | MatchRecord::from_input | YES | STRUCTURALLY_VERIFIED | reports/club_history_screen_decode.md |
| 0x007cc990 | record_utils.cpp | record name-id store | REPLACED_BY_RUST | MatchRecord | YES | STRUCTURALLY_VERIFIED | reports/club_history_screen_decode.md |
| 0x007ccaa0 | record_utils.cpp | biggest-win compare | PORTED_EXACT | MatchRecord::beats_win | YES | BEHAVIOURALLY_EXACT | reports/club_history_screen_decode.md |
| 0x007ccb70 | record_utils.cpp | biggest-defeat compare | PORTED_EXACT | MatchRecord::beats_defeat | YES | BEHAVIOURALLY_EXACT | reports/club_history_screen_decode.md |
| 0x007ccc50 | record_utils.cpp | highest-scoring compare | PORTED_EXACT | MatchRecord::beats_scoring | YES | BEHAVIOURALLY_EXACT | reports/club_history_screen_decode.md |
| 0x007cce50 | record_utils.cpp | FUN_007cce50 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007cd010 | record_utils.cpp | FUN_007cd010 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007cd1d0 | record_utils.cpp | FUN_007cd1d0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007cd330 | record_utils.cpp | FUN_007cd330 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007cd490 | record_utils.cpp | FUN_007cd490 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007cd5e0 | record_utils.cpp | FUN_007cd5e0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007ce0e0 | record_utils.cpp | FUN_007ce0e0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x007ce250 | record_utils.cpp | FUN_007ce250 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x007ce270 | record_utils.cpp | FUN_007ce270 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x007ce2e0 | record_utils.cpp | FUN_007ce2e0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x007ce440 | record_utils.cpp | FUN_007ce440 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x007ce460 | record_utils.cpp | FUN_007ce460 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x007ce470 | record_utils.cpp | FUN_007ce470 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x007ce490 | record_utils.cpp | FUN_007ce490 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x007ce840 | record_utils.cpp | FUN_007ce840 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x007ceb90 | record_utils.cpp | FUN_007ceb90 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007cecc0 | record_utils.cpp | FUN_007cecc0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x007cf090 | record_utils.cpp | FUN_007cf090 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x007cf3a0 | record_utils.cpp | FUN_007cf3a0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |

## regen

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x0078d720 | player_regen.cpp | FUN_0078d720 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0078da90 | player_regen.cpp | FUN_0078da90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0078db30 | player_regen.cpp | FUN_0078db30 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0078dc90 | player_regen.cpp | FUN_0078dc90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0078e970 | player_regen.cpp | regen fill-club-squad | PORTED_BEHAVIOURAL | regen_fill_club_squad | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/player_regen.rs |
| 0x0078f1c0 | player_regen.cpp | FUN_0078f1c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0078f1d0 | player_regen.cpp | FUN_0078f1d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0078f200 | player_regen.cpp | regen free-agent selector | PORTED_BEHAVIOURAL | select_free_agent | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/player_regen.rs |
| 0x0078f2b0 | player_regen.cpp | FUN_0078f2b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0078f4f0 | player_regen.cpp | regen suitability score | PORTED_BEHAVIOURAL | regen_suitability_score | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/player_regen.rs |
| 0x0078f6b0 | player_regen.cpp | FUN_0078f6b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0078fd10 | player_regen.cpp | FUN_0078fd10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00790200 | player_regen.cpp | FUN_00790200 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00790220 | player_regen.cpp | FUN_00790220 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00790270 | player_regen.cpp | FUN_00790270 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007903a0 | player_regen.cpp | FUN_007903a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007904f0 | player_regen.cpp | FUN_007904f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00790600 | player_regen.cpp | FUN_00790600 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007906a0 | player_regen.cpp | FUN_007906a0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007907c0 | player_regen.cpp | FUN_007907c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00790860 | player_regen.cpp | FUN_00790860 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007909e0 | player_regen.cpp | FUN_007909e0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00790ad0 | player_regen.cpp | FUN_00790ad0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00790b20 | player_regen.cpp | FUN_00790b20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00790ec0 | player_regen.cpp | FUN_00790ec0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00790ef0 | player_regen.cpp | FUN_00790ef0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00791200 | player_regen.cpp | FUN_00791200 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00791630 | player_regen.cpp | FUN_00791630 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007919a0 | player_regen.cpp | FUN_007919a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00792690 | player_regen.cpp | FUN_00792690 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00792810 | player_regen.cpp | FUN_00792810 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00793030 | player_regen.cpp | FUN_00793030 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00793200 | player_regen.cpp | FUN_00793200 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00793690 | player_regen.cpp | FUN_00793690 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00793e10 | player_regen.cpp | FUN_00793e10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00795310 | player_regen.cpp | FUN_00795310 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00795430 | player_regen.cpp | FUN_00795430 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00795570 | player_regen.cpp | FUN_00795570 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007958e0 | player_regen.cpp | FUN_007958e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00795b10 | player_regen.cpp | FUN_00795b10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00796020 | player_regen.cpp | FUN_00796020 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00796590 | player_regen.cpp | FUN_00796590 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007965f0 | player_regen.cpp | FUN_007965f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007966a0 | player_regen.cpp | FUN_007966a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007966e0 | player_regen.cpp | FUN_007966e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00796840 | player_regen.cpp | FUN_00796840 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |

## render

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00415a80 | award_screens.cpp | FUN_00415a80 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00415ad0 | award_screens.cpp | FUN_00415ad0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00415b10 | award_screens.cpp | FUN_00415b10 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00415b70 | award_screens.cpp | FUN_00415b70 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00415bd0 | award_screens.cpp | FUN_00415bd0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00415c90 | award_screens.cpp | FUN_00415c90 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00416810 | award_screens.cpp | FUN_00416810 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00417e40 | award_screens.cpp | FUN_00417e40 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0041bf30 | background.cpp | FUN_0041bf30 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0041c410 | background.cpp | FUN_0041c410 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0041c620 | background.cpp | FUN_0041c620 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0041c770 | background.cpp | FUN_0041c770 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0041cad0 | background.cpp | FUN_0041cad0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0041caf0 | background.cpp | FUN_0041caf0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0041cc20 | background.cpp | FUN_0041cc20 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0041cc50 | background.cpp | FUN_0041cc50 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0041cf60 | background.cpp | FUN_0041cf60 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0041d0b0 | background.cpp | FUN_0041d0b0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0041d1c0 | background.cpp | FUN_0041d1c0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0041d2e0 | background.cpp | FUN_0041d2e0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0041d430 | background.cpp | FUN_0041d430 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0041d520 | background.cpp | FUN_0041d520 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0041d620 | background.cpp | FUN_0041d620 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0041d720 | background.cpp | FUN_0041d720 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0041d820 | background.cpp | FUN_0041d820 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0041d920 | background.cpp | FUN_0041d920 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0041da20 | background.cpp | FUN_0041da20 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00457050 | club_screens.cpp | FUN_00457050 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004570b0 | club_screens.cpp | FUN_004570b0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004570f0 | club_screens.cpp | FUN_004570f0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00457130 | club_screens.cpp | FUN_00457130 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00457190 | club_screens.cpp | FUN_00457190 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0045da20 | club_screens.cpp | FUN_0045da20 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0045da30 | club_screens.cpp | FUN_0045da30 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0045dab0 | club_screens.cpp | FUN_0045dab0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0045dbc0 | club_screens.cpp | FUN_0045dbc0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00461bf0 | club_screens.cpp | FUN_00461bf0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00468cd0 | club_screens.cpp | FUN_00468cd0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00468d00 | club_screens.cpp | FUN_00468d00 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00468d10 | club_screens.cpp | FUN_00468d10 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00468d20 | club_screens.cpp | FUN_00468d20 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00468d30 | club_screens.cpp | FUN_00468d30 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00468d40 | club_screens.cpp | FUN_00468d40 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0046a730 | club_screens.cpp | FUN_0046a730 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0046a770 | club_screens.cpp | FUN_0046a770 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0046b9b0 | club_screens.cpp | FUN_0046b9b0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0046c980 | club_screens.cpp | FUN_0046c980 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0046e0a0 | club_screens.cpp | FUN_0046e0a0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0046e640 | club_screens.cpp | FUN_0046e640 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0046ebe0 | club_screens.cpp | FUN_0046ebe0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0046f180 | club_screens.cpp | FUN_0046f180 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0046f710 | club_screens.cpp | FUN_0046f710 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0046fcd0 | club_screens.cpp | FUN_0046fcd0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x004707a0 | club_screens.cpp | FUN_004707a0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x004708a0 | club_screens.cpp | FUN_004708a0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x004709a0 | club_screens.cpp | FUN_004709a0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00470d60 | club_screens.cpp | FUN_00470d60 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x004729d0 | club_screens.cpp | FUN_004729d0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00472e00 | club_screens.cpp | FUN_00472e00 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00473da0 | club_screens.cpp | FUN_00473da0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00474fb0 | club_screens.cpp | FUN_00474fb0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00477590 | club_screens.cpp | FUN_00477590 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00477b60 | club_screens.cpp | FUN_00477b60 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00478240 | club_screens.cpp | FUN_00478240 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00480200 | club_screens.cpp | FUN_00480200 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00485c20 | club_screens.cpp | FUN_00485c20 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00486630 | club_screens.cpp | FUN_00486630 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00488280 | club_screens.cpp | FUN_00488280 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x004883f0 | club_screens.cpp | FUN_004883f0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00489720 | club_screens.cpp | FUN_00489720 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00489760 | club_screens.cpp | FUN_00489760 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0048c5b0 | club_screens.cpp | FUN_0048c5b0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0048c5e0 | club_screens.cpp | FUN_0048c5e0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0048ca10 | club_screens.cpp | FUN_0048ca10 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0048cd00 | club_screens.cpp | FUN_0048cd00 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0048f740 | club_screens.cpp | FUN_0048f740 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00548db0 | display.cpp | FUN_00548db0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00548e60 | display.cpp | FUN_00548e60 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00548e80 | display.cpp | FUN_00548e80 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00548eb0 | display.cpp | FUN_00548eb0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00549d30 | display.cpp | FUN_00549d30 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00549dd0 | display.cpp | FUN_00549dd0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00549e80 | display.cpp | FUN_00549e80 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00549f30 | display.cpp | FUN_00549f30 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0054a080 | display.cpp | FUN_0054a080 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0054a820 | display.cpp | FUN_0054a820 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0054ac30 | display.cpp | FUN_0054ac30 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0054adf0 | display.cpp | FUN_0054adf0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0054b3a0 | display.cpp | FUN_0054b3a0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0054baf0 | display.cpp | FUN_0054baf0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0054bdd0 | display.cpp | FUN_0054bdd0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0054bf50 | display.cpp | FUN_0054bf50 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x005516a0 | display.cpp | FUN_005516a0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005516c0 | display.cpp | FUN_005516c0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005516e0 | display.cpp | FUN_005516e0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00551720 | display.cpp | FUN_00551720 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005517c0 | display.cpp | FUN_005517c0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00551860 | display.cpp | FUN_00551860 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00551910 | display.cpp | FUN_00551910 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005519c0 | display.cpp | FUN_005519c0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00551a70 | display.cpp | FUN_00551a70 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00551b20 | display.cpp | FUN_00551b20 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00551bd0 | display.cpp | FUN_00551bd0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00551c90 | display.cpp | FUN_00551c90 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00551cc0 | display.cpp | FUN_00551cc0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00551e60 | display.cpp | FUN_00551e60 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00552040 | display.cpp | FUN_00552040 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00552130 | display.cpp | FUN_00552130 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00553070 | display.cpp | FUN_00553070 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00553110 | display.cpp | FUN_00553110 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005531e0 | display.cpp | FUN_005531e0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005533b0 | display.cpp | FUN_005533b0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x005534a0 | display.cpp | FUN_005534a0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x005534e0 | display.cpp | FUN_005534e0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x005cc310 | goldcup.cpp | graphics device/framebuffer init | UI_GDI_DOMAIN | WindowInitConfig | YES | STRUCTURALLY_VERIFIED | memory:gui-core-ported.md |
| 0x005cc4f0 | goldcup.cpp | RGB555 pixel-format install | PORTED_EXACT | PackedSurface::rgb555 | INDIRECT | BYTE_EXACT | memory:gdi-renderer-is-ground-truth.md |
| 0x005ccba0 | goldcup.cpp | present dirty rectangle | UI_GDI_DOMAIN | dispatch_dirty_blit | YES | STRUCTURALLY_VERIFIED | memory:directdraw-callsites-found.md |
| 0x005cd420 | goldcup.cpp | line drawer (H/V/Bresenham; dashed) | PORTED_EXACT | line;draw_line | TEST_ONLY | BYTE_EXACT | memory:gdi-renderer-is-ground-truth.md |
| 0x005cd840 | goldcup.cpp | rect fill/outline | PORTED_EXACT | rect;fill_rect | TEST_ONLY | BYTE_EXACT | memory:gdi-renderer-is-ground-truth.md |
| 0x005cd870 | goldcup.cpp | stipple blit | PORTED_EXACT | PackedSurface::draw_stipple | INDIRECT | BYTE_EXACT | memory:gdi-renderer-is-ground-truth.md |
| 0x005cdac0 | goldcup.cpp | save background behind popup | PORTED_EXACT | save_rect | TEST_ONLY | BYTE_EXACT | memory:gdi-renderer-is-ground-truth.md |
| 0x005cdcc0 | goldcup.cpp | restore background / pixel-buffer copy | PORTED_EXACT | restore_rect | TEST_ONLY | BYTE_EXACT | memory:gdi-renderer-is-ground-truth.md |
| 0x005cdfd0 | goldcup.cpp | darken rect (65536-entry LUT) | PORTED_EXACT | darken_rect | TEST_ONLY | BYTE_EXACT | memory:gdi-renderer-is-ground-truth.md |
| 0x005ce250 | goldcup.cpp | palette derivation (17 named + 9 packs) | PORTED_BEHAVIOURAL | derive_palette | INDIRECT | STRUCTURALLY_VERIFIED | memory:gui-core-ported.md |
| 0x005ce2d0 | goldcup.cpp | colour-scale per-channel /100 | PORTED_EXACT | colour_scale | INDIRECT | BYTE_EXACT | memory:gdi-renderer-is-ground-truth.md |
| 0x005ce4f0 | goldcup.cpp | RGB555/565 pixel packer | PORTED_EXACT | pack_rgb | TEST_ONLY | BYTE_EXACT | memory:gdi-renderer-is-ground-truth.md |
| 0x005ce890 | goldcup.cpp | fnt font loader + Latin-1 alias | PORTED_BEHAVIOURAL | load_font_bytes | YES | STRUCTURALLY_VERIFIED | memory:gui-core-ported.md |
| 0x005ced50 | goldcup.cpp | glyph renderer (4bpp nibble alpha) | PORTED_PARTIAL | draw_glyph;blit_glyph | TEST_ONLY | PARTIAL | memory:gdi-renderer-is-ground-truth.md |
| 0x005cf7b0 | goldcup.cpp | font metrics (height/measure/kerning) | PORTED_EXACT | font_height;measure_string;kerning_delta | YES | STRUCTURALLY_VERIFIED | memory:gui-core-ported.md |
| 0x005cf8e0 | goldcup.cpp | panel/bevel frame drawer | PORTED_EXACT | draw_panel;bevel_box | TEST_ONLY | BYTE_EXACT | memory:gdi-renderer-is-ground-truth.md |
| 0x005d0870 | goldcup.cpp | word-wrapped multi-line text box | PORTED_PARTIAL | draw_wrapped_text | TEST_ONLY | PARTIAL | memory:gdi-renderer-is-ground-truth.md |
| 0x005d7790 | gui_utils.cpp | FUN_005d7790 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x005d7aa0 | gui_utils.cpp | Layer-2 widget renderer hot path | PORTED_PARTIAL | crates/cm-render/src/packed_widget.rs | INDIRECT | PARTIAL | memory:layer2-widget-renderer-status.md |
| 0x005d7ec0 | gui_utils.cpp | FUN_005d7ec0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005d7fb0 | gui_utils.cpp | FUN_005d7fb0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x005d8640 | gui_utils.cpp | FUN_005d8640 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x005d86d0 | gui_utils.cpp | FUN_005d86d0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x005d8770 | guio.cpp | FUN_005d8770 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005d89a0 | guio.cpp | FUN_005d89a0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005d8ad0 | guio.cpp | FUN_005d8ad0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005d8b40 | guio.cpp | FUN_005d8b40 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x006547c0 | langlib.cpp | l10n template resolver | PORTED_BEHAVIOURAL | resolve_template | YES | STRUCTURALLY_VERIFIED | memory:gui-core-ported.md |
| 0x00654e90 | langlib.cpp | language-bank lookup (bsearch) | PORTED_BEHAVIOURAL | LangBank::lookup | YES | STRUCTURALLY_VERIFIED | memory:gui-core-ported.md |
| 0x006554c0 | langlib.cpp | inflection engine | PORTED_PARTIAL | inflect_token | YES | PARTIAL | memory:gui-core-ported.md |
| 0x006fe420 | match_screens.cpp | FUN_006fe420 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007020d0 | match_screens.cpp | FUN_007020d0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00703a90 | match_screens.cpp | FUN_00703a90 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00703e20 | match_screens.cpp | FUN_00703e20 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00703fb0 | match_screens.cpp | FUN_00703fb0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007040f0 | match_screens.cpp | FUN_007040f0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00704c10 | match_screens.cpp | FUN_00704c10 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007059c0 | match_screens.cpp | FUN_007059c0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00706a60 | match_screens.cpp | FUN_00706a60 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00707740 | match_screens.cpp | FUN_00707740 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00707f90 | match_screens.cpp | FUN_00707f90 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007088e0 | match_screens.cpp | FUN_007088e0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00709de0 | match_screens.cpp | FUN_00709de0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00709ee0 | match_screens.cpp | FUN_00709ee0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0070a4f0 | match_screens.cpp | FUN_0070a4f0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0070aed0 | match_screens.cpp | FUN_0070aed0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0070b900 | match_screens.cpp | FUN_0070b900 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0070cc40 | match_screens.cpp | FUN_0070cc40 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0070ce50 | match_screens.cpp | FUN_0070ce50 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0070d080 | match_screens.cpp | FUN_0070d080 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0070f280 | match_screens.cpp | FUN_0070f280 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0070f470 | match_screens.cpp | FUN_0070f470 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0070f730 | match_screens.cpp | FUN_0070f730 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00710710 | match_screens.cpp | FUN_00710710 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007e5bd0 | scrman.cpp | FUN_007e5bd0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007e6a70 | scrman.cpp | FUN_007e6a70 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007e6c30 | scrman.cpp | FUN_007e6c30 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007e6cd0 | scrman.cpp | FUN_007e6cd0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007e6d70 | scrman.cpp | FUN_007e6d70 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007e6da0 | scrman.cpp | FUN_007e6da0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007e73b0 | scrman.cpp | FUN_007e73b0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007e74a0 | scrman.cpp | FUN_007e74a0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007e76c0 | scrman.cpp | FUN_007e76c0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007e7bf0 | scrman.cpp | FUN_007e7bf0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007e7d20 | scrman.cpp | FUN_007e7d20 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007e7d90 | scrman.cpp | FUN_007e7d90 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007e7de0 | scrman.cpp | FUN_007e7de0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007e7e10 | scrman.cpp | FUN_007e7e10 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007e7e60 | scrman.cpp | FUN_007e7e60 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007e7fc0 | scrman.cpp | FUN_007e7fc0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007e8190 | scrman.cpp | FUN_007e8190 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007e86e0 | scrman.cpp | FUN_007e86e0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007e8e70 | scrman.cpp | FUN_007e8e70 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007e9480 | scrman.cpp | FUN_007e9480 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007e9770 | scrman.cpp | FUN_007e9770 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007ea0c0 | scrman.cpp | FUN_007ea0c0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007ea3c0 | scrman.cpp | FUN_007ea3c0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007ea7d0 | scrman.cpp | FUN_007ea7d0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007ea820 | scrman.cpp | FUN_007ea820 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007eaa70 | scrman.cpp | FUN_007eaa70 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007eaaa0 | scrman.cpp | FUN_007eaaa0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007eb0d0 | scrman.cpp | FUN_007eb0d0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007eb130 | scrman.cpp | FUN_007eb130 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007eb160 | scrman.cpp | FUN_007eb160 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007eb180 | scrman.cpp | FUN_007eb180 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007eb350 | scrman.cpp | FUN_007eb350 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007eb4b0 | scrman.cpp | FUN_007eb4b0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007eb6d0 | scrman.cpp | FUN_007eb6d0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007eb830 | scrman.cpp | FUN_007eb830 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007ebb20 | scrman.cpp | FUN_007ebb20 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007ebda0 | scrman.cpp | FUN_007ebda0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007ebdc0 | scrman.cpp | FUN_007ebdc0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007ebea0 | scrman.cpp | FUN_007ebea0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x007ebf30 | scrman.cpp | FUN_007ebf30 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0085e570 | staff_screens.cpp | FUN_0085e570 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00861700 | staff_screens.cpp | FUN_00861700 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00862d80 | staff_screens.cpp | FUN_00862d80 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x008663b0 | staff_screens.cpp | FUN_008663b0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00866eb0 | staff_screens.cpp | FUN_00866eb0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x00868290 | staff_screens.cpp | FUN_00868290 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0086f9c0 | staff_screens.cpp | FUN_0086f9c0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0086f9d0 | staff_screens.cpp | FUN_0086f9d0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0086f9e0 | staff_screens.cpp | FUN_0086f9e0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0086f9f0 | staff_screens.cpp | FUN_0086f9f0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0086fa00 | staff_screens.cpp | FUN_0086fa00 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0086fa10 | staff_screens.cpp | FUN_0086fa10 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0086fa20 | staff_screens.cpp | FUN_0086fa20 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0086fa40 | staff_screens.cpp | FUN_0086fa40 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x0086fa50 | staff_screens.cpp | FUN_0086fa50 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x008760d0 | staff_screens.cpp | FUN_008760d0 | UI_GDI_DOMAIN |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00876290 | staff_screens.cpp | FUN_00876290 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |
| 0x008766f0 | staff_screens.cpp | FUN_008766f0 | UI_GDI_DOMAIN |  | NO | UNVERIFIED |  |

## rng

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x008fc4f0 | utils.cpp | pool rand (rand_mod) | PORTED_EXACT | GameRng::rand_mod;SimRng::rand;cm_rng::MatchRng::random | YES | BYTE_EXACT | memory/c10-11-rng-byte-exact.md;memory:player-init-decode.md |
| 0x008fc5d0 | utils.cpp | rng seed / srand-equivalent | PORTED_EXACT | GameRng::new;SimRng::seed | YES | STRUCTURALLY_VERIFIED | memory/c10-11-rng-byte-exact.md |
| 0x00935a8a | zipdir.cpp | MSVC LCG srand | PORTED_EXACT | GameRng::lcg_srand | INDIRECT | STATE_EXACT | memory/c10-11-rng-byte-exact.md |
| 0x00935a94 | zipdir.cpp | MSVC LCG rand() | PORTED_EXACT | GameRng::msvc_rand;GameRng::lcg_next;SimRng::lcg | YES | BYTE_EXACT | memory/c10-11-rng-byte-exact.md |

## save-load

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x005176c0 | database.cpp | database section writer/loader hub | REPLACED_BY_RUST | save_game | YES | STRUCTURALLY_VERIFIED | memory/sav-file-format.md |
| 0x0057c1a0 | file_screens.cpp | foreground-league set writer | REPLACED_BY_RUST | RuntimeSaveGame::nation_tiers | YES | STRUCTURALLY_VERIFIED | memory/league-tier-model.md |
| 0x00814870 | setup.cpp | load_game_data loader | REPLACED_BY_RUST | load_game | YES | STRUCTURALLY_VERIFIED | memory/save-load-status.md |
| 0x00818060 | setup.cpp | save_game_data orchestrator | REPLACED_BY_RUST | save_game | YES | STRUCTURALLY_VERIFIED | memory/save-load-status.md |

## scouting

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00489790 | club_screens.cpp | scouting decode (pending) | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | memory:scouting-and-transfers-status.md |
| 0x00599d60 | fog_of_war.cpp | attribute-fog display filter | PORTED_PARTIAL | AttributeReveal;PlayerKnowledge::reveal | YES | PARTIAL | rust:scouting.rs |
| 0x0059a150 | fog_of_war.cpp | FUN_0059a150 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059a360 | fog_of_war.cpp | FUN_0059a360 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059a380 | fog_of_war.cpp | FUN_0059a380 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059a3f0 | fog_of_war.cpp | FUN_0059a3f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059a460 | fog_of_war.cpp | FUN_0059a460 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059a480 | fog_of_war.cpp | FUN_0059a480 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059a580 | fog_of_war.cpp | FUN_0059a580 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059a690 | fog_of_war.cpp | FUN_0059a690 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059a7f0 | fog_of_war.cpp | FUN_0059a7f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059a8e0 | fog_of_war.cpp | FUN_0059a8e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059a970 | fog_of_war.cpp | FUN_0059a970 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059aa60 | fog_of_war.cpp | FUN_0059aa60 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059ad60 | fog_of_war.cpp | FUN_0059ad60 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059b050 | fog_of_war.cpp | FUN_0059b050 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059b1b0 | fog_of_war.cpp | FUN_0059b1b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059b1d0 | fog_of_war.cpp | FUN_0059b1d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059b3a0 | fog_of_war.cpp | FUN_0059b3a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059bf70 | fog_of_war.cpp | FUN_0059bf70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059c030 | fog_of_war.cpp | FUN_0059c030 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059c1d0 | fog_of_war.cpp | FUN_0059c1d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059c570 | fog_of_war.cpp | FUN_0059c570 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059c630 | fog_of_war.cpp | FUN_0059c630 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059c640 | fog_of_war.cpp | FUN_0059c640 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059c6b0 | fog_of_war.cpp | FUN_0059c6b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059c6e0 | fog_of_war.cpp | FUN_0059c6e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059c750 | fog_of_war.cpp | FUN_0059c750 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059c840 | fog_of_war.cpp | FUN_0059c840 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059cad0 | fog_of_war.cpp | FUN_0059cad0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007974b0 | player_search.cpp | scout-task result consumer | PORTED_PARTIAL | ScoutBook::assignments | YES | PARTIAL | memory:scouting-and-transfers-status.md |
| 0x00799190 | player_search.cpp | scout-task builder | PORTED_PARTIAL | ScoutBook::assign | YES | STRUCTURALLY_VERIFIED | memory:scouting-and-transfers-status.md |
| 0x007de900 | scout_manager.cpp | FUN_007de900 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007dec40 | scout_manager.cpp | FUN_007dec40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007dec90 | scout_manager.cpp | FUN_007dec90 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007ded80 | scout_manager.cpp | FUN_007ded80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007dee70 | scout_manager.cpp | FUN_007dee70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007defd0 | scout_manager.cpp | FUN_007defd0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007df0c0 | scout_manager.cpp | FUN_007df0c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007df1b0 | scout_manager.cpp | scout weekly tick | PORTED_PARTIAL | ScoutBook::weekly_tick | YES | PARTIAL | memory:scouting-and-transfers-status.md |
| 0x007df410 | scout_manager.cpp | FUN_007df410 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007df630 | scout_manager.cpp | FUN_007df630 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007df790 | scout_manager.cpp | FUN_007df790 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007df920 | scout_manager.cpp | FUN_007df920 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007dfbc0 | scout_manager.cpp | FUN_007dfbc0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007dfd50 | scout_manager.cpp | FUN_007dfd50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007e01c0 | scout_manager.cpp | FUN_007e01c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007e07b0 | scout_manager.cpp | FUN_007e07b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007e08f0 | scout_manager.cpp | FUN_007e08f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007e0da0 | scout_manager.cpp | FUN_007e0da0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007e0f10 | scout_manager.cpp | FUN_007e0f10 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007e1080 | scout_manager.cpp | FUN_007e1080 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007e1190 | scout_manager.cpp | FUN_007e1190 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007e1276 | scout_manager.cpp | FUN_007e1276 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007e1330 | scout_manager.cpp | FUN_007e1330 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007e1430 | scout_manager.cpp | FUN_007e1430 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007e16c0 | scout_manager.cpp | FUN_007e16c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007e1830 | scout_manager.cpp | FUN_007e1830 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007e18c0 | scout_manager.cpp | FUN_007e18c0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007e1a30 | scout_manager.cpp | FUN_007e1a30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007e1c00 | scout_manager.cpp | FUN_007e1c00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007e1eb0 | scout_manager.cpp | FUN_007e1eb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007e42f0 | scout_manager.cpp | FUN_007e42f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007e4410 | scout_manager.cpp | FUN_007e4410 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007e4440 | scout_manager.cpp | FUN_007e4440 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007e44b0 | scout_manager.cpp | FUN_007e44b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007e4750 | scout_manager.cpp | FUN_007e4750 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007e47f0 | scout_manager.cpp | FUN_007e47f0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007e4920 | scout_manager.cpp | FUN_007e4920 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00822e30 | shortlist_manager.cpp | FUN_00822e30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00823140 | shortlist_manager.cpp | FUN_00823140 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00823560 | shortlist_manager.cpp | FUN_00823560 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00823a80 | shortlist_manager.cpp | FUN_00823a80 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00823d20 | shortlist_manager.cpp | FUN_00823d20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008246b0 | shortlist_manager.cpp | FUN_008246b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008246c0 | shortlist_manager.cpp | FUN_008246c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00824a30 | shortlist_manager.cpp | FUN_00824a30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00825050 | shortlist_manager.cpp | FUN_00825050 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00825290 | shortlist_manager.cpp | FUN_00825290 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008253e0 | shortlist_manager.cpp | FUN_008253e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00825e90 | shortlist_manager.cpp | FUN_00825e90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0082b3b0 | shortlist_manager.cpp | FUN_0082b3b0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0082b410 | shortlist_manager.cpp | FUN_0082b410 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0082b4b0 | shortlist_manager.cpp | FUN_0082b4b0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0082b650 | shortlist_manager.cpp | FUN_0082b650 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0082b720 | shortlist_manager.cpp | FUN_0082b720 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0082ba70 | shortlist_manager.cpp | FUN_0082ba70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0082be30 | shortlist_manager.cpp | FUN_0082be30 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0082bf30 | shortlist_manager.cpp | FUN_0082bf30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0082c140 | shortlist_manager.cpp | FUN_0082c140 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0082cb30 | shortlist_manager.cpp | FUN_0082cb30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0082d140 | shortlist_manager.cpp | FUN_0082d140 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0082d5b0 | shortlist_manager.cpp | FUN_0082d5b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0082d5e0 | shortlist_manager.cpp | FUN_0082d5e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0082d600 | shortlist_manager.cpp | FUN_0082d600 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0082d630 | shortlist_manager.cpp | FUN_0082d630 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0082d870 | shortlist_manager.cpp | FUN_0082d870 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0082d990 | shortlist_manager.cpp | FUN_0082d990 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00832ed0 | shortlist_manager.cpp | FUN_00832ed0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00832f50 | shortlist_manager.cpp | FUN_00832f50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008330f0 | shortlist_manager.cpp | FUN_008330f0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00833d80 | shortlist_manager.cpp | FUN_00833d80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00834050 | shortlist_manager.cpp | FUN_00834050 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008340a0 | shortlist_manager.cpp | FUN_008340a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00834320 | shortlist_manager.cpp | FUN_00834320 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00834390 | shortlist_manager.cpp | FUN_00834390 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008343d0 | shortlist_manager.cpp | FUN_008343d0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008344d0 | shortlist_manager.cpp | FUN_008344d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00834670 | shortlist_manager.cpp | FUN_00834670 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00834720 | shortlist_manager.cpp | FUN_00834720 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00834890 | shortlist_manager.cpp | FUN_00834890 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00834a50 | shortlist_manager.cpp | FUN_00834a50 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |

## screens

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x004150e0 | award_screens.cpp | International Awards screen setup | PORTED_BEHAVIOURAL | build_screen_4150e0 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x00417870 | award_screens.cpp | Nominations screen setup | PORTED_BEHAVIOURAL | build_screen_417870 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x00454620 | club_records.cpp | club dashboard setup | PORTED_BEHAVIOURAL | build_club_dashboard | INDIRECT | STRUCTURALLY_VERIFIED | memory:wired-screens-are-approximations.md |
| 0x004551c0 | club_screens.cpp | club home-screen draw | UNKNOWN | crates/cm-domain/src/lib.rs;crates/cm-domain/src/screen_club_dashboard.rs;crates/cm-render/src/view_render.rs | NO | UNVERIFIED | memory:dashboard-manager-model.md |
| 0x0046bdf0 | club_screens.cpp | club History screen setup | PORTED_BEHAVIOURAL | build_screen_46bdf0 | INDIRECT | STRUCTURALLY_VERIFIED | memory:club-history-screen.md |
| 0x00474760 | club_screens.cpp | Arrange Friendly Tour screen setup | PORTED_BEHAVIOURAL | build_screen_474760 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004751b0 | club_screens.cpp | Arrange Tour Of Nation screen setup | PORTED_BEHAVIOURAL | build_screen_4751b0 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x00476ef0 | club_screens.cpp | Invited Clubs screen setup | PORTED_BEHAVIOURAL | build_screen_476ef0 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x0047ea60 | club_screens.cpp | squad-number kit panel | UNKNOWN | crates/cm-domain/src/lib.rs;crates/cm-domain/src/screen_batch11.rs | NO | UNVERIFIED | memory:squad-number-not-at-45.md |
| 0x00488f70 | club_screens.cpp | squad position-filter (consumer of 0x005a2030) | UNKNOWN | crates/cm-domain/src/lib.rs | NO | UNVERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x00494640 | comp_screens.cpp | competition Rounds screen setup | PORTED_BEHAVIOURAL | build_screen_494640 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004a0c10 | comp_screens.cpp | competition-list screen builder | PORTED_BEHAVIOURAL | build_comp_list_screen | INDIRECT | STRUCTURALLY_VERIFIED | memory:wired-screens-are-approximations.md |
| 0x004a16a0 | comp_screens.cpp | competition command pre-dispatcher (sibling) | PORTED_PARTIAL | dispatch_comp_command_val9 | INDIRECT | PARTIAL | memory:menu-command-tree.md |
| 0x004a17f0 | comp_screens.cpp | fixture-view screen launcher | PORTED_BEHAVIOURAL | launch_fixture_view | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004a1fd0 | comp_screens.cpp | competition command pre-dispatcher | PORTED_PARTIAL | dispatch_comp_command | INDIRECT | PARTIAL | memory:menu-command-tree.md |
| 0x004a2190 | comp_screens.cpp | league-table/division comp screen setup | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch29.rs | INDIRECT | STRUCTURALLY_VERIFIED | memory:wired-screens-are-approximations.md |
| 0x004a2200 | comp_screens.cpp | FIFA World Rankings screen model | PORTED_BEHAVIOURAL | build_fifa_world_rankings_screen | INDIRECT | STRUCTURALLY_VERIFIED | memory:fifa-rankings-mechanism.md |
| 0x004a28c0 | comp_screens.cpp | UEFA-coefficients screen launcher | PORTED_BEHAVIOURAL | launch_uefa_coefs_screen | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004a3220 | comp_screens.cpp | competition loader pre-dispatcher | PORTED_BEHAVIOURAL | dispatch_comp_loader | INDIRECT | STRUCTURALLY_VERIFIED | memory:menu-command-tree.md |
| 0x004a3770 | comp_screens.cpp | competition-stages walker | PORTED_PARTIAL | build_competition_stages_walker | INDIRECT | PARTIAL | memory:screen-builders-ported.md |
| 0x004a3d20 | comp_screens.cpp | competition-list row builder | PORTED_BEHAVIOURAL | build_comp_list_row | INDIRECT | STRUCTURALLY_VERIFIED | memory:wired-screens-are-approximations.md |
| 0x004a3f10 | comp_screens.cpp | competition fixture row builder | PORTED_BEHAVIOURAL | build_comp_fixture_row | INDIRECT | STRUCTURALLY_VERIFIED | memory:wired-screens-are-approximations.md |
| 0x004a5610 | comp_screens.cpp | competition slot-probe (22-slot) | PORTED_BEHAVIOURAL | comp_slot_probe | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004a5760 | comp_screens.cpp | extract teams from fixture id | PORTED_BEHAVIOURAL | extract_fixture_teams | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004a8f10 | comp_stats.cpp | drain pending comp-news flags | PORTED_BEHAVIOURAL | drain_pending_comp_news | INDIRECT | STRUCTURALLY_VERIFIED | memory:news-generation-logic.md |
| 0x004a9000 | comp_stats.cpp | comp-history flag byte setter | PORTED_BEHAVIOURAL | set_comp_history_team_of_month | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004a92d0 | comp_stats.cpp | team stat column names | PORTED_EXACT | team_stat_column_name | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004a95d0 | comp_stats.cpp | player stat column names | PORTED_EXACT | player_stat_column_name | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004a9980 | comp_stats.cpp | player-history attribute reader | PORTED_BEHAVIOURAL | read_player_attribute_value | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004a9d10 | comp_stats.cpp | team attribute value reader | PORTED_BEHAVIOURAL | read_team_attribute_value | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004aa480 | comp_stats.cpp | player-attr formatted reader | PORTED_EXACT | format_player_attribute | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004abad0 | comp_stats.cpp | history-scope label | PORTED_EXACT | history_scope_label | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004abbe0 | comp_stats.cpp | season-record column names | PORTED_EXACT | season_record_column_name | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004abf80 | comp_stats.cpp | per-tab screen enable gate | PORTED_BEHAVIOURAL | screen_enable_gate | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004e2c70 | contract_manager.cpp | player screen setup | PORTED_BEHAVIOURAL | build_screen_4e2c70 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004e38d0 | contract_screens.cpp | Set Role At Club screen setup | PORTED_BEHAVIOURAL | build_screen_4e38d0 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004e42b0 | contract_screens.cpp | empty screen builder | PORTED_PARTIAL | build_screen_4e42b0 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004e6680 | contract_screens.cpp | empty screen builder | PORTED_PARTIAL | build_screen_4e6680 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004ebfa0 | contract_screens.cpp | Terminate Contract screen setup | PORTED_BEHAVIOURAL | build_screen_4ebfa0 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004ec590 | contract_screens.cpp | Game Credits screen setup | PORTED_BEHAVIOURAL | build_screen_4ec590 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x004fd1f0 | contract_screens.cpp | Web Sites screen setup | PORTED_BEHAVIOURAL | build_screen_4fd1f0 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x00548560 | discipline.cpp | Please-Confirm dialog setup | PORTED_BEHAVIOURAL | build_screen_548560 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x005488f0 | discipline.cpp | empty screen builder | PORTED_PARTIAL | build_screen_5488f0 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x00548b40 | discipline.cpp | ScreenManager session sub-object ctor | PORTED_BEHAVIOURAL | SessionSubObject::new | INDIRECT | STRUCTURALLY_VERIFIED | rust:cm-render/scrman.rs |
| 0x0058a740 | find_screens.cpp | Continent screen setup | PORTED_BEHAVIOURAL | build_screen_58a740 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x0058d000 | find_screens.cpp | Nation screen setup | PORTED_BEHAVIOURAL | build_screen_58d000 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x005dad10 | history.cpp | Ok dialog setup | PORTED_BEHAVIOURAL | build_screen_5dad10 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x005db600 | history.cpp | empty screen builder | PORTED_PARTIAL | build_screen_5db600 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x00697440 | manager_screens.cpp | Please-Confirm dialog setup | PORTED_BEHAVIOURAL | build_screen_697440 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x00697dc0 | manager_screens.cpp | Please-Confirm dialog setup | PORTED_BEHAVIOURAL | build_screen_697dc0 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x00698160 | manager_screens.cpp | Please-Confirm dialog setup | PORTED_BEHAVIOURAL | build_screen_698160 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x006fd7b0 | match_screens.cpp | day-part Results screen setup | PORTED_BEHAVIOURAL | build_screen_6fd7b0 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x00700f20 | match_screens.cpp | Latest Scores screen builder | UNKNOWN | crates/cm-domain/src/lib.rs;crates/cm-domain/src/menu.rs;crates/cm-domain/src/screen_batch3.rs;crates/cm-render/src/screen_wire_batch3.rs;crates/cm-ui-app/src/main.rs;crates/cm-ui-app/src/screens.rs | NO | UNVERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x00701070 | match_screens.cpp | Latest Scores screen setup | PORTED_BEHAVIOURAL | build_screen_701070 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x007013d0 | match_screens.cpp | match aggregate/result screen setup | PORTED_BEHAVIOURAL | build_screen_7013d0 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x007491e0 | media.cpp | global/menu-bar command dispatcher | PORTED_PARTIAL | crates/cm-render/src/dispatcher.rs;crates/cm-domain/src/sidebar_dispatcher.rs | INDIRECT | PARTIAL | memory:screen-builders-ported.md |
| 0x0074bf60 | media.cpp | club-context command dispatcher | PORTED_PARTIAL | crates/cm-render/src/dispatcher_club_toolbar.rs;crates/cm-domain/src/sidebar_dispatcher.rs | INDIRECT | PARTIAL | memory:ui-five-layer-architecture.md |
| 0x00763590 | network.cpp | ScreenManager network-buffer ctor | REPLACED_BY_RUST | NetworkBuffer::new | INDIRECT | STRUCTURALLY_VERIFIED | rust:cm-render/scrman.rs |
| 0x0076f2f0 | news.cpp | news home screen setup | PORTED_PARTIAL | crates/cm-render/src/screen_news.rs | INDIRECT | PARTIAL | memory:news-first-real-diff.md |
| 0x0076ffb0 | news_screens.cpp | news home callback registrar | UNKNOWN | crates/cm-domain/src/lib.rs;crates/cm-domain/src/screen_manager_batch.rs;crates/cm-domain/src/sidebar_dispatcher.rs;crates/cm-render/src/dispatcher.rs | NO | UNVERIFIED | memory:news-screen-geometry.md |
| 0x007719b0 | news_screens.cpp | Send-Message-To-All screen setup | PORTED_BEHAVIOURAL | build_screen_7719b0 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x007cdc70 | record_utils.cpp | Please-Confirm dialog setup | PORTED_BEHAVIOURAL | build_screen_7cdc70 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x007e4520 | scout_manager.cpp | ScreenManager ctor + struct layout | PORTED_BEHAVIOURAL | ScreenManager::new | INDIRECT | STRUCTURALLY_VERIFIED | rust:cm-render/scrman.rs |
| 0x007e46a0 | scout_manager.cpp | ScreenManager reset | PORTED_BEHAVIOURAL | ScreenManager::reset | INDIRECT | STRUCTURALLY_VERIFIED | rust:cm-render/scrman.rs |
| 0x007e6420 | scrman.cpp | pending-push staging | PORTED_BEHAVIOURAL | ScreenManager::stage_pending_push | INDIRECT | STRUCTURALLY_VERIFIED | rust:cm-render/scrman.rs |
| 0x007e6570 | scrman.cpp | screen-visible gate (skip pushes) | PORTED_BEHAVIOURAL | build_* None arm | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x007e7130 | scrman.cpp | widget-slot writer | REPLACED_BY_RUST | View slot fields (serde) | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x007fb050 | search_screens.cpp | search Filters screen setup | PORTED_BEHAVIOURAL | build_screen_7fb050 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x00804020 | setup.cpp | title screen setup | PORTED_BEHAVIOURAL | build_screen_804020 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x008053d0 | setup.cpp | Select Leagues screen | UNKNOWN | crates/cm-domain/src/lib.rs;crates/cm-domain/src/menu.rs;crates/cm-domain/src/screen_batch8.rs;crates/cm-domain/src/sidebar_dispatcher.rs;crates/cm-render/src/view_render.rs;crates/cm-ui-app/src/main.rs;crates/cm-ui-app/src/screens.rs | NO | UNVERIFIED | memory:start-new-game-flow.md |
| 0x00807280 | setup.cpp | Select Start Season screen | UNKNOWN | crates/cm-domain/src/lib.rs;crates/cm-render/src/screen_pre_boot.rs;crates/cm-ui-app/src/game_state.rs;crates/cm-ui-app/src/screens.rs;crates/cm-widget/src/lib.rs | NO | UNVERIFIED | memory:start-new-game-flow.md |
| 0x00808ae0 | setup.cpp | title screen setup | PORTED_BEHAVIOURAL | build_screen_808ae0 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x00810ce0 | setup.cpp | title/splash screen setup | PORTED_BEHAVIOURAL | build_screen_810ce0 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x008596b0 | staff_screens.cpp | shortlist-action screen setup | PORTED_BEHAVIOURAL | build_screen_8596b0 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x0088def0 | tactics_screens.cpp | Team Instructions screen setup | PORTED_BEHAVIOURAL | build_screen_88def0 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x00890e50 | tactics_screens.cpp | large multi-tab screen setup | PORTED_BEHAVIOURAL | build_screen_890e50 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x00893500 | tactics_screens.cpp | empty screen builder | PORTED_PARTIAL | build_screen_893500 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x008a2180 | training_schedule.cpp | club Training screen setup | PORTED_BEHAVIOURAL | build_screen_8a2180 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x008a6580 | training_screens.cpp | large multi-tab screen setup | PORTED_BEHAVIOURAL | build_screen_8a6580 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x008d6310 | transfer_offer.cpp | player screen family | UNKNOWN | crates/cm-domain/src/lib.rs;crates/cm-domain/src/screen_batch22.rs;crates/cm-domain/src/screen_batch23.rs | NO | UNVERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x008dfb10 | transfer_screens.cpp | contract-offer/move screen builder | PORTED_BEHAVIOURAL | build_contract_offer_or_move | INDIRECT | STRUCTURALLY_VERIFIED | memory:wired-screens-are-approximations.md |
| 0x008dfc20 | transfer_screens.cpp | small 3-slot screen builder | PORTED_BEHAVIOURAL | build_small_three_slot | INDIRECT | STRUCTURALLY_VERIFIED | memory:wired-screens-are-approximations.md |
| 0x008dfdf0 | transfer_screens.cpp | wage-offer/move screen builder | PORTED_BEHAVIOURAL | build_wage_offer_or_move | INDIRECT | STRUCTURALLY_VERIFIED | memory:wired-screens-are-approximations.md |
| 0x008e01d0 | transfer_screens.cpp | Transfer Bid screen setup | PORTED_BEHAVIOURAL | build_screen_8e01d0 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |
| 0x008e0710 | transfer_screens.cpp | 2-slot screen builder | PORTED_BEHAVIOURAL | build_two_slot_0710 | INDIRECT | STRUCTURALLY_VERIFIED | memory:wired-screens-are-approximations.md |
| 0x008e0b60 | transfer_screens.cpp | 2-slot screen builder | PORTED_BEHAVIOURAL | build_two_slot_0b60 | INDIRECT | STRUCTURALLY_VERIFIED | memory:wired-screens-are-approximations.md |
| 0x008e9e60 | transfer_screens.cpp | empty screen builder | PORTED_PARTIAL | build_screen_8e9e60 | INDIRECT | STRUCTURALLY_VERIFIED | memory:screen-builders-ported.md |

## search

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x007968c0 | player_search.cpp | FUN_007968c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00796990 | player_search.cpp | FUN_00796990 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00796a30 | player_search.cpp | FUN_00796a30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00796ad0 | player_search.cpp | FUN_00796ad0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00796b20 | player_search.cpp | FUN_00796b20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00796bf0 | player_search.cpp | FUN_00796bf0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00796ca0 | player_search.cpp | FUN_00796ca0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00796d10 | player_search.cpp | FUN_00796d10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007971d0 | player_search.cpp | FUN_007971d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00797320 | player_search.cpp | FUN_00797320 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007978d0 | player_search.cpp | FUN_007978d0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00797950 | player_search.cpp | FUN_00797950 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00797a70 | player_search.cpp | FUN_00797a70 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00797bc0 | player_search.cpp | FUN_00797bc0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00797dc0 | player_search.cpp | FUN_00797dc0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007983a0 | player_search.cpp | FUN_007983a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00799500 | player_search.cpp | FUN_00799500 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007995b0 | player_search.cpp | FUN_007995b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007997f0 | player_search.cpp | FUN_007997f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00799870 | player_search.cpp | FUN_00799870 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00799b30 | player_search.cpp | FUN_00799b30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0079a6b0 | player_search.cpp | FUN_0079a6b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0079b9b0 | player_search.cpp | FUN_0079b9b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0079c0a0 | player_search.cpp | FUN_0079c0a0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0079c170 | player_search.cpp | FUN_0079c170 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0079df30 | player_search.cpp | FUN_0079df30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0079e5f0 | player_search.cpp | FUN_0079e5f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0079f7e0 | player_search.cpp | FUN_0079f7e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0079f9e0 | player_search.cpp | FUN_0079f9e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0079fb90 | player_search.cpp | FUN_0079fb90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a0540 | player_search.cpp | FUN_007a0540 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a1910 | player_search.cpp | FUN_007a1910 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a4dd0 | player_search.cpp | FUN_007a4dd0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a5220 | player_search.cpp | FUN_007a5220 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a5650 | player_search.cpp | FUN_007a5650 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a5a40 | player_search.cpp | FUN_007a5a40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a5b70 | player_search.cpp | FUN_007a5b70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a5bb0 | player_search.cpp | FUN_007a5bb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a5c00 | player_search.cpp | FUN_007a5c00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a5c50 | player_search.cpp | FUN_007a5c50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a5ca0 | player_search.cpp | FUN_007a5ca0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a5d10 | player_search.cpp | FUN_007a5d10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a60b0 | player_search.cpp | FUN_007a60b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a6220 | player_search.cpp | FUN_007a6220 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a6980 | player_search.cpp | FUN_007a6980 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a69b0 | player_search.cpp | FUN_007a69b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a7f10 | player_search.cpp | FUN_007a7f10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a7f90 | player_search.cpp | FUN_007a7f90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007a8010 | player_search.cpp | FUN_007a8010 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007ec9e0 | search_eng.cpp | FUN_007ec9e0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007ecab0 | search_eng.cpp | FUN_007ecab0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007ecac0 | search_eng.cpp | FUN_007ecac0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007ecdf0 | search_eng.cpp | FUN_007ecdf0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007ed9f0 | search_eng.cpp | FUN_007ed9f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007eda90 | search_eng.cpp | FUN_007eda90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007edb10 | search_eng.cpp | FUN_007edb10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007edb60 | search_eng.cpp | FUN_007edb60 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007edcd0 | search_eng.cpp | FUN_007edcd0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007edd40 | search_eng.cpp | FUN_007edd40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007edf30 | search_eng.cpp | FUN_007edf30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007eead0 | search_filters.cpp | FUN_007eead0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007eecb0 | search_filters.cpp | FUN_007eecb0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x007eef30 | search_filters.cpp | FUN_007eef30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x007efc70 | search_filters.cpp | FUN_007efc70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |

## season-roll

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x005bfd90 | game.cpp | season_roll_dispatcher | PORTED_BEHAVIOURAL | SeasonRollScheduler::fire_for_day | YES | STRUCTURALLY_VERIFIED | memory:league-dates-and-comp-wiring.md |
| 0x005e4370 | host_country.cpp | season-roll driver companion | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |

## setup

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x008077e0 | setup.cpp | FUN_008077e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0080ba60 | setup.cpp | FUN_0080ba60 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0080fb10 | setup.cpp | FUN_0080fb10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00819a40 | setup.cpp | FUN_00819a40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00821f80 | setup.cpp | FUN_00821f80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00822080 | setup.cpp | FUN_00822080 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00822400 | setup.cpp | FUN_00822400 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008226f0 | setup.cpp | FUN_008226f0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00822960 | setup.cpp | FUN_00822960 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00822b10 | setup.cpp | FUN_00822b10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00822cd0 | setup.cpp | FUN_00822cd0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |

## squad

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00842ce0 | squad_manager.cpp | FUN_00842ce0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00842de0 | squad_manager.cpp | FUN_00842de0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00842ee0 | squad_manager.cpp | FUN_00842ee0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00843ad0 | squad_manager.cpp | FUN_00843ad0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00843c00 | squad_manager.cpp | FUN_00843c00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00843fe0 | squad_manager.cpp | FUN_00843fe0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00844060 | squad_manager.cpp | FUN_00844060 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00844140 | squad_manager.cpp | FUN_00844140 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008442b0 | squad_manager.cpp | FUN_008442b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00844790 | squad_manager.cpp | FUN_00844790 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00844910 | squad_manager.cpp | FUN_00844910 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |

## stadium

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00583fc0 | finance.cpp | stadium-expansion transaction + chairman-overrun-gate (CONFLICT) | PORTED_EXACT | apply_stadium_expansion;compute_expansion_cost;chairman_approves_overrun;stadium_meets_capacity_target | INDIRECT | STRUCTURALLY_VERIFIED | rust:c14_stadium_expansion.rs |
| 0x0058a310 | finance.cpp | stadium-expansion news broadcast (template 0x1780) | PORTED_EXACT | StadiumExpansionNews | INDIRECT | STRUCTURALLY_VERIFIED | rust:c14_stadium_expansion.rs |

## startup

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x005b6940 | friendly.cpp | init / DB-load entry | UNKNOWN | crates/cm-app/src/main.rs | NO | UNVERIFIED | rust:crates/cm-app/src/main.rs |
| 0x005b6f10 | game.cpp | startup intro/logo flow | UNKNOWN | crates/cm-app/src/main.rs;crates/cm-render/src/background.rs;crates/cm-render/src/fade.rs | NO | UNVERIFIED | rust:crates/cm-app/src/main.rs |
| 0x00672270 | league_stage.cpp | main-loop helper | UNKNOWN | crates/cm-app/src/main.rs | NO | UNVERIFIED | rust:crates/cm-app/src/main.rs |
| 0x00672770 | main.cpp | per-iteration message-pump tick shell | UNKNOWN | crates/cm-app/src/main.rs;crates/cm-domain/src/game.rs | NO | UNVERIFIED | rust:crates/cm-app/src/main.rs |
| 0x00803e00 | setup.cpp | startup setup (CM3_QSTART/-seed) | UNKNOWN | crates/cm-app/src/main.rs;crates/cm-domain/src/screen_batch19.rs | NO | UNVERIFIED | rust:crates/cm-app/src/main.rs |

## tactics

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x0059cc40 | formation.cpp | FUN_0059cc40 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0059d190 | formation.cpp | FUN_0059d190 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059d220 | formation.cpp | FUN_0059d220 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059d4c0 | formation.cpp | FUN_0059d4c0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0059d580 | formation.cpp | FUN_0059d580 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059d770 | formation.cpp | FUN_0059d770 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059d870 | formation.cpp | per-slot player-instruction setter | PORTED_PARTIAL | TacticSlot | YES | PARTIAL | memory:tactics-port-status.md |
| 0x0059df80 | formation.cpp | FUN_0059df80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059e0f0 | formation.cpp | FUN_0059e0f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059e430 | formation.cpp | FUN_0059e430 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059e720 | formation.cpp | FUN_0059e720 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059e8c0 | formation.cpp | FUN_0059e8c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059ed70 | formation.cpp | FUN_0059ed70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059ee80 | formation.cpp | FUN_0059ee80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059f1f0 | formation.cpp | FUN_0059f1f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059f580 | formation.cpp | FUN_0059f580 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0059fad0 | formation.cpp | FUN_0059fad0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0059fbf0 | formation.cpp | FUN_0059fbf0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0059fc20 | formation.cpp | FUN_0059fc20 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0059fc70 | formation.cpp | FUN_0059fc70 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0059fcb0 | formation.cpp | FUN_0059fcb0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x0059fd70 | formation.cpp | FUN_0059fd70 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x005a0020 | formation.cpp | FUN_005a0020 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005a1470 | formation.cpp | global team-settings setters | PORTED_PARTIAL | TeamSettings | YES | PARTIAL | memory:tactics-port-status.md |
| 0x005a1b20 | formation.cpp | FUN_005a1b20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005a1dc0 | formation.cpp | FUN_005a1dc0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005a2170 | formation.cpp | FUN_005a2170 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005a30d0 | formation.cpp | FUN_005a30d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x006c34c0 | match_man.cpp | team tempo+mentality bitmask | PORTED_BEHAVIOURAL | team_settings | YES | BEHAVIOURALLY_EXACT | rust:tactic_file.rs |
| 0x006c5c40 | match_man.cpp | team score (sum ratings / opp_rep x8) | PORTED_BEHAVIOURAL | team_score | YES | BEHAVIOURALLY_EXACT | memory:tactics-port-status.md |
| 0x006c8930 | match_man.cpp | position rating (player-in-role) | PORTED_PARTIAL | position_rating;ATTR_CURVE | YES | PARTIAL | memory:tactics-port-status.md |
| 0x0087ea70 | tactics.cpp | tactics-AI XI selection (faithful picker) | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | memory:tactics-port-status.md |
| 0x00880e90 | tactics.cpp | FUN_00880e90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00880f40 | tactics.cpp | FUN_00880f40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00881310 | tactics.cpp | AI/per-club formation resolver | NOT_YET_PORTED | FLAT_442_ROLES | BLOCKED | PARTIAL | memory:tactics-port-status.md |
| 0x00881480 | tactics.cpp | FUN_00881480 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00881740 | tactics.cpp | FUN_00881740 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00881b40 | tactics.cpp | FUN_00881b40 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00881c90 | tactics.cpp | FUN_00881c90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00881f00 | tactics.cpp | FUN_00881f00 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00881f70 | tactics.cpp | FUN_00881f70 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00882090 | tactics.cpp | FUN_00882090 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00882240 | tactics.cpp | FUN_00882240 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00882350 | tactics.cpp | FUN_00882350 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00882410 | tactics.cpp | FUN_00882410 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00882530 | tactics.cpp | FUN_00882530 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00882860 | tactics.cpp | FUN_00882860 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00882a70 | tactics.cpp | FUN_00882a70 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00882b90 | tactics.cpp | FUN_00882b90 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00882cf0 | tactics.cpp | FUN_00882cf0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00882e00 | tactics.cpp | FUN_00882e00 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00883180 | tactics.cpp | FUN_00883180 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00883270 | tactics.cpp | FUN_00883270 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00884280 | tactics.cpp | FUN_00884280 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00884700 | tactics_screens.cpp | TacticScreenView widget builder | NOT_YET_PORTED | crates/cm-domain/src/screen_batch21.rs;crates/cm-domain/src/tactic_dispatcher.rs | BLOCKED | PARTIAL | memory:tactics-port-status.md |
| 0x0088a720 | tactics_screens.cpp | FUN_0088a720 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0088a730 | tactics_screens.cpp | FUN_0088a730 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0088a740 | tactics_screens.cpp | FUN_0088a740 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0088a770 | tactics_screens.cpp | FUN_0088a770 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0088a790 | tactics_screens.cpp | FUN_0088a790 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0088a7d0 | tactics_screens.cpp | FUN_0088a7d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0088a7f0 | tactics_screens.cpp | FUN_0088a7f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0088a810 | tactics_screens.cpp | FUN_0088a810 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0088a850 | tactics_screens.cpp | tactics-screen event dispatcher | PORTED_PARTIAL | TacticCmd;dispatch | INDIRECT | PARTIAL | memory:tactics-port-status.md |
| 0x0088dde0 | tactics_screens.cpp | FUN_0088dde0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00890b30 | tactics_screens.cpp | FUN_00890b30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008939c0 | tactics_screens.cpp | tactic-record initialiser | NOT_YET_PORTED | crates/cm-domain/src/screen_batch21.rs;crates/cm-domain/src/tactic_dispatcher.rs | BLOCKED | PARTIAL | memory:tactics-port-status.md |
| 0x008948b0 | tactics_screens.cpp | FUN_008948b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00895450 | tactics_screens.cpp | FUN_00895450 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008955a0 | tactics_screens.cpp | FUN_008955a0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008958f0 | tactics_screens.cpp | FUN_008958f0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x00895a50 | tactics_screens.cpp | FUN_00895a50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00895b50 | tactics_screens.cpp | FUN_00895b50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00895c10 | tactics_screens.cpp | tct/pct tactic loader | PORTED_EXACT | load_tactic;parse_tactic | YES | BYTE_EXACT | memory:agevak-cross-check.md |
| 0x00896260 | tactics_screens.cpp | FUN_00896260 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008963b0 | tactics_screens.cpp | FUN_008963b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008964d0 | tactics_screens.cpp | FUN_008964d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008965e0 | tactics_screens.cpp | FUN_008965e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008966b0 | tactics_screens.cpp | FUN_008966b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00896790 | tactics_screens.cpp | FUN_00896790 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00896840 | tactics_screens.cpp | FUN_00896840 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00896b90 | tactics_screens.cpp | FUN_00896b90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00896e70 | tactics_screens.cpp | FUN_00896e70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008970e0 | tactics_screens.cpp | FUN_008970e0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008971f0 | tactics_screens.cpp | FUN_008971f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008972f0 | tactics_screens.cpp | FUN_008972f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008973f0 | tactics_screens.cpp | FUN_008973f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00897610 | tactics_screens.cpp | FUN_00897610 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00897790 | tactics_screens.cpp | FUN_00897790 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008978a0 | tactics_screens.cpp | FUN_008978a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008979a0 | tactics_screens.cpp | FUN_008979a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00897ac0 | tactics_screens.cpp | FUN_00897ac0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00897cb0 | tactics_screens.cpp | FUN_00897cb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008980b0 | tactics_screens.cpp | FUN_008980b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00898190 | tactics_screens.cpp | FUN_00898190 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00898340 | tactics_screens.cpp | FUN_00898340 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00898470 | tactics_screens.cpp | FUN_00898470 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00898740 | tactics_screens.cpp | FUN_00898740 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00898a10 | tactics_screens.cpp | FUN_00898a10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00898c40 | tactics_screens.cpp | FUN_00898c40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x00899dc0 | tactics_screens.cpp | FUN_00899dc0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089a1f0 | tactics_screens.cpp | FUN_0089a1f0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089a2b0 | tactics_screens.cpp | FUN_0089a2b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089a7d0 | tactics_screens.cpp | FUN_0089a7d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089a840 | tactics_screens.cpp | FUN_0089a840 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |

## tick

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00413980 | australia_rules.cpp | background subsystem F | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x00419c30 | awol.cpp | manager-job lifecycle C | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x00535600 | date.cpp | news-manager pacing A | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x0053fe40 | discipline.cpp | background subsystem D | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x00553aa0 | dispute.cpp | manager-job lifecycle D | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x00585ae0 | finance.cpp | media/board mood pass B | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x0058fd80 | fine.cpp | manager-job lifecycle E | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x00594950 | fix_man.cpp | year-rollover tick hook | PORTED_BEHAVIOURAL | RuntimeSaveGame::hook_year_rollover | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x00595580 | fix_man.cpp | fixture/news cleanup | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x005b7f10 | game.cpp | background subsystem A | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x005b8390 | game.cpp | event-drain predicate | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x005b85b0 | game.cpp | daily AI dispatcher (staff/transfers/AI) | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x005c0d20 | game.cpp | player-ranking summary B | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x005c0f90 | game.cpp | player-ranking summary A | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x00614e90 | index.cpp | background subsystem C | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x00674c10 | manager_manager.cpp | manager-job lifecycle A | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x00752d40 | national_teams.cpp | tie-participant notification | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | memory:sim-findings-2026-09.md |
| 0x0078dd80 | player_regen.cpp | media/scouting pass C | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x007e4940 | scrman.cpp | post-hotseat finalize | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x007ead30 | scrman.cpp | news-manager pacing B | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x00808a70 | setup.cpp | per-seat hotseat processing | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x00823210 | shortlist_manager.cpp | monthly hook (date%30) | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x00823ad0 | shortlist_manager.cpp | media/board mood pass A | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x00844940 | squad_manager.cpp | manager-job lifecycle B | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x00856d50 | staff_records.cpp | post-comp dispatch | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x0089de30 | training_manager.cpp | media/board mood pass E | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x008f2900 | uefa_seeding.cpp | background subsystem E | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |
| 0x009123a0 | weather.cpp | background subsystem B | NOT_YET_PORTED | crates/cm-domain/src/lib.rs | NO | HYPOTHESIS | rust:crates/cm-domain/src/lib.rs |

## training

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x0089d7d0 | training_manager.cpp | FUN_0089d7d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089db50 | training_manager.cpp | FUN_0089db50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089de50 | training_manager.cpp | training weekly tick | PORTED_EXACT | PlayerDevelopmentBook::weekly_tick | YES | STRUCTURALLY_VERIFIED | memory:heuristic-kill-campaign.md |
| 0x0089e930 | training_manager.cpp | FUN_0089e930 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089e950 | training_manager.cpp | FUN_0089e950 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089eab0 | training_manager.cpp | FUN_0089eab0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089eb70 | training_manager.cpp | FUN_0089eb70 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089ef80 | training_manager.cpp | FUN_0089ef80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089f350 | training_manager.cpp | apply category accumulator to type10 attrs | PORTED_EXACT | PlayerDevelopmentBook | YES | STRUCTURALLY_VERIFIED | memory:heuristic-kill-campaign.md |
| 0x0089fb80 | training_manager.cpp | FUN_0089fb80 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089fd10 | training_manager.cpp | FUN_0089fd10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089fdb0 | training_manager.cpp | FUN_0089fdb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x0089ff30 | training_manager.cpp | FUN_0089ff30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008a00a0 | training_manager.cpp | FUN_008a00a0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008a0160 | training_manager.cpp | FUN_008a0160 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008a03c0 | training_manager.cpp | FUN_008a03c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008a04b0 | training_manager.cpp | FUN_008a04b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008a0940 | training_manager.cpp | coach-quality averaging | PORTED_BEHAVIOURAL | PlayerDevelopmentBook::build | YES | BEHAVIOURALLY_EXACT | memory:heuristic-kill-campaign.md |
| 0x008a0a40 | training_manager.cpp | FUN_008a0a40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008a0bc0 | training_manager.cpp | new position/side familiarity rating | NOT_YET_PORTED |  | NO | UNVERIFIED | memory:heuristic-kill-campaign.md |
| 0x008a0ee0 | training_manager.cpp | FUN_008a0ee0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008a1120 | training_manager.cpp | FUN_008a1120 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008a1290 | training_manager.cpp | FUN_008a1290 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008a1440 | training_manager.cpp | FUN_008a1440 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008a1520 | training_manager.cpp | FUN_008a1520 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008a1580 | training_manager.cpp | FUN_008a1580 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008a15c0 | training_manager.cpp | training-intensity dial magnitudes | PORTED_EXACT | DIAL_NONE;DIAL_LIGHT;DIAL_MEDIUM;DIAL_INTENSIVE | YES | BYTE_EXACT | memory:heuristic-kill-campaign.md |
| 0x008a16c0 | training_manager.cpp | FUN_008a16c0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008a16d0 | training_manager.cpp | FUN_008a16d0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008a1770 | training_manager.cpp | FUN_008a1770 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008a1860 | training_manager.cpp | FUN_008a1860 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008a1990 | training_manager.cpp | FUN_008a1990 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008a1b40 | training_manager.cpp | FUN_008a1b40 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |

## transfer

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x008d2750 | transfer_offer.cpp | FUN_008d2750 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d2af0 | transfer_offer.cpp | FUN_008d2af0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d47d0 | transfer_offer.cpp | FUN_008d47d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d4a30 | transfer_offer.cpp | FUN_008d4a30 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008d4eb0 | transfer_offer.cpp | FUN_008d4eb0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d4ed0 | transfer_offer.cpp | FUN_008d4ed0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d4f30 | transfer_offer.cpp | FUN_008d4f30 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d5160 | transfer_offer.cpp | FUN_008d5160 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d52b0 | transfer_offer.cpp | FUN_008d52b0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d52f0 | transfer_offer.cpp | FUN_008d52f0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008d5460 | transfer_offer.cpp | FUN_008d5460 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d5500 | transfer_offer.cpp | FUN_008d5500 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d5520 | transfer_offer.cpp | FUN_008d5520 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d57c0 | transfer_offer.cpp | FUN_008d57c0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d5a20 | transfer_offer.cpp | FUN_008d5a20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d5b90 | transfer_offer.cpp | FUN_008d5b90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d5c00 | transfer_offer.cpp | FUN_008d5c00 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d5e10 | transfer_offer.cpp | FUN_008d5e10 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d5e90 | transfer_offer.cpp | FUN_008d5e90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d6040 | transfer_offer.cpp | FUN_008d6040 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008d6170 | transfer_offer.cpp | FUN_008d6170 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008fc820 | virtual_staff.cpp | FUN_008fc820 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008fca20 | virtual_staff.cpp | FUN_008fca20 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008fcbe0 | virtual_staff.cpp | FUN_008fcbe0 | NOT_YET_PORTED |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008fcd90 | virtual_staff.cpp | FUN_008fcd90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008fce20 | virtual_staff.cpp | FUN_008fce20 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008fd010 | virtual_staff.cpp | FUN_008fd010 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008fd550 | virtual_staff.cpp | FUN_008fd550 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008fd5d0 | virtual_staff.cpp | FUN_008fd5d0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008fdc90 | virtual_staff.cpp | FUN_008fdc90 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x008fded0 | virtual_staff.cpp | FUN_008fded0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |

## transfer-ai

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x008a8a90 | transfer_manager.cpp | FUN_008a8a90 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008a8e80 | transfer_manager.cpp | FUN_008a8e80 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008a9080 | transfer_manager.cpp | FUN_008a9080 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008aa510 | transfer_manager.cpp | FUN_008aa510 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008ab360 | transfer_manager.cpp | FUN_008ab360 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008aba30 | transfer_manager.cpp | FUN_008aba30 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008abdc0 | transfer_manager.cpp | FUN_008abdc0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008abeb0 | transfer_manager.cpp | FUN_008abeb0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008ac460 | transfer_manager.cpp | FUN_008ac460 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008ac6e0 | transfer_manager.cpp | FUN_008ac6e0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008aca90 | transfer_manager.cpp | FUN_008aca90 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008ad6c0 | transfer_manager.cpp | FUN_008ad6c0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008ad870 | transfer_manager.cpp | FUN_008ad870 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008adfa0 | transfer_manager.cpp | FUN_008adfa0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008ae2f0 | transfer_manager.cpp | FUN_008ae2f0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008ae710 | transfer_manager.cpp | FUN_008ae710 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008ae7e0 | transfer_manager.cpp | FUN_008ae7e0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008ae8d0 | transfer_manager.cpp | FUN_008ae8d0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008ae920 | transfer_manager.cpp | FUN_008ae920 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008aeb50 | transfer_manager.cpp | FUN_008aeb50 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008aeea0 | transfer_manager.cpp | FUN_008aeea0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008aefe0 | transfer_manager.cpp | FUN_008aefe0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008af0c0 | transfer_manager.cpp | FUN_008af0c0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008af1e0 | transfer_manager.cpp | FUN_008af1e0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008af920 | transfer_manager.cpp | FUN_008af920 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008affb0 | transfer_manager.cpp | FUN_008affb0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b0370 | transfer_manager.cpp | FUN_008b0370 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b0540 | transfer_manager.cpp | FUN_008b0540 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b0710 | transfer_manager.cpp | FUN_008b0710 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b0910 | transfer_manager.cpp | FUN_008b0910 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b0a50 | transfer_manager.cpp | FUN_008b0a50 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b0eb0 | transfer_manager.cpp | FUN_008b0eb0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b0f10 | transfer_manager.cpp | FUN_008b0f10 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b1220 | transfer_manager.cpp | FUN_008b1220 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b1540 | transfer_manager.cpp | FUN_008b1540 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b1830 | transfer_manager.cpp | FUN_008b1830 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b19b0 | transfer_manager.cpp | FUN_008b19b0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b1e70 | transfer_manager.cpp | FUN_008b1e70 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b2470 | transfer_manager.cpp | FUN_008b2470 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b2490 | transfer_manager.cpp | FUN_008b2490 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b24b0 | transfer_manager.cpp | FUN_008b24b0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008b2520 | transfer_manager.cpp | FUN_008b2520 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008b25c0 | transfer_manager.cpp | FUN_008b25c0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b26f0 | transfer_manager.cpp | FUN_008b26f0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b2860 | transfer_manager.cpp | FUN_008b2860 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b28e0 | transfer_manager.cpp | FUN_008b28e0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b2d50 | transfer_manager.cpp | FUN_008b2d50 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008b2ea0 | transfer_manager.cpp | FUN_008b2ea0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008b3020 | transfer_manager.cpp | FUN_008b3020 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b3160 | transfer_manager.cpp | FUN_008b3160 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b3280 | transfer_manager.cpp | FUN_008b3280 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008b3400 | transfer_manager.cpp | FUN_008b3400 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b3b40 | transfer_manager.cpp | FUN_008b3b40 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b3c20 | transfer_manager.cpp | FUN_008b3c20 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b3e10 | transfer_manager.cpp | FUN_008b3e10 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008b3ec0 | transfer_manager.cpp | FUN_008b3ec0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b40d0 | transfer_manager.cpp | FUN_008b40d0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b42c0 | transfer_manager.cpp | FUN_008b42c0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008b4330 | transfer_manager.cpp | FUN_008b4330 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008b4390 | transfer_manager.cpp | FUN_008b4390 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b44d0 | transfer_manager.cpp | FUN_008b44d0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b45c0 | transfer_manager.cpp | FUN_008b45c0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008b46f0 | transfer_manager.cpp | FUN_008b46f0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b49b0 | transfer_manager.cpp | FUN_008b49b0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b4e80 | transfer_manager.cpp | FUN_008b4e80 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b50f0 | transfer_manager.cpp | FUN_008b50f0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b57a0 | transfer_manager.cpp | FUN_008b57a0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b58c0 | transfer_manager.cpp | FUN_008b58c0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b6030 | transfer_manager.cpp | FUN_008b6030 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b6240 | transfer_manager.cpp | FUN_008b6240 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b63f0 | transfer_manager.cpp | FUN_008b63f0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b6780 | transfer_manager.cpp | FUN_008b6780 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b7900 | transfer_manager.cpp | FUN_008b7900 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b7b50 | transfer_manager.cpp | FUN_008b7b50 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b7ca0 | transfer_manager.cpp | FUN_008b7ca0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b7df0 | transfer_manager.cpp | FUN_008b7df0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b7f40 | transfer_manager.cpp | FUN_008b7f40 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b8150 | transfer_manager.cpp | FUN_008b8150 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b83a0 | transfer_manager.cpp | FUN_008b83a0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b8550 | transfer_manager.cpp | FUN_008b8550 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b85f0 | transfer_manager.cpp | FUN_008b85f0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b8690 | transfer_manager.cpp | FUN_008b8690 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b8730 | transfer_manager.cpp | FUN_008b8730 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b89a0 | transfer_manager.cpp | FUN_008b89a0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008b8a00 | transfer_manager.cpp | FUN_008b8a00 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b8c40 | transfer_manager.cpp | FUN_008b8c40 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b8fa0 | transfer_manager.cpp | FUN_008b8fa0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b8fd0 | transfer_manager.cpp | FUN_008b8fd0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008b9090 | transfer_manager.cpp | FUN_008b9090 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b9610 | transfer_manager.cpp | FUN_008b9610 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008b9990 | transfer_manager.cpp | FUN_008b9990 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008b9a70 | transfer_manager.cpp | FUN_008b9a70 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008b9be0 | transfer_manager.cpp | FUN_008b9be0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008ba1b0 | transfer_manager.cpp | FUN_008ba1b0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008ba2a0 | transfer_manager.cpp | FUN_008ba2a0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008babe0 | transfer_manager.cpp | FUN_008babe0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008bacf0 | transfer_manager.cpp | FUN_008bacf0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008bade0 | transfer_manager.cpp | FUN_008bade0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008baed0 | transfer_manager.cpp | FUN_008baed0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008bb140 | transfer_manager.cpp | FUN_008bb140 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008bb980 | transfer_manager.cpp | FUN_008bb980 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008bb9f0 | transfer_manager.cpp | FUN_008bb9f0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008bbed0 | transfer_manager.cpp | FUN_008bbed0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008bc4d0 | transfer_manager.cpp | FUN_008bc4d0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008bc640 | transfer_manager.cpp | FUN_008bc640 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008bc710 | transfer_manager.cpp | FUN_008bc710 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008bc840 | transfer_manager.cpp | FUN_008bc840 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008bc8e0 | transfer_manager.cpp | FUN_008bc8e0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008bccf0 | transfer_manager.cpp | FUN_008bccf0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008bd7e0 | transfer_manager.cpp | FUN_008bd7e0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008bdad0 | transfer_manager.cpp | FUN_008bdad0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008bdc10 | transfer_manager.cpp | FUN_008bdc10 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008bddf0 | transfer_manager.cpp | FUN_008bddf0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008bde30 | transfer_manager.cpp | FUN_008bde30 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008bdf00 | transfer_manager.cpp | FUN_008bdf00 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008bdf60 | transfer_manager.cpp | FUN_008bdf60 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008bdfc0 | transfer_manager.cpp | FUN_008bdfc0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008be190 | transfer_manager.cpp | FUN_008be190 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008bee00 | transfer_manager.cpp | FUN_008bee00 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008bf710 | transfer_manager.cpp | FUN_008bf710 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008bffc0 | transfer_manager.cpp | FUN_008bffc0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c00b0 | transfer_manager.cpp | FUN_008c00b0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c0540 | transfer_manager.cpp | FUN_008c0540 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c2190 | transfer_manager.cpp | FUN_008c2190 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c2440 | transfer_manager.cpp | FUN_008c2440 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c3be0 | transfer_manager.cpp | FUN_008c3be0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c3e70 | transfer_manager.cpp | FUN_008c3e70 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c46c0 | transfer_manager.cpp | FUN_008c46c0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008c4860 | transfer_manager.cpp | FUN_008c4860 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c4cc0 | transfer_manager.cpp | FUN_008c4cc0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c4fa0 | transfer_manager.cpp | FUN_008c4fa0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c5300 | transfer_manager.cpp | FUN_008c5300 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008c54d0 | transfer_manager.cpp | FUN_008c54d0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c5900 | transfer_manager.cpp | FUN_008c5900 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008c5ae0 | transfer_manager.cpp | FUN_008c5ae0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c5e80 | transfer_manager.cpp | FUN_008c5e80 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c6470 | transfer_manager.cpp | FUN_008c6470 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c75d0 | transfer_manager.cpp | FUN_008c75d0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c78e0 | transfer_manager.cpp | FUN_008c78e0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c8380 | transfer_manager.cpp | FUN_008c8380 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c8560 | transfer_manager.cpp | FUN_008c8560 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c8880 | transfer_manager.cpp | FUN_008c8880 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c8a60 | transfer_manager.cpp | FUN_008c8a60 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c8ed0 | transfer_manager.cpp | FUN_008c8ed0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008c9350 | transfer_manager.cpp | FUN_008c9350 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008ca810 | transfer_manager.cpp | FUN_008ca810 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008caa70 | transfer_manager.cpp | FUN_008caa70 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008cac50 | transfer_manager.cpp | FUN_008cac50 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008cb130 | transfer_manager.cpp | FUN_008cb130 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008cb300 | transfer_manager.cpp | FUN_008cb300 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008cb770 | transfer_manager.cpp | FUN_008cb770 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008cb960 | transfer_manager.cpp | FUN_008cb960 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008cc990 | transfer_manager.cpp | FUN_008cc990 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008ccfc0 | transfer_manager.cpp | FUN_008ccfc0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008cd580 | transfer_manager.cpp | FUN_008cd580 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008cd5b0 | transfer_manager.cpp | FUN_008cd5b0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008cd8c0 | transfer_manager.cpp | FUN_008cd8c0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008ce130 | transfer_manager.cpp | FUN_008ce130 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008ce350 | transfer_manager.cpp | FUN_008ce350 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008ce760 | transfer_manager.cpp | FUN_008ce760 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008ce9a0 | transfer_manager.cpp | FUN_008ce9a0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008ceef0 | transfer_manager.cpp | FUN_008ceef0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008cf080 | transfer_manager.cpp | FUN_008cf080 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008cf3b0 | transfer_manager.cpp | FUN_008cf3b0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008cf550 | transfer_manager.cpp | FUN_008cf550 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008cf9e0 | transfer_manager.cpp | FUN_008cf9e0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008cfcb0 | transfer_manager.cpp | FUN_008cfcb0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008d0110 | transfer_manager.cpp | FUN_008d0110 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008d02a0 | transfer_manager.cpp | FUN_008d02a0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008d0580 | transfer_manager.cpp | FUN_008d0580 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008d0760 | transfer_manager.cpp | FUN_008d0760 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008d0b60 | transfer_manager.cpp | FUN_008d0b60 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008d0f70 | transfer_manager.cpp | FUN_008d0f70 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008d1270 | transfer_manager.cpp | FUN_008d1270 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008d1440 | transfer_manager.cpp | FUN_008d1440 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008d15f0 | transfer_manager.cpp | FUN_008d15f0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008d1780 | transfer_manager.cpp | FUN_008d1780 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008d1ac0 | transfer_manager.cpp | FUN_008d1ac0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008d1ee0 | transfer_manager.cpp | FUN_008d1ee0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008d2190 | transfer_manager.cpp | FUN_008d2190 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008d2500 | transfer_manager.cpp | FUN_008d2500 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008d25b0 | transfer_manager.cpp | FUN_008d25b0 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008d2640 | transfer_manager.cpp | FUN_008d2640 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008d2660 | transfer_manager.cpp | FUN_008d2660 | OUT_OF_SCOPE |  | NO | UNVERIFIED |  |
| 0x008d26d0 | transfer_manager.cpp | FUN_008d26d0 | OUT_OF_SCOPE |  | NO | STRUCTURALLY_VERIFIED |  |

## transfer-rules

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x005d6810 | greece_rules.cpp | FUN_005d6810 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005d6960 | greece_rules.cpp | FUN_005d6960 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x005d6af0 | greece_rules.cpp | FUN_005d6af0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |

## transfers

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x004539f0 | club_records.cpp | transfer-queue preprocessing | UNKNOWN | crates/cm-app/src/main.rs;crates/cm-domain/src/club_season_records.rs;crates/cm-domain/src/lib.rs | NO | UNVERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x004d0b00 | contract_manager.cpp | played-XI morale delta | PORTED_PARTIAL | crates/cm-domain/src/lib.rs | INDIRECT | PARTIAL | rust:crates/cm-domain/src/lib.rs |
| 0x004d2710 | contract_manager.cpp | player-morale display thresholds | PORTED_EXACT | morale_label | YES | BYTE_EXACT | rust:transfer.rs |
| 0x004d79c0 | contract_manager.cpp | contract-cost readback | PORTED_PARTIAL | contract_cost_readback | INDIRECT | STRUCTURALLY_VERIFIED | memory:transfer-deep-ports-done.md |
| 0x004dfbd0 | contract_manager.cpp | squad-status 7-branch enum | PORTED_BEHAVIOURAL | SquadStatus | INDIRECT | STRUCTURALLY_VERIFIED | memory:scouting-and-transfers-status.md |
| 0x00580a90 | finance.cpp | player-rating wage-cap cascade | PORTED_PARTIAL | resolve_wage_cap | INDIRECT | STRUCTURALLY_VERIFIED | memory:transfer-deep-ports-done.md |
| 0x00594220 | finland_rules.cpp | loan recall/send-back date gate | PORTED_EXACT | compute_earliest_recall | INDIRECT | STATE_EXACT | memory:transfer-deep-ports-done.md |
| 0x00596fa0 | fix_man.cpp | contract date-window helper | UNKNOWN | crates/cm-app/src/main.rs;crates/cm-domain/src/lib.rs | NO | UNVERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x005ea590 | human_manager.cpp | related-club seniority gate | NOT_YET_PORTED | crates/cm-app/src/main.rs;crates/cm-domain/src/c13_promotion_apply.rs;crates/cm-domain/src/eng_second_fixtures.rs;crates/cm-domain/src/human_manager.rs;crates/cm-domain/src/screen_batch28.rs;crates/cm-domain/src/screen_batch9.rs;crates/cm-domain/src/transfer.rs;crates/cm-render/src/dispatcher_club_toolbar.rs | BLOCKED | UNVERIFIED | memory:transfer-deep-ports-done.md |
| 0x00672260 | league_stage.cpp | queue helper (transfer emit) | UNKNOWN | crates/cm-domain/src/lib.rs | NO | UNVERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x006ce0e0 | match_man.cpp | wage-estimate helper | PORTED_PARTIAL | predict_wage | INDIRECT | BEHAVIOURALLY_EXACT | memory:transfer-deep-ports-done.md |
| 0x0076e180 | news.cpp | transfer normal-club dispatch | UNKNOWN | crates/cm-app/src/main.rs;crates/cm-domain/src/lib.rs;crates/cm-domain/src/person_news.rs | NO | UNVERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x0076e390 | news.cpp | transfer linked +0x53 dispatch | UNKNOWN | crates/cm-app/src/main.rs;crates/cm-domain/src/lib.rs | NO | UNVERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x008286f0 | shortlist_manager.cpp | scout throttle + bucket + work-permit | PORTED_EXACT | scout_throttle;scout_bucket_range;foreign_player_permit | INDIRECT | STRUCTURALLY_VERIFIED | memory:scouting-and-transfers-status.md |
| 0x0082dab0 | shortlist_manager.cpp | mentor-loyalty bypass allowlist | PORTED_EXACT | mentor_loyalty_bypass | INDIRECT | STRUCTURALLY_VERIFIED | rust:transfer.rs |
| 0x00843590 | squad_manager.cpp | position-candidate placement tree | PORTED_EXACT | crates/cm-domain/src/transfer.rs | INDIRECT | STRUCTURALLY_VERIFIED | rust:transfer.rs |
| 0x00848da0 | staff_contracts.cpp | canonical contract/offer composer | PORTED_PARTIAL | compose_wage_offer | INDIRECT | PARTIAL | memory:scouting-and-transfers-status.md |
| 0x0084d5d0 | staff_contracts.cpp | per-role x87 weekly-wage cascade | PORTED_EXACT | wage_formula_by_role | INDIRECT | BYTE_EXACT | memory:transfer-deep-ports-done.md |

## util

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x004b5230 | comp_text.cpp | FUN_004b5230 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x004b55b0 | comp_text.cpp | FUN_004b55b0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x004b5f70 | comp_text.cpp | FUN_004b5f70 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x008fafd0 | utils.cpp | FUN_008fafd0 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008fb300 | utils.cpp | FUN_008fb300 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008fb580 | utils.cpp | FUN_008fb580 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008fb660 | utils.cpp | FUN_008fb660 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008fb6c0 | utils.cpp | FUN_008fb6c0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x008fb6f0 | utils.cpp | FUN_008fb6f0 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008fb7f0 | utils.cpp | FUN_008fb7f0 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x008fb810 | utils.cpp | FUN_008fb810 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x008fb830 | utils.cpp | FUN_008fb830 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x008fba40 | utils.cpp | thunk_FUN_009362a5 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008fba50 | utils.cpp | FUN_008fba50 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x008fc050 | utils.cpp | FUN_008fc050 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008fc0d0 | utils.cpp | FUN_008fc0d0 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008fc180 | utils.cpp | FUN_008fc180 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008fc450 | utils.cpp | FUN_008fc450 | NON_USEFUL |  | NO | STRUCTURALLY_VERIFIED |  |
| 0x008fc670 | utils.cpp | FUN_008fc670 | NON_USEFUL |  | NO | UNVERIFIED |  |
| 0x008fc810 | utils.cpp | FUN_008fc810 | NON_USEFUL |  | NO | UNVERIFIED |  |

## view-model

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00524850 | database.cpp | club_job_role_name_switch | PORTED_EXACT | role_for_job | YES | BEHAVIOURALLY_EXACT | rust:general_info.rs |
| 0x005289a0 | database.cpp | multi_position_name_formatter | PORTED_BEHAVIOURAL | position_full_name | YES | BEHAVIOURALLY_EXACT | memory:no-inferring-display-codes.md |
| 0x0052c3f0 | database.cpp | attribute-name function | UNKNOWN | crates/cm-domain/src/lib.rs;crates/cm-render/src/screen_club_squad_faithful.rs | NO | UNVERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x0052d090 | database.cpp | type10_attribute_offset_dispatch | PORTED_BEHAVIOURAL | PROFILE_ATTR_OFFSETS | YES | STRUCTURALLY_VERIFIED | memory:type10-real-attribute-offsets.md |
| 0x00546a40 | discipline.cpp | ban_scope_adjective | PORTED_BEHAVIOURAL | World::competition_scope_adjective | YES | STRUCTURALLY_VERIFIED | rust:player_profile.rs |
| 0x00562580 | england_awards.cpp | english_award_pool_resolver | PORTED_PARTIAL | World::resolve_award_text | YES | PARTIAL | memory:deferred-awards-engine.md |
| 0x005a2030 | formation.cpp | position eligibility-bits decoder | PORTED_EXACT | PlayerView::position_eligibility_bits | YES | STRUCTURALLY_VERIFIED | rust:crates/cm-domain/src/lib.rs |
| 0x00850fd0 | staff_contracts.cpp | release_clause_short_code_formatter | PORTED_EXACT | ReleaseClauses::short_code | YES | BEHAVIOURALLY_EXACT | memory:contract-clauses-generated-at-boot.md |

## widgets

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x00403240 | area.cpp | z-order bubble-insert widget | PORTED_EXACT | insert_widget_z_order | YES | STRUCTURALLY_VERIFIED | memory:gui-core-ported.md |
| 0x00403390 | area.cpp | area_rebuild_layout_tables (layout engine) | PORTED_EXACT | area_rebuild_layout_tables | YES | STRUCTURALLY_VERIFIED | memory:ui-five-layer-architecture.md |
| 0x00403a20 | area.cpp | frame-metric lookup (3057B stride) | PORTED_PARTIAL | frame_lookup;FrameMetrics | INDIRECT | STRUCTURALLY_VERIFIED | memory:layer2-widget-renderer-status.md |
| 0x00549580 | display.cpp | widget spawner (18-arg registrar) | PORTED_EXACT | spawn_widget | YES | STRUCTURALLY_VERIFIED | memory:gui-core-ported.md |
| 0x00549790 | display.cpp | area spawner | PORTED_EXACT | spawn_area | YES | STRUCTURALLY_VERIFIED | memory:gui-core-ported.md |
| 0x005d1b10 | goldcup.cpp | cursor state (Default/Hand/Busy) | PORTED_BEHAVIOURAL | CursorState | YES | STRUCTURALLY_VERIFIED | memory:gui-core-ported.md |
| 0x005d1c30 | goldcup.cpp | message-box layout | PORTED_BEHAVIOURAL | compute_msgbox_layout | YES | STRUCTURALLY_VERIFIED | memory:gui-core-ported.md |
| 0x005d75b0 | gui_utils.cpp | nav-bar Back/Next builder | PORTED_PARTIAL | crates/cm-render/src/dispatcher.rs | INDIRECT | PARTIAL | memory:news-screen-geometry.md |
| 0x005d7bd0 | gui_utils.cpp | per-widget renderer (THE draw layer) | PORTED_BEHAVIOURAL | render_widget | INDIRECT | STRUCTURALLY_VERIFIED | memory:widgets-not-screens.md |
| 0x00745540 | media.cpp | sidebar msg dispatch + in-game menu bar | PORTED_BEHAVIOURAL | SidebarMsg;MenuBar::in_game | YES | STRUCTURALLY_VERIFIED | memory:menu-command-tree.md |
| 0x007eaac0 | scrman.cpp | scrollbar/scroll-region manager | PORTED_BEHAVIOURAL | crates/cm-render/src/scrollbar.rs | INDIRECT | PARTIAL | rust:cm-render |

## (unclassified)

| DD VA | source | semantic | status | Rust | reach | conf | evidence |
|---|---|---|---|---|---|---|---|
| 0x0005e9b0 |  |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x004279c0 | bra_champ_cup.cpp | FUN_004279c0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00441f00 | cash.cpp | FUN_00441f00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00449e70 | club_records.cpp | FUN_00449e70 | PORTED_BEHAVIOURAL | crates/cm-domain/src/club_season_records.rs | YES | UNVERIFIED |  |
| 0x0046a670 | club_screens.cpp | FUN_0046a670 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch28.rs | YES | UNVERIFIED |  |
| 0x00487480 | club_screens.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x004878e0 | club_screens.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher_club_toolbar.rs | YES | UNVERIFIED |  |
| 0x004938d0 | comp.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/c15_1_world_apply.rs;crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x0049a5b0 | comp_screens.cpp | FUN_0049a5b0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0049eb30 | comp_screens.cpp | FUN_0049eb30 | PORTED_BEHAVIOURAL | crates/cm-domain/src/typed_records.rs | YES | UNVERIFIED |  |
| 0x004a89d0 | comp_stats.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x004a98d0 | comp_stats.cpp | FUN_004a98d0 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch30.rs | YES | UNVERIFIED |  |
| 0x004b6230 | comp_util.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x004b6e20 | comp_util.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x004c0cf0 | comp_util.cpp | FUN_004c0cf0 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004c5c50 | comp_util.cpp | FUN_004c5c50 | NOT_YET_PORTED |  | NO | UNVERIFIED |  |
| 0x004c9320 | conmebol_liber.cpp | FUN_004c9320 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004caa90 | conmebol_liber.cpp | FUN_004caa90 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004d35a0 | contract_manager.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x004d3700 | contract_manager.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/c13_promotion_apply.rs;crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x004e8ba0 | contract_screens.cpp | FUN_004e8ba0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x004ebf60 | contract_screens.cpp | FUN_004ebf60 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch14.rs | YES | UNVERIFIED |  |
| 0x0050cc90 | cup_stage.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x00524f20 | database.cpp | FUN_00524f20 | PORTED_BEHAVIOURAL | crates/cm-domain/src/screen_batch28.rs;crates/cm-domain/src/transfer.rs | YES | UNVERIFIED |  |
| 0x00533d80 | date.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x005349f0 | date.cpp | FUN_005349f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00536df0 | date.cpp | FUN_00536df0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00536f90 | date.cpp | FUN_00536f90 | PORTED_BEHAVIOURAL | crates/cm-domain/src/next_match.rs | YES | UNVERIFIED |  |
| 0x00548d50 | discipline.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x00548de0 | display.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed_widget.rs;crates/cm-render/src/packed_widget_globals.rs | YES | UNVERIFIED |  |
| 0x0054be80 | display.cpp | FUN_0054be80 | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x005588f0 | eng_conf.cpp | FUN_005588f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0055ceb0 | eng_first.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x0055e7c0 | eng_prm.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x0055e9b0 | eng_prm.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x0055ec00 | eng_prm.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x0055ee40 | eng_prm.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x0055f080 | eng_second.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs;crates/cm-domain/src/year_end_statuses.rs | YES | UNVERIFIED |  |
| 0x0055f540 | eng_second.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/exe_date.rs | YES | UNVERIFIED |  |
| 0x005603d0 | eng_second.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x00560520 | eng_second.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/season_roll_scheduler.rs | YES | UNVERIFIED |  |
| 0x005605c0 | eng_second.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/lib.rs;crates/cm-domain/src/season_roll_scheduler.rs | YES | UNVERIFIED |  |
| 0x00560780 | eng_second.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x00560810 | eng_second.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x005622a0 | eng_third.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x00562330 | eng_third.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x00584150 | finance.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x00585060 | finance.cpp | FUN_00585060 | PORTED_BEHAVIOURAL | crates/cm-domain/src/finance.rs | YES | UNVERIFIED |  |
| 0x0058af50 | find_screens.cpp | FUN_0058af50 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00598d20 | fix_man.cpp | FUN_00598d20 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs;crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x0059b550 | fog_of_war.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed_glyph.rs;crates/cm-render/src/packed_widget_globals.rs | YES | UNVERIFIED |  |
| 0x005cd330 | goldcup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed.rs;crates/cm-render/src/packed_panel.rs;crates/cm-render/src/packed_text.rs;crates/cm-render/tests/verify_clip_against_exe.rs | YES | UNVERIFIED |  |
| 0x005cd3e0 | goldcup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed.rs;crates/cm-render/src/packed_capture.rs;crates/cm-render/src/packed_panel.rs;crates/cm-render/src/packed_text.rs;crates/cm-render/tests/verify_line_against_exe.rs | YES | UNVERIFIED |  |
| 0x005cd730 | goldcup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed.rs;crates/cm-render/src/packed_capture.rs;crates/cm-render/src/packed_panel.rs;crates/cm-render/tests/verify_rect_against_exe.rs | YES | UNVERIFIED |  |
| 0x005cd930 | goldcup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed.rs;crates/cm-render/src/packed_widget.rs;crates/cm-render/src/scrman.rs;crates/cm-render/tests/verify_save_restore_against_exe.rs | YES | UNVERIFIED |  |
| 0x005cda90 | goldcup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed.rs;crates/cm-render/src/packed_capture.rs;crates/cm-render/src/packed_widget.rs;crates/cm-render/tests/verify_save_restore_against_exe.rs | YES | UNVERIFIED |  |
| 0x005cdb50 | goldcup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed_icon_loader.rs;crates/cm-render/src/packed_widget.rs;crates/cm-render/src/packed_widget_globals.rs | YES | UNVERIFIED |  |
| 0x005cdd30 | goldcup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed_widget.rs | YES | UNVERIFIED |  |
| 0x005cdd60 | goldcup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed.rs;crates/cm-render/src/packed_capture.rs;crates/cm-render/src/packed_panel.rs;crates/cm-render/tests/verify_darken_against_exe.rs | YES | UNVERIFIED |  |
| 0x005ce240 | goldcup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed.rs;crates/cm-render/src/packed_panel.rs;crates/cm-render/src/scrman.rs;crates/cm-render/tests/verify_pack_rgb_against_exe.rs | YES | UNVERIFIED |  |
| 0x005ce3e0 | goldcup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/tests/news_pixel_diff_against_exe.rs | YES | UNVERIFIED |  |
| 0x005ce430 | goldcup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x005ceaa0 | goldcup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed_capture.rs;crates/cm-render/src/packed_glyph.rs;crates/cm-render/src/packed_text.rs;crates/cm-render/src/packed_widget_globals.rs;crates/cm-render/tests/verify_glyph_against_exe.rs | YES | UNVERIFIED |  |
| 0x005cf2a0 | goldcup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed_text.rs | YES | UNVERIFIED |  |
| 0x005cf4d0 | goldcup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed_glyph.rs | YES | UNVERIFIED |  |
| 0x005cf570 | goldcup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed_capture.rs;crates/cm-render/src/packed_panel.rs;crates/cm-render/src/packed_widget.rs;crates/cm-render/src/widget_pool.rs;crates/cm-render/tests/verify_panel_against_exe.rs | YES | UNVERIFIED |  |
| 0x005d03a0 | goldcup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed_capture.rs;crates/cm-render/src/packed_text.rs;crates/cm-render/src/packed_widget.rs;crates/cm-render/src/packed_widget_globals.rs;crates/cm-render/tests/verify_wrapped_text_against_exe.rs | YES | UNVERIFIED |  |
| 0x005d0ce0 | goldcup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed_panel.rs | YES | UNVERIFIED |  |
| 0x005d0ec0 | goldcup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed_panel.rs | YES | UNVERIFIED |  |
| 0x005d70a0 | gui_utils.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/screen_news.rs | YES | UNVERIFIED |  |
| 0x005d76c0 | gui_utils.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/screen_nav_back_next.rs;crates/cm-render/src/view_render.rs;crates/cm-render/src/widget_pool.rs | YES | UNVERIFIED |  |
| 0x005d7710 | gui_utils.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/manager_creation.rs | YES | UNVERIFIED |  |
| 0x005d8260 | gui_utils.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed_widget.rs;crates/cm-render/src/packed_widget_globals.rs | YES | UNVERIFIED |  |
| 0x005d8410 | gui_utils.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed_widget.rs;crates/cm-render/src/packed_widget_globals.rs | YES | UNVERIFIED |  |
| 0x00621e00 | ire_munster_cup.cpp | FUN_00621e00 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00627f80 | ita_c_cup.cpp | FUN_00627f80 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00654380 | langlib.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/screen_news.rs;crates/cm-render/src/screen_wire_batch3.rs | YES | UNVERIFIED |  |
| 0x00666500 | langlib.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/year_end_statuses.rs | YES | UNVERIFIED |  |
| 0x00667560 | langlib.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x00667660 | langlib.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x00667f40 | league.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x00668450 | league.cpp |  | UNKNOWN | crates/cm-domain/src/eng_second_fixtures.rs;crates/cm-domain/src/exe_date.rs;crates/cm-domain/src/lib.rs | INDIRECT | UNVERIFIED |  |
| 0x00669340 | league.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x0066b900 | league.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs;crates/cm-domain/src/game_rng.rs | YES | UNVERIFIED |  |
| 0x0066c800 | league.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x0066ea90 | league.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x0066ee40 | league.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs;crates/cm-domain/src/english_traditional.rs | YES | UNVERIFIED |  |
| 0x0066ef70 | league.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x0066efd0 | league.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x0066f890 | league.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x00671e40 | league_stage.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed_text.rs | YES | UNVERIFIED |  |
| 0x00671ef0 | league_stage.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed_text.rs | YES | UNVERIFIED |  |
| 0x00671fd0 | league_stage.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed_text.rs | YES | UNVERIFIED |  |
| 0x00672e10 | main.cpp | FUN_00672e10 | PORTED_BEHAVIOURAL | crates/cm-render/src/dispatcher.rs | YES | UNVERIFIED |  |
| 0x006e7a60 | match_pl.cpp | FUN_006e7a60 | PORTED_BEHAVIOURAL | crates/cm-app/src/main.rs | YES | UNVERIFIED |  |
| 0x006f1a50 | match_pl.cpp | FUN_006f1a50 | PORTED_BEHAVIOURAL | crates/cm-domain/src/match_engine_exe.rs | YES | UNVERIFIED |  |
| 0x00762ac0 | national_teams_screens.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x007631d0 | network.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x0076a750 | news.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x0076bcee | news.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/news_action_classify.rs | YES | UNVERIFIED |  |
| 0x0076ed20 | news.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/screen_news.rs | YES | UNVERIFIED |  |
| 0x0076fdb0 | news.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/screen_news.rs | YES | UNVERIFIED |  |
| 0x00770f40 | news_screens.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/screen_news.rs | YES | UNVERIFIED |  |
| 0x00784e70 | officials_manager.cpp |  | PORTED_BEHAVIOURAL | crates/cm-domain/src/eng_second_fixtures.rs | YES | UNVERIFIED |  |
| 0x007e3f20 | scout_manager.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x007e4340 | scout_manager.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x007fadf0 | search_screens.cpp | FUN_007fadf0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x008070a3 | setup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-ui-app/src/main.rs | YES | UNVERIFIED |  |
| 0x0083ece0 | spa_second.cpp | FUN_0083ece0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0087ce60 | swe_second.cpp | FUN_0087ce60 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0089a970 | tcpip.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x0089aeb0 | tcpip.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x008fa820 | usa_open_cup.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/scrman.rs | YES | UNVERIFIED |  |
| 0x00903ed0 | wc_asia_league.cpp | FUN_00903ed0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00914ff0 | wel_first.cpp | FUN_00914ff0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x0091a3d0 | world_club_champ.cpp | FUN_0091a3d0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x009203f0 | year_award.cpp | FUN_009203f0 | UNKNOWN |  | INDIRECT | UNVERIFIED |  |
| 0x00933579 | zipdir.cpp |  | PORTED_BEHAVIOURAL | crates/cm-render/src/packed_sprintf.rs;crates/cm-render/src/packed_widget.rs | YES | UNVERIFIED |  |

## Superseded interpretations

| DD VA | previous | why superseded | current |
|---|---|---|---|
| 0x004d5a20 | foul-interaction / match-event leaf hypothesis | later decode identified it as contract-manager logic, not match |  |
| 0x008fcbe0 | player_regen ghost-id cleanup | decompile source is virtual_staff.cpp squad DE-registration; player_regen.rs actually cites FUN_008FC4F0 (the RNG). The two addresses were conflated. |  |
