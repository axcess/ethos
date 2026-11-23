//! Fixed-layout genome (DESIGN §6). Phase 1 seeds sapients from a hand-tuned distribution;
//! Phase 2 will evolve the same type from primordial life.
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

pub const N_TRAITS: usize = 6;
pub mod tr {
    pub const BOLD: usize = 0;
    pub const SOCIABLE: usize = 1;
    pub const CURIOUS: usize = 2;
    pub const AGGRESSIVE: usize = 3;
    pub const PATIENT: usize = 4;
    pub const CONFORM: usize = 5;
    pub const NAMES: [&str; 6] = ["boldness", "sociability", "curiosity", "aggression", "patience", "conformity"];
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genome {
    // body
    pub size: f32,
    pub grip: f32,
    pub face: f32,
    pub hue: [f32; 3],
    pub hard_parts: f32,
    // metabolism
    pub efficiency: f32,
    pub temp_opt: f32,
    pub temp_tol: f32,
    pub max_age: f32,
    // senses
    pub vision: f32,
    pub chemo: f32,
    // reproduction
    pub fertility: f32,
    pub parental_care: f32,
    pub mating_sig: [f32; 3],
    // cognition
    pub capacity: f32,
    pub learning: f32,
    pub social_learning: f32,
    pub play: f32,
    // social
    pub sociality: f32,
    pub kin_bias: f32,
    pub dominance: f32,
    pub cooperation: f32,
    // personality baselines
    pub traits: [f32; N_TRAITS],
    // voice (constrains phonology, drives spoken sound in later phases)
    pub pitch: f32,
    pub lips: f32,
    pub tongue: f32,
    pub throat: f32,
    // animal body & ecology (Phase 2): 0 = grazer of plants and mats … 1 = hunter of animals
    #[serde(default)]
    pub diet: f32,
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub armor: f32,
    /// preferred depth: 0 surface … 1 bottom
    #[serde(default)]
    pub layer: f32,
}

fn c01(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

impl Genome {
    /// A founding sapient drawn from the Phase 1 seed distribution.
    pub fn seed(r: &mut Rng, cap_min: f32, cap_max: f32, age_min: f32, age_max: f32, lineage: &Genome) -> Genome {
        let n = |r: &mut Rng, base: f32, sd: f32| c01(base + r.normal() * sd);
        let mut traits = [0.0; N_TRAITS];
        for (i, t) in traits.iter_mut().enumerate() {
            *t = n(r, lineage.traits[i], 0.18);
        }
        Genome {
            size: n(r, lineage.size, 0.05),
            grip: n(r, 0.8, 0.05),
            face: n(r, 0.8, 0.05),
            hue: [n(r, lineage.hue[0], 0.04), n(r, lineage.hue[1], 0.05), n(r, lineage.hue[2], 0.05)],
            hard_parts: n(r, 0.4, 0.1),
            efficiency: n(r, 0.5, 0.1),
            temp_opt: n(r, lineage.temp_opt, 0.04),
            temp_tol: n(r, 0.5, 0.1),
            max_age: r.range(age_min, age_max),
            vision: n(r, 0.6, 0.1),
            chemo: n(r, 0.5, 0.1),
            fertility: n(r, 0.5, 0.12),
            parental_care: n(r, 0.6, 0.12),
            mating_sig: [n(r, lineage.mating_sig[0], 0.03), n(r, lineage.mating_sig[1], 0.03), n(r, lineage.mating_sig[2], 0.03)],
            capacity: r.range(cap_min, cap_max),
            learning: n(r, 0.5, 0.12),
            social_learning: n(r, 0.5, 0.15),
            play: n(r, 0.5, 0.15),
            sociality: n(r, lineage.sociality, 0.15),
            kin_bias: n(r, 0.6, 0.15),
            dominance: n(r, 0.4, 0.18),
            cooperation: n(r, lineage.cooperation, 0.15),
            traits,
            pitch: n(r, lineage.pitch, 0.08),
            lips: n(r, lineage.lips, 0.05),
            tongue: n(r, lineage.tongue, 0.05),
            throat: n(r, lineage.throat, 0.05),
            diet: n(r, 0.4, 0.05),
            speed: n(r, 0.5, 0.05),
            armor: n(r, 0.2, 0.05),
            layer: n(r, 0.8, 0.05),
        }
    }

    /// A lineage template: each founding band is drawn around its own template.
    pub fn lineage(r: &mut Rng) -> Genome {
        let mut traits = [0.0; N_TRAITS];
        for t in traits.iter_mut() {
            *t = r.range(0.3, 0.7);
        }
        Genome {
            size: r.range(0.4, 0.6),
            grip: 0.8,
            face: 0.8,
            hue: [r.f32(), r.range(0.3, 0.8), r.range(0.3, 0.7)],
            hard_parts: 0.4,
            efficiency: 0.5,
            temp_opt: r.range(0.4, 0.6),
            temp_tol: 0.5,
            max_age: 40.0,
            vision: 0.6,
            chemo: 0.5,
            fertility: 0.5,
            parental_care: 0.6,
            mating_sig: [r.f32(), r.f32(), r.f32()],
            capacity: 0.85,
            learning: 0.5,
            social_learning: 0.5,
            play: 0.5,
            sociality: r.range(0.4, 0.7),
            kin_bias: 0.6,
            dominance: 0.4,
            cooperation: r.range(0.35, 0.65),
            traits,
            pitch: r.f32(),
            lips: r.range(0.3, 1.0),
            tongue: r.range(0.3, 1.0),
            throat: r.range(0.0, 1.0),
            diet: 0.4,
            speed: 0.5,
            armor: 0.2,
            layer: 0.8,
        }
    }

    /// The first animals of a lineage, descended from a multicellular grazing strain.
    pub fn from_strain(s: &crate::evo::StrainGenome, r: &mut Rng) -> Genome {
        let n = |r: &mut Rng, base: f32, sd: f32| c01(base + r.normal() * sd);
        let mut traits = [0.0; N_TRAITS];
        for t in traits.iter_mut() {
            *t = r.range(0.2, 0.8);
        }
        Genome {
            size: n(r, s.size * 0.6, 0.05),
            grip: n(r, 0.05, 0.03),
            face: n(r, 0.1, 0.05),
            hue: [s.hue, r.range(0.3, 0.8), r.range(0.3, 0.7)],
            hard_parts: n(r, s.defense, 0.05),
            efficiency: n(r, 0.5, 0.1),
            temp_opt: s.temp_opt,
            temp_tol: s.temp_tol,
            max_age: r.range(4.0, 8.0),
            vision: n(r, 0.2, 0.05),
            chemo: n(r, 0.5, 0.1),
            fertility: n(r, 0.7, 0.1),
            parental_care: n(r, 0.05, 0.03),
            mating_sig: [r.f32(), r.f32(), r.f32()],
            capacity: n(r, 0.03, 0.02),
            learning: n(r, 0.2, 0.05),
            social_learning: n(r, 0.05, 0.03),
            play: n(r, 0.05, 0.03),
            sociality: n(r, 0.2, 0.1),
            kin_bias: n(r, 0.2, 0.1),
            dominance: n(r, 0.3, 0.1),
            cooperation: n(r, 0.1, 0.05),
            traits,
            pitch: r.f32(),
            lips: r.f32(),
            tongue: r.f32(),
            throat: r.f32(),
            diet: n(r, 0.1, 0.05),
            speed: n(r, s.motility, 0.05),
            armor: n(r, s.defense, 0.05),
            layer: s.layer,
        }
    }

    /// Asexual copy with mutation (used when an animal group splits off).
    pub fn mutant(&self, r: &mut Rng, rate: f32, sd: f32) -> Genome {
        Genome::child(self, self, r, rate, sd)
    }

    /// Distance between two animals' genomes, for species clustering.
    pub fn distance(&self, o: &Genome) -> f32 {
        let d = [
            self.diet - o.diet,
            self.size - o.size,
            self.speed - o.speed,
            self.armor - o.armor,
            self.layer - o.layer,
            self.capacity - o.capacity,
            self.sociality - o.sociality,
            (self.mating_sig[0] - o.mating_sig[0]) * 2.0,
            (self.mating_sig[1] - o.mating_sig[1]) * 2.0,
            (self.mating_sig[2] - o.mating_sig[2]) * 2.0,
        ];
        libm::sqrtf(d.iter().map(|x| x * x).sum::<f32>() / d.len() as f32)
    }

    /// Sexual reproduction: uniform crossover by gene, then Gaussian mutation.
    pub fn child(a: &Genome, b: &Genome, r: &mut Rng, rate: f32, sd: f32) -> Genome {
        let mut g = a.clone();
        macro_rules! cross {
            ($($f:ident),*) => { $( if r.chance(0.5) { g.$f = b.$f; } if r.chance(rate) { g.$f = c01(g.$f + r.normal() * sd); } )* };
        }
        cross!(diet, speed, armor, layer, size, grip, face, hard_parts, efficiency, temp_opt, temp_tol, vision, chemo, fertility, parental_care,
               capacity, learning, social_learning, play, sociality, kin_bias, dominance, cooperation, pitch, lips, tongue, throat);
        if r.chance(0.5) {
            g.max_age = b.max_age;
        }
        g.max_age = (g.max_age + r.normal() * 2.0).max(10.0);
        for i in 0..3 {
            if r.chance(0.5) {
                g.hue[i] = b.hue[i];
            }
            if r.chance(0.5) {
                g.mating_sig[i] = b.mating_sig[i];
            }
            if r.chance(rate) {
                g.mating_sig[i] = c01(g.mating_sig[i] + r.normal() * sd * 0.5);
            }
        }
        for i in 0..N_TRAITS {
            if r.chance(0.5) {
                g.traits[i] = b.traits[i];
            }
            if r.chance(rate * 2.0) {
                g.traits[i] = c01(g.traits[i] + r.normal() * sd);
            }
        }
        g
    }
}
