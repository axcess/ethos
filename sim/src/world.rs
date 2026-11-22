//! Phase 1 simplified tank: a 2-D top-down grid of cells with depth, habitats, materials,
//! regrowing food, temperature from the climate driver, predators (gulpers), drifter herds
//! and camps. (The full 3-D field model arrives in Phase 2.)
use crate::climate::DayClimate;
use crate::config::{Config, WorldCfg};
use crate::discovery::mat;
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

pub const W: usize = 64;
pub const H: usize = 32;
pub const CELL: f32 = 8.0;
pub const PW: usize = 8;
pub const PH: usize = 4;
pub const N_PLACES: usize = PW * PH;
pub const TANK_W: f32 = W as f32 * CELL;
pub const TANK_H: f32 = H as f32 * CELL;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Habitat {
    Shallows,
    Sand,
    Reef,
    Kelp,
    Deep,
    Vent,
    Rock,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cell {
    pub depth: f32,
    pub hab: Habitat,
    pub mats: u16,
    pub algae: f32,
    pub algae_cap: f32,
    pub shell: f32,
    pub shell_cap: f32,
    pub temp: f32,
    pub garden: f32,
    pub bed: bool,
    pub heat: f32,
    /// Wear from harvesting (0..1): worn beds regrow slowly until rested (fallow).
    pub fatigue: f32,
    pub harvested_today: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Place {
    pub id: u8,
    pub cx: f32,
    pub cy: f32,
    pub mats: u16,
    pub hab: Habitat,
    pub vent: bool,
    pub east: bool,
    pub passable: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Predator {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub tx: f32,
    pub ty: f32,
    pub energy: f32,
    pub age_days: f32,
    pub target: Option<u32>,
    pub alive: bool,
    pub cooldown: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Herd {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub tx: f32,
    pub ty: f32,
    pub count: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Structure {
    pub node: u8,
    pub material: u16,
    pub condition: f32,
    pub built: u64,
    pub builder: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Camp {
    pub id: u16,
    pub place: u8,
    pub x: f32,
    pub y: f32,
    pub founded: u64,
    pub founder: u32,
    pub name: Vec<u8>,
    pub structures: Vec<Structure>,
    pub abandoned: Option<u64>,
    /// Discovery nodes recorded with tally marks here (readable by members).
    pub records: Vec<u8>,
    pub members: u32,
    pub culture: u16,
}

impl Camp {
    pub fn has(&self, node: u8) -> bool {
        self.structures.iter().any(|s| s.node == node && s.condition > 0.2)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    pub cells: Vec<Cell>,
    pub places: Vec<Place>,
    pub predators: Vec<Predator>,
    pub herds: Vec<Herd>,
    pub camps: Vec<Camp>,
    pub climate: DayClimate,
    pub prev_climate: DayClimate,
    pub ridge_x: f32,
    pub next_pred_id: u32,
    pub bloom: f32,
    /// Scenario multiplier on regrowth (famine < 1 < windfall), per place.
    pub regrow_mult: Vec<f32>,
    /// Fouling of the water by crowded camps (0..1) per place.
    pub foul: Vec<f32>,
    /// Diagnostics: regrowth produced today (algae, shellfish).
    #[serde(skip)]
    pub grown_today: (f32, f32),
    /// Attacks per place in the current cycle (ground truth for belief checks).
    pub attacks_by_place: Vec<u32>,
    pub attacks_by_place_prev: Vec<u32>,
}

pub fn cell_index(x: f32, y: f32) -> usize {
    let cx = ((x / CELL) as isize).clamp(0, W as isize - 1) as usize;
    let cy = ((y / CELL) as isize).clamp(0, H as isize - 1) as usize;
    cy * W + cx
}
pub fn place_of(x: f32, y: f32) -> u8 {
    let px = ((x / (TANK_W / PW as f32)) as isize).clamp(0, PW as isize - 1) as usize;
    let py = ((y / (TANK_H / PH as f32)) as isize).clamp(0, PH as isize - 1) as usize;
    (py * PW + px) as u8
}
pub fn sq(x: f32) -> f32 {
    x * x
}
pub fn dist(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = ax - bx;
    let dy = ay - by;
    libm::sqrtf(dx * dx + dy * dy)
}

fn hash2(seed: u64, x: i32, y: i32, k: u64) -> f32 {
    let h = crate::rng::mix(seed ^ ((x as u64) << 32) ^ (y as u64 & 0xffff_ffff) ^ k.wrapping_mul(0x9E37));
    (h >> 40) as f32 / (1u64 << 24) as f32
}
/// Smooth value noise in [0,1].
fn noise(seed: u64, x: f32, y: f32, scale: f32, k: u64) -> f32 {
    let (fx, fy) = (x / scale, y / scale);
    let (ix, iy) = (libm::floorf(fx) as i32, libm::floorf(fy) as i32);
    let (tx, ty) = (fx - ix as f32, fy - iy as f32);
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let (sx, sy) = (s(tx), s(ty));
    let a = hash2(seed, ix, iy, k);
    let b = hash2(seed, ix + 1, iy, k);
    let c = hash2(seed, ix, iy + 1, k);
    let d = hash2(seed, ix + 1, iy + 1, k);
    let top = a + (b - a) * sx;
    let bot = c + (d - c) * sx;
    top + (bot - top) * sy
}

impl World {
    pub fn generate(seed: u64, cfg: &Config, r: &mut Rng) -> World {
        let wc = &cfg.world;
        let ridge_x = (W as f32 / 2.0 + r.range(-6.0, 6.0)) * CELL;
        let pass_a = r.range(3.0, 12.0);
        let pass_b = r.range(18.0, 28.0);
        let mut cells = Vec::with_capacity(W * H);
        for cy in 0..H {
            for cx in 0..W {
                let (x, y) = (cx as f32, cy as f32);
                let n1 = 0.65 * noise(seed, x, y, 12.0, 1) + 0.35 * noise(seed, x, y, 5.0, 2);
                let mut depth = (0.1 + 0.55 * n1 + 0.45 * (y / H as f32)).clamp(0.0, 1.0);
                let mut hab;
                let n2 = noise(seed, x, y, 6.0, 3);
                let n3 = noise(seed, x, y, 4.0, 4);
                let mut mats = 0u16;
                if depth < 0.38 {
                    hab = Habitat::Shallows;
                    mats |= mat::SAND | mat::SHELL;
                    if n3 > 0.55 {
                        mats |= mat::CLAY;
                    }
                } else if depth < 0.6 {
                    hab = if n2 > 0.55 { Habitat::Reef } else if n3 > 0.5 { Habitat::Kelp } else { Habitat::Sand };
                } else if depth < 0.78 {
                    hab = if n3 > 0.45 { Habitat::Kelp } else { Habitat::Sand };
                } else {
                    hab = Habitat::Deep;
                    mats |= mat::STONE;
                }
                match hab {
                    Habitat::Reef => {
                        mats |= mat::SHELL | mat::STONE;
                        if n3 > 0.6 {
                            mats |= mat::SPONGE;
                        }
                    }
                    Habitat::Kelp => {
                        mats |= mat::KELP;
                        if n2 > 0.7 {
                            mats |= mat::GLOW;
                        }
                    }
                    Habitat::Sand => {
                        mats |= mat::SAND;
                        if n2 > 0.45 {
                            mats |= mat::CLAY;
                        }
                    }
                    Habitat::Deep => {
                        if n2 > 0.6 {
                            mats |= mat::GLOW;
                        }
                    }
                    _ => {}
                }
                if noise(seed, x, y, 3.0, 5) > 0.78 {
                    mats |= mat::STONE;
                }
                if cfg.world.ridge {
                    let rx = (x + 0.5) * CELL;
                    if (rx - ridge_x).abs() < CELL * 1.2 && !((y - pass_a).abs() < 1.6 || (y - pass_b).abs() < 1.6) {
                        hab = Habitat::Rock;
                        depth = 0.05;
                        mats = mat::STONE;
                    }
                }
                let (acap, scap) = match hab {
                    Habitat::Shallows => (40.0, 7.0),
                    Habitat::Kelp => (24.0, 6.0),
                    Habitat::Reef => (14.0, 26.0),
                    Habitat::Sand => (10.0, 6.0),
                    Habitat::Deep => (3.0, 3.0),
                    Habitat::Vent => (8.0, 4.0),
                    Habitat::Rock => (0.0, 0.0),
                };
                let (acap, scap) = (acap * wc.algae_cap_scale, scap * wc.shell_cap_scale);
                cells.push(Cell {
                    depth,
                    hab,
                    mats,
                    algae: acap * 0.35,
                    algae_cap: acap,
                    shell: scap * 0.45,
                    shell_cap: scap,
                    temp: 12.0,
                    garden: 0.0,
                    fatigue: 0.0,
                    harvested_today: false,
                    bed: false,
                    heat: 0.0,
                });
            }
        }
        // Hydrothermal vents in deep water, at least one per side of the ridge when possible.
        let mut vents = Vec::new();
        for v in 0..cfg.world.vents {
            for _ in 0..200 {
                let cx = r.below(W as u32) as usize;
                let cy = r.below(H as u32) as usize;
                let east = (cx as f32 + 0.5) * CELL > ridge_x;
                if cells[cy * W + cx].depth > 0.62 && cells[cy * W + cx].hab != Habitat::Rock && (v % 2 == 0) != east {
                    vents.push((cx, cy));
                    break;
                }
            }
        }
        for &(vx, vy) in &vents {
            for cy in vy.saturating_sub(3)..(vy + 4).min(H) {
                for cx in vx.saturating_sub(3)..(vx + 4).min(W) {
                    let d = (sq(cx as f32 - vx as f32) + sq(cy as f32 - vy as f32)).sqrt();
                    let c = &mut cells[cy * W + cx];
                    if c.hab == Habitat::Rock {
                        continue;
                    }
                    c.heat = c.heat.max((1.0 - d / 4.0).max(0.0) * 9.0);
                    if d < 1.5 {
                        c.hab = Habitat::Vent;
                        c.mats |= mat::VENT | mat::STONE;
                        if d < 0.5 {
                            c.mats |= mat::VENTGLASS;
                        }
                        c.algae_cap = 8.0;
                        c.shell_cap = 10.0;
                    }
                }
            }
        }
        let mut places = Vec::new();
        for py in 0..PH {
            for px in 0..PW {
                let id = (py * PW + px) as u8;
                let (cx, cy) = ((px as f32 + 0.5) * TANK_W / PW as f32, (py as f32 + 0.5) * TANK_H / PH as f32);
                let mut m = 0u16;
                let mut counts = [0u32; 7];
                let mut vent = false;
                let mut rock = 0;
                for yy in py * (H / PH)..(py + 1) * (H / PH) {
                    for xx in px * (W / PW)..(px + 1) * (W / PW) {
                        let c = &cells[yy * W + xx];
                        m |= c.mats;
                        counts[c.hab as usize] += 1;
                        vent |= c.hab == Habitat::Vent;
                        if c.hab == Habitat::Rock {
                            rock += 1;
                        }
                    }
                }
                let hab_i = (0..7).filter(|i| *i != Habitat::Rock as usize).max_by_key(|i| (counts[*i], 7 - *i)).unwrap();
                let hab = [Habitat::Shallows, Habitat::Sand, Habitat::Reef, Habitat::Kelp, Habitat::Deep, Habitat::Vent, Habitat::Rock][hab_i];
                places.push(Place { id, cx, cy, mats: m, hab, vent, east: cx > ridge_x, passable: rock < 40 });
            }
        }
        let mut herds = Vec::new();
        for i in 0..wc.herds {
            let (x, y) = (r.range(20.0, TANK_W - 20.0), r.range(20.0, TANK_H - 20.0));
            herds.push(Herd { id: i, x, y, tx: x, ty: y, count: wc.herd_max * r.range(0.4, 0.8) });
        }
        let mut world = World {
            cells,
            places,
            predators: Vec::new(),
            herds,
            camps: Vec::new(),
            climate: DayClimate::default(),
            prev_climate: DayClimate::default(),
            ridge_x,
            next_pred_id: 0,
            bloom: 0.0,
            regrow_mult: vec![1.0; N_PLACES],
            foul: vec![0.0; N_PLACES],
            grown_today: (0.0, 0.0),
            attacks_by_place: vec![0; N_PLACES],
            attacks_by_place_prev: vec![0; N_PLACES],
        };
        for _ in 0..cfg.predators.initial {
            world.spawn_predator(r, None);
        }
        world
    }

    pub fn spawn_predator(&mut self, r: &mut Rng, at: Option<(f32, f32)>) {
        let (x, y) = at.unwrap_or_else(|| (r.range(10.0, TANK_W - 10.0), r.range(TANK_H * 0.5, TANK_H - 10.0)));
        let id = self.next_pred_id;
        self.next_pred_id += 1;
        self.predators.push(Predator { id, x, y, tx: x, ty: y, energy: 1.5, age_days: 0.0, target: None, alive: true, cooldown: 0 });
    }

    pub fn on_same_side(&self, ax: f32, bx: f32) -> bool {
        (ax > self.ridge_x) == (bx > self.ridge_x)
    }

    /// Travel distance, detouring through a ridge pass if the ridge lies between.
    pub fn travel_dist(&self, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
        let d = dist(ax, ay, bx, by);
        if self.on_same_side(ax, bx) {
            d
        } else {
            d * 1.6 + 20.0
        }
    }

    /// Daily environment update: temperatures, light, regrowth.
    /// Camps foul the water around them in proportion to how many live there.
    pub fn update_foul(&mut self, wc: &WorldCfg) {
        let mut load = vec![0.0f32; N_PLACES];
        for c in self.camps.iter().filter(|c| c.abandoned.is_none()) {
            load[c.place as usize] += c.members as f32;
        }
        for p in 0..N_PLACES {
            let f = &mut self.foul[p];
            *f = (*f * (1.0 - wc.foul_clear) + load[p] * wc.foul_per_person).clamp(0.0, 1.0);
        }
    }

    pub fn daily(&mut self, wc: &WorldCfg) {
        let c = &self.climate;
        if c.storm {
            self.bloom = (self.bloom + wc.storm_bloom).min(1.5);
        }
        self.bloom *= 0.85;
        let deep_t = 8.0;
        let light_base = c.brightness * (0.3 + c.lamp_frac) * (1.0 - 0.7 * c.ice);
        let mut grown = (0.0f32, 0.0f32);
        for y in 0..H {
            for x in 0..W {
                let i = y * W + x;
                let place = place_of((x as f32 + 0.5) * CELL, (y as f32 + 0.5) * CELL) as usize;
                let mult = self.regrow_mult[place] * (1.0 - 0.5 * self.foul[place]);
                let cell = &mut self.cells[i];
                if cell.hab == Habitat::Rock {
                    continue;
                }
                cell.temp = c.surface_t + (deep_t - c.surface_t) * cell.depth * 0.8 + cell.heat;
                let light = light_base * (1.0 - 0.8 * cell.depth);
                let tf = libm::expf(-sq((cell.temp - 20.0) / 9.0));
                let cap = cell.algae_cap * (1.0 + cell.garden * 0.4 + self.bloom * 0.5);
                if cap > 0.0 {
                    let a = cell.algae;
                    let growth = wc.algae_regrow * light * tf * a * (1.0 - a / cap) + 0.008 * cap * light * tf;
                    // tending (weeding, clearing, spreading) makes beds grow faster; wear undoes it
                    let worn = (1.0 + wc.garden_growth * cell.garden) * (1.0 - wc.fatigue_effect * cell.fatigue);
                    let na = (a + growth * mult * worn).clamp(0.0, cap);
                    grown.0 += (na - a).max(0.0);
                    cell.algae = na;
                }
                let scap = cell.shell_cap;
                if scap > 0.0 {
                    let s = cell.shell;
                    let allee = if cell.bed { 0.0 } else { wc.shell_allee };
                    let g = wc.shell_regrow * s * (1.0 - s / scap) * ((s / scap - allee) * 4.0).clamp(-1.0, 1.0);
                    let ns = (s + g * mult + 0.0005 * scap).clamp(0.0, scap);
                    grown.1 += (ns - s).max(0.0);
                    cell.shell = ns;
                }
                cell.garden *= 0.995;
                // rest restores worn beds; continued harvest keeps them worn
                cell.fatigue = (cell.fatigue - if cell.harvested_today { wc.fatigue_rest * 0.2 } else { wc.fatigue_rest }).max(0.0);
                cell.harvested_today = false;
            }
        }
        self.grown_today = grown;
    }

    /// Food available to forage at a place: (best cell index, amount)
    pub fn best_cells(&self, place: u8, k: usize) -> Vec<usize> {
        let p = place as usize;
        let (px, py) = (p % PW, p / PW);
        let mut v: Vec<(f32, usize)> = Vec::with_capacity(64);
        for yy in py * (H / PH)..(py + 1) * (H / PH) {
            for xx in px * (W / PW)..(px + 1) * (W / PW) {
                let i = yy * W + xx;
                let c = &self.cells[i];
                v.push((c.algae + c.shell * 1.5, i));
            }
        }
        v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap().then(a.1.cmp(&b.1)));
        v.into_iter().take(k).map(|x| x.1).collect()
    }

    pub fn place_food(&self, place: u8) -> f32 {
        self.best_cells(place, 6).iter().map(|&i| self.cells[i].algae + self.cells[i].shell * 1.5).sum::<f32>()
    }

    pub fn place_temp(&self, place: u8) -> f32 {
        let p = &self.places[place as usize];
        self.cells[cell_index(p.cx, p.cy)].temp
    }

    pub fn cell_center(i: usize) -> (f32, f32) {
        (((i % W) as f32 + 0.5) * CELL, ((i / W) as f32 + 0.5) * CELL)
    }
}
