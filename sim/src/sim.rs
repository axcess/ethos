//! The simulation: world + people + the deterministic main loop.
use crate::climate::{ClimateDriver, Dataset};
use crate::concepts::{cue, outcome, Concept, Kind, ALL_KINDS};
use crate::config::Config;
use crate::discovery::{n, need, Know, KnowSource, Registry, RegistryEntry, Via, NODES};
use crate::events::{EvKind, Event};
use crate::genome::Genome;
use crate::language::{Lexicon, Phonology};
use crate::metrics::{CycleCounters, CycleStats};
use crate::mind::{drv, emo, kin, Episode, EpKind, Mind, Rel};
use crate::person::{Act, Person, N_ACT};
use crate::rng::{fnv64, tag, Rng};
use crate::thought::Thought;
use crate::world::{dist, place_of, Camp, World, N_PLACES, PH, PW};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub leader: u32,
    pub members: Vec<u32>,
    pub herd: u32,
    pub end: u64,
    pub place: u8,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NodeState {
    pub holders: u32,
    pub peak: u32,
    pub first_tick: Option<u64>,
    pub lost: Vec<u64>,
    pub rediscovered: Vec<u64>,
    pub inventions: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThoughtMode {
    None,
    Tracked,
    All,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sim {
    pub cfg: Config,
    pub seed: u64,
    pub tick0: u64,
    pub tick: u64,
    pub world: World,
    pub people: Vec<Person>,
    pub alive: Vec<u32>,
    pub climate: ClimateDriver,
    pub tasks: Vec<Task>,
    pub next_task: u32,
    pub next_camp: u16,
    pub registry: Registry,
    pub nodes: Vec<NodeState>,
    pub events: Vec<Event>,
    pub rng: Rng,
    pub pred_rng: Rng,
    pub dis_rng: Rng,
    pub obs: crate::social::Observer,
    pub counters: CycleCounters,
    pub stats: Vec<CycleStats>,
    pub tracked: Vec<u32>,
    pub thought_mode: ThoughtMode,
    #[serde(skip)]
    pub thought_out: Vec<Thought>,
    pub decisions: u64,
    pub last_extreme_day: u64,
    pub scenario: Vec<crate::scenario::Injection>,
    pub proto_phon: Phonology,
    #[serde(skip)]
    pub ds: Option<Arc<Dataset>>,
    #[serde(skip)]
    pub at_place: Vec<Vec<u32>>,
    pub extinct: bool,
    #[serde(default)]
    pub evo: Option<crate::evo::Evo>,
}

pub fn pair_mut<T>(v: &mut [T], a: usize, b: usize) -> (&mut T, &mut T) {
    assert!(a != b);
    if a < b {
        let (x, y) = v.split_at_mut(b);
        (&mut x[a], &mut y[0])
    } else {
        let (x, y) = v.split_at_mut(a);
        (&mut y[0], &mut x[b])
    }
}

impl Sim {
    pub fn new(seed: u64, cfg: Config, ds: Arc<Dataset>) -> Sim {
        let mut rng = Rng::stream(seed, tag::WORLD, 0);
        let mut map_rng = Rng::stream(seed, tag::MAP, 0);
        let world = World::generate(seed, &cfg, &mut map_rng);
        let climate = ClimateDriver::new(seed, &cfg.climate, &ds);
        let tpc = cfg.time.ticks_per_cycle();
        let tick0 = 64 * tpc;
        let species = Genome::lineage(&mut rng);
        let proto_phon = Phonology::generate(&mut rng, species.lips, species.tongue, species.throat);
        let mut sim = Sim {
            seed,
            tick0,
            tick: tick0,
            world,
            people: Vec::new(),
            alive: Vec::new(),
            climate,
            tasks: Vec::new(),
            next_task: 0,
            next_camp: 0,
            registry: Registry::default(),
            nodes: vec![NodeState::default(); n::COUNT],
            events: Vec::new(),
            rng,
            pred_rng: Rng::stream(seed, tag::PRED, 0),
            dis_rng: Rng::stream(seed, tag::DISEASE, 0),
            obs: crate::social::Observer::default(),
            counters: CycleCounters::default(),
            stats: Vec::new(),
            tracked: Vec::new(),
            thought_mode: ThoughtMode::Tracked,
            thought_out: Vec::new(),
            decisions: 0,
            last_extreme_day: 0,
            scenario: Vec::new(),
            proto_phon,
            ds: Some(ds),
            at_place: vec![Vec::new(); N_PLACES],
            extinct: false,
            evo: None,
            cfg,
        };
        if sim.cfg.evolution.enabled {
            // a lifeless tank: no people, no beds, no hand-made fauna; everything must evolve
            sim.evo = Some(crate::evo::Evo::new(seed));
            for c in sim.world.cells.iter_mut() {
                c.algae = 0.0;
                c.shell = 0.0;
                c.life_algae = 0.0;
                c.life_shell = 0.0;
            }
            sim.world.herds.clear();
            sim.world.predators.clear();
        } else {
            sim.found_population(&species);
        }
        sim.new_day();
        sim
    }

    pub fn ds(&self) -> &Dataset {
        self.ds.as_ref().expect("climate dataset not attached")
    }
    pub fn tpd(&self) -> u64 {
        self.cfg.time.ticks_per_day
    }
    pub fn tpc(&self) -> u64 {
        self.cfg.time.ticks_per_cycle()
    }
    pub fn day(&self) -> u64 {
        (self.tick - self.tick0) / self.tpd()
    }
    pub fn cycle(&self) -> u64 {
        (self.tick - self.tick0) / self.tpc()
    }
    pub fn cycle_of(&self, t: u64) -> u64 {
        t.saturating_sub(self.tick0) / self.tpc()
    }
    pub fn is_night(&self, t: u64) -> bool {
        let tod = (t % self.tpd()) as f32 / self.tpd() as f32;
        tod > self.world.climate.lamp_frac.max(0.25)
    }
    pub fn adult(&self, i: usize) -> bool {
        self.people[i].age_cycles(self.tick, self.tpc()) >= self.cfg.people.adult_cycles
    }
    pub fn name(&self, id: u32) -> String {
        self.people.get(id as usize).map(|p| p.display_name()).unwrap_or_else(|| format!("#{id}"))
    }
    pub fn camp_name(&self, c: u16) -> String {
        crate::language::name_case(&self.world.camps[c as usize].name)
    }
    pub fn place_name(&self, p: u8) -> String {
        let pl = &self.world.places[p as usize];
        let side = if pl.east { "east" } else { "west" };
        let depth = ["front", "near", "far", "back"][(p as usize) / PW];
        let hab = format!("{:?}", pl.hab).to_lowercase();
        format!("{side} {depth} {hab} ({p})")
    }

    // ---------------------------------------------------------------- founding

    fn found_population(&mut self, species: &Genome) {
        let pc = self.cfg.people.clone();
        let tpc = self.tpc();
        // proto-language shared by all founders
        let mut proto = Lexicon::default();
        for k in ALL_KINDS {
            let w = self.proto_phon.coin(&mut self.rng);
            proto.adopt(Concept::Kind(*k), w, 0.7);
        }
        for p in 0..N_PLACES as u8 {
            let w = self.proto_phon.coin(&mut self.rng);
            proto.adopt(Concept::Place(p), w, 0.6);
        }
        let per_band = (pc.initial / pc.bands.max(1)).max(2);
        let mut used_places: Vec<u8> = Vec::new();
        for b in 0..pc.bands {
            let east = b % 2 == 1;
            // best unused place on this side
            let mut best = None;
            let mut best_s = f32::MIN;
            for pl in &self.world.places {
                if pl.east != east || !pl.passable || used_places.iter().any(|u| {
                    let (a, c) = (*u as usize, pl.id as usize);
                    (a % PW).abs_diff(c % PW) <= 1 && (a / PW).abs_diff(c / PW) <= 1
                }) {
                    continue;
                }
                let s = self.world.place_food(pl.id) + self.rng.f32() * 20.0;
                if s > best_s {
                    best_s = s;
                    best = Some(pl.id);
                }
            }
            let place = best.unwrap_or(if east { (PW - 1) as u8 } else { 0 });
            used_places.push(place);
            let band_lineage = {
                let mut l = Genome::lineage(&mut self.rng);
                l.pitch = species.pitch;
                l.lips = species.lips;
                l.tongue = species.tongue;
                l.throat = species.throat;
                l.size = species.size;
                l
            };
            let camp_id = self.new_camp(place, u32::MAX, b as u16);
            let first = self.people.len() as u32;
            for _ in 0..per_band {
                let id = self.people.len() as u32;
                let mut prng = Rng::stream(self.seed, tag::PERSON, id as u64);
                let g = Genome::seed(&mut prng, pc.capacity_min, pc.capacity_max, pc.max_age_min, pc.max_age_max, &band_lineage);
                let age = if prng.chance(0.25) { prng.range(1.0, pc.adult_cycles) } else { prng.range(pc.adult_cycles, 28.0) };
                let born = self.tick - (age * tpc as f32) as u64;
                let female = prng.chance(0.5);
                let mut p = self.make_person(id, g, female, born, b as u16, self.proto_phon.clone(), prng);
                p.home = Some(camp_id);
                let (cx, cy) = (self.world.camps[camp_id as usize].x, self.world.camps[camp_id as usize].y);
                let jx = p.rng.range(-6.0, 6.0);
                let jy = p.rng.range(-6.0, 6.0);
                p.from = (cx + jx, cy + jy);
                p.to = p.from;
                p.stash = 3.0;
                p.energy = 0.75;
                // proto vocabulary with gaps
                for (c, v) in &proto.entries {
                    if p.rng.chance(0.85) {
                        let s = p.rng.range(0.4, 0.8);
                        p.mind.lex.adopt(*c, v[0].0.clone(), s);
                    }
                }
                p.name = loop {
                    let w = self.proto_phon.coin(&mut p.rng);
                    if w.len() >= 3 {
                        break w;
                    }
                };
                p.name_gloss = "founder".into();
                self.people.push(p);
            }
            let last = self.people.len() as u32;
            // pair bonds among adults, parents for the young
            let ids: Vec<u32> = (first..last).collect();
            let adults: Vec<u32> = ids.iter().copied().filter(|&i| self.adult(i as usize)).collect();
            let mut free_f: Vec<u32> = adults.iter().copied().filter(|&i| self.people[i as usize].female).collect();
            let mut free_m: Vec<u32> = adults.iter().copied().filter(|&i| !self.people[i as usize].female).collect();
            let mut pairs = Vec::new();
            while !free_f.is_empty() && !free_m.is_empty() && self.rng.chance(0.75) {
                let f = free_f.remove(self.rng.below(free_f.len() as u32) as usize);
                let m = free_m.remove(self.rng.below(free_m.len() as u32) as usize);
                self.people[f as usize].mate = Some(m);
                self.people[m as usize].mate = Some(f);
                pairs.push((f, m));
            }
            let young: Vec<u32> = ids.iter().copied().filter(|&i| !self.adult(i as usize)).collect();
            for &c in &young {
                if pairs.is_empty() {
                    break;
                }
                let (f, m) = pairs[self.rng.below(pairs.len() as u32) as usize];
                self.people[c as usize].mother = Some(f);
                self.people[c as usize].father = Some(m);
                self.people[f as usize].children.push(c);
                self.people[m as usize].children.push(c);
            }
            // everyone in a band knows everyone
            for &a in &ids {
                for &bb in &ids {
                    if a == bb {
                        continue;
                    }
                    let k = self.kin_between(a, bb);
                    let nm = self.people[bb as usize].name.clone();
                    let tick = self.tick;
                    let cap = self.cfg.mind.model_cap;
                    let p = &mut self.people[a as usize];
                    let aff = p.rng.range(0.05, 0.35) + if k != kin::NONE { 0.4 } else { 0.0 };
                    let tr = p.rng.range(0.35, 0.6) + if k != kin::NONE { 0.2 } else { 0.0 };
                    let m = p.mind.model_mut(bb, tick, cap);
                    m.affinity = aff;
                    m.trust = tr;
                    m.kin = k;
                    p.mind.lex.adopt(Concept::Person(bb), nm, 0.6);
                }
                let own = self.people[a as usize].name.clone();
                self.people[a as usize].mind.lex.adopt(Concept::Person(a), own, 0.9);
                let cname = self.world.camps[camp_id as usize].name.clone();
                self.people[a as usize].mind.lex.adopt(Concept::Camp(camp_id), cname, 0.7);
            }
            self.world.camps[camp_id as usize].founder = first;
        }
        // tracked individuals for thought logs: a deterministic sample
        let n = self.people.len() as u32;
        let stride = (n / 12).max(1);
        self.tracked = (0..n).step_by(stride as usize).take(12).collect();
        self.alive = (0..n).collect();
        for i in 0..self.people.len() {
            self.people[i].next_decision = self.tick + self.people[i].rng.below(64) as u64;
            self.people[i].act_start = self.tick;
            self.people[i].last_upd = self.tick;
        }
    }

    /// How strongly `i` sees `j` as one of "us" (0..1), and as one of "them" (0..1).
    pub fn us_them(&self, i: usize, j: usize) -> (f32, f32) {
        match self.people[i].mind.identity {
            Some((camp, s)) => {
                if self.people[j].home == Some(camp) {
                    (s, 0.0)
                } else if self.people[j].home.is_some() {
                    (0.0, s)
                } else {
                    (0.0, s * 0.5)
                }
            }
            None => (0.0, 0.0),
        }
    }

    pub fn kin_between(&self, a: u32, b: u32) -> u8 {
        let (pa, pb) = (&self.people[a as usize], &self.people[b as usize]);
        if pa.mate == Some(b) {
            kin::MATE
        } else if pa.mother == Some(b) || pa.father == Some(b) {
            kin::PARENT
        } else if pb.mother == Some(a) || pb.father == Some(a) {
            kin::CHILD
        } else if pa.mother.is_some() && (pa.mother == pb.mother || pa.father == pb.father) {
            kin::SIBLING
        } else {
            kin::NONE
        }
    }

    pub fn make_person(&self, id: u32, g: Genome, female: bool, born: u64, culture: u16, phon: Phonology, mut rng: Rng) -> Person {
        let innovation = if rng.chance(self.cfg.language.innovator_chance) { Some(phon.random_law(&mut rng)) } else { None };
        let traits = g.traits;
        let caps = crate::mind::Caps::from_capacity(g.capacity, &self.cfg.mind);
        Person {
            id,
            alive: true,
            name: Vec::new(),
            name_gloss: String::new(),
            female,
            born,
            died: None,
            death_cause: None,
            mother: None,
            father: None,
            mate: None,
            children: Vec::new(),
            genome: g,
            culture,
            phon,
            innovation,
            home: None,
            energy: 0.8,
            health: 1.0,
            carried: 0.0,
            stash: 0.0,
            last_upd: self.tick,
            infected: None,
            immune_until: 0,
            healed_today: false,
            pregnant: None,
            act: Act::Idle,
            act_start: self.tick,
            next_decision: self.tick + 8,
            from: (0.0, 0.0),
            to: (0.0, 0.0),
            intent: None,
            mind: Mind::new(traits, caps),
            knowledge: Vec::new(),
            skill_forage: 0.2,
            skill_hunt: 0.1,
            activity: [0.0; N_ACT],
            activity_total: [0; N_ACT],
            needs: [0.0; need::N],
            rng,
            saw_predator_today: false,
            alarm_until: 0,
            life: Vec::new(),
            thoughts: Vec::new(),
            roles: 0,
            notability: 0.0,
            inventions: 0,
            stranger: false,
            predators_killed: 0,
            founded_camps: 0,
            prestige: 0.0,
            consider_move_day: 0,
            species: 0,
        }
    }

    pub fn new_camp(&mut self, place: u8, founder: u32, culture: u16) -> u16 {
        let id = self.next_camp;
        self.next_camp += 1;
        let pl = &self.world.places[place as usize];
        let (x, y) = (pl.cx + self.rng.range(-10.0, 10.0), pl.cy + self.rng.range(-8.0, 8.0));
        let name = if founder != u32::MAX {
            self.people[founder as usize].phon.coin(&mut self.rng)
        } else {
            self.proto_phon.coin(&mut self.rng)
        };
        self.world.camps.push(Camp {
            id,
            place,
            x,
            y,
            founded: self.tick,
            founder,
            name,
            structures: Vec::new(),
            abandoned: None,
            records: Vec::new(),
            members: 0,
            culture,
        });
        id
    }

    // ---------------------------------------------------------------- events & logs

    fn fauna_substeps(&mut self, n: u32, dt: f32) {
        let Some(mut evo) = self.evo.take() else { return };
        if !evo.creatures.is_empty() {
            let fcfg = self.cfg.fauna.clone();
            let mut fr = evo.frng.take().unwrap_or_else(|| Rng::new(2));
            for _ in 0..n {
                evo.fauna_step(&mut self.world, &fcfg, self.tick, dt, &mut fr);
            }
            evo.frng = Some(fr);
        }
        self.evo = Some(evo);
    }

    /// A herd whose genes reach capacity 0.35 becomes a band of individuals with full minds
    /// (DESIGN §5.4–5.5): the same Mind code as "start with people" tanks, limited by their genes.
    fn promote_minds(&mut self) {
        let Some(mut evo) = self.evo.take() else { return };
        let Some(k) = evo
            .creatures
            .iter()
            .enumerate()
            .filter(|(_, c)| evo.can_promote(c, self.tick, self.tpc()))
            .max_by(|a, b| a.1.g.capacity.partial_cmp(&b.1.g.capacity).unwrap().then(b.0.cmp(&a.0)))
            .map(|(k, _)| k)
        else {
            self.evo = Some(evo);
            return;
        };
        let herd = evo.creatures[k].clone();
        let n = herd.m.min(24.0).floor() as u32;
        evo.creatures[k].m -= n as f32;
        if evo.creatures[k].m < 1.0 {
            evo.creatures[k].alive = false;
        }
        evo.mind_species.entry(herd.species).or_insert(self.tick);
        let first_ever = evo.minds_began.is_none();
        if first_ever {
            evo.minds_began = Some(self.tick);
        }
        self.evo = Some(evo);
        let tpc = self.tpc();
        let culture = 1000 + herd.species as u16;
        let mut rng = Rng::stream(self.seed, tag::WORLD, self.tick ^ 0xA11CE);
        let phon = Phonology::generate(&mut rng, herd.g.lips, herd.g.tongue, herd.g.throat);
        let place = place_of(herd.x, herd.y);
        let camp = self.new_camp(place, u32::MAX, culture);
        let first = self.people.len() as u32;
        for _ in 0..n {
            let id = self.people.len() as u32;
            let mut prng = Rng::stream(self.seed, tag::PERSON, id as u64);
            let g = herd.g.mutant(&mut prng, 0.3, 0.03);
            let female = prng.chance(0.5);
            let age = prng.range(0.5, (g.max_age * 0.5).max(1.0));
            let born = self.tick.saturating_sub((age * tpc as f32) as u64);
            let mut p = self.make_person(id, g, female, born, culture, phon.clone(), prng);
            p.species = herd.species;
            p.home = Some(camp);
            let (cx, cy) = (self.world.camps[camp as usize].x, self.world.camps[camp as usize].y);
            p.from = (cx + p.rng.range(-6.0, 6.0), cy + p.rng.range(-6.0, 6.0));
            p.to = p.from;
            p.energy = herd.energy.max(0.5);
            p.stash = 1.0;
            p.name = loop {
                let w = phon.coin(&mut p.rng);
                if w.len() >= 3 {
                    break w;
                }
            };
            p.name_gloss = format!("of species A{}", herd.species);
            let own = p.name.clone();
            p.mind.lex.adopt(Concept::Person(id), own, 0.9);
            p.next_decision = self.tick + p.rng.below(64) as u64;
            p.act_start = self.tick;
            p.last_upd = self.tick;
            p.life_note(self.tick, format!("among the first of species A{} to think as an individual", herd.species));
            self.people.push(p);
            self.alive.push(id);
        }
        let last = self.people.len() as u32;
        self.world.camps[camp as usize].founder = first;
        // a herd knows its members
        let tick = self.tick;
        let cap = self.cfg.mind.model_cap;
        for a in first..last {
            for b in first..last {
                if a != b {
                    let pa = &mut self.people[a as usize];
                    let m = pa.mind.model_mut(b, tick, cap);
                    m.affinity = pa.rng.range(0.1, 0.4);
                    m.trust = 0.5;
                }
            }
        }
        if self.tracked.len() < 24 {
            self.tracked.extend(first..last.min(first + 6));
        }
        let cname = self.camp_name(camp);
        let text = if first_ever {
            format!("The first minds: {n} animals of species A{} begin to think as individuals, at {cname} in the {}", herd.species, self.place_name(place))
        } else {
            format!("Another band of thinking animals ({n}, species A{}) gathers at {cname}", herd.species)
        };
        self.emit(EvKind::EvolutionaryFirst, (first..last).take(4).collect(), place, text);
    }

    fn emit_fauna(&mut self, _t: u64, e: crate::fauna::FaunaEvent) {
        use crate::fauna::FaunaEvent::*;
        let (kind, text) = match e {
            FirstAnimals { species, x, y } => (EvKind::EvolutionaryFirst, format!("The first animals: a mat of grazing cells becomes swimming creatures (species A{species}) at the {}", self.place_name(place_of(x, y)))),
            NewSpecies { species, parent, what } => (EvKind::Speciation, format!("A new animal species A{species} ({what}) branches off from A{}", parent.unwrap_or(0))),
            Extinct { species, what, days } => (EvKind::SpeciesExtinct, format!("Animal species A{species} ({what}) dies out after {days} days")),
            FirstHunter { species } => (EvKind::EvolutionaryFirst, format!("The first hunters: species A{species} lives by eating other animals")),
            Cleverest { species, capacity } => (EvKind::EvolutionaryFirst, format!("A new height of intelligence: species A{species} reaches capacity {capacity:.2}")),
        };
        self.emit(kind, vec![], 0, text);
    }

    fn emit_evo(&mut self, _t: u64, e: crate::evo::EvoEvent) {
        use crate::evo::EvoEvent::*;
        let (kind, place, text) = match e {
            LifeBegins { cell, vent, .. } => {
                let (x, y) = World::cell_center(cell);
                let pl = place_of(x, y);
                (EvKind::LifeBegins, pl, format!("Life begins: a self-copying film appears {} at the {}", if vent { "at a hot vent" } else { "in the sunlit shallows" }, self.place_name(pl)))
            }
            Speciation { species, parent, kind } => (EvKind::Speciation, 0, match parent {
                Some(p) => format!("A new species ({species}) of {kind} branches off from species {p}"),
                None => format!("The first species ({species}) of {kind}"),
            }),
            Extinction { species, kind, lived_days } => (EvKind::SpeciesExtinct, 0, format!("Species {species} ({kind}) dies out after {lived_days} days")),
            MassExtinction { lost, of } => (EvKind::MassExtinction, 0, format!("Mass extinction: {lost} of {of} species lost within a cycle")),
            FirstOf { kind, species } => (EvKind::EvolutionaryFirst, 0, format!("For the first time in this tank: a {kind} (species {species})")),
        };
        self.emit(kind, vec![], place, text);
    }

    pub fn emit(&mut self, kind: EvKind, who: Vec<u32>, place: u8, text: String) {
        let mut imp = kind.base_importance();
        for w in &who {
            if let Some(p) = self.people.get(*w as usize) {
                imp += (p.notability * 0.02).min(0.3);
            }
        }
        let cycle = self.cycle();
        self.events.push(Event { tick: self.tick, cycle, kind, who, place, text, importance: imp.min(1.0) });
    }

    pub fn log_thought(&mut self, i: usize, th: Thought) {
        let cap = self.cfg.mind.thought_ring;
        let id = self.people[i].id;
        let keep = match self.thought_mode {
            ThoughtMode::None => false,
            ThoughtMode::All => true,
            ThoughtMode::Tracked => self.tracked.contains(&id) || self.people[i].notability >= 8.0,
        };
        if keep {
            self.thought_out.push(th.clone());
        }
        let p = &mut self.people[i];
        if p.thoughts.len() >= cap {
            p.thoughts.remove(0);
        }
        p.thoughts.push(th);
    }

    pub fn episode(&mut self, i: usize, kind: EpKind, other: Option<u32>, valence: f32, salience: f32) {
        let place = {
            let (x, y) = self.people[i].pos(self.tick);
            place_of(x, y)
        };
        let cap = self.cfg.mind.episodic_cap;
        let tick = self.tick;
        self.people[i].mind.remember(Episode { tick, kind, place, other, valence, salience }, cap);
    }

    // ---------------------------------------------------------------- main loop

    pub fn run_cycles(&mut self, cycles: u64) {
        let end = self.tick0 + cycles * self.tpc();
        while self.tick < end && !self.extinct {
            self.step();
        }
    }

    pub fn step(&mut self) {
        // deep time: with no people, the world advances a day at a time (animals in 4 sub-steps)
        if self.evo.is_some() && self.alive.is_empty() {
            self.fauna_substeps(4, 0.25);
            let next_day = (self.tick / self.tpd() + 1) * self.tpd();
            self.tick = next_day;
            self.new_day();
            return;
        }
        let step = self.cfg.time.step_ticks;
        let before_day = self.tick / self.tpd();
        // animals keep living alongside minds: a quarter-day sub-step every 256 ticks
        if self.evo.is_some() && (self.tick % 256) < step {
            self.fauna_substeps(1, 0.25);
        }
        self.tick += step;
        if self.tick / self.tpd() != before_day {
            self.new_day();
        }
        self.rebuild_index();
        self.predators_step();
        self.threats_step();
        self.resolve_tasks();
        let now = self.tick;
        let ids: Vec<u32> = self.alive.clone();
        for id in ids {
            let i = id as usize;
            if self.people[i].alive && self.people[i].next_decision <= now {
                self.complete(i);
                if self.people[i].alive {
                    self.decide(i, None);
                }
            }
        }
    }

    pub fn rebuild_index(&mut self) {
        if self.at_place.len() != N_PLACES {
            self.at_place = vec![Vec::new(); N_PLACES];
        }
        for v in self.at_place.iter_mut() {
            v.clear();
        }
        let t = self.tick;
        for &id in &self.alive {
            let (x, y) = self.people[id as usize].pos(t);
            self.at_place[place_of(x, y) as usize].push(id);
        }
    }

    /// People (alive) within `r` of (x,y), using the place index.
    pub fn near(&self, x: f32, y: f32, r: f32) -> Vec<u32> {
        let p = place_of(x, y) as usize;
        let (px, py) = ((p % PW) as isize, (p / PW) as isize);
        let mut out = Vec::new();
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (nx, ny) = (px + dx, py + dy);
                if nx < 0 || ny < 0 || nx >= PW as isize || ny >= PH as isize {
                    continue;
                }
                for &id in &self.at_place[(ny as usize) * PW + nx as usize] {
                    let (qx, qy) = self.people[id as usize].pos(self.tick);
                    if dist(x, y, qx, qy) <= r {
                        out.push(id);
                    }
                }
            }
        }
        out
    }

    pub fn place_of_person(&self, i: usize) -> u8 {
        let (x, y) = self.people[i].pos(self.tick);
        place_of(x, y)
    }

    pub fn at_home(&self, i: usize) -> bool {
        match self.people[i].home {
            Some(c) => {
                let camp = &self.world.camps[c as usize];
                let (x, y) = self.people[i].pos(self.tick);
                dist(x, y, camp.x, camp.y) < 14.0
            }
            None => false,
        }
    }

    // ---------------------------------------------------------------- daily

    pub fn new_day(&mut self) {
        let dpc = self.cfg.time.days_per_cycle();
        let day = self.day();
        let cycle = self.cycle();
        let din = day % dpc;
        if din == 0 && day > 0 {
            self.end_cycle(cycle - 1);
        }
        let ds = self.ds.clone().expect("dataset");
        let c = self.climate.day(cycle, din, dpc, &ds);
        // extreme weather events
        let anomaly = c.real_tmean_c - ds.clim_tmean[c.real_doy as usize];
        if anomaly.abs() > 11.0 && day >= self.last_extreme_day + 8 {
            self.last_extreme_day = day;
            let kind = if anomaly < 0.0 { "cold snap" } else { "heat wave" };
            let text = format!(
                "A {kind} grips the tank (mirroring Chicago, {} day {}: {:.0}°C, {:+.0}° from normal)",
                c.real_year, c.real_doy, c.real_tmean_c, anomaly
            );
            self.emit(EvKind::ClimateExtreme, vec![], 0, text);
        }
        self.world.prev_climate = std::mem::replace(&mut self.world.climate, c);
        if let Some(mut evo) = self.evo.take() {
            let ecfg = self.cfg.evolution.clone();
            evo.daily(&mut self.world, &ecfg, self.tick, day);
            let fcfg = self.cfg.fauna.clone();
            let mut fr = evo.frng.take().unwrap_or_else(|| Rng::new(2));
            evo.animal_origins(&fcfg, self.tick, &mut fr);
            let promote = evo.creatures.iter().any(|c| evo.can_promote(c, self.tick, self.tpc())) && fr.chance(0.05) && self.alive.len() < 400;
            if day % 12 == 6 {
                evo.fauna_species(&fcfg, self.tick);
            }
            evo.frng = Some(fr);
            for (t, e) in std::mem::take(&mut evo.fauna_events) {
                self.emit_fauna(t, e);
            }
            for (t, e) in std::mem::take(&mut evo.events) {
                self.emit_evo(t, e);
            }
            self.evo = Some(evo);
            if promote {
                self.promote_minds();
            }
        }
        self.world.update_foul(&self.cfg.world);
        self.world.daily(&self.cfg.world);
        self.counters.algae_grown += self.world.grown_today.0;
        self.counters.shell_grown += self.world.grown_today.1;
        crate::scenario::apply_day(self, day);
        self.herds_daily();
        self.predators_daily();
        self.disease_daily();
        self.people_daily();
        self.knowledge_census();
        if day % self.cfg.observer.group_every_days == 0 {
            crate::social::detect_groups(self);
        }
        if self.alive.is_empty() && !self.extinct && self.evo.is_none() {
            self.extinct = true;
            self.emit(EvKind::Extinction, vec![], 0, "The last of the people is gone.".into());
        }
    }

    fn end_cycle(&mut self, cycle: u64) {
        crate::social::cycle_observe(self, cycle);
        // predator immigration (a trickle from the wider tank)
        let imm = self.cfg.predators.immigration_per_cycle;
        let alive_preds = self.world.predators.iter().filter(|p| p.alive).count() as u32;
        // gulpers drift in from the wider tank only where there is prey to support them
        let herd_animals: f32 = self.world.herds.iter().map(|h| h.count).sum();
        let prey = (herd_animals / (self.cfg.world.herds as f32 * 10.0)).min(1.0);
        let imm = if alive_preds < self.cfg.predators.initial { imm.max(0.8) * prey } else { imm * prey };
        if alive_preds < self.cfg.predators.max && self.pred_rng.chance(imm) {
            let r = &mut self.pred_rng;
            self.world.spawn_predator(r, None);
        }
        self.world.attacks_by_place_prev = std::mem::replace(&mut self.world.attacks_by_place, vec![0; N_PLACES]);
        self.world.predators.retain(|p| p.alive);
        let alive_people = self.alive.clone();
        let people = &self.people;
        let is_alive = |id: u32| people.get(id as usize).map_or(false, |p| p.alive);
        let mut prunes: Vec<(usize, Lexicon)> = Vec::new();
        for &id in &alive_people {
            let p = &people[id as usize];
            let lcap = self.cfg.mind.lexicon_cap.min(p.mind.caps.lexicon);
            if p.mind.lex.len() <= lcap {
                continue;
            }
            let mut lex = p.mind.lex.clone();
            let known = |q: u32| p.mind.model(q).is_some();
            lex.prune(lcap, is_alive, known);
            prunes.push((id as usize, lex));
        }
        for (i, lex) in prunes {
            self.people[i].mind.lex = lex;
        }
        self.tasks.retain(|t| t.end + 2048 > self.tick);
    }

    fn herds_daily(&mut self) {
        let g = self.cfg.world.herd_growth;
        let max = self.cfg.world.herd_max;
        let ridge = self.world.ridge_x;
        let foods: Vec<f32> = self.world.herds.iter().map(|h| (self.world.cells[crate::world::cell_index(h.x, h.y)].algae / 20.0).min(1.0)).collect();
        for (hk, h) in self.world.herds.iter_mut().enumerate() {
            if h.count < 1.0 {
                h.count = 0.0;
                continue;
            }
            let food = foods[hk];
            h.count += g * h.count * (1.0 - h.count / max) * (0.3 + food);
            if dist(h.x, h.y, h.tx, h.ty) < 5.0 || self.rng.chance(0.1) {
                h.tx = (h.x + self.rng.range(-60.0, 60.0)).clamp(10.0, crate::world::TANK_W - 10.0);
                h.ty = (h.y + self.rng.range(-40.0, 40.0)).clamp(10.0, crate::world::TANK_H - 10.0);
                if (h.x > ridge) != (h.tx > ridge) {
                    h.tx = h.x;
                }
            }
            let d = dist(h.x, h.y, h.tx, h.ty).max(1e-3);
            let s = 12.0f32.min(d);
            h.x += (h.tx - h.x) / d * s;
            h.y += (h.ty - h.y) / d * s;
        }
        // rare herd re-seeding from outside (drifters wander in)
        for h in self.world.herds.iter_mut() {
            if h.count < 1.0 && self.rng.chance(0.03) {
                h.count = 4.0;
            }
        }
    }

    fn predators_daily(&mut self) {
        let pc = self.cfg.predators.clone();
        let tpc = self.tpc() as f32;
        let tpd = self.tpd() as f32;
        let mut births = Vec::new();
        for p in self.world.predators.iter_mut().filter(|p| p.alive) {
            p.age_days += 1.0;
            p.energy -= pc.energy_per_day;
            if p.energy <= 0.0 || p.age_days * tpd > pc.lifespan_cycles * tpc {
                p.alive = false;
                continue;
            }
            if p.energy > pc.breed_energy && p.age_days > 96.0 {
                p.energy -= 1.4;
                births.push((p.x, p.y));
            }
        }
        let alive_n = self.world.predators.iter().filter(|p| p.alive).count() as u32;
        for (k, (x, y)) in births.into_iter().enumerate() {
            if alive_n + (k as u32) < pc.max {
                let r = &mut self.pred_rng;
                self.world.spawn_predator(r, Some((x, y)));
            }
        }
    }

    fn predators_step(&mut self) {
        let step = self.cfg.time.step_ticks as f32;
        let speed = self.cfg.predators.speed * step;
        let sense = self.cfg.predators.sense;
        let night = self.is_night(self.tick);
        let retarget = (self.tick / self.cfg.time.step_ticks) % 4 == 0;
        let tick = self.tick;
        for k in 0..self.world.predators.len() {
            if !self.world.predators[k].alive {
                continue;
            }
            let (px, py) = (self.world.predators[k].x, self.world.predators[k].y);
            if self.world.predators[k].cooldown > tick {
                continue;
            }
            if let Some(t) = self.world.predators[k].target {
                if !self.people[t as usize].alive {
                    self.world.predators[k].target = None;
                }
            }
            let hungry = self.world.predators[k].energy < 0.9;
            if retarget && self.world.predators[k].target.is_none() && hungry {
                let r = sense * if night { 1.25 } else { 1.0 };
                let cands = self.near(px, py, r);
                let mut best = None;
                let mut bd = f32::MAX;
                for id in cands {
                    let p = &self.people[id as usize];
                    let (x, y) = p.pos(tick);
                    // walled camps deter
                    if let Some(h) = p.home {
                        let c = &self.world.camps[h as usize];
                        if c.has(n::WALL) && dist(x, y, c.x, c.y) < 16.0 && self.pred_rng.chance(0.8) {
                            continue;
                        }
                    }
                    // crowds deter: gulpers take the isolated, or anyone in the dark
                    let crowd = self.at_place[place_of(x, y) as usize].iter().filter(|&&o| o != id && {
                        let (ox, oy) = self.people[o as usize].pos(tick);
                        dist(x, y, ox, oy) < 14.0
                    }).count();
                    if crowd >= 3 && !(night && crowd < 6) {
                        continue;
                    }
                    let d = dist(px, py, x, y) + if self.adult(id as usize) { 0.0 } else { -8.0 };
                    if d < bd {
                        bd = d;
                        best = Some(id);
                    }
                }
                // most of the time a drifter herd is the easier meal
                let herd_near = self.world.herds.iter().any(|h| h.count >= 1.0 && dist(px, py, h.x, h.y) < sense * 2.5);
                if best.is_some() && herd_near && self.pred_rng.chance(0.6) {
                    best = None;
                }
                self.world.predators[k].target = best;
                if best.is_none() {
                    if let Some(h) = self.world.herds.iter().filter(|h| h.count >= 1.0).min_by(|a, b| dist(px, py, a.x, a.y).partial_cmp(&dist(px, py, b.x, b.y)).unwrap().then(a.id.cmp(&b.id))) {
                        if dist(px, py, h.x, h.y) < sense * 2.5 {
                            let (hx, hy) = (h.x, h.y);
                            self.world.predators[k].tx = hx;
                            self.world.predators[k].ty = hy;
                        }
                    }
                }
            }
            let pr = &self.world.predators[k];
            let (tx, ty, has_t) = match pr.target {
                Some(t) => {
                    let (x, y) = self.people[t as usize].pos(tick);
                    (x, y, true)
                }
                None => (pr.tx, pr.ty, false),
            };
            let d = dist(px, py, tx, ty);
            if has_t && d < 3.0 {
                let victim = self.world.predators[k].target.unwrap();
                self.attack(k, victim);
                continue;
            }
            let pr = &mut self.world.predators[k];
            if !has_t && d < 5.0 {
                // reached its waypoint: eat a drifter if one is here, then wander
                if hungry {
                    for h in self.world.herds.iter_mut() {
                        if h.count >= 1.0 && dist(px, py, h.x, h.y) < 12.0 && self.pred_rng.chance(0.6) {
                            h.count -= 1.0;
                            pr.energy += 1.0;
                            pr.cooldown = tick + 2048;
                            break;
                        }
                    }
                }
                pr.tx = (px + self.pred_rng.range(-80.0, 80.0)).clamp(5.0, crate::world::TANK_W - 5.0);
                pr.ty = (py + self.pred_rng.range(-60.0, 60.0)).clamp(5.0, crate::world::TANK_H - 5.0);
                continue;
            }
            let s = speed.min(d).max(0.0);
            if d > 1e-3 {
                pr.x += (tx - px) / d * s;
                pr.y += (ty - py) / d * s;
            }
        }
    }

    fn attack(&mut self, k: usize, victim: u32) {
        let tick = self.tick;
        let vi = victim as usize;
        let (x, y) = self.people[vi].pos(tick);
        let place = place_of(x, y);
        let night = self.is_night(tick);
        let witnesses: Vec<u32> = self.near(x, y, 30.0).into_iter().filter(|&w| w != victim).collect();
        let defenders = witnesses.len().min(6) as f32;
        let p = &self.people[vi];
        let mut defense = 0.15 * p.genome.size + 0.05 * defenders;
        if p.knows(n::SPEAR) {
            defense += 0.25;
        }
        if night && p.knows(n::GLOWLAMP) {
            defense += 0.1;
        }
        if !self.adult(vi) {
            defense *= 0.4;
        }
        let kill = self.cfg.predators.kill_chance * (1.0 - defense).max(0.05);
        let roll = self.pred_rng.f32();
        self.world.attacks_by_place[place as usize] += 1;
        self.counters.attacks += 1;
        let pr = &mut self.world.predators[k];
        pr.target = None;
        pr.cooldown = tick + 1024;
        let vname = self.name(victim);
        if roll < kill {
            self.world.predators[k].energy += 0.8;
            self.emit(EvKind::PredatorAttack, vec![victim], place, format!("A gulper took {vname} at the {}", self.place_name(place)));
            self.die(vi, "taken by a gulper");
        } else {
            self.world.predators[k].energy += 0.1;
            let pr = &mut self.world.predators[k];
            pr.tx = (pr.x + self.pred_rng.range(-90.0, 90.0)).clamp(5.0, crate::world::TANK_W - 5.0);
            pr.ty = (pr.y + self.pred_rng.range(-60.0, 60.0)).clamp(5.0, crate::world::TANK_H - 5.0);
            let p = &mut self.people[vi];
            p.health -= 0.3;
            p.mind.feel(emo::FEAR, 0.8);
            p.mind.today_out |= 1 << outcome::ATTACK;
            p.needs[need::PREDATOR] += 1.0;
            let pm = &mut p.mind.places[place as usize];
            pm.danger = (pm.danger + 0.5).min(1.0);
            pm.danger_src = 0;
            pm.danger_distorted = false;
            self.episode(vi, EpKind::Attacked, None, -0.9, 0.95);
            if self.people[vi].health <= 0.0 {
                self.emit(EvKind::PredatorAttack, vec![victim], place, format!("{vname} died of wounds from a gulper"));
                self.die(vi, "wounds from a gulper");
            } else {
                self.emit(EvKind::PredatorAttack, vec![victim], place, format!("{vname} survived a gulper attack"));
            }
        }
        for w in witnesses {
            let wi = w as usize;
            let close = self.people[wi].mind.model(victim).map_or(0.0, |m| m.affinity.max(0.0) + if m.kin != kin::NONE { 0.5 } else { 0.0 });
            let p = &mut self.people[wi];
            p.mind.feel(emo::FEAR, 0.4 + close * 0.3);
            p.mind.today_out |= 1 << outcome::ATTACK;
            p.needs[need::PREDATOR] += 0.5;
            let pm = &mut p.mind.places[place as usize];
            pm.danger = (pm.danger + 0.3).min(1.0);
            pm.danger_src = 0;
            pm.danger_distorted = false;
            self.episode(wi, EpKind::SawAttack, Some(victim), -0.6, 0.5 + close * 0.3);
        }
    }

    /// Perception of predators: may interrupt what people are doing.
    fn threats_step(&mut self) {
        let tick = self.tick;
        let night = self.is_night(tick);
        let mut alerts: Vec<(u32, u32)> = Vec::new();
        for k in 0..self.world.predators.len() {
            let pr = &self.world.predators[k];
            if !pr.alive {
                continue;
            }
            let (px, py, pid) = (pr.x, pr.y, pr.id);
            let hunting = pr.target.map(|t| self.people[t as usize].pos(tick));
            let mut seen_only = Vec::new();
            for id in self.near(px, py, 60.0) {
                let p = &self.people[id as usize];
                if p.alarm_until > tick {
                    continue;
                }
                let (x, y) = p.pos(tick);
                let mut range = 20.0 + 35.0 * p.genome.vision;
                if night {
                    range *= if p.knows(n::GLOWLAMP) { 0.9 } else { 0.5 };
                }
                let d = dist(x, y, px, py);
                if d <= range && self.pred_rng.chance(0.6) {
                    let menacing = d < 22.0 || hunting.map_or(false, |(hx, hy)| dist(x, y, hx, hy) < 35.0);
                    if menacing {
                        alerts.push((id, pid));
                    } else {
                        seen_only.push(id);
                    }
                }
            }
            for id in seen_only {
                self.people[id as usize].saw_predator_today = true;
            }
        }
        let mut alerted: Vec<u32> = Vec::new();
        for (id, pid) in alerts {
            if alerted.contains(&id) {
                continue;
            }
            alerted.push(id);
            let i = id as usize;
            self.people[i].saw_predator_today = true;
            self.people[i].alarm_until = tick + 160;
            // alarm call: knowers alert people around them
            if self.people[i].knows(n::ALARM) {
                self.use_know(i, n::ALARM);
                let (x, y) = self.people[i].pos(tick);
                for o in self.near(x, y, 45.0) {
                    if o != id && !alerted.contains(&o) && self.people[o as usize].alarm_until <= tick {
                        alerted.push(o);
                        self.people[o as usize].saw_predator_today = true;
                        self.people[o as usize].alarm_until = tick + 160;
                        self.interrupt(o as usize, Some(pid));
                    }
                }
            }
            self.interrupt(i, Some(pid));
        }
    }

    pub fn interrupt(&mut self, i: usize, threat: Option<u32>) {
        if !self.people[i].alive {
            return;
        }
        // leaving whatever we were doing (a hunt partner may notice)
        let (x, y) = self.people[i].pos(self.tick);
        self.people[i].from = (x, y);
        self.people[i].to = (x, y);
        self.decide(i, threat);
    }

    fn disease_daily(&mut self) {
        let dc = self.cfg.disease.clone();
        let tick = self.tick;
        let tpd = self.tpd();
        let dpc = self.cfg.time.days_per_cycle() as f32;
        let max_foul = self.world.foul.iter().fold(0.0f32, |a, b| a.max(*b));
        if !self.alive.is_empty() && self.dis_rng.chance(dc.spontaneous_per_cycle / dpc * (1.0 + 4.0 * max_foul)) {
            // fevers tend to break out where the water is foulest
            let foulest = (0..N_PLACES).max_by(|a, b| self.world.foul[*a].partial_cmp(&self.world.foul[*b]).unwrap().then(b.cmp(a))).unwrap_or(0);
            let cands: Vec<u32> = if max_foul > 0.2 && !self.at_place[foulest].is_empty() { self.at_place[foulest].clone() } else { self.alive.clone() };
            let k = self.dis_rng.below(cands.len() as u32) as usize;
            let id = cands[k];
            if self.people[id as usize].immune_until <= tick && self.people[id as usize].infected.is_none() {
                self.infect(id as usize);
                let place = self.place_of_person(id as usize);
                self.emit(EvKind::Outbreak, vec![id], place, format!("A shivering fever appears: {} falls sick", self.name(id)));
            }
        }
        // transmission within places
        for pl in 0..N_PLACES {
            let here = self.at_place[pl].clone();
            let n_inf = here.iter().filter(|&&id| self.people[id as usize].infected.is_some()).count();
            if n_inf == 0 {
                continue;
            }
            let crowd = 1.0 + self.cfg.world.crowd_disease * (here.len() as f32 / 40.0 + 2.0 * self.world.foul[pl]);
            for &id in &here {
                let p = &mut self.people[id as usize];
                if !p.alive {
                    continue;
                }
                p.mind.today_cues |= 1 << cue::SICK_NEARBY;
                if p.infected.is_some() || p.immune_until > tick {
                    continue;
                }
                let careful = p.knows(n::QUARANTINE) || p.mind.belief(Concept::Kind(Kind::Sickness), Rel::Spreads) > 0.5;
                if let Some(k) = p.knowledge.iter_mut().find(|k| k.node == n::QUARANTINE) {
                    k.prof = (k.prof + 0.03).min(1.0);
                }
                let exposure = if careful { 0.3 } else { 1.0 };
                let pinf = 1.0 - libm::powf(1.0 - (dc.beta * exposure * crowd).min(0.9), n_inf as f32);
                if self.dis_rng.chance(pinf) {
                    self.infect(id as usize);
                }
            }
        }
        // progression
        let ids = self.alive.clone();
        for id in ids {
            let i = id as usize;
            let Some(since) = self.people[i].infected else { continue };
            let days = (tick - since) / tpd;
            let healed = self.people[i].healed_today;
            let cooked = self.people[i].knows(n::VENT_COOK);
            let p = &mut self.people[i];
            p.health -= 0.03;
            p.needs[need::SICK] += 0.5;
            let mort = dc.mortality_per_day * if healed { 0.35 } else { 1.0 } * if cooked { 0.8 } else { 1.0 } * (1.6 - p.health).max(0.3);
            let dur = if healed { dc.duration_days * 2 / 3 } else { dc.duration_days };
            if self.dis_rng.chance(mort) || p.health <= 0.0 {
                self.die(i, "the shivering fever");
            } else if days >= dur {
                let p = &mut self.people[i];
                p.infected = None;
                p.immune_until = tick + (dc.immunity_cycles * self.cfg.time.ticks_per_cycle() as f32) as u64;
                self.episode(i, EpKind::Recovered, None, 0.5, 0.3);
            }
            if self.people[i].alive {
                self.people[i].healed_today = false;
            }
        }
    }

    pub fn infect(&mut self, i: usize) {
        let tick = self.tick;
        let p = &mut self.people[i];
        p.infected = Some(tick);
        p.mind.today_out |= 1 << outcome::SICKNESS;
        p.mind.feel(emo::FEAR, 0.3);
        self.counters.infections += 1;
        self.episode(i, EpKind::FellSick, None, -0.5, 0.5);
    }

    fn people_daily(&mut self) {
        let pc = self.cfg.people.clone();
        let tick = self.tick;
        let tpc = self.tpc();
        let ids = self.alive.clone();
        let c = self.world.climate.clone();
        let pc_prev = self.world.prev_climate.clone();
        let dpc = self.cfg.time.days_per_cycle();
        let din = self.day() % dpc;
        let season = (din / self.cfg.time.days_per_season) as usize;
        let late = din % self.cfg.time.days_per_season >= self.cfg.time.days_per_season * 2 / 3;
        let mut world_cues: u32 = 1 << (cue::SEASON0 + season);
        if c.lamp_frac < pc_prev.lamp_frac - 0.001 {
            world_cues |= 1 << cue::LAMP_SHORTENING;
        }
        if c.lamp_frac > pc_prev.lamp_frac + 0.001 {
            world_cues |= 1 << cue::LAMP_LENGTHENING;
        }
        if c.surface_t < pc_prev.surface_t - 0.5 {
            world_cues |= 1 << cue::TEMP_FALLING;
        }
        if c.surface_t > pc_prev.surface_t + 0.5 {
            world_cues |= 1 << cue::TEMP_RISING;
        }
        if c.storm {
            world_cues |= 1 << cue::STORM;
        }
        if c.ice > 0.2 {
            world_cues |= 1 << cue::ICE;
        }
        if c.brightness < 0.7 {
            world_cues |= 1 << cue::DIM;
        }
        let mcfg = self.cfg.mind.clone();
        for id in ids {
            let i = id as usize;
            if !self.people[i].alive {
                continue;
            }
            self.update_energy(i);
            let age = self.people[i].age_cycles(tick, tpc);
            let adult = age >= pc.adult_cycles;
            // mortality: age
            let max_age = self.people[i].genome.max_age;
            if age > max_age && self.people[i].rng.chance(0.05) || age > max_age * 0.85 && self.people[i].rng.chance(0.004) {
                self.die(i, "old age");
                continue;
            }
            // temperature
            let (x, y) = self.people[i].pos(tick);
            let temp = self.world.cells[crate::world::cell_index(x, y)].temp;
            let comfort = pc.temp_comfort + (self.people[i].genome.temp_opt - 0.5) * 4.0 - (self.people[i].genome.temp_tol - 0.5) * 3.0;
            let mut cues = world_cues;
            if temp < comfort {
                cues |= 1 << cue::COLD;
                let mut shelter = 0.0f32;
                if let Some(h) = self.people[i].home {
                    let camp = &self.world.camps[h as usize];
                    if camp.has(n::HALL) {
                        shelter = 0.7;
                    } else if camp.has(n::BURROW) {
                        shelter = 0.4;
                    }
                }
                let stress = ((comfort - temp) / 6.0).min(1.5) * (1.0 - shelter);
                if shelter > 0.0 {
                    self.use_know(i, if shelter > 0.5 { n::HALL } else { n::BURROW });
                }
                if stress > 0.05 {
                    let p = &mut self.people[i];
                    p.health -= pc.cold_damage * stress * if adult { 1.0 } else { 1.6 };
                    p.energy = (p.energy - 0.12 * stress).max(0.0); // burning food to keep warm
                    p.needs[need::COLD] += stress;
                    p.mind.today_out |= 1 << outcome::COLD_STRESS;
                    if stress > 0.5 && p.rng.chance(0.2) {
                        self.episode(i, EpKind::Cold, None, -0.4, 0.3);
                    }
                }
            } else if temp > comfort + 8.0 {
                cues |= 1 << cue::WARM;
            }
            // hunger
            {
                let p = &mut self.people[i];
                if p.energy <= 0.01 {
                    p.health -= pc.starve_damage;
                    p.needs[need::HUNGER] += 1.0;
                }
                if p.energy < 0.25 {
                    p.mind.today_out |= 1 << outcome::HUNGER;
                    p.needs[need::HUNGER] += 0.3;
                } else if p.energy > 0.7 && p.stash > 4.0 {
                    p.mind.today_out |= 1 << outcome::PLENTY;
                }
                if p.health <= 0.0 {
                    let cause = if p.mind.today_out & (1 << outcome::COLD_STRESS) != 0 {
                        "cold and hunger"
                    } else if p.energy <= 0.01 {
                        "starvation"
                    } else {
                        "injuries"
                    };
                    self.die(i, cause);
                    continue;
                }
                if p.energy > 0.3 && p.infected.is_none() && p.mind.today_out & (1 << outcome::COLD_STRESS) == 0 {
                    p.health = (p.health + 0.04).min(1.0);
                }
                // spoilage
                let mut rate = self.cfg.world.spoil_rate;
                if p.knows(n::JAR) {
                    rate *= 0.2;
                } else if p.knows(n::CACHE) {
                    rate *= 0.5;
                }
                if p.stash > 1.0 {
                    for k in p.knowledge.iter_mut().filter(|k| k.node == n::CACHE || k.node == n::JAR) {
                        k.prof = (k.prof + 0.004).min(1.0);
                    }
                }
                let spoiled = p.stash * rate;
                p.stash -= spoiled;
                self.counters.food_spoiled += spoiled;
                if spoiled > 0.25 {
                    p.needs[need::SPOILED] += spoiled;
                }
                if p.stash < 1.5 {
                    cues |= 1 << cue::STASH_LOW;
                }
                if p.saw_predator_today {
                    cues |= 1 << cue::PREDATOR_SEEN;
                }
                p.saw_predator_today = false;
            }
            if let Some(h) = self.people[i].home {
                // crowded: many mouths for what the home grounds seem to offer
                let camp = &self.world.camps[h as usize];
                let food_here = self.people[i].mind.places[camp.place as usize].food;
                if camp.members > 25 || (camp.members > 10 && food_here < 0.3) {
                    cues |= 1 << cue::CROWDED;
                }
            }
            if late && (self.people[i].knows(n::LAMPWATCH) || self.people[i].knows(n::CALENDAR)) {
                cues |= 1 << cue::SEASON_LATE;
                self.use_know(i, n::LAMPWATCH);
                self.use_know(i, n::CALENDAR);
            }
            // infants are fed by parents with food
            // parents and kin feed infants, and hungry children until they are grown
            if age < 2.0 || (!adult && self.people[i].energy < 0.5) {
                self.feed_infant(i);
            }
            // pregnancy
            if let Some((due, father)) = self.people[i].pregnant {
                if tick >= due {
                    self.people[i].pregnant = None;
                    self.birth(i, father);
                }
            }
            self.try_conceive(i, adult, age);
            self.people[i].prestige *= 0.985;
            self.update_identity(i, tick);
            // beliefs fade unless reinforced; hearsay and superstition fade faster
            {
                let m = &mut self.people[i].mind;
                for b in m.beliefs.iter_mut() {
                    let fade = if b.rel == Rel::Forbidden {
                        0.001 // norms are cultural and slow to fade
                    } else if b.rel == Rel::Cursed || b.source.is_some() {
                        0.006
                    } else {
                        0.002
                    };
                    b.conf -= fade;
                }
                m.beliefs.retain(|b| b.conf > 0.05);
            }
            // unpractised skills fade; practices and beliefs fade more slowly
            {
                let p = &mut self.people[i];
                let mut forgot = Vec::new();
                for k in p.knowledge.iter_mut() {
                    let kind = NODES[k.node as usize].kind;
                    let rate = match kind {
                        crate::discovery::NodeKind::Practice | crate::discovery::NodeKind::Belief | crate::discovery::NodeKind::Institution => 0.0012,
                        _ => 0.0025,
                    };
                    k.prof -= rate;
                    if k.prof < 0.05 {
                        forgot.push(k.node);
                    }
                }
                if !forgot.is_empty() {
                    p.knowledge.retain(|k| k.prof >= 0.05);
                    for f in forgot {
                        let txt = format!("forgot the {}", NODES[f as usize].name);
                        p.life_note(tick, txt);
                    }
                }
            }
            // drives, emotions, memory upkeep
            let dependents = self.people[i].children.iter().filter(|&&c| self.people[c as usize].alive && self.people[c as usize].age_cycles(tick, tpc) < 4.0).count() as f32;
            let knows_lampwatch = self.people[i].knows(n::LAMPWATCH);
            let knows_rites = self.people[i].knows(n::BURIAL) || self.people[i].knows(n::ANCESTOR);
            let p = &mut self.people[i];
            let m = &mut p.mind;
            m.drives[drv::HUNGER] = 1.0 - p.energy;
            m.drives[drv::SAFETY] = (m.emo[emo::FEAR] * 0.7 + m.drives[drv::SAFETY] * 0.3).min(1.0);
            m.drives[drv::BELONGING] = (m.drives[drv::BELONGING] + 0.08 * (0.5 + m.traits[1])).min(1.0);
            m.drives[drv::STATUS] = (m.drives[drv::STATUS] * 0.9 + 0.02 * (0.3 + p.genome.dominance + m.emo[emo::SHAME])).min(1.0);
            // keep relationship values bounded; unseen acquaintances fade
            for md in m.models.iter_mut() {
                md.affinity = md.affinity.clamp(-1.0, 1.0);
                md.trust = md.trust.clamp(0.0, 1.0);
                md.respect = md.respect.clamp(0.0, 1.0);
                md.ledger = md.ledger.clamp(-10.0, 10.0);
                md.grudge *= 0.995;
                if tick.saturating_sub(md.last_seen) > 4 * self.cfg.time.ticks_per_cycle() && md.kin == kin::NONE {
                    md.affinity *= 0.99;
                }
                for s in md.seen.iter_mut() {
                    *s *= 0.995;
                }
            }
            m.drives[drv::CURIOSITY] = (m.drives[drv::CURIOSITY] + 0.06 * (0.3 + m.traits[2])).min(1.0);
            m.drives[drv::REPRO] = if adult && p.mate.is_none() && age < p.genome.max_age * 0.7 { (m.drives[drv::REPRO] + 0.05).min(1.0) } else { 0.0 };
            // (tiredness accrues continuously in update_energy)
            m.drives[drv::CARE] = (m.drives[drv::CARE] * 0.5 + dependents * 0.12).min(0.9);
            for e in m.emo.iter_mut() {
                *e *= mcfg.emotion_decay;
            }
            for k in 0..p.needs.len() {
                p.needs[k] *= 0.94;
            }
            for a in p.activity.iter_mut() {
                *a *= 0.985;
            }
            for pm in m.places.iter_mut() {
                pm.danger += (0.1 - pm.danger) * 0.01;
                pm.conf *= 0.995;
            }
            for t in 0..m.traits.len() {
                m.traits[t] += (m.base_traits[t] - m.traits[t]) * 0.002;
            }
            m.today_cues |= cues;
            let (tc, to) = (m.today_cues, m.today_out);
            m.cont.end_day(tc, to, mcfg.contingency_decay);
            m.expect = m.cont.expect(tc, mcfg.rule_min_dp + m.caps.rule_margin(), mcfg.rule_min_n);
            m.today_cues = 0;
            m.today_out = 0;
            m.cont.told.retain(|t| t.2 > 0.3);
            for t in m.cont.told.iter_mut() {
                t.2 *= 0.995;
            }
            // a learned rule "sick-nearby → sickness" becomes a belief that sickness spreads
            let (dp, _, nn) = m.cont.delta_p(cue::SICK_NEARBY, outcome::SICKNESS);
            if dp > 0.25 && nn >= 4.0 {
                m.believe(Concept::Kind(Kind::Sickness), Rel::Spreads, 0.1, None, id, tick, false, mcfg.belief_cap);
            }
            // a learned seasonal rule ties the Lamp to the seasons
            let (dp2, _, nn2) = m.cont.delta_p(cue::LAMP_SHORTENING, outcome::HUNGER);
            if dp2 > 0.25 && nn2 >= 6.0 && knows_lampwatch {
                m.believe(Concept::Kind(Kind::Lamp), Rel::Governs, 0.08, None, id, tick, false, mcfg.belief_cap);
            }
            // grief resolves faster with burial / rites
            let g = m.emo[emo::GRIEF];
            if g > 0.0 && knows_rites {
                m.emo[emo::GRIEF] = g * 0.85;
            }
        }
        self.alive.retain(|&id| self.people[id as usize].alive);
        // camp membership & upkeep
        for c in self.world.camps.iter_mut() {
            c.members = 0;
        }
        for &id in &self.alive {
            if let Some(h) = self.people[id as usize].home {
                self.world.camps[h as usize].members += 1;
            }
        }
        let mut abandoned = Vec::new();
        for c in self.world.camps.iter_mut() {
            for s in c.structures.iter_mut() {
                s.condition -= if c.abandoned.is_some() { 0.02 } else { 0.004 };
            }
            c.structures.retain(|s| s.condition > 0.0);
            if c.members == 0 && c.abandoned.is_none() {
                c.abandoned = Some(tick);
                abandoned.push((c.id, c.place));
            }
            if c.members > 0 && c.abandoned.is_some() {
                c.abandoned = None;
            }
        }
        for (cid, place) in abandoned {
            let nm = self.camp_name(cid);
            self.emit(EvKind::CampAbandoned, vec![], place, format!("The camp of {nm} stands empty"));
        }
    }

    pub fn update_energy(&mut self, i: usize) {
        let tpd = self.tpd() as f32;
        let hunger = self.cfg.people.hunger_per_day;
        let tick = self.tick;
        let p = &mut self.people[i];
        let dt = tick.saturating_sub(p.last_upd) as f32 / tpd;
        let child = if (tick - p.born) < (self.cfg.people.adult_cycles * self.cfg.time.ticks_per_cycle() as f32) as u64 { 0.7 } else { 1.0 };
        let burn = hunger * dt * (0.8 + 0.4 * p.genome.size) * (1.2 - 0.4 * p.genome.efficiency) * child;
        p.energy = (p.energy - burn).max(0.0);
        if p.act != Act::Sleep {
            p.mind.drives[drv::REST] = (p.mind.drives[drv::REST] + dt * 1.1).min(1.0);
        }
        p.last_upd = tick;
    }

    fn feed_infant(&mut self, i: usize) {
        let need = (0.8 - self.people[i].energy).max(0.0);
        if need <= 0.0 {
            return;
        }
        let mut carers: Vec<u32> = [self.people[i].mother, self.people[i].father].into_iter().flatten().collect();
        // older kin and generous camp-mates help when parents cannot
        if let Some(h) = self.people[i].home {
            for &o in &self.alive {
                if carers.len() >= 6 {
                    break;
                }
                let q = &self.people[o as usize];
                if o as usize != i && q.home == Some(h) && q.stash > 3.0 && !carers.contains(&o) && (self.kin_between(o, i as u32) != kin::NONE || q.genome.cooperation > 0.6) {
                    carers.push(o);
                }
            }
        }
        let mut need_units = need / 0.25;
        for par in carers {
            let pi = par as usize;
            if !self.people[pi].alive || need_units <= 0.0 {
                continue;
            }
            let is_parent = self.people[i].mother == Some(par) || self.people[i].father == Some(par);
            let care = if is_parent { 0.7 + self.people[pi].genome.parental_care * 0.3 } else { 0.5 * self.people[pi].genome.cooperation };
            let (pp, child) = pair_mut(&mut self.people, pi, i);
            let units = need_units.min(pp.stash + pp.carried) * care;
            if units <= 0.0 {
                continue;
            }
            let from_stash = units.min(pp.stash);
            pp.stash -= from_stash;
            pp.carried -= units - from_stash;
            child.energy = (child.energy + units * 0.25).min(1.0);
            self.counters.food_eaten += units;
            pp.mind.drives[drv::CARE] = (pp.mind.drives[drv::CARE] + 0.05).min(0.9);
            need_units -= units;
        }
    }

    fn try_conceive(&mut self, i: usize, adult: bool, age: f32) {
        let p = &self.people[i];
        if !adult || !p.female || p.pregnant.is_some() || p.energy < 0.35 || age > p.genome.max_age * 0.75 || p.stash + p.carried < 2.5 {
            return;
        }
        if self.alive.len() as u32 >= self.cfg.people.max_population {
            return;
        }
        let Some(m) = p.mate else { return };
        let mi = m as usize;
        if !self.people[mi].alive || self.people[mi].home != p.home {
            return;
        }
        let young = p.children.iter().filter(|&&c| self.people[c as usize].alive && self.people[c as usize].age_cycles(self.tick, self.tpc()) < 2.0).count();
        if young >= 1 {
            return;
        }
        // minds that can plan and expect hard times hold off on children (anticipation, DESIGN §7.7)
        let foresight = if p.mind.caps.planning { 1.0 - 0.85 * p.mind.expect[outcome::HUNGER].0 } else { 1.0 };
        let pr = self.cfg.people.conceive_per_day * (0.5 + p.genome.fertility) * (p.energy - 0.2).max(0.0) * 2.0 * foresight;
        if self.people[i].rng.chance(pr) {
            let due = self.tick + self.cfg.people.gestation_days * self.tpd();
            self.people[i].pregnant = Some((due, m));
        }
    }

    fn birth(&mut self, mi: usize, father: u32) {
        let id = self.people.len() as u32;
        let mut rng = Rng::stream(self.seed, tag::PERSON, id as u64);
        let fg = self.people[father as usize].genome.clone();
        let g = Genome::child(&self.people[mi].genome, &fg, &mut rng, 0.1, 0.06);
        let female = rng.chance(0.5);
        let phon = self.people[mi].phon.clone();
        let culture = self.people[mi].culture;
        let mut c = self.make_person(id, g, female, self.tick, culture, phon, rng);
        c.mother = Some(mi as u32);
        c.father = Some(father);
        c.home = self.people[mi].home;
        let pos = self.people[mi].pos(self.tick);
        c.from = pos;
        c.to = pos;
        c.energy = 0.7;
        // the mother names the child, sometimes after a salient thing
        let place = place_of(pos.0, pos.1);
        let mother = &mut self.people[mi];
        let (name, gloss) = if mother.rng.chance(self.cfg.language.compound_chance) {
            let topics = [Concept::Place(place), Concept::Kind(Kind::Lamp), Concept::Kind(Kind::Vent), Concept::Kind(Kind::Shell)];
            let topic = topics[mother.rng.below(topics.len() as u32) as usize];
            match mother.mind.lex.best(&topic).cloned() {
                Some(w) => {
                    let mut nm = w;
                    nm.truncate(4);
                    let suf = mother.phon.coin(&mut mother.rng);
                    nm.extend(suf.iter().take(2));
                    let g = match topic {
                        Concept::Place(p) => format!("born-at-place-{p}"),
                        Concept::Kind(k) => format!("{}-child", k.gloss().to_lowercase()),
                        _ => String::new(),
                    };
                    (nm, g)
                }
                None => (mother.phon.coin(&mut mother.rng), String::new()),
            }
        } else {
            (mother.phon.coin(&mut mother.rng), String::new())
        };
        let mut name = name;
        while name.len() < 3 {
            let m = &mut self.people[mi];
            let extra = m.phon.coin(&mut m.rng);
            name.extend(extra);
        }
        c.name = name.clone();
        c.name_gloss = gloss;
        c.mind.lex.adopt(Concept::Person(id), name.clone(), 0.9);
        let tick = self.tick;
        let cap = self.cfg.mind.model_cap;
        for par in [mi as u32, father] {
            let pp = &mut self.people[par as usize];
            pp.children.push(id);
            pp.mind.lex.adopt(Concept::Person(id), name.clone(), 0.8);
            let m = pp.mind.model_mut(id, tick, cap);
            m.kin = kin::CHILD;
            m.affinity = 0.8;
            m.trust = 0.7;
            pp.mind.feel(emo::JOY, 0.6);
        }
        for (par, k) in [(mi as u32, kin::PARENT), (father, kin::PARENT)] {
            let pn = self.people[par as usize].name.clone();
            let m = c.mind.model_mut(par, tick, cap);
            m.kin = k;
            m.affinity = 0.8;
            m.trust = 0.8;
            c.mind.lex.adopt(Concept::Person(par), pn, 0.5);
        }
        c.next_decision = self.tick + 32;
        let mname = self.people[mi].display_name();
        let fname = self.name(father);
        let cname = crate::language::name_case(&name);
        c.life_note(tick, format!("born to {mname} and {fname}"));
        self.people.push(c);
        self.alive.push(id);
        self.people[mi].life_note(tick, format!("gave birth to {cname}"));
        self.episode(mi, EpKind::ChildBorn, Some(id), 0.8, 0.6);
        self.counters.births += 1;
        self.emit(EvKind::Birth, vec![id, mi as u32, father], place, format!("{cname} is born to {mname} and {fname}"));
    }

    pub fn die(&mut self, i: usize, cause: &str) {
        if !self.people[i].alive {
            return;
        }
        let tick = self.tick;
        let id = self.people[i].id;
        let place = self.place_of_person(i);
        {
            let p = &mut self.people[i];
            p.alive = false;
            p.died = Some(tick);
            p.death_cause = Some(cause.to_string());
            p.life_note(tick, format!("died: {cause}"));
        }
        self.counters.record_death(cause);
        let name = self.name(id);
        let age = self.people[i].age_cycles(tick, self.tpc());
        self.emit(EvKind::Death, vec![id], place, format!("{name} died ({cause}) aged {age:.0} cycles"));
        // grief for those who knew them
        let cap = self.cfg.mind.belief_cap;
        let ids = self.alive.clone();
        for o in ids {
            let oi = o as usize;
            if oi == i || !self.people[oi].alive {
                continue;
            }
            let Some(m) = self.people[oi].mind.model(id) else { continue };
            let (k, aff) = (m.kin, m.affinity);
            if k == kin::NONE && aff < 0.35 {
                continue;
            }
            let w = if k != kin::NONE { 0.9 } else { aff };
            let p = &mut self.people[oi];
            p.mind.feel(emo::GRIEF, w);
            p.mind.today_out |= 1 << outcome::DEATH;
            p.needs[need::GRIEF] += w * 2.0;
            if p.mate == Some(id) {
                p.mate = None;
            }
            let kind = if k != kin::NONE { EpKind::KinDied } else { EpKind::FriendDied };
            self.episode(oi, kind, Some(id), -0.9, 0.6 + w * 0.4);
            if self.people[oi].knows(n::ANCESTOR) {
                self.people[oi].mind.believe(Concept::Kind(Kind::Ancestors), Rel::Watch, 0.1, None, o, tick, false, cap);
            }
        }
        // remove from any hunt
        for t in self.tasks.iter_mut() {
            t.members.retain(|&m| m != id);
        }
    }

    // ---------------------------------------------------------------- knowledge

    /// Identity grows from living among camp-mates one likes, under a shared name; it fades
    /// when one moves away, and a new one takes its place.
    fn update_identity(&mut self, i: usize, tick: u64) {
        let _ = tick;
        let adult = self.adult(i);
        let home = self.people[i].home;
        let p = &self.people[i];
        if !p.mind.caps.language || !adult {
            return;
        }
        let liking = match home {
            Some(h) => {
                let (mut n, mut sum) = (0, 0.0f32);
                for md in &p.mind.models {
                    let q = &self.people[md.who as usize];
                    if q.alive && q.home == Some(h) {
                        n += 1;
                        sum += md.affinity.max(0.0) + if md.kin != kin::NONE { 0.3 } else { 0.0 };
                    }
                }
                if n > 0 { (sum / n as f32).min(1.0) } else { 0.0 }
            }
            None => 0.0,
        };
        let named = home.map_or(false, |h| p.mind.lex.best(&Concept::Camp(h)).is_some());
        let m = &mut self.people[i].mind;
        m.identity = match (m.identity, home) {
            (Some((c, s)), Some(h)) if c == h => {
                let grow = if named { 0.004 + 0.02 * liking } else { 0.0 };
                Some((c, (s + grow - 0.002).clamp(0.0, 1.0)))
            }
            (Some((c, s)), Some(h)) => {
                // living elsewhere: the old allegiance fades until the new home takes over
                let s2 = s * 0.97;
                if s2 < 0.15 { Some((h, 0.1)) } else { Some((c, s2)) }
            }
            (Some((c, s)), None) => {
                let s2 = s * 0.98;
                if s2 < 0.05 { None } else { Some((c, s2)) }
            }
            (None, Some(h)) => Some((h, 0.1)),
            (None, None) => None,
        };
    }

    /// Successful use of a material is evidence against it being cursed.
    pub fn disconfirm_curse(&mut self, i: usize, mats: u16) {
        let m = &mut self.people[i].mind;
        for b in m.beliefs.iter_mut() {
            if b.rel == Rel::Cursed {
                if let Concept::Kind(k) = b.subject {
                    for mm in crate::discovery::mat::list(mats) {
                        if crate::decide::mat_kind(mm) == k {
                            b.conf -= 0.08;
                        }
                    }
                }
            }
        }
    }

    /// Using a skill where others can see it: bystanders who respect the user may copy it
    /// (prestige-biased social learning, DESIGN §7.10).
    pub fn demonstrate(&mut self, i: usize, node: u8) {
        if !self.people[i].knows(node) || !self.people[i].rng.chance(0.2) {
            return;
        }
        let (x, y) = self.people[i].pos(self.tick);
        let iid = i as u32;
        let rate = self.cfg.discovery.imitation;
        let material = self.people[i].know(node).map_or(0, |k| k.material);
        for o in self.near(x, y, 25.0) {
            let oi = o as usize;
            if o == iid || self.people[oi].knows(node) {
                continue;
            }
            let respect = self.people[oi].mind.model(iid).map_or(0.0, |m| m.respect);
            let p_learn = rate * (0.3 + respect * 2.0) * (0.5 + self.people[oi].genome.social_learning);
            if self.people[oi].rng.chance(p_learn) {
                if self.learn(oi, node, material, KnowSource::Imitated(iid)) {
                    self.people[i].prestige += 0.05;
                    self.counters.imitated += 1;
                }
            }
        }
    }

    /// Practising a discovery keeps it (and its prerequisites) fresh.
    pub fn use_know(&mut self, i: usize, node: u8) {
        let p = &mut self.people[i];
        if let Some(k) = p.knowledge.iter_mut().find(|k| k.node == node) {
            k.prof = (k.prof + 0.03).min(1.0);
            for &q in NODES[node as usize].prereq {
                if let Some(kq) = p.knowledge.iter_mut().find(|k| k.node == q) {
                    kq.prof = (kq.prof + 0.02).min(1.0);
                }
            }
        }
    }

    pub fn learn(&mut self, i: usize, node: u8, material: u16, src: KnowSource) -> bool {
        let tick = self.tick;
        let p = &mut self.people[i];
        if let Some(k) = p.knowledge.iter_mut().find(|k| k.node == node) {
            k.prof = (k.prof + 0.1).min(1.0);
            return false;
        }
        let prof = match src {
            KnowSource::Invented => 0.6,
            KnowSource::Taught(_) => 0.5,
            KnowSource::Imitated(_) => 0.3,
            KnowSource::Record => 0.3,
        };
        p.knowledge.push(Know { node, material, prof, from: src, tick });
        let via = match src {
            KnowSource::Invented => Via::Invented,
            KnowSource::Taught(_) => Via::Taught,
            KnowSource::Imitated(_) => Via::Imitated,
            KnowSource::Record => Via::Record,
        };
        let culture = p.culture;
        let pid = p.id;
        let idx = match self.registry.key_index(node, material) {
            Some(x) => x,
            None => {
                let d = &NODES[node as usize];
                self.registry.entries.push(RegistryEntry {
                    node,
                    material,
                    name: Registry::variant_name(node, material),
                    function: d.function.into(),
                    first_tick: tick,
                    first_by: pid,
                    first_culture: culture,
                    inventions: 0,
                    spread: [0; 5],
                    holders: 0,
                    peak_holders: 0,
                    lost: Vec::new(),
                    rediscovered: Vec::new(),
                });
                self.registry.entries.len() - 1
            }
        };
        self.registry.entries[idx].spread[via as usize] += 1;
        self.registry.entries[idx].holders += 1;
        let ns = &mut self.nodes[node as usize];
        ns.holders += 1;
        if via == Via::Invented {
            ns.inventions += 1;
            self.registry.entries[idx].inventions += 1;
            self.counters.inventions += 1;
            let was_lost = !ns.lost.is_empty() && ns.lost.len() > ns.rediscovered.len();
            let first = ns.first_tick.is_none();
            if first {
                ns.first_tick = Some(tick);
            }
            let place = self.place_of_person(i);
            let name = self.name(pid);
            let vname = Registry::variant_name(node, material);
            if was_lost {
                self.nodes[node as usize].rediscovered.push(tick);
                self.registry.entries[idx].rediscovered.push(tick);
                self.emit(EvKind::Rediscovery, vec![pid], place, format!("{name} rediscovers the lost {vname}"));
            } else if first {
                self.emit(EvKind::Discovery, vec![pid], place, format!("{name} invents the {vname}: {}", NODES[node as usize].desc));
            } else if self.registry.entries[idx].inventions == 1 {
                self.emit(EvKind::Discovery, vec![pid], place, format!("{name} independently devises the {vname}"));
            }
        } else if via == Via::Record {
            let ns = &mut self.nodes[node as usize];
            if !ns.lost.is_empty() && ns.lost.len() > ns.rediscovered.len() {
                ns.rediscovered.push(tick);
                let place = self.place_of_person(i);
                let name = self.name(pid);
                self.emit(EvKind::Rediscovery, vec![pid], place, format!("{name} recovers the lost {} from old tally marks", NODES[node as usize].name));
            }
        }
        true
    }

    fn knowledge_census(&mut self) {
        let mut holders = vec![0u32; n::COUNT];
        for e in self.registry.entries.iter_mut() {
            e.holders = 0;
        }
        for &id in &self.alive {
            for k in &self.people[id as usize].knowledge {
                holders[k.node as usize] += 1;
                if let Some(x) = self.registry.key_index(k.node, k.material) {
                    self.registry.entries[x].holders += 1;
                }
            }
        }
        for e in self.registry.entries.iter_mut() {
            e.peak_holders = e.peak_holders.max(e.holders);
        }
        for node in 0..n::COUNT {
            let prev = self.nodes[node].holders;
            self.nodes[node].holders = holders[node];
            self.nodes[node].peak = self.nodes[node].peak.max(holders[node]);
            if prev > 0 && holders[node] == 0 {
                let preserved = self.world.camps.iter().any(|c| c.abandoned.is_none() && (c.records.contains(&(node as u8)) || c.has(node as u8)));
                if !preserved {
                    let t = self.tick;
                    self.nodes[node].lost.push(t);
                    for e in self.registry.entries.iter_mut().filter(|e| e.node as usize == node) {
                        e.lost.push(t);
                    }
                    self.counters.knowledge_lost += 1;
                    self.emit(EvKind::KnowledgeLost, vec![], 0, format!("The {} is forgotten: no one alive knows it", NODES[node].name));
                }
            }
        }
    }

    // ---------------------------------------------------------------- hashing

    pub fn state_hash(&self) -> u64 {
        let bytes = bincode::serialize(self).expect("serialize");
        fnv64(&bytes)
    }

    pub fn snapshot(&self) -> Vec<u8> {
        bincode::serialize(self).expect("serialize")
    }

    pub fn restore(bytes: &[u8], ds: Arc<Dataset>) -> Sim {
        let mut s: Sim = bincode::deserialize(bytes).expect("deserialize");
        s.ds = Some(ds);
        s.at_place = vec![Vec::new(); N_PLACES];
        s.rebuild_index();
        s
    }
}
