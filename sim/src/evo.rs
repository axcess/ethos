//! Life below the minds (DESIGN §5.2): 2.5-D water layers, prebiotic chemistry,
//! abiogenesis and evolving microbial strains, clustered into species by the observer.
//! Only active in evolution mode; "start with people" tanks never touch it.
use crate::rng::{fnv64, Rng};
use crate::world::{Habitat, World, H, W};
use serde::{Deserialize, Serialize};

pub const LAYERS: usize = 3;
pub const SURFACE: usize = 0;
pub const MID: usize = 1;
pub const BOTTOM: usize = 2;
/// Maximum strains sharing one cell-layer.
pub const SLOTS: usize = 4;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct EvolutionCfg {
    pub enabled: bool,
    /// Daily chance scale of life arising where the prebiotic soup is rich.
    pub abiogenesis: f32,
    pub soup_threshold: f32,
    /// Maximum growth rate per day of a well-suited strain.
    pub growth: f32,
    /// Biomass a cell-layer can hold.
    pub layer_capacity: f32,
    /// Chance per unit biomass per day of a mutant strain appearing.
    pub mutation: f32,
    pub mutation_size: f32,
    pub dispersal: f32,
    pub max_strains: usize,
    pub species_distance: f32,
}
impl Default for EvolutionCfg {
    fn default() -> Self {
        EvolutionCfg {
            enabled: false,
            abiogenesis: 0.02,
            soup_threshold: 1.0,
            growth: 0.6,
            layer_capacity: 40.0,
            mutation: 0.004,
            mutation_size: 0.08,
            dispersal: 0.08,
            max_strains: 2500,
            species_distance: 0.22,
        }
    }
}

/// Compact genome of a microbial strain (all values 0..1).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct StrainGenome {
    /// trophic mix (normalised to sum 1): sunlight, vent chemistry, eating other life
    pub photo: f32,
    pub chemo: f32,
    pub graze: f32,
    pub temp_opt: f32,
    pub temp_tol: f32,
    pub size: f32,
    pub multicell: f32,
    pub defense: f32,
    /// preferred water layer: 0 surface … 1 bottom
    pub layer: f32,
    pub motility: f32,
    pub hue: f32,
}

impl StrainGenome {
    fn normalise(mut self) -> Self {
        let s = (self.photo + self.chemo + self.graze).max(1e-4);
        self.photo /= s;
        self.chemo /= s;
        self.graze /= s;
        self
    }
    pub fn mutate(&self, r: &mut Rng, sd: f32) -> StrainGenome {
        let m = |x: f32, r: &mut Rng| (x + r.normal() * sd).clamp(0.0, 1.0);
        StrainGenome {
            photo: m(self.photo, r),
            chemo: m(self.chemo, r),
            graze: m(self.graze, r),
            temp_opt: m(self.temp_opt, r),
            temp_tol: m(self.temp_tol, r),
            size: m(self.size, r),
            multicell: m(self.multicell, r),
            defense: m(self.defense, r),
            layer: m(self.layer, r),
            motility: m(self.motility, r),
            hue: (self.hue + r.normal() * sd * 0.5).rem_euclid(1.0),
        }
        .normalise()
    }
    pub fn distance(&self, o: &StrainGenome) -> f32 {
        let d = [
            self.photo - o.photo,
            self.chemo - o.chemo,
            self.graze - o.graze,
            self.temp_opt - o.temp_opt,
            self.size - o.size,
            self.multicell - o.multicell,
            self.defense - o.defense,
            self.layer - o.layer,
        ];
        libm::sqrtf(d.iter().map(|x| x * x).sum::<f32>() / d.len() as f32)
    }
    /// What kind of life this is, for reports.
    pub fn kind(&self) -> &'static str {
        if self.multicell > 0.5 {
            if self.graze > 0.5 {
                "multicellular grazer"
            } else if self.photo >= self.chemo {
                "multicellular producer"
            } else {
                "multicellular vent-feeder"
            }
        } else if self.graze > 0.5 {
            "microbial grazer"
        } else if self.photo >= self.chemo {
            "photosynthetic microbe"
        } else {
            "vent microbe"
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Strain {
    pub id: u32,
    pub g: StrainGenome,
    pub parent: Option<u32>,
    pub born: u64,
    pub extinct: Option<u64>,
    pub biomass: f32,
    pub peak: f32,
    pub species: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Species {
    pub id: u32,
    pub founded: u64,
    pub extinct: Option<u64>,
    pub parent: Option<u32>,
    pub kind: String,
    pub biomass: f32,
    pub peak: f32,
    pub centroid: StrainGenome,
}

/// Physical state of one water layer in one cell.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Layer {
    pub light: f32,
    pub temp: f32,
    pub oxygen: f32,
    pub nutrient: f32,
    pub detritus: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvoEvent {
    LifeBegins { cell: usize, strain: u32, vent: bool },
    Speciation { species: u32, parent: Option<u32>, kind: String },
    Extinction { species: u32, kind: String, lived_days: u64 },
    MassExtinction { lost: u32, of: u32 },
    FirstOf { kind: String, species: u32 },
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Evo {
    pub layers: Vec<[Layer; LAYERS]>,
    pub soup: Vec<f32>,
    /// strains present in each cell-layer: index cell * LAYERS + layer
    pub pop: Vec<Vec<(u32, f32)>>,
    pub strains: Vec<Strain>,
    pub species: Vec<Species>,
    pub life_began: Option<u64>,
    pub kinds_seen: Vec<String>,
    pub events: Vec<(u64, EvoEvent)>,
    /// biomass of macro-producers and filter-feeders on the bottom, per cell (feeds algae/shellfish beds)
    pub macro_photo: Vec<f32>,
    pub macro_filter: Vec<f32>,
    pub recent_extinctions: Vec<u64>,
    /// (tick, living species) samples, for detecting collapses in diversity
    pub diversity: Vec<(u64, u32)>,
    pub rng: Option<Rng>,
    pub creatures: Vec<crate::fauna::Creature>,
    pub animal_species: Vec<crate::fauna::AnimalSpecies>,
    pub next_creature: u32,
    pub animals_began: Option<u64>,
    pub fauna_events: Vec<(u64, crate::fauna::FaunaEvent)>,
    pub first_hunter_seen: bool,
    pub cleverest_mark: f32,
    pub frng: Option<Rng>,
    /// diagnostics: animals born, starved, died old, eaten (cumulative)
    pub fauna_tally: [f32; 4],
    #[serde(skip)]
    pub crowd: Vec<f32>,
    /// Slots of extinct strains, reused so the table stays bounded.
    pub free: Vec<u32>,
    pub live: usize,
    pub strains_ever: u64,
}

fn temp_c(x: f32) -> f32 {
    4.0 + 28.0 * x
}

impl Evo {
    pub fn new(seed: u64) -> Evo {
        let n = W * H;
        Evo {
            layers: vec![[Layer { light: 0.0, temp: 12.0, oxygen: 0.6, nutrient: 1.0, detritus: 0.0 }; LAYERS]; n],
            soup: vec![0.0; n],
            pop: vec![Vec::new(); n * LAYERS],
            macro_photo: vec![0.0; n],
            macro_filter: vec![0.0; n],
            rng: Some(Rng::stream(seed, 9, 0)),
            frng: Some(Rng::stream(seed, 10, 0)),
            ..Default::default()
        }
    }

    pub fn total_biomass(&self) -> f32 {
        self.strains.iter().filter(|s| s.extinct.is_none()).map(|s| s.biomass).sum()
    }
    pub fn live_strains(&self) -> usize {
        self.live
    }
    pub fn live_species(&self) -> usize {
        self.species.iter().filter(|s| s.extinct.is_none()).count()
    }

    /// One day of chemistry and microbial life.
    pub fn daily(&mut self, world: &mut World, cfg: &EvolutionCfg, tick: u64, day: u64) {
        let mut r = self.rng.take().unwrap_or_else(|| Rng::new(1));
        self.physics(world);
        self.chemistry_and_origins(world, cfg, tick, &mut r);
        self.growth(world, cfg, tick, &mut r);
        self.disperse(cfg, &mut r);
        self.census(tick);
        if day % 12 == 0 {
            self.cluster_species(cfg, tick);
        }
        self.couple_to_beds(world);
        self.rng = Some(r);
    }

    fn physics(&mut self, world: &World) {
        let c = &world.climate;
        let light_base = c.brightness * (0.3 + c.lamp_frac) * (1.0 - 0.7 * c.ice);
        let mut next_n: Vec<[f32; LAYERS]> = self.layers.iter().map(|l| [l[0].nutrient, l[1].nutrient, l[2].nutrient]).collect();
        for i in 0..W * H {
            let cell = &world.cells[i];
            let ly = &mut self.layers[i];
            if cell.hab == Habitat::Rock {
                continue;
            }
            let d = cell.depth;
            ly[SURFACE].light = light_base;
            ly[MID].light = light_base * (1.0 - 0.5 * d);
            ly[BOTTOM].light = light_base * (1.0 - 0.8 * d);
            ly[SURFACE].temp = c.surface_t;
            ly[BOTTOM].temp = cell.temp;
            ly[MID].temp = 0.5 * (c.surface_t + cell.temp);
            // oxygen: exchange at the surface (blocked by ice), mixing downwards
            ly[SURFACE].oxygen += (1.0 - ly[SURFACE].oxygen) * 0.3 * (1.0 - c.ice);
            for k in 1..LAYERS {
                let mix = 0.1 + 0.02 * c.wind;
                let avg = 0.5 * (ly[k - 1].oxygen + ly[k].oxygen);
                ly[k].oxygen += (avg - ly[k].oxygen) * mix;
                ly[k - 1].oxygen += (avg - ly[k - 1].oxygen) * mix;
            }
            // nutrients: vents feed the bottom, storms the surface, detritus is recycled
            ly[BOTTOM].nutrient += cell.heat * 0.02;
            if c.storm {
                ly[SURFACE].nutrient += 0.3;
            }
            for k in 0..LAYERS {
                let rec = ly[k].detritus * 0.05;
                ly[k].detritus -= rec;
                ly[k].nutrient = (ly[k].nutrient + rec).min(6.0);
            }
            next_n[i] = [ly[0].nutrient, ly[1].nutrient, ly[2].nutrient];
        }
        // slow horizontal spreading of nutrients
        for y in 0..H {
            for x in 0..W {
                let i = y * W + x;
                if world.cells[i].hab == Habitat::Rock {
                    continue;
                }
                for k in 0..LAYERS {
                    let mut sum = 0.0;
                    let mut n = 0.0;
                    for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                        let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                        if nx < 0 || ny < 0 || nx >= W as i32 || ny >= H as i32 {
                            continue;
                        }
                        let j = ny as usize * W + nx as usize;
                        if world.cells[j].hab == Habitat::Rock {
                            continue;
                        }
                        sum += self.layers[j][k].nutrient;
                        n += 1.0;
                    }
                    if n > 0.0 {
                        next_n[i][k] += (sum / n - self.layers[i][k].nutrient) * 0.05;
                    }
                }
            }
        }
        for i in 0..W * H {
            for k in 0..LAYERS {
                self.layers[i][k].nutrient = next_n[i][k].max(0.0);
            }
        }
    }

    fn chemistry_and_origins(&mut self, world: &World, cfg: &EvolutionCfg, tick: u64, r: &mut Rng) {
        for i in 0..W * H {
            let cell = &world.cells[i];
            if cell.hab == Habitat::Rock {
                continue;
            }
            let ly = &self.layers[i];
            let alive_here: f32 = (0..LAYERS).map(|k| self.pop[i * LAYERS + k].iter().map(|e| e.1).sum::<f32>()).sum();
            let mut s = self.soup[i];
            s += cell.heat * 0.01 + ly[SURFACE].light * ly[SURFACE].nutrient.min(2.0) * 0.016 * (1.0 - cell.depth) - s * 0.01;
            if alive_here > 0.5 {
                s *= 0.9; // life consumes the soup
            }
            self.soup[i] = s;
            // once life exists it is rarer for it to arise again (living things consume the soup)
            let rarity = if self.life_began.is_some() { 0.05 } else { 1.0 };
            if alive_here < 0.01 && s > cfg.soup_threshold && r.chance(cfg.abiogenesis * rarity * (s - cfg.soup_threshold)) {
                let vent = cell.heat > 1.0;
                let g = StrainGenome {
                    photo: if vent { 0.1 } else { 0.8 },
                    chemo: if vent { 0.8 } else { 0.1 },
                    graze: 0.1,
                    temp_opt: ((ly[BOTTOM].temp - 4.0) / 28.0).clamp(0.0, 1.0),
                    temp_tol: 0.3,
                    size: 0.05,
                    multicell: 0.0,
                    defense: 0.05,
                    layer: if vent { 0.9 } else { 0.3 },
                    motility: 0.1,
                    hue: r.f32(),
                }
                .normalise();
                let layer = if vent { BOTTOM } else { SURFACE };
                let id = self.new_strain(g, None, tick);
                self.pop[i * LAYERS + layer].push((id, 0.5));
                self.soup[i] *= 0.5;
                if self.life_began.is_none() {
                    self.life_began = Some(tick);
                    self.events.push((tick, EvoEvent::LifeBegins { cell: i, strain: id, vent }));
                }
            }
        }
    }

    fn new_strain(&mut self, g: StrainGenome, parent: Option<u32>, tick: u64) -> u32 {
        let species = parent.map(|p| self.strains[p as usize].species).unwrap_or(u32::MAX);
        self.live += 1;
        self.strains_ever += 1;
        match self.free.pop() {
            Some(id) => {
                self.strains[id as usize] = Strain { id, g, parent, born: tick, extinct: None, biomass: 0.0, peak: 0.0, species };
                id
            }
            None => {
                let id = self.strains.len() as u32;
                self.strains.push(Strain { id, g, parent, born: tick, extinct: None, biomass: 0.0, peak: 0.0, species });
                id
            }
        }
    }

    fn growth(&mut self, world: &World, cfg: &EvolutionCfg, tick: u64, r: &mut Rng) {
        let mut births: Vec<(usize, u32, f32)> = Vec::new();
        for i in 0..W * H {
            let cell = &world.cells[i];
            if cell.hab == Habitat::Rock {
                continue;
            }
            for k in 0..LAYERS {
                let idx = i * LAYERS + k;
                if self.pop[idx].is_empty() {
                    continue;
                }
                let lay = self.layers[i][k];
                let vent = if k == BOTTOM { cell.heat } else { 0.0 };
                let total: f32 = self.pop[idx].iter().map(|e| e.1).sum();
                let room = (1.0 - total / cfg.layer_capacity).max(-0.5);
                let nut = lay.nutrient / (lay.nutrient + 0.5);
                // effective body size needs multicellularity; a grazer can only eat what it is big
                // enough to eat, so prey and grazers are drawn into an arms race of size
                let ents: Vec<(u32, f32, StrainGenome)> = self.pop[idx].iter().map(|&(s, b)| (s, b, self.strains[s as usize].g)).collect();
                let es = |g: &StrainGenome| g.size * (0.3 + 0.7 * g.multicell);
                let n_e = ents.len();
                // edible[a][b]: how much of b grazer a can eat
                let mut eat = [[0.0f32; SLOTS + 1]; SLOTS + 1];
                for a in 0..n_e {
                    for b in 0..n_e {
                        if a != b {
                            let (ga, gb) = (&ents[a].2, &ents[b].2);
                            let fit = (0.5 + (es(ga) - es(gb)) * 1.5).clamp(0.05, 1.0);
                            eat[a][b] = ents[b].1 * (1.0 - 0.7 * gb.defense) * fit;
                        }
                    }
                }
                let mut changes: Vec<f32> = vec![0.0; n_e];
                let mut losses: Vec<f32> = vec![0.0; n_e];
                let mut nutrient_used = 0.0f32;
                for a in 0..n_e {
                    let (_, b, g) = (ents[a].0, ents[a].1, ents[a].2);
                    let z = (lay.temp - temp_c(g.temp_opt)) / (2.0 + 10.0 * g.temp_tol);
                    let tf = libm::expf(-z * z);
                    let oxy = if g.graze > 0.3 { (lay.oxygen * 1.5).min(1.0) } else { 1.0 };
                    // multicellular producers shade out and outcompete thin films for light
                    let photo = g.photo * lay.light * nut * (0.7 + 0.6 * g.multicell);
                    let chemo = g.chemo * (vent * 0.15).min(1.0);
                    let avail: f32 = (0..n_e).map(|o| eat[a][o]).sum();
                    let prey = avail / (avail + 5.0);
                    let graze = g.graze * prey * (0.6 + 0.8 * es(&g)) * oxy;
                    // light and vent energy need room to grow into; grazing eats what is there and makes room
                    let gain = cfg.growth * b * tf * ((photo + chemo) * room.max(0.0) + graze);
                    let upkeep = b * (0.03 + 0.02 * g.size + 0.07 * g.size * g.size + 0.02 * g.multicell + 0.04 * g.multicell * g.multicell + 0.02 * g.motility + 0.02 * g.defense + 0.01 * g.temp_tol);
                    let eaten = cfg.growth * b * tf * graze * 1.3;
                    if avail > 0.0 {
                        for o in 0..n_e {
                            losses[o] += eaten * eat[a][o] / avail;
                        }
                    }
                    nutrient_used += cfg.growth * b * tf * photo * 0.05;
                    changes[a] = gain - upkeep - if room < 0.0 { b * (-room) * 0.2 } else { 0.0 };
                }
                let mut dead = 0.0f32;
                for o in 0..n_e {
                    changes[o] -= losses[o].min(ents[o].1 * 0.9);
                }
                let lay_mut = &mut self.layers[i][k];
                lay_mut.nutrient = (lay_mut.nutrient - nutrient_used).max(0.0);
                for (n, e) in self.pop[idx].iter_mut().enumerate() {
                    let nb = (e.1 + changes[n]).max(0.0);
                    if nb < e.1 {
                        dead += e.1 - nb;
                    }
                    e.1 = nb;
                    // photosynthesis makes oxygen, everything else uses it
                    let g = &self.strains[e.0 as usize].g;
                    lay_mut.oxygen = (lay_mut.oxygen + e.1 * (g.photo * 0.002 - (1.0 - g.photo) * 0.0015)).clamp(0.0, 1.0);
                    if e.1 > 0.3 && r.chance(cfg.mutation * e.1) {
                        births.push((idx, e.0, e.1 * 0.05));
                        e.1 *= 0.95;
                    }
                }
                lay_mut.detritus += dead * 0.5;
                self.pop[idx].retain(|e| e.1 > 0.005);
            }
        }
        // mutants
        for (idx, parent, b) in births {
            if self.live >= cfg.max_strains {
                // a crowded world: the mutant joins its parent instead
                if let Some(e) = self.pop[idx].iter_mut().find(|e| e.0 == parent) {
                    e.1 += b;
                }
                continue;
            }
            let g = self.strains[parent as usize].g.mutate(r, cfg.mutation_size);
            let id = self.new_strain(g, Some(parent), tick);
            self.pop[idx].push((id, b));
            if self.pop[idx].len() > SLOTS {
                self.pop[idx].sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
                self.pop[idx].truncate(SLOTS);
            }
        }
    }

    fn disperse(&mut self, cfg: &EvolutionCfg, r: &mut Rng) {
        let mut moves: Vec<(usize, u32, f32)> = Vec::new();
        for i in 0..W * H {
            let (x, y) = ((i % W) as i32, (i / W) as i32);
            for k in 0..LAYERS {
                let idx = i * LAYERS + k;
                for e in self.pop[idx].iter_mut() {
                    let g = &self.strains[e.0 as usize].g;
                    let frac = cfg.dispersal * (0.3 + g.motility);
                    if e.1 < 0.05 {
                        continue;
                    }
                    let amount = e.1 * frac;
                    // drift with the water to a neighbour, or swim toward the preferred layer
                    let want = ((g.layer * (LAYERS as f32 - 0.01)) as usize).min(LAYERS - 1);
                    let (tx, ty, tk) = if want != k && r.chance(0.5 + 0.5 * g.motility) {
                        (x, y, if want > k { k + 1 } else { k - 1 })
                    } else {
                        let d = r.below(4);
                        let (dx, dy) = [(1, 0), (-1, 0), (0, 1), (0, -1)][d as usize];
                        (x + dx, y + dy, k)
                    };
                    if tx < 0 || ty < 0 || tx >= W as i32 || ty >= H as i32 {
                        continue;
                    }
                    e.1 -= amount;
                    moves.push(((ty as usize * W + tx as usize) * LAYERS + tk, e.0, amount));
                }
            }
        }
        for (idx, s, b) in moves {
            let v = &mut self.pop[idx];
            if let Some(e) = v.iter_mut().find(|e| e.0 == s) {
                e.1 += b;
            } else if v.len() < SLOTS {
                v.push((s, b));
            } else if let Some(weak) = v.iter_mut().min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()) {
                // an incomer displaces the weakest resident only if it arrives in force
                if b > weak.1 {
                    *weak = (s, b);
                }
            }
        }
    }

    fn census(&mut self, tick: u64) {
        let mut bio = vec![0.0f32; self.strains.len()];
        for v in &self.pop {
            for &(s, b) in v {
                bio[s as usize] += b;
            }
        }
        for s in self.strains.iter_mut() {
            if s.extinct.is_some() {
                continue;
            }
            s.biomass = bio[s.id as usize];
            s.peak = s.peak.max(s.biomass);
            if s.biomass <= 0.0 {
                s.extinct = Some(tick);
                self.live -= 1;
                self.free.push(s.id);
            }
        }
    }

    /// Group living strains into species: a strain belongs to its parent's species unless it has
    /// drifted further than `species_distance` from that species' centroid (speciation).
    /// Fold near-identical strains of a species into the larger one, so the strain table holds
    /// real variation and new mutants always have room.
    fn merge_twins(&mut self, tick: u64) {
        let mut by_species: std::collections::BTreeMap<u32, Vec<u32>> = std::collections::BTreeMap::new();
        for s in self.strains.iter().filter(|s| s.extinct.is_none()) {
            by_species.entry(s.species).or_default().push(s.id);
        }
        let mut remap: Vec<(u32, u32)> = Vec::new();
        for (_, mut ids) in by_species {
            ids.sort_by(|a, b| self.strains[*b as usize].biomass.partial_cmp(&self.strains[*a as usize].biomass).unwrap().then(a.cmp(b)));
            let mut keep: Vec<u32> = Vec::new();
            for id in ids {
                let g = self.strains[id as usize].g;
                match keep.iter().find(|k| self.strains[**k as usize].g.distance(&g) < 0.035) {
                    Some(&into) => remap.push((id, into)),
                    None => keep.push(id),
                }
            }
        }
        if remap.is_empty() {
            return;
        }
        let map: std::collections::BTreeMap<u32, u32> = remap.iter().copied().collect();
        for v in self.pop.iter_mut() {
            for e in v.iter_mut() {
                if let Some(&into) = map.get(&e.0) {
                    e.0 = into;
                }
            }
            // combine duplicates now sharing an id
            v.sort_by_key(|e| e.0);
            let mut out: Vec<(u32, f32)> = Vec::with_capacity(v.len());
            for &(s, b) in v.iter() {
                match out.last_mut() {
                    Some(l) if l.0 == s => l.1 += b,
                    _ => out.push((s, b)),
                }
            }
            *v = out;
        }
        for (gone, _) in remap {
            let s = &mut self.strains[gone as usize];
            if s.extinct.is_none() {
                s.extinct = Some(tick);
                s.biomass = 0.0;
                self.live -= 1;
                self.free.push(gone);
            }
        }
    }

    fn cluster_species(&mut self, cfg: &EvolutionCfg, tick: u64) {
        self.merge_twins(tick);
        // a new kind of life is noted the first time any established strain is of that kind
        let mut new_kinds: Vec<(String, u32)> = Vec::new();
        for s in self.strains.iter().filter(|s| s.extinct.is_none() && s.biomass > 2.0 && s.species != u32::MAX) {
            let k = s.g.kind().to_string();
            if !self.kinds_seen.contains(&k) && !new_kinds.iter().any(|x| x.0 == k) {
                new_kinds.push((k, s.species));
            }
        }
        for (k, sp) in new_kinds {
            self.kinds_seen.push(k.clone());
            self.events.push((tick, EvoEvent::FirstOf { kind: k, species: sp }));
        }
        let live: Vec<u32> = self.strains.iter().filter(|s| s.extinct.is_none()).map(|s| s.id).collect();
        for &sid in &live {
            let s = self.strains[sid as usize].clone();
            let sp = s.species;
            let fits = sp != u32::MAX && self.species.get(sp as usize).map_or(false, |x| x.extinct.is_none() && x.centroid.distance(&s.g) <= cfg.species_distance);
            if !fits {
                let id = self.species.len() as u32;
                let parent = if sp == u32::MAX { None } else { Some(sp) };
                let kind = s.g.kind().to_string();
                self.species.push(Species { id, founded: tick, extinct: None, parent, kind: kind.clone(), biomass: 0.0, peak: 0.0, centroid: s.g });
                self.strains[sid as usize].species = id;
                // relatives that also left the old species follow the new one
                self.events.push((tick, EvoEvent::Speciation { species: id, parent, kind: kind.clone() }));
                if !self.kinds_seen.contains(&kind) {
                    self.kinds_seen.push(kind.clone());
                    self.events.push((tick, EvoEvent::FirstOf { kind, species: id }));
                }
            }
        }
        // centroids, biomass and extinctions
        let n = self.species.len();
        let mut sum = vec![(0.0f32, [0.0f32; 11]); n];
        for &sid in &live {
            let s = &self.strains[sid as usize];
            let g = s.g;
            let w = s.biomass.max(1e-4);
            let e = &mut sum[s.species as usize];
            e.0 += w;
            for (k, v) in [g.photo, g.chemo, g.graze, g.temp_opt, g.temp_tol, g.size, g.multicell, g.defense, g.layer, g.motility, g.hue].iter().enumerate() {
                e.1[k] += v * w;
            }
        }
        let mut lost = 0u32;
        let alive_before = self.live_species() as u32;
        for sp in self.species.iter_mut() {
            if sp.extinct.is_some() {
                continue;
            }
            let (w, v) = sum[sp.id as usize];
            if w <= 0.0 {
                sp.extinct = Some(tick);
                lost += 1;
                self.recent_extinctions.push(tick);
                self.events.push((tick, EvoEvent::Extinction { species: sp.id, kind: sp.kind.clone(), lived_days: (tick - sp.founded) / 1024 }));
                continue;
            }
            sp.biomass = w;
            sp.peak = sp.peak.max(w);
            sp.centroid = StrainGenome {
                photo: v[0] / w,
                chemo: v[1] / w,
                graze: v[2] / w,
                temp_opt: v[3] / w,
                temp_tol: v[4] / w,
                size: v[5] / w,
                multicell: v[6] / w,
                defense: v[7] / w,
                layer: v[8] / w,
                motility: v[9] / w,
                hue: v[10] / w,
            };
            sp.kind = sp.centroid.kind().to_string();
        }
        // a mass extinction: the number of living species falls to under half of its recent peak
        let _ = (lost, alive_before);
        let window = 2 * 48 * 1024;
        let now = self.live_species() as u32;
        self.diversity.push((tick, now));
        self.diversity.retain(|(t, _)| tick.saturating_sub(*t) < window);
        let peak = self.diversity.iter().map(|d| d.1).max().unwrap_or(0);
        if peak >= 8 && now * 2 < peak {
            self.events.push((tick, EvoEvent::MassExtinction { lost: peak - now, of: peak }));
            self.diversity.clear();
            self.diversity.push((tick, now));
        }
        let _ = fnv64;
    }

    /// Macro-producers grow algae beds and filter-feeders shellfish beds: the food people and
    /// creatures depend on exists only where it has evolved.
    fn couple_to_beds(&mut self, world: &mut World) {
        for i in 0..W * H {
            let mut photo = 0.0;
            let mut filter = 0.0;
            for k in [MID, BOTTOM] {
                for &(s, b) in &self.pop[i * LAYERS + k] {
                    let g = &self.strains[s as usize].g;
                    if g.multicell > 0.35 && g.photo > 0.5 {
                        photo += b * g.multicell;
                    }
                    if g.multicell > 0.35 && g.graze > 0.4 && g.size > 0.35 {
                        filter += b * g.multicell;
                    }
                }
            }
            self.macro_photo[i] = photo;
            self.macro_filter[i] = filter;
            world.cells[i].life_algae = (photo / 8.0).min(1.0);
            world.cells[i].life_shell = (filter / 8.0).min(1.0);
        }
    }
}
