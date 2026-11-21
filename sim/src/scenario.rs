//! Scenario tests (ROADMAP Phase 1). These are *test inputs* for headless experiments
//! only; a broadcast tank never runs with injections.
use crate::concepts::{Concept, ALL_KINDS};
use crate::discovery::n;
use crate::events::EvKind;
use crate::genome::Genome;
use crate::language::Phonology;
use crate::rng::{tag, Rng};
use crate::sim::Sim;
use crate::world::N_PLACES;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Injection {
    pub kind: String,
    pub day: u64,
    pub end_day: Option<u64>,
    pub applied: bool,
    pub ended: bool,
}

pub const ALL: &[&str] = &["famine", "predator_surge", "stranger", "leader_death", "windfall", "plague", "harsh_winter", "drought_year"];

/// Parse "kind" or "kind@cycle".
pub fn parse(spec: &str, dpc: u64) -> Result<Injection, String> {
    let (kind, cyc) = match spec.split_once('@') {
        Some((k, c)) => (k.to_string(), c.parse::<u64>().map_err(|_| format!("bad cycle in {spec}"))?),
        None => (spec.to_string(), 6),
    };
    if !ALL.contains(&kind.as_str()) {
        return Err(format!("unknown scenario '{kind}'. Known: {}", ALL.join(", ")));
    }
    Ok(Injection { kind, day: cyc * dpc, end_day: None, applied: false, ended: false })
}

pub fn apply_day(sim: &mut Sim, day: u64) {
    let dps = sim.cfg.time.days_per_season;
    let mut todo = Vec::new();
    for (k, inj) in sim.scenario.iter().enumerate() {
        if !inj.applied && day >= inj.day {
            todo.push((k, true));
        } else if inj.applied && !inj.ended && inj.end_day.map_or(false, |e| day >= e) {
            todo.push((k, false));
        }
    }
    for (k, start) in todo {
        let kind = sim.scenario[k].kind.clone();
        if start {
            sim.scenario[k].applied = true;
            let text = apply(sim, &kind, day, dps);
            if let Some(d) = scenario_duration(&kind, dps) {
                sim.scenario[k].end_day = Some(day + d);
            } else {
                sim.scenario[k].ended = true;
            }
            sim.emit(EvKind::Scenario, vec![], 0, format!("[scenario] {kind}: {text}"));
        } else {
            sim.scenario[k].ended = true;
            end(sim, &kind);
            sim.emit(EvKind::Scenario, vec![], 0, format!("[scenario] {kind} ends"));
        }
    }
}

fn scenario_duration(kind: &str, dps: u64) -> Option<u64> {
    match kind {
        "famine" => Some(dps * 2),
        "windfall" => Some(dps * 2),
        _ => None,
    }
}

fn largest_camp(sim: &Sim) -> Option<u16> {
    sim.world.camps.iter().filter(|c| c.abandoned.is_none()).max_by_key(|c| (c.members, u16::MAX - c.id)).map(|c| c.id)
}

fn apply(sim: &mut Sim, kind: &str, _day: u64, _dps: u64) -> String {
    let mut r = Rng::stream(sim.seed, tag::SCENARIO, sim.tick);
    match kind {
        "famine" => {
            for m in sim.world.regrow_mult.iter_mut() {
                *m = 0.1;
            }
            for c in sim.world.cells.iter_mut() {
                c.algae *= 0.3;
                c.shell *= 0.5;
            }
            for h in sim.world.herds.iter_mut() {
                h.count *= 0.4;
            }
            "algae dies back across the tank for two seasons".into()
        }
        "windfall" => {
            let east = r.chance(0.5);
            for p in 0..N_PLACES {
                if sim.world.places[p].east == east {
                    sim.world.regrow_mult[p] = 3.0;
                }
            }
            for c in sim.world.cells.iter_mut() {
                c.algae = (c.algae * 1.8).min(c.algae_cap * 1.5);
            }
            for h in sim.world.herds.iter_mut() {
                h.count = (h.count + 10.0).min(sim.cfg.world.herd_max);
            }
            format!("a great bloom on the {} side", if east { "east" } else { "west" })
        }
        "predator_surge" => {
            let n0 = sim.cfg.predators.initial.max(4) * 3;
            for _ in 0..n0 {
                let rr = &mut sim.pred_rng;
                sim.world.spawn_predator(rr, None);
            }
            format!("{n0} gulpers swarm in from the depths")
        }
        "plague" => {
            let mut infected = 0;
            if let Some(c) = largest_camp(sim) {
                let members: Vec<u32> = sim.alive.iter().copied().filter(|&i| sim.people[i as usize].home == Some(c)).collect();
                for &m in members.iter().take(5) {
                    sim.infect(m as usize);
                    infected += 1;
                }
            }
            format!("a fever breaks out; {infected} fall sick at once")
        }
        "leader_death" => {
            let mut best = None;
            let mut bs = -1.0f32;
            let mut resp: std::collections::BTreeMap<u32, f32> = Default::default();
            for &o in &sim.alive {
                for m in &sim.people[o as usize].mind.models {
                    *resp.entry(m.who).or_default() += m.respect.max(0.0) * (1.0 + m.followed as f32 * 0.2);
                }
            }
            for &id in &sim.alive {
                if !sim.adult(id as usize) {
                    continue;
                }
                let s = resp.get(&id).copied().unwrap_or(0.0) + sim.people[id as usize].notability;
                if s > bs {
                    bs = s;
                    best = Some(id);
                }
            }
            match best {
                Some(id) => {
                    let name = sim.name(id);
                    sim.die(id as usize, "vanished into the deep (scenario)");
                    format!("{name}, the most respected of all, vanishes")
                }
                None => "no one to lose".into(),
            }
        }
        "stranger" => {
            let id = sim.people.len() as u32;
            let mut prng = Rng::stream(sim.seed, tag::PERSON, id as u64);
            let lin = Genome::lineage(&mut prng);
            let pc = sim.cfg.people.clone();
            let g = Genome::seed(&mut prng, pc.capacity_min, pc.capacity_max, pc.max_age_min, pc.max_age_max, &lin);
            let phon = Phonology::generate(&mut prng, lin.lips, lin.tongue, lin.throat);
            let born = sim.tick - (15.0 * sim.tpc() as f32) as u64;
            let female = prng.chance(0.5);
            let mut p = sim.make_person(id, g, female, born, 900, phon, prng);
            for k in ALL_KINDS {
                let w = p.phon.coin(&mut p.rng);
                p.mind.lex.adopt(Concept::Kind(*k), w, 0.8);
            }
            for pl in 0..N_PLACES as u8 {
                let w = p.phon.coin(&mut p.rng);
                p.mind.lex.adopt(Concept::Place(pl), w, 0.6);
            }
            p.name = p.phon.coin(&mut p.rng);
            p.name_gloss = "stranger".into();
            let own = p.name.clone();
            p.mind.lex.adopt(Concept::Person(id), own, 0.9);
            p.stranger = true;
            p.stash = 0.0;
            p.carried = 3.0;
            // the stranger brings unfamiliar knowledge
            let gifts = [n::CORDAGE, n::SPEAR, n::REMEDY, n::BLADE];
            let tick = sim.tick;
            for g in gifts {
                if sim.nodes[g as usize].holders == 0 {
                    p.knowledge.push(crate::discovery::Know { node: g, material: crate::discovery::NODES[g as usize].materials.first().copied().unwrap_or(0), prof: 0.8, from: crate::discovery::KnowSource::Invented, tick });
                }
            }
            let gift_names: Vec<&str> = p.knowledge.iter().map(|k| crate::discovery::NODES[k.node as usize].name).collect();
            let (x, y) = match largest_camp(sim) {
                Some(c) => (sim.world.camps[c as usize].x + 20.0, sim.world.camps[c as usize].y + 10.0),
                None => (100.0, 100.0),
            };
            p.from = (x, y);
            p.to = (x, y);
            p.next_decision = sim.tick + 8;
            p.act_start = sim.tick;
            p.life_note(tick, "arrived from beyond the glass-edge, alone".into());
            let name = p.display_name();
            sim.people.push(p);
            sim.alive.push(id);
            sim.tracked.push(id);
            format!("{name}, a stranger speaking an unknown tongue, arrives knowing: {}", if gift_names.is_empty() { "nothing new".into() } else { gift_names.join(", ") })
        }
        "harsh_winter" | "drought_year" => {
            let years: &[u32] = if kind == "harsh_winter" { &[1977, 1978, 1979] } else { &[1934, 1936, 1988] };
            let c0 = sim.cycle();
            for (k, y) in years.iter().enumerate() {
                sim.climate.overrides.push((c0 + k as u64, *y));
            }
            format!("the tank's climate follows Chicago {}–{}", years[0], years[years.len() - 1])
        }
        _ => "unknown".into(),
    }
}

fn end(sim: &mut Sim, kind: &str) {
    if kind == "famine" || kind == "windfall" {
        for m in sim.world.regrow_mult.iter_mut() {
            *m = 1.0;
        }
    }
}
