//! Load a CM0102 install into the owned Rust world model and export JSON.
//! Usage: cargo run -p cm-domain --example export_world -- D:/cm0102 [output.json]

fn main() {
    let root = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "D:/cm0102".into());
    let output = std::env::args().nth(2);

    let world = cm_domain::World::load_from_install(std::path::Path::new(&root))
        .expect("load world from install");

    if let Some(path) = output {
        world
            .write_json_file(std::path::Path::new(&path))
            .expect("write json");
        println!("exported world snapshot to {}", path);
    } else {
        println!("{}", world.to_pretty_json().expect("serialize world"));
    }
}
