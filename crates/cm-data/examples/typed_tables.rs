//! Load the first typed inferred tables from a CM0102 `Data` directory.
//! Usage: cargo run -p cm-data --example typed_tables -- D:/cm0102/Data

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "D:/cm0102/Data".into());
    let data_dir = std::path::Path::new(&path);

    let cities = cm_data::load_city_table(&data_dir.join("city.dat")).expect("load city.dat");
    let officials =
        cm_data::load_officials_table(&data_dir.join("officials.dat")).expect("load officials.dat");
    let first_names =
        cm_data::load_name_table(&data_dir.join("first_names.dat")).expect("load first_names.dat");
    let second_names = cm_data::load_name_table(&data_dir.join("second_names.dat"))
        .expect("load second_names.dat");
    let common_names = cm_data::load_name_table(&data_dir.join("common_names.dat"))
        .expect("load common_names.dat");
    let references =
        cm_data::ReferenceData::load_from_data_dir(data_dir).expect("load reference data");
    let stadiums =
        cm_data::load_stadium_table(&data_dir.join("stadium.dat")).expect("load stadium.dat");
    let staff_comps = cm_data::load_staff_comp_table(&data_dir.join("staff_comp.dat"))
        .expect("load staff_comp.dat");
    let club_comps =
        cm_data::load_club_comp_table(&data_dir.join("club_comp.dat")).expect("load club_comp.dat");
    let nation_comps = cm_data::load_nation_comp_table(&data_dir.join("nation_comp.dat"))
        .expect("load nation_comp.dat");

    println!("{}:", path);
    println!(
        "cities       size {} count {}",
        cm_data::CityTable::LAYOUT.size,
        cities.count()
    );
    println!(
        "officials    size {} count {}",
        cm_data::OfficialsTable::LAYOUT.size,
        officials.count()
    );
    println!(
        "first names  size {} count {}",
        cm_data::NameTable::LAYOUT.size,
        first_names.count()
    );
    println!(
        "second names size {} count {}",
        cm_data::NameTable::LAYOUT.size,
        second_names.count()
    );
    println!(
        "common names size {} count {}",
        cm_data::NameTable::LAYOUT.size,
        common_names.count()
    );
    println!(
        "stadiums     size {} count {}",
        cm_data::StadiumTable::LAYOUT.size,
        stadiums.count()
    );
    println!(
        "staff comps  size {} count {}",
        cm_data::StaffCompTable::LAYOUT.size,
        staff_comps.count()
    );
    println!(
        "club comps   size {} count {}",
        cm_data::ClubCompTable::LAYOUT.size,
        club_comps.count()
    );
    println!(
        "nation comps size {} count {}",
        cm_data::NationCompTable::LAYOUT.size,
        nation_comps.count()
    );

    if let Some(city) = cities.record(0) {
        println!(
            "sample city     id {} name {} tail_u16[0] {:?} tail_u32[0] {:?}",
            city.id(),
            city.name(),
            city.tail_u16(0),
            city.tail_u32(0)
        );
    }
    if let Some(official) = officials.record(0) {
        println!(
            "sample official id {} u32[1] {:?} u16[6] {:?} tail-byte {}",
            official.id(),
            official.u32_slot(1),
            official.u16_slot(6),
            official.trailing_byte()
        );
    }
    if let Some(name) = first_names.record(1) {
        println!(
            "sample first    text {} footer-bytes {}",
            name.text(),
            name.unknown_footer().len()
        );
    }
    if let Some(name) = second_names.record(0) {
        println!("sample second   text {}", name.text());
    }
    if let Some(name) = common_names.record(1) {
        println!("sample common   text {}", name.text());
    }
    if let Some(stadium) = stadiums.record(0) {
        println!(
            "sample stadium  id {} name {}",
            stadium.id(),
            stadium.name()
        );
    }
    if let Some(comp) = staff_comps.record(0) {
        println!(
            "sample staffcmp id {} long {} short {}",
            comp.id(),
            comp.long_name(),
            comp.short_name()
        );
    }
    if let Some(comp) = club_comps.record(0) {
        println!(
            "sample clubcmp  id {} long {} short {}",
            comp.id(),
            comp.long_name(),
            comp.short_name()
        );
    }
    if let Some(comp) = nation_comps.record(0) {
        println!(
            "sample natcmp   id {} long {} short {}",
            comp.id(),
            comp.long_name(),
            comp.short_name()
        );
    }
    println!(
        "decoded refs    cities {} officials {} first {} second {} common {} stadiums {} staffcmp {} clubcmp {} nationcmp {} staffhist {} staffcmphist {} clubcmphist {} nationcmphist {}",
        references.cities.len(),
        references.officials.len(),
        references.first_names.len(),
        references.second_names.len(),
        references.common_names.len(),
        references.stadiums.len(),
        references.staff_competitions.len(),
        references.club_competitions.len(),
        references.nation_competitions.len(),
        references.staff_history.len(),
        references.staff_comp_history.len(),
        references.club_comp_history.len(),
        references.nation_comp_history.len()
    );
}
