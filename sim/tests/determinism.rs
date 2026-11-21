//! Determinism contract (DESIGN §1, D-006): same seed + config + dataset → identical state,
//! and a snapshot restored mid-run continues to the identical state.
use std::sync::Arc;
use tank_sim::climate::Dataset;
use tank_sim::config::Config;
use tank_sim::sim::{Sim, ThoughtMode};

fn dataset() -> Arc<Dataset> {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/../data/climate/chicago_daily.csv");
    Arc::new(Dataset::load(p).expect("climate dataset"))
}

fn small_cfg() -> Config {
    let mut c = Config::default();
    c.people.initial = 48;
    c.people.bands = 2;
    c
}

fn run(seed: u64, cycles: u64) -> Sim {
    let mut s = Sim::new(seed, small_cfg(), dataset());
    s.thought_mode = ThoughtMode::None;
    s.run_cycles(cycles);
    s
}

#[test]
fn same_seed_same_state() {
    assert_eq!(run(5, 2).state_hash(), run(5, 2).state_hash());
}

#[test]
fn different_seed_different_state() {
    assert_ne!(run(5, 1).state_hash(), run(6, 1).state_hash());
}

#[test]
fn snapshot_restore_continues_identically() {
    let full = run(9, 2);
    let half = run(9, 1);
    let mut restored = Sim::restore(&half.snapshot(), dataset());
    restored.run_cycles(2);
    assert_eq!(full.state_hash(), restored.state_hash());
}

#[test]
fn climate_dataset_is_the_bundled_one() {
    let ds = dataset();
    assert_eq!(ds.first_year, 1929);
    assert_eq!(ds.last_year(), 2025);
    // the coldest years in the record are the late-1970s blizzard winters
    let mean = |y: u32| ds.years[(y - 1929) as usize].iter().map(|d| (d.tmax + d.tmin) * 0.5).sum::<f32>() / 365.0;
    assert!(mean(1979) < mean(2012));
}
