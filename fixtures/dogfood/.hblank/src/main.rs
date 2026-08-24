mod examples {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/examples.rs"));
}

fn main() {
    hblank::run_harness();
}
