//! Evolving animals (DESIGN §5.3): super-individual creatures descended from multicellular
//! grazing strains. Behaviour comes from genes; a little intelligence (`capacity`) buys better
//! choices at a metabolic and developmental cost, so it rises only where it pays.
use crate::evo::{Evo, StrainGenome, BOTTOM, LAYERS, MID};
use crate::genome::Genome;
use crate::rng::Rng;
use crate::world::{dist, World, CELL, H, TANK_H, TANK_W, W};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct FaunaCfg {
    /// Daily chance that a dense patch of multicellular grazers gives rise to animals.
    pub origin_chance: f32,
    pub origin_min_biomass: f32,
    pub max_agents: usize,
    /// Chance per birth event that offspring split off as a separate, mutated group.
    pub split_chance: f32,
    pub mutation_rate: f32,
    pub mutation_size: f32,
    pub species_distance: f32,
    /// Energy cost multiplier of intelligence (capacity).
    pub capacity_cost: f32,
    /// Extra days to maturity per unit of capacity (a long childhood).
    pub capacity_maturity: f32,
}
impl Default for FaunaCfg {
    fn default() -> Self {
        FaunaCfg {
            origin_chance: 0.02,
            origin_min_biomass: 12.0,
            max_agents: 3000,
            split_chance: 0.12,
            mutation_rate: 0.3,
            mutation_size: 0.05,
            species_distance: 0.12,
            capacity_cost: 0.3,
            capacity_maturity: 40.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Creature {
    pub id: u32,
    pub species: u32,
    pub g: Genome,
    pub x: f32,
    pub y: f32,
    pub energy: f32,
    pub age_days: f32,
    /// how many near-identical animals this agent stands for
    pub m: f32,
    pub alive: bool,
    pub mem: Option<(f32, f32)>,
    pub born: u64,
    pub gen: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimalSpecies {
    pub id: u32,
    pub founded: u64,
    pub extinct: Option<u64>,
    pub parent: Option<u32>,
    pub centroid: Genome,
    pub population: f32,
    pub peak: f32,
}

impl AnimalSpecies {
    pub fn describe(&self) -> String {
        let g = &self.centroid;
        let size = if g.size < 0.25 { "tiny" } else if g.size < 0.5 { "small" } else if g.size < 0.75 { "large" } else { "huge" };
        let diet = if g.diet < 0.3 { "grazer" } else if g.diet < 0.65 { "omnivore" } else { "hunter" };
        let social = if g.sociality > 0.6 { ", schooling" } else { "" };
        let mind = if g.capacity > 0.25 { ", clever" } else { "" };
        format!("{size} {diet}{social}{mind}")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FaunaEvent {
    FirstAnimals { species: u32, x: f32, y: f32 },
    NewSpecies { species: u32, parent: Option<u32>, what: String },
    Extinct { species: u32, what: String, days: u64 },
    FirstHunter { species: u32 },
    Cleverest { species: u32, capacity: f32 },
}

/// Days of growth before an animal can breed: bigger and cleverer animals mature slower.
fn maturity_days(g: &Genome, cfg: &FaunaCfg) -> f32 {
    12.0 + 40.0 * g.size + cfg.capacity_maturity * g.capacity
}

fn cell_of(x: f32, y: f32) -> usize {
    let cx = ((x / CELL) as isize).clamp(0, W as isize - 1) as usize;
    let cy = ((y / CELL) as isize).clamp(0, H as isize - 1) as usize;
    cy * W + cx
}

/// Coarse buckets (4×4 cells) for neighbour queries.
const BW: usize = W / 4;
const BH: usize = H / 4;
fn bucket(x: f32, y: f32) -> usize {
    let bx = ((x / (CELL * 4.0)) as isize).clamp(0, BW as isize - 1) as usize;
    let by = ((y / (CELL * 4.0)) as isize).clamp(0, BH as isize - 1) as usize;
    by * BW + bx
}

impl Evo {
    /// Where dense multicellular grazers live, animals may arise.
    pub fn animal_origins(&mut self, cfg: &FaunaCfg, tick: u64, r: &mut Rng) {
        if self.creatures.len() >= cfg.max_agents {
            return;
        }
        let mut found: Option<(usize, u32, f32)> = None;
        for i in 0..W * H {
            for k in [MID, BOTTOM] {
                for &(s, b) in &self.pop[i * LAYERS + k] {
                    let g = &self.strains[s as usize].g;
                    if g.graze > 0.5 && g.multicell > 0.5 && g.size > 0.45 && b > cfg.origin_min_biomass {
                        if found.map_or(true, |f| b > f.2) {
                            found = Some((i, s, b));
                        }
                    }
                }
            }
        }
        let Some((cell, sid, b)) = found else { return };
        if !r.chance(cfg.origin_chance) {
            return;
        }
        let sg: StrainGenome = self.strains[sid as usize].g;
        let g = Genome::from_strain(&sg, r);
        let (x, y) = World::cell_center(cell);
        let species = self.new_animal_species(&g, None, tick);
        let id = self.next_creature;
        self.next_creature += 1;
        self.creatures.push(Creature { id, species, g, x, y, energy: 0.6, age_days: 30.0, m: (b * 0.5).max(4.0), alive: true, mem: None, born: tick, gen: 0 });
        // the founding population is taken from the strain
        for k in [MID, BOTTOM] {
            for e in self.pop[cell * LAYERS + k].iter_mut() {
                if e.0 == sid {
                    e.1 *= 0.5;
                }
            }
        }
        if self.animals_began.is_none() {
            self.animals_began = Some(tick);
            self.fauna_events.push((tick, FaunaEvent::FirstAnimals { species, x, y }));
        }
    }

    fn new_animal_species(&mut self, g: &Genome, parent: Option<u32>, tick: u64) -> u32 {
        let id = self.animal_species.len() as u32;
        self.animal_species.push(AnimalSpecies { id, founded: tick, extinct: None, parent, centroid: g.clone(), population: 0.0, peak: 0.0 });
        if parent.is_some() {
            let what = self.animal_species[id as usize].describe();
            self.fauna_events.push((tick, FaunaEvent::NewSpecies { species: id, parent, what }));
        }
        id
    }

    /// One behavioural sub-step for every animal (`dt` in days).
    pub fn fauna_step(&mut self, world: &mut World, cfg: &FaunaCfg, tick: u64, dt: f32, r: &mut Rng) {
        if self.creatures.is_empty() {
            return;
        }
        // buckets for neighbour queries
        let mut buckets: Vec<Vec<u32>> = vec![Vec::new(); BW * BH];
        for (k, c) in self.creatures.iter().enumerate() {
            if c.alive {
                buckets[bucket(c.x, c.y)].push(k as u32);
            }
        }
        let n = self.creatures.len();
        // how many animals already feed in each cell (crowded spots are worth less)
        let mut crowd = vec![0.0f32; W * H];
        for c in self.creatures.iter().filter(|c| c.alive) {
            crowd[cell_of(c.x, c.y)] += c.m;
        }
        self.crowd = crowd;
        let mut kills: Vec<(usize, usize)> = Vec::new();
        for k in 0..n {
            if !self.creatures[k].alive {
                continue;
            }
            let c = self.creatures[k].clone();
            let g = &c.g;
            let cap = g.capacity;
            let sense = 10.0 + 30.0 * g.vision + 25.0 * cap;
            // nearby animals
            let (bx, by) = ((bucket(c.x, c.y) % BW) as isize, (bucket(c.x, c.y) / BW) as isize);
            let mut threat: Option<(f32, f32, f32)> = None;
            let mut prey: Option<(usize, f32)> = None;
            let mut _kin_near = 0.0f32;
            let mut kin_n = 0.0f32;
            let mut kin_cx = 0.0;
            let mut kin_cy = 0.0;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let (nx, ny) = (bx + dx, by + dy);
                    if nx < 0 || ny < 0 || nx >= BW as isize || ny >= BH as isize {
                        continue;
                    }
                    for &o in &buckets[ny as usize * BW + nx as usize] {
                        let o = o as usize;
                        if o == k {
                            continue;
                        }
                        let oc = &self.creatures[o];
                        if !oc.alive {
                            continue;
                        }
                        let d = dist(c.x, c.y, oc.x, oc.y);
                        if d > sense {
                            continue;
                        }
                        // kin are recognised by close genetic similarity, not by species label
                        if oc.g.distance(&c.g) < 0.06 {
                            _kin_near += oc.m.min(20.0);
                            kin_n += 1.0;
                            kin_cx += oc.x;
                            kin_cy += oc.y;
                        } else if oc.g.diet > 0.5 && oc.g.size > g.size * 1.1 {
                            if threat.map_or(true, |t| d < t.2) {
                                threat = Some((oc.x, oc.y, d));
                            }
                        } else if g.diet > 0.12 && oc.g.size < g.size * (0.85 + 0.3 * g.diet) {
                            let value = oc.g.size * (1.0 - oc.g.armor * 0.6) / (1.0 + d / 20.0);
                            if prey.map_or(true, |p| value > p.1) {
                                prey = Some((o, value));
                            }
                        }
                    }
                }
            }
            let mut c = c;
            let step = (6.0 + 22.0 * c.g.speed) * dt * 4.0;
            // metabolism: big, fast, armoured and clever animals burn more
            let burn = 0.22 * dt * (0.6 + c.g.size) * (0.8 + 0.4 * c.g.speed) * (1.0 + cfg.capacity_cost * cap) + 0.03 * dt * c.g.armor;
            c.energy -= burn;
            c.age_days += dt;
            // decide: flee, hunt, graze-move, school
            if let Some((tx, ty, d)) = threat {
                // anticipation: clever animals start fleeing earlier and choose better escape lines
                let (ex, ey) = (c.x - tx, c.y - ty);
                let l = (ex * ex + ey * ey).sqrt().max(1.0);
                let boost = 1.0 + cap;
                c.x += ex / l * step * boost;
                c.y += ey / l * step * boost;
                let _ = d;
            } else if let (Some((p, _)), true) = (prey, c.energy < 0.8 && r.chance(0.3 + 0.7 * c.g.diet)) {
                let pc = &self.creatures[p];
                let (px, py) = (pc.x, pc.y);
                let d = dist(c.x, c.y, px, py);
                let mv = step.min(d);
                if d > 0.1 {
                    c.x += (px - c.x) / d * mv;
                    c.y += (py - c.y) / d * mv;
                }
                if d <= step {
                    // the strike: speed, size, cleverness vs armour, escape and the safety of numbers
                    let prey_school = 1.0 + pc.g.sociality * (pc.m.min(30.0) / 10.0);
                    let p_hit = ((0.25 + 0.5 * (c.g.speed - pc.g.speed).max(-0.5) + 0.6 * (c.g.size - pc.g.size) + 0.3 * cap - 0.25 * pc.g.capacity) * (1.0 - 0.6 * pc.g.armor) / prey_school).clamp(0.02, 0.9);
                    if r.chance(p_hit) {
                        kills.push((k, p));
                    }
                }
            } else {
                // graze: even brainless animals follow the food gradient to neighbouring cells;
                // cleverer ones also weigh spots further off, and remember good places
                let here = self.food_at(world, c.x, c.y, &c.g);
                let mut best = (c.x, c.y, here);
                let reach = (CELL * 1.2).max(step);
                for (dx, dy) in [(1.0f32, 0.0f32), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
                    if !r.chance(0.4 + 0.6 * c.g.chemo) {
                        continue;
                    }
                    let (ox, oy) = ((c.x + dx * reach).clamp(2.0, TANK_W - 2.0), (c.y + dy * reach).clamp(2.0, TANK_H - 2.0));
                    let f = self.food_at(world, ox, oy, &c.g);
                    if f > best.2 {
                        best = (ox, oy, f);
                    }
                }
                let options = (cap * 12.0) as usize;
                // hungry animals with nothing better nearby set off to roam
                let range = if c.energy < 0.4 && best.2 <= here { reach * 4.0 } else { reach * (1.0 + 3.0 * cap) };
                if c.energy < 0.4 && best.2 <= here * 1.05 {
                    let ang = r.range(0.0, 6.283);
                    best = ((c.x + range * libm::cosf(ang)).clamp(2.0, TANK_W - 2.0), (c.y + range * libm::sinf(ang)).clamp(2.0, TANK_H - 2.0), here);
                }
                for _ in 0..options {
                    let (ox, oy) = ((c.x + r.range(-range, range)).clamp(2.0, TANK_W - 2.0), (c.y + r.range(-range, range)).clamp(2.0, TANK_H - 2.0));
                    let f = self.food_at(world, ox, oy, &c.g);
                    if f > best.2 {
                        best = (ox, oy, f);
                    }
                }
                if cap > 0.1 {
                    if let Some((mx, my)) = c.mem {
                        let f = self.food_at(world, mx, my, &c.g) * 0.9;
                        if f > best.2 && dist(c.x, c.y, mx, my) < step * 3.0 {
                            let d = dist(c.x, c.y, mx, my).max(0.1);
                            best = (c.x + (mx - c.x) / d * step.min(d), c.y + (my - c.y) / d * step.min(d), f);
                        }
                    }
                }
                // social animals stay with their school
                if c.g.sociality > 0.4 && kin_n > 0.0 {
                    let (sx, sy) = (kin_cx / kin_n, kin_cy / kin_n);
                    let pull = (c.g.sociality - 0.4) * 0.6;
                    best.0 += (sx - best.0) * pull;
                    best.1 += (sy - best.1) * pull;
                }
                c.x = best.0;
                c.y = best.1;
                let eaten = self.eat_at(world, c.x, c.y, &c.g, dt, c.m);
                c.energy = (c.energy + eaten).min(1.0);
                if cap > 0.1 && best.2 > 2.0 {
                    c.mem = Some((c.x, c.y));
                }
            }
            if world.cells[cell_of(c.x, c.y)].hab == crate::world::Habitat::Rock {
                c.x = self.creatures[k].x;
                c.y = self.creatures[k].y;
            }
            self.creatures[k] = c;
        }
        // predation: a pack makes several kills, shared among its members
        for (hunter, victim) in kills {
            if !self.creatures[victim].alive || !self.creatures[hunter].alive {
                continue;
            }
            let pack = self.creatures[hunter].m;
            let taken = (pack * 0.25).max(1.0).min(self.creatures[victim].m);
            let food = taken * (self.creatures[victim].g.size * 2.0 + 0.4);
            self.fauna_tally[3] += taken;
            let v = &mut self.creatures[victim];
            v.m -= taken;
            if v.m < 1.0 {
                v.alive = false;
            }
            let h = &mut self.creatures[hunter];
            h.energy = (h.energy + food * 0.25 / h.m.max(1.0)).min(1.0);
        }
        self.fauna_life_cycle(cfg, tick, dt, r);
    }

    /// Food an animal could get here: plants and mats for grazers.
    fn food_at(&self, world: &World, x: f32, y: f32, g: &Genome) -> f32 {
        let i = cell_of(x, y);
        let c = &world.cells[i];
        let k = if g.layer > 0.66 { BOTTOM } else if g.layer > 0.33 { MID } else { 0 };
        let mats: f32 = (0..LAYERS)
            .map(|kk| {
                let eff = if kk == k { 1.0 } else if (kk as isize - k as isize).abs() == 1 { 0.5 } else { 0.0 };
                eff * self.pop[i * LAYERS + kk].iter().map(|e| e.1).sum::<f32>()
            })
            .sum();
        (1.0 - g.diet) * (c.algae + c.shell + mats) / (1.0 + 0.3 * self.crowd.get(i).copied().unwrap_or(0.0))
    }

    /// A group of `m` animals feeds here; returns energy gained per animal.
    /// A herd of `m` animals feeds across the cells it spreads over (bigger herds range wider);
    /// returns energy gained per animal.
    fn eat_at(&mut self, world: &mut World, x: f32, y: f32, g: &Genome, dt: f32, m: f32) -> f32 {
        let per = (0.4 + g.size) * dt * (1.0 - g.diet) * 2.0;
        let mut need = per * m;
        let mut got = 0.0f32;
        let (cx, cy) = (((x / CELL) as isize).clamp(0, W as isize - 1), ((y / CELL) as isize).clamp(0, H as isize - 1));
        let radius = (1.0 + libm::sqrtf(m) / 4.0).min(3.0) as isize;
        let k = if g.layer > 0.66 { BOTTOM } else if g.layer > 0.33 { MID } else { 0 };
        let cells: Vec<usize> = {
            let mut v = Vec::new();
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let (nx, ny) = (cx + dx, cy + dy);
                    if nx >= 0 && ny >= 0 && nx < W as isize && ny < H as isize {
                        v.push(ny as usize * W + nx as usize);
                    }
                }
            }
            v
        };
        let share = 1.0 / cells.len() as f32;
        for &i in &cells {
            let mut want = need.min(per * m * share * 1.5);
            // refuges: grazing stops short of the last of a bed, so it can regrow
            let c = &mut world.cells[i];
            let a = (want * 0.6).min((c.algae - 0.2 * c.algae_cap).max(0.0) * 0.5);
            c.algae -= a;
            want -= a;
            let s = (want * 0.5).min((c.shell - 0.2 * c.shell_cap).max(0.0) * 0.5);
            c.shell -= s;
            want -= s;
            let mut mats = 0.0;
            // graze mats in the preferred layer fully, neighbouring layers at half efficiency
            for kk in 0..LAYERS {
                let eff = if kk == k { 1.0 } else if (kk as isize - k as isize).abs() == 1 { 0.5 } else { 0.0 };
                if eff == 0.0 || want <= 0.0 {
                    continue;
                }
                for e in self.pop[i * LAYERS + kk].iter_mut() {
                    // well-defended mats are harder to graze, and thin mats are left alone
                    let def = self.strains[e.0 as usize].g.defense;
                    let t = (want / eff).min((e.1 - 1.5).max(0.0) * 0.3 * (1.0 - 0.6 * def));
                    e.1 -= t;
                    want -= t * eff;
                    mats += t * eff;
                }
            }
            let here = a + s * 1.5 + mats;
            got += here;
            need = (need - here).max(0.0);
            if need <= 0.0 {
                break;
            }
        }
        got / m.max(1.0) * 0.25
    }

    fn fauna_life_cycle(&mut self, cfg: &FaunaCfg, tick: u64, dt: f32, r: &mut Rng) {
        let mut newborn: Vec<Creature> = Vec::new();
        let n = self.creatures.len();
        let room = cfg.max_agents.saturating_sub(n);
        for k in 0..n {
            let c = &mut self.creatures[k];
            if !c.alive {
                continue;
            }
            // starvation thins the group; old age too
            if c.energy < 0.1 {
                // a hungry group loses its weakest members
                let lost = c.m * 0.3 * dt * (0.1 - c.energy.max(0.0)) * 10.0;
                self.fauna_tally[1] += lost;
                c.m -= lost;
                c.energy = c.energy.max(0.0);
            }
            if c.age_days > c.g.max_age * 48.0 {
                self.fauna_tally[2] += c.m * 0.05 * dt;
                c.m *= 1.0 - 0.05 * dt;
            }
            if c.m < 1.0 {
                self.fauna_tally[1] += c.m.max(0.0);
                c.alive = false;
                continue;
            }
            // breeding: well-fed, mature groups grow; some young split off as a new, mutated group
            if c.energy > 0.55 && c.age_days > maturity_days(&c.g, cfg) {
                // fewer young where the feeding ground is crowded (density-dependent breeding)
                let local = self.crowd.get(cell_of(c.x, c.y)).copied().unwrap_or(0.0);
                let crowd_room = 1.0 / (1.0 + local / 12.0);
                let rate = 0.25 * dt * (c.energy - 0.55) * (0.5 + c.g.fertility) * (1.0 - 0.5 * c.g.size) * crowd_room;
                let young = c.m * rate;
                self.fauna_tally[0] += young;
                c.energy -= 0.25 * dt;
                // a new, mutated group buds off only from a herd big enough to spare two animals
                if c.m >= 4.0 && r.chance(cfg.split_chance * dt * 4.0) && newborn.len() < room {
                    let g = c.g.mutant(r, cfg.mutation_rate, cfg.mutation_size);
                    let m = (young * 4.0).max(2.0).min(c.m * 0.5);
                    c.m -= m - young;
                    newborn.push(Creature { id: 0, species: c.species, g, x: c.x + r.range(-6.0, 6.0), y: c.y + r.range(-6.0, 6.0), energy: 0.5, age_days: 0.0, m, alive: true, mem: None, born: tick, gen: c.gen + 1 });
                } else {
                    c.m += young;
                }
            }
            if c.m > 60.0 && newborn.len() < room {
                let half = c.m * 0.5;
                c.m -= half;
                let g = if r.chance(0.5) { c.g.mutant(r, cfg.mutation_rate, cfg.mutation_size) } else { c.g.clone() };
                let ang = r.range(0.0, 6.283);
                newborn.push(Creature { id: 0, species: c.species, g, x: (c.x + 20.0 * libm::cosf(ang)).clamp(2.0, TANK_W - 2.0), y: (c.y + 20.0 * libm::sinf(ang)).clamp(2.0, TANK_H - 2.0), energy: c.energy, age_days: c.age_days, m: half, alive: true, mem: c.mem, born: tick, gen: c.gen + 1 });
            }
            c.m = c.m.min(400.0);
        }
        for mut b in newborn {
            b.id = self.next_creature;
            self.next_creature += 1;
            self.creatures.push(b);
        }
        self.creatures.retain(|c| c.alive);
    }

    /// Re-cluster animal species: a group whose genes have drifted far from its species becomes a new one.
    pub fn fauna_species(&mut self, cfg: &FaunaCfg, tick: u64) {
        let n = self.creatures.len();
        for k in 0..n {
            let sp = self.creatures[k].species;
            let far = self.animal_species[sp as usize].centroid.distance(&self.creatures[k].g) > cfg.species_distance;
            if far {
                let g = self.creatures[k].g.clone();
                let id = self.new_animal_species(&g, Some(sp), tick);
                self.creatures[k].species = id;
            }
        }
        let ns = self.animal_species.len();
        let mut pop = vec![0.0f32; ns];
        let mut best_cap = vec![0.0f32; ns];
        for c in &self.creatures {
            pop[c.species as usize] += c.m;
            best_cap[c.species as usize] = best_cap[c.species as usize].max(c.g.capacity);
        }
        for sp in self.animal_species.iter_mut() {
            if sp.extinct.is_some() {
                continue;
            }
            sp.population = pop[sp.id as usize];
            sp.peak = sp.peak.max(sp.population);
            if sp.population < 1.0 {
                sp.extinct = Some(tick);
                let what = sp.describe();
                self.fauna_events.push((tick, FaunaEvent::Extinct { species: sp.id, what, days: (tick - sp.founded) / 1024 }));
            }
        }
        // centroid drift: follow the living members (population-weighted)
        let mut acc: Vec<Option<(f32, Genome)>> = vec![None; ns];
        for c in &self.creatures {
            let e = &mut acc[c.species as usize];
            match e {
                None => *e = Some((c.m, c.g.clone())),
                Some((w, g)) => {
                    let t = c.m / (*w + c.m);
                    g.diet += (c.g.diet - g.diet) * t;
                    g.size += (c.g.size - g.size) * t;
                    g.speed += (c.g.speed - g.speed) * t;
                    g.armor += (c.g.armor - g.armor) * t;
                    g.capacity += (c.g.capacity - g.capacity) * t;
                    g.sociality += (c.g.sociality - g.sociality) * t;
                    for z in 0..3 {
                        g.mating_sig[z] += (c.g.mating_sig[z] - g.mating_sig[z]) * t;
                    }
                    *w += c.m;
                }
            }
        }
        for (k, e) in acc.into_iter().enumerate() {
            if let Some((_, g)) = e {
                self.animal_species[k].centroid = g;
            }
        }
        // milestones
        if !self.first_hunter_seen {
            if let Some(sp) = self.animal_species.iter().find(|s| s.extinct.is_none() && s.centroid.diet > 0.6) {
                self.first_hunter_seen = true;
                let id = sp.id;
                self.fauna_events.push((tick, FaunaEvent::FirstHunter { species: id }));
            }
        }
        let top = self.creatures.iter().map(|c| c.g.capacity).fold(0.0f32, f32::max);
        for mark in [0.15f32, 0.25, 0.35, 0.55, 0.75] {
            if top >= mark && self.cleverest_mark < mark {
                self.cleverest_mark = mark;
                if let Some(c) = self.creatures.iter().find(|c| c.g.capacity >= mark) {
                    let sp = c.species;
                    self.fauna_events.push((tick, FaunaEvent::Cleverest { species: sp, capacity: mark }));
                }
            }
        }
    }
}
