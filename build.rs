use std::process::exit;

#[path ="src/lib.rs"]
mod lib;

fn main() {
    use build_script::*;

    cargo_rerun_if_changed("gen");
    cargo_rerun_if_changed("src");
    cargo_rerun_if_changed("clusters");
    cargo_rerun_if_changed("Cargo.toml");
    cargo_rerun_if_changed("Cargo.lock");

    lib::build();
}
