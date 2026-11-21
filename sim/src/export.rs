//! Output writers: thought rendering, person summaries, run files and the debug-viewer export.
use crate::concepts::{cue, outcome, Concept};
use crate::discovery::{Registry, NODES};
use crate::genome::tr;
use crate::mind::{drv, emo, Rel};
use crate::person::Act;
use crate::sim::Sim;
use crate::social::role;
use crate::thought::{result, saw, why, Thought, NONE};
use crate::world::{Habitat, H, W};
use serde::Serialize;
use std::io::Write;

pub fn when(sim: &Sim, t: u64) -> String {
    let rel = t.saturating_sub(sim.tick0);
    let tpd = sim.tpd();
    let dpc = sim.cfg.time.days_per_cycle();
    let day = rel / tpd;
    let hour = ((rel % tpd) as f32 / tpd as f32 * 24.0 + 6.0) % 24.0;
    format!("c{}d{:02} {:02}:{:02}", day / dpc, day % dpc, hour as u32, ((hour.fract()) * 60.0) as u32)
}

pub fn act_text(sim: &Sim, a: &Act) -> String {
    let nm = |w: u32| sim.name(w);
    match *a {
        Act::Idle => "wait".into(),
        Act::Forage { place } => format!("forage at the {}", sim.place_name(place)),
        Act::Store => "carry food home to the stash".into(),
        Act::Eat => "eat".into(),
        Act::Rest => "rest".into(),
        Act::Sleep => "sleep".into(),
        Act::Flee => "flee".into(),
        Act::Socialize { who } => format!("spend time with {}", nm(who)),
        Act::Share { who } => format!("give food to {}", nm(who)),
        Act::AskHelp { who } => format!("ask {} for food", nm(who)),
        Act::Hunt { .. } => "hunt drifters together".into(),
        Act::Defend { .. } => "stand and fight the gulper".into(),
        Act::Teach { who, node } => format!("teach {} the {}", nm(who), NODES[node as usize].name),
        Act::Experiment => "tinker and experiment".into(),
        Act::Build { node } => format!("build a {}", NODES[node as usize].name),
        Act::Steal { who } => format!("steal food from {}", nm(who)),
        Act::Punish { who } => format!("punish {}", nm(who)),
        Act::Court { who } => format!("court {}", nm(who)),
        Act::Explore { place } => format!("explore the {}", sim.place_name(place)),
        Act::Migrate { camp } => format!("move to the camp of {}", sim.camp_name(camp)),
        Act::Found { place } => format!("leave and found a new camp at the {}", sim.place_name(place)),
        Act::Mourn => "mourn".into(),
        Act::Heal { who } => format!("tend {}'s fever", nm(who)),
        Act::Play => "play".into(),
        Act::Follow { who } => format!("go along with {}", nm(who)),
    }
}

pub fn render_thought(sim: &Sim, t: &Thought) -> String {
    let p = &sim.people[t.who as usize];
    let age = p.age_cycles(t.tick, sim.tpc());
    let mut parts: Vec<String> = Vec::new();
    if t.felt.0 != NONE {
        parts.push(format!("{}({:.1})", emo::NAMES[t.felt.0 as usize], t.felt.1));
    }
    if t.drive.1 > 0.4 {
        parts.push(format!("{}({:.1})", drv::NAMES[t.drive.0 as usize], t.drive.1));
    }
    for s in t.saw.iter() {
        if s.0 != NONE {
            let who = match s.0 {
                saw::GULPER | saw::PHANTOM | saw::HERD | saw::FOOD_LOW => String::new(),
                _ => format!(" {}", sim.name(s.1)),
            };
            parts.push(format!("saw {}{}", saw::NAMES[s.0 as usize], who));
        }
    }
    if t.expect.0 != NONE {
        let c = if (t.expect.1 as usize) < cue::N { cue::NAMES[t.expect.1 as usize] } else { "?" };
        parts.push(format!("expects {} (after {}, {:.0}%)", outcome::NAMES[t.expect.0 as usize], c, t.expect.2 * 100.0));
    }
    let because: Vec<String> = t.why.iter().filter(|w| w.0 != NONE).map(|w| format!("{} {:.1}", why::NAMES[w.0 as usize], w.1)).collect();
    let res = match t.result {
        result::ACCEPTED => " [yes]",
        result::REFUSED => " [no]",
        _ => "",
    };
    let intr = if t.interrupt { "!! " } else { "" };
    let rname = role_names(p.roles);
    format!(
        "[{}] {} ({:.0}{}): {}{} → {}{}{}",
        when(sim, t.tick),
        p.display_name(),
        age,
        if rname.is_empty() { String::new() } else { format!(", {rname}") },
        intr,
        if parts.is_empty() { "calm".into() } else { parts.join(" + ") },
        act_text(sim, &t.chose),
        if because.is_empty() { String::new() } else { format!(" because {}", because.join(", ")) },
        res
    )
}

pub fn role_names(bits: u16) -> String {
    (0..role::N).filter(|k| bits & (1 << k) != 0 && *k != role::STORYTELLER).map(|k| role::NAMES[k]).collect::<Vec<_>>().join("/")
}

pub fn belief_text(sim: &Sim, subject: &Concept, rel: Rel) -> String {
    let s = match subject {
        Concept::Person(p) => sim.name(*p),
        Concept::Kind(k) => k.gloss().to_lowercase(),
        Concept::Place(p) => sim.place_name(*p),
        Concept::Camp(c) => sim.camp_name(*c),
        Concept::Node(nn) => NODES[*nn as usize].name.into(),
    };
    let r = match rel {
        Rel::Forbidden => "is forbidden",
        Rel::Thief => "is a thief",
        Rel::Heals => "is a healer",
        Rel::Leads => "should be followed",
        Rel::Teaches => "is a teacher",
        Rel::Cursed => "is cursed",
        Rel::Liar => "lies",
        Rel::Spreads => "spreads from the sick",
        Rel::Watch => "watch over us",
        Rel::Governs => "governs the seasons",
        Rel::Sacred => "is sacred",
    };
    format!("{s} {r}")
}

#[derive(Serialize)]
pub struct PersonOut {
    pub id: u32,
    pub name: String,
    pub name_ipa: String,
    pub gloss: String,
    pub culture: u16,
    pub female: bool,
    pub born_cycle: f32,
    pub died_cycle: Option<f32>,
    pub cause: Option<String>,
    pub age: f32,
    pub mother: Option<String>,
    pub father: Option<String>,
    pub mate: Option<String>,
    pub children: usize,
    pub home: Option<String>,
    pub roles: String,
    pub notability: f32,
    pub inventions: u32,
    pub knowledge: Vec<String>,
    pub traits: Vec<(String, f32)>,
    pub beliefs: Vec<String>,
    pub rules: Vec<String>,
    pub feared_places: Vec<String>,
    pub friends: Vec<String>,
    pub enemies: Vec<String>,
    pub lexicon_size: usize,
    pub life: Vec<String>,
    pub thoughts: Vec<String>,
}

pub fn person_out(sim: &Sim, id: u32) -> PersonOut {
    let p = &sim.people[id as usize];
    let tpc = sim.tpc() as f32;
    let cyc = |t: u64| (t as f32 - sim.tick0 as f32) / tpc;
    let end = p.died.unwrap_or(sim.tick);
    let mut beliefs: Vec<(f32, String)> = p.mind.beliefs.iter().filter(|b| b.conf > 0.25).map(|b| (b.conf, format!("{} ({:.0}%{})", belief_text(sim, &b.subject, b.rel), b.conf * 100.0, if b.distorted { ", garbled rumor" } else { "" }))).collect();
    beliefs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let rules = p.mind.cont.rules(sim.cfg.mind.rule_min_dp, sim.cfg.mind.rule_min_n).into_iter().take(6).map(|(c, o, dp)| format!("{} → {} (ΔP {:.2})", cue::NAMES[c], outcome::NAMES[o], dp)).collect();
    let feared_places = p.mind.places.iter().enumerate().filter(|(_, pm)| pm.danger > 0.5).map(|(k, pm)| format!("{} ({:.0}%{})", sim.place_name(k as u8), pm.danger * 100.0, if pm.danger_src != 0 { ", heard it" } else { "" })).collect();
    let mut friends: Vec<(f32, String)> = p.mind.models.iter().filter(|m| m.affinity > 0.5).map(|m| (m.affinity, sim.name(m.who))).collect();
    friends.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let enemies = p.mind.models.iter().filter(|m| m.grudge > 0.35).map(|m| format!("{} (grudge {:.1})", sim.name(m.who), m.grudge)).collect();
    PersonOut {
        id,
        name: p.display_name(),
        name_ipa: crate::language::ipa(&p.name),
        gloss: p.name_gloss.clone(),
        culture: p.culture,
        female: p.female,
        born_cycle: cyc(p.born),
        died_cycle: p.died.map(cyc),
        cause: p.death_cause.clone(),
        age: p.age_cycles(end, sim.tpc()),
        mother: p.mother.map(|m| sim.name(m)),
        father: p.father.map(|m| sim.name(m)),
        mate: p.mate.map(|m| sim.name(m)),
        children: p.children.len(),
        home: p.home.map(|h| sim.camp_name(h)),
        roles: role_names(p.roles),
        notability: p.notability,
        inventions: p.inventions,
        knowledge: p.knowledge.iter().map(|k| Registry::variant_name(k.node, k.material)).collect(),
        traits: (0..6).map(|k| (tr::NAMES[k].to_string(), (p.mind.traits[k] * 100.0).round() / 100.0)).collect(),
        beliefs: beliefs.into_iter().take(10).map(|b| b.1).collect(),
        rules,
        feared_places,
        friends: friends.into_iter().take(6).map(|f| f.1).collect(),
        enemies,
        lexicon_size: p.mind.lex.len(),
        life: p.life.iter().map(|l| format!("[{}] {}", when(sim, l.tick), l.text)).collect(),
        thoughts: p.thoughts.iter().rev().take(16).rev().map(|t| render_thought(sim, t)).collect(),
    }
}

pub fn write_run(sim: &Sim, dir: &str, summary: &crate::metrics::RunSummary) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(format!("{dir}/summary.json"), serde_json::to_string_pretty(summary).unwrap())?;
    // cycles.csv
    let mut f = std::io::BufWriter::new(std::fs::File::create(format!("{dir}/cycles.csv"))?);
    writeln!(f, "cycle,real_year,surface_t,pop,adults,births,deaths,starve,predator,disease,old,cold,violence,camps,groups,formed,splits,merges,predators,herd,food,known_nodes,inventions,lost_total,languages,min_intel,coherence,lexicon,grammar_tier,comm_success,roles,belief_div,false_frac,culture_false,beh_entropy,beh_div,change,shares,asks,granted,hunts_won,hunts_lost,thefts,punish,heals,taught")?;
    for s in &sim.stats {
        let c = &s.c;
        writeln!(
            f,
            "{},{},{:.1},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.0},{},{},{},{},{:.2},{:.3},{:.0},{},{:.3},{},{:.3},{:.3},{},{:.3},{:.3},{:.3},{},{},{},{},{},{},{},{},{}",
            s.cycle, s.real_year, s.surface_t, s.pop, s.adults, c.births, c.deaths, c.deaths_starvation, c.deaths_predator, c.deaths_disease, c.deaths_old, c.deaths_cold, c.deaths_violence,
            s.camps, s.groups, c.groups_formed, c.group_splits, c.group_merges, s.predators, s.herd_animals, s.food_total, s.known_nodes, c.inventions, s.knowledge_lost_total, s.languages,
            s.min_intelligibility, s.coherence, s.lexicon_mean, s.max_grammar_tier, s.comm_success, s.roles_distinct, s.belief_diversity, s.false_belief_frac, s.culture_specific_false,
            s.behavior_entropy, s.behavior_divergence, s.change_index, c.shares, c.asks, c.asks_granted, c.hunts_won, c.hunts_lost, c.thefts, c.punishments, c.heals, c.taught
        )?;
    }
    drop(f);
    // events
    let mut f = std::io::BufWriter::new(std::fs::File::create(format!("{dir}/events.jsonl"))?);
    for e in &sim.events {
        writeln!(f, "{}", serde_json::to_string(e).unwrap())?;
    }
    drop(f);
    // readable chronicle of important events
    let mut f = std::io::BufWriter::new(std::fs::File::create(format!("{dir}/chronicle.txt"))?);
    for e in sim.events.iter().filter(|e| e.importance >= 0.35) {
        writeln!(f, "[{}] {:?}: {}", when(sim, e.tick), e.kind, e.text)?;
    }
    drop(f);
    std::fs::write(format!("{dir}/registry.json"), serde_json::to_string_pretty(&sim.registry).unwrap())?;
    std::fs::write(format!("{dir}/languages.json"), serde_json::to_string_pretty(&sim.obs.languages).unwrap())?;
    let people: Vec<PersonOut> = (0..sim.people.len() as u32).map(|id| person_out(sim, id)).collect();
    std::fs::write(format!("{dir}/people.json"), serde_json::to_string(&people).unwrap())?;
    Ok(())
}

// ---------------------------------------------------------------- debug viewer export

#[derive(Serialize, Default)]
pub struct Viewer {
    pub seed: u64,
    pub w: usize,
    pub h: usize,
    pub cell: f32,
    pub ridge_x: f32,
    pub tpd: u64,
    pub dpc: u64,
    pub tick0: u64,
    pub hab: Vec<u8>,
    pub depth: Vec<u8>,
    pub place_names: Vec<String>,
    pub act_names: Vec<String>,
    pub role_names: Vec<String>,
    pub people: Vec<VPerson>,
    pub camps: Vec<VCamp>,
    pub frames: Vec<VFrame>,
    pub events: Vec<VEvent>,
    #[serde(skip)]
    pub last_frame_tick: u64,
}

#[derive(Serialize)]
pub struct VPerson {
    pub id: u32,
    pub name: String,
    pub culture: u16,
    pub female: bool,
    pub born: u64,
    pub died: Option<u64>,
    pub cause: Option<String>,
    pub mother: Option<u32>,
    pub father: Option<u32>,
    pub hue: f32,
}

#[derive(Serialize)]
pub struct VCamp {
    pub id: u16,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub founded: u64,
}

#[derive(Serialize)]
pub struct VFrame {
    pub t: u64,
    /// flattened per person: id, x*10, y*10, act, home(+1 or 0), group(+1 or 0), flags(1=sick 2=child 4=stranger), roles
    pub p: Vec<i32>,
    /// predators: x, y
    pub g: Vec<i32>,
    /// herds: x, y, count
    pub d: Vec<i32>,
    /// camps: id, members, structure-node bitmask, records
    pub c: Vec<i32>,
    /// algae per place (0..255)
    pub a: Vec<u8>,
    /// speech: person id → rendered thought
    pub s: Vec<(u32, String)>,
    pub climate: (u32, f32, f32, f32),
}

#[derive(Serialize)]
pub struct VEvent {
    pub t: u64,
    pub k: String,
    pub who: Vec<u32>,
    pub text: String,
    pub imp: f32,
}

impl Viewer {
    pub fn new(sim: &Sim) -> Viewer {
        let mut v = Viewer {
            seed: sim.seed,
            w: W,
            h: H,
            cell: crate::world::CELL,
            ridge_x: sim.world.ridge_x,
            tpd: sim.tpd(),
            dpc: sim.cfg.time.days_per_cycle(),
            tick0: sim.tick0,
            ..Default::default()
        };
        for c in &sim.world.cells {
            v.hab.push(match c.hab {
                Habitat::Shallows => 0,
                Habitat::Sand => 1,
                Habitat::Reef => 2,
                Habitat::Kelp => 3,
                Habitat::Deep => 4,
                Habitat::Vent => 5,
                Habitat::Rock => 6,
            });
            v.depth.push((c.depth * 255.0) as u8);
        }
        v.place_names = (0..crate::world::N_PLACES as u8).map(|p| sim.place_name(p)).collect();
        v.act_names = crate::person::ACT_NAMES.iter().map(|s| s.to_string()).collect();
        v.role_names = role::NAMES.iter().map(|s| s.to_string()).collect();
        v
    }

    pub fn capture(&mut self, sim: &Sim) {
        let t = sim.tick;
        let mut f = VFrame { t, p: Vec::new(), g: Vec::new(), d: Vec::new(), c: Vec::new(), a: Vec::new(), s: Vec::new(), climate: (sim.world.climate.real_year, sim.world.climate.surface_t, sim.world.climate.lamp_frac, sim.world.climate.ice) };
        for &id in &sim.alive {
            let p = &sim.people[id as usize];
            let (x, y) = p.pos(t);
            let flags = if p.infected.is_some() { 1 } else { 0 } | if !sim.adult(id as usize) { 2 } else { 0 } | if p.stranger { 4 } else { 0 };
            f.p.extend_from_slice(&[
                id as i32,
                (x * 10.0) as i32,
                (y * 10.0) as i32,
                p.act.kind() as i32,
                p.home.map_or(0, |h| h as i32 + 1),
                sim.obs.membership.get(&id).map_or(0, |g| *g as i32 + 1),
                flags,
                p.roles as i32,
            ]);
            if let Some(th) = p.thoughts.last() {
                if th.tick > self.last_frame_tick {
                    f.s.push((id, render_thought(sim, th)));
                }
            }
        }
        for pr in sim.world.predators.iter().filter(|p| p.alive) {
            f.g.extend_from_slice(&[(pr.x * 10.0) as i32, (pr.y * 10.0) as i32]);
        }
        for h in sim.world.herds.iter().filter(|h| h.count >= 1.0) {
            f.d.extend_from_slice(&[(h.x * 10.0) as i32, (h.y * 10.0) as i32, h.count as i32]);
        }
        for c in sim.world.camps.iter().filter(|c| c.abandoned.is_none()) {
            let mut bits = 0i32;
            for s in &c.structures {
                bits |= 1 << s.node;
            }
            f.c.extend_from_slice(&[c.id as i32, c.members as i32, bits, c.records.len() as i32]);
        }
        for p in 0..crate::world::N_PLACES as u8 {
            f.a.push((sim.world.place_food(p) / 2.0).min(255.0) as u8);
        }
        self.last_frame_tick = t;
        self.frames.push(f);
    }

    pub fn finish(&mut self, sim: &Sim, from: u64, to: u64) {
        self.people = sim
            .people
            .iter()
            .map(|p| VPerson { id: p.id, name: p.display_name(), culture: p.culture, female: p.female, born: p.born, died: p.died, cause: p.death_cause.clone(), mother: p.mother, father: p.father, hue: p.genome.hue[0] })
            .collect();
        self.camps = sim.world.camps.iter().map(|c| VCamp { id: c.id, name: sim.camp_name(c.id), x: c.x, y: c.y, founded: c.founded }).collect();
        self.events = sim
            .events
            .iter()
            .filter(|e| e.tick >= from && e.tick <= to && e.kind != crate::events::EvKind::Birth)
            .map(|e| VEvent { t: e.tick, k: format!("{:?}", e.kind), who: e.who.clone(), text: e.text.clone(), imp: e.importance })
            .collect();
    }
}
