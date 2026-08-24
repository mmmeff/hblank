pub use hblank::gpui;

mod fixtures {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/fixtures.rs"));
}

fn main() {
    hblank::run_harness();
}

