//! D1: one Mind, many capacities (DESIGN §5.4, D-005).
use std::sync::Arc;
use tank_sim::climate::Dataset;
use tank_sim::config::{Config, MindCfg};
use tank_sim::mind::Caps;
use tank_sim::person::Act;
use tank_sim::sim::{Sim, ThoughtMode};

fn dataset() -> Arc<Dataset> {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/../data/climate/chicago_daily.csv");
    Arc::new(Dataset::load(p).expect("climate dataset"))
}

#[test]
fn limits_grow_with_capacity_and_features_switch_on() {
    let cfg = MindCfg::default();
    let mut prev: Option<Caps> = None;
    for k in 0..=20 {
        let c = Caps::from_capacity(k as f32 * 0.05, &cfg);
        if let Some(p) = prev {
            assert!(c.episodic >= p.episodic && c.beliefs >= p.beliefs && c.models >= p.models && c.lexicon >= p.lexicon);
        }
        prev = Some(c);
    }
    let low = Caps::from_capacity(0.25, &cfg);
    assert!(!low.language && !low.planning && !low.culture && low.lexicon == 0);
    let mid = Caps::from_capacity(0.45, &cfg);
    assert!(mid.language && !mid.planning);
    let typical = Caps::from_capacity(0.85, &cfg);
    assert!(typical.language && typical.planning && typical.culture);
    assert_eq!(typical.models, cfg.model_cap, "config limits are the values for a typical sapient");
}

fn run_at(capacity: f32) -> Sim {
    let mut cfg = Config::default();
    cfg.people.initial = 40;
    cfg.people.bands = 2;
    cfg.people.capacity_min = capacity;
    cfg.people.capacity_max = capacity + 0.01;
    let mut s = Sim::new(4, cfg, dataset());
    s.thought_mode = ThoughtMode::None;
    s.run_cycles(2);
    s
}

#[test]
fn low_capacity_minds_run_the_same_code_with_fewer_features() {
    let s = run_at(0.3);
    let mut acted = 0;
    for p in &s.people {
        let c = p.mind.caps;
        // kin are always remembered (even the simplest minds know their kin); the cap is on everyone else
        let non_kin = p.mind.models.iter().filter(|m| m.kin == tank_sim::mind::kin::NONE).count();
        assert!(non_kin <= c.models, "{non_kin} non-kin acquaintances > cap {}", c.models);
        assert!(p.mind.episodes.len() <= c.episodic);
        assert!(p.mind.beliefs.len() <= c.beliefs);
        assert!(p.activity_total[Act::Teach { who: 0, node: 0 }.kind()] == 0, "no teaching below 0.55");
        assert!(p.activity_total[Act::Experiment.kind()] == 0, "no experimenting below 0.55");
        assert!(p.intent.is_none(), "no planning below 0.55");
        assert_eq!(p.mind.comm_try, 0, "no naming game below 0.35");
        acted += p.activity_total.iter().sum::<u32>();
    }
    assert!(acted > 1000, "low-capacity minds still forage, eat, sleep, flee and socialise");
    // the typical sapient tank still teaches and talks
    let hi = run_at(0.85);
    assert!(hi.people.iter().any(|p| p.mind.comm_try > 0));
}
