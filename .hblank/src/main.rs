pub use hblank::gpui;

#[path = "../../catalog/controls.rs"]
mod catalog_controls;

mod fixtures {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/generated/fixtures.rs"
    ));
}

fn main() {
    hblank::run_harness();
}
