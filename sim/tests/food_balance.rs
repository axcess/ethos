//! D6 food-balance safeguard: people can only gather food that existed at the start or
//! regrew since. Any action that creates food from nothing (like the old shellfish-bed bug)
//! makes this fail.
use std::sync::Arc;
use tank_sim::climate::Dataset;
use tank_sim::config::Config;
use tank_sim::sim::{Sim, ThoughtMode};

fn dataset() -> Arc<Dataset> {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/../data/climate/chicago_daily.csv");
    Arc::new(Dataset::load(p).expect("climate dataset"))
}

/// Gathered food counts shellfish at 1.5× (it is more nourishing), so stock is valued the same way.
fn stock(s: &Sim) -> f64 {
    s.world.cells.iter().map(|c| c.algae as f64 + 1.5 * c.shell as f64).sum()
}

fn check(seed: u64, cycles: u64) {
    let mut cfg = Config::default();
    cfg.people.initial = 60;
    cfg.people.bands = 2;
    let mut s = Sim::new(seed, cfg, dataset());
    s.thought_mode = ThoughtMode::None;
    // everyone knows everything, so every food-touching action (gardens, beds, nets, jars…) runs
    for i in 0..s.people.len() {
        for node in 0..tank_sim::discovery::n::COUNT as u8 {
            let m = tank_sim::discovery::NODES[node as usize].materials.first().copied().unwrap_or(0);
            s.learn(i, node, m, tank_sim::discovery::KnowSource::Invented);
        }
    }
    let start = stock(&s);
    s.run_cycles(cycles);
    let mut gathered = s.counters.food_gathered as f64;
    let mut grown = s.counters.algae_grown as f64 + 1.5 * s.counters.shell_grown as f64;
    for c in &s.stats {
        gathered += c.c.food_gathered as f64;
        grown += c.c.algae_grown as f64 + 1.5 * c.c.shell_grown as f64;
    }
    let available = start + grown;
    assert!(gathered > 0.0, "nobody gathered anything");
    assert!(
        gathered <= available * 1.001 + 1.0,
        "seed {seed}: gathered {gathered:.0} but only {available:.0} existed (start {start:.0} + regrowth {grown:.0})"
    );
    // no hidden sources: what remains can't exceed what the books allow (natural die-back is a
    // legitimate loss, so `end` may be lower)
    let end = stock(&s);
    let allowed = start + grown - gathered;
    assert!(end <= allowed * 1.001 + 1.0, "seed {seed}: {end:.0} food remains but the books allow only {allowed:.0}");
}

#[test]
fn food_is_conserved_seed_1() {
    check(1, 4);
}

#[test]
fn food_is_conserved_seed_2() {
    check(2, 4);
}
