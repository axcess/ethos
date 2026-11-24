//! Dev diagnostic: run an evolution tank until the first minds appear, then trace their food day by day.
//! cargo run --release --example minds_trace -- SEED DAYS
use std::sync::Arc;
use tank_sim::climate::Dataset;
use tank_sim::config::Config;
use tank_sim::sim::{Sim, ThoughtMode};

fn main() {
    let seed: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(9);
    let days: u64 = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(30);
    let ds = Arc::new(Dataset::load(concat!(env!("CARGO_MANIFEST_DIR"), "/../data/climate/chicago_daily.csv")).unwrap());
    let cfg = Config::load(concat!(env!("CARGO_MANIFEST_DIR"), "/../config/presets/evolution.toml")).unwrap();
    let mut s = Sim::new(seed, cfg, ds);
    s.thought_mode = ThoughtMode::None;
    let tpd = s.tpd();
    while s.alive.is_empty() && s.cycle() < 800 {
        s.step();
    }
    println!("minds appear in cycle {} day {}", s.cycle(), (s.tick / tpd) % 48);
    for _ in 0..days {
        let before = s.tick / tpd;
        while s.tick / tpd == before {
            s.step();
        }
        let n = s.alive.len();
        if n == 0 {
            println!("all dead");
            break;
        }
        let mut camps: Vec<u16> = s.alive.iter().filter_map(|&i| s.people[i as usize].home).collect();
        camps.sort();
        camps.dedup();
        let mut line = format!("day {:>2} alive {:>3} energy {:.2} carried {:.1} stash {:.1} |", (s.tick / tpd) % 48, n,
            s.alive.iter().map(|&i| s.people[i as usize].energy).sum::<f32>() / n as f32,
            s.alive.iter().map(|&i| s.people[i as usize].carried).sum::<f32>() / n as f32,
            s.alive.iter().map(|&i| s.people[i as usize].stash).sum::<f32>() / n as f32);
        for c in camps.iter().take(3) {
            let pl = s.world.camps[*c as usize].place;
            let cells = s.world.best_cells(pl, 3);
            let cs: Vec<String> = cells.iter().map(|&i| { let c = &s.world.cells[i]; format!("a{:.1}/{:.1} s{:.1}/{:.1} la{:.2}", c.algae, c.algae_cap, c.shell, c.shell_cap, c.life_algae) }).collect();
            line += &format!(" camp{} place{} food{:.1} [{}]", c, pl, s.world.place_food(pl), cs.join(", "));
        }
        println!("{line}");
        let (mut fa, mut mated, mut samehome, mut stash, mut en) = (0, 0, 0, 0, 0);
        for &i in &s.alive {
            let p = &s.people[i as usize];
            if !p.female || !s.adult(i as usize) { continue; }
            fa += 1;
            if let Some(m) = p.mate { mated += 1; if s.people[m as usize].alive && s.people[m as usize].home == p.home { samehome += 1; } }
            if p.stash + p.carried >= 2.5 { stash += 1; }
            if p.energy >= 0.35 { en += 1; }
        }
        println!("   adult females {fa} mated {mated} same-home {samehome} stash-ok {stash} energy-ok {en} pregnant {}", s.alive.iter().filter(|&&i| s.people[i as usize].pregnant.is_some()).count());
    }
}
