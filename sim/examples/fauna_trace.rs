//! Dev diagnostic: run an evolution tank until animals appear, then trace them day by day.
//! cargo run --release --example fauna_trace -- SEED
use std::sync::Arc;
use tank_sim::climate::Dataset;
use tank_sim::config::Config;
use tank_sim::sim::{Sim, ThoughtMode};

fn main() {
    let seed: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(6);
    let ds = Arc::new(Dataset::load(concat!(env!("CARGO_MANIFEST_DIR"), "/../data/climate/chicago_daily.csv")).unwrap());
    let mut cfg = Config::load(concat!(env!("CARGO_MANIFEST_DIR"), "/../config/presets/evolution.toml")).unwrap();
    cfg.evolution.enabled = true;
    let mut s = Sim::new(seed, cfg, ds);
    s.thought_mode = ThoughtMode::None;
    let tpd = s.tpd();
    let mut day = 0;
    while s.evo.as_ref().unwrap().creatures.is_empty() && day < 48 * 800 {
        s.step();
        day = (s.tick - s.tick0) / tpd;
    }
    println!("animals appear on day {day} (cycle {})", day / 48);
    let days: usize = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(120);
    let every: usize = std::env::args().nth(3).and_then(|s| s.parse().ok()).unwrap_or(1);
    for d in 0..days {
        let before = s.tick / tpd;
        while s.tick / tpd == before {
            s.step();
        }
        let e = s.evo.as_ref().unwrap();
        let n: f32 = e.creatures.iter().map(|c| c.m).sum();
        let en: f32 = if n > 0.0 { e.creatures.iter().map(|c| c.energy * c.m).sum::<f32>() / n } else { 0.0 };
        let mut food_near = 0.0;
        for c in &e.creatures {
            let cx = (c.x / 8.0) as usize;
            let cy = (c.y / 8.0) as usize;
            let i = cy.min(31) * 64 + cx.min(63);
            let cell = &s.world.cells[i];
            let mats: f32 = (0..3).map(|k| e.pop[i * 3 + k].iter().map(|x| x.1).sum::<f32>()).sum();
            food_near += cell.algae + cell.shell + mats;
        }
        let g = e.creatures.first().map(|c| (c.g.size, c.g.speed, c.g.diet, c.g.capacity, c.g.layer, c.age_days));
        let diets: Vec<f32> = e.creatures.iter().map(|c| c.g.diet).collect();
        let dmax = diets.iter().cloned().fold(0.0f32, f32::max);
        let caps = e.creatures.iter().map(|c| c.g.capacity).fold(0.0f32, f32::max);
        let mean_m = if e.creatures.is_empty() { 0.0 } else { n / e.creatures.len() as f32 };
        if d % every != 0 { continue; }
        let mats: f32 = e.total_biomass();
        let algae: f32 = s.world.cells.iter().map(|c| c.algae).sum();
        println!("   diet max {:.2} cap max {:.2} mean herd {:.1} | season day {:>2} surface {:.1}C ice {:.2} | mats {:.0} algae {:.0}", dmax, caps, mean_m, s.day() % 48, s.world.climate.surface_t, s.world.climate.ice, mats, algae);
        println!(
            "day {:>5} herds {:>3} animals {:>7.1} energy {:.2} food {:>6.1} born/starved/old/eaten {:?} first {:?}",
            s.day(), e.creatures.len(), n, en, food_near / e.creatures.len().max(1) as f32, e.fauna_tally.map(|x| (x * 10.0).round() / 10.0), g.map(|g| (g.0, g.2, g.5))
        );
        if e.creatures.is_empty() {
            break;
        }
    }
}
