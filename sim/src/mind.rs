//! Mind data structures (DESIGN §7): drives, personality, emotion, episodic memory,
//! semantic beliefs, contingency-learned predictions and theory of mind.
use crate::concepts::{cue, outcome, Concept};
use crate::genome::N_TRAITS;
use crate::language::Lexicon;
use crate::world::N_PLACES;
use serde::{Deserialize, Serialize};

pub const N_DRIVES: usize = 8;
pub mod drv {
    pub const HUNGER: usize = 0;
    pub const SAFETY: usize = 1;
    pub const REST: usize = 2;
    pub const BELONGING: usize = 3;
    pub const STATUS: usize = 4;
    pub const CURIOSITY: usize = 5;
    pub const REPRO: usize = 6;
    pub const CARE: usize = 7;
    pub const NAMES: [&str; 8] = ["hunger", "safety", "rest", "belonging", "status", "curiosity", "mating", "care-for-kin"];
}

pub const N_EMO: usize = 8;
pub mod emo {
    pub const FEAR: usize = 0;
    pub const ANGER: usize = 1;
    pub const GRIEF: usize = 2;
    pub const JOY: usize = 3;
    pub const TRUST: usize = 4;
    pub const SHAME: usize = 5;
    pub const SURPRISE: usize = 6;
    pub const DISGUST: usize = 7;
    pub const NAMES: [&str; 8] = ["afraid", "angry", "grieving", "glad", "trusting", "ashamed", "surprised", "disgusted"];
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EpKind {
    Attacked,
    SawAttack,
    KinDied,
    FriendDied,
    GotFood,
    Starving,
    WasShared,
    WasRefused,
    StolenFrom,
    CaughtStealing,
    Punished,
    Helped,
    Abandoned,
    Discovered,
    Learned,
    ChildBorn,
    FellSick,
    Recovered,
    Cold,
    HuntWon,
    HuntLost,
    UnfairShare,
    FalseAlarm,
    MetStranger,
    Bonded,
    Migrated,
    PredatorKilled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Episode {
    pub tick: u64,
    pub kind: EpKind,
    pub place: u8,
    pub other: Option<u32>,
    pub valence: f32,
    pub salience: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rel {
    /// subject (an action concept) is forbidden; object = sanction
    Forbidden,
    /// subject (person) is a thief
    Thief,
    /// subject (person) heals
    Heals,
    /// subject (person) leads / should be followed
    Leads,
    /// subject (person) teaches
    Teaches,
    /// subject (material concept) is cursed — always a false belief
    Cursed,
    /// subject (person) is a liar / unreliable
    Liar,
    /// subject (Kind::Sickness) spreads from the sick (true)
    Spreads,
    /// subject (Kind::Ancestors) watch over us
    Watch,
    /// subject (Kind::Lamp) governs the seasons
    Governs,
    /// subject is sacred
    Sacred,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Belief {
    pub subject: Concept,
    pub rel: Rel,
    pub conf: f32,
    pub source: Option<u32>,
    pub origin: u32,
    pub formed: u64,
    pub distorted: bool,
}

/// The cognitive map: what one mind believes about each place.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PlaceMem {
    pub food: f32,
    pub danger: f32,
    pub conf: f32,
    pub last_visit: u64,
    /// Where the danger belief came from: 0 = own experience, else told by (id + 1)
    pub danger_src: u32,
    pub danger_distorted: bool,
}

/// Contingency learning tables (cue → outcome within a horizon of days).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contingency {
    pub n_days: f32,
    pub n_cue: [f32; cue::N],
    pub n_out: [f32; outcome::N],
    pub n_co: [[f32; outcome::N]; cue::N],
    /// Rolling history (most recent last) of daily cue bitsets and outcome bitsets.
    pub hist_cues: Vec<u32>,
    pub hist_out: Vec<u8>,
    /// Pseudo-evidence received from others for (cue, outcome): (strength, teller)
    pub told: Vec<(u8, u8, f32, u32)>,
}

impl Default for Contingency {
    fn default() -> Self {
        Contingency {
            n_days: 0.0,
            n_cue: [0.0; cue::N],
            n_out: [0.0; outcome::N],
            n_co: [[0.0; outcome::N]; cue::N],
            hist_cues: Vec::new(),
            hist_out: Vec::new(),
            told: Vec::new(),
        }
    }
}

pub const HORIZON: usize = 10;

impl Contingency {
    /// Close a day: record today's cues/outcomes and update counts for the day whose horizon ended.
    pub fn end_day(&mut self, cues: u32, outs: u8, decay: f32) {
        self.hist_cues.push(cues);
        self.hist_out.push(outs);
        if self.hist_cues.len() > HORIZON + 1 {
            self.hist_cues.remove(0);
            self.hist_out.remove(0);
        }
        if self.hist_cues.len() < HORIZON + 1 {
            return;
        }
        // decay
        self.n_days *= decay;
        for c in 0..cue::N {
            self.n_cue[c] *= decay;
            for o in 0..outcome::N {
                self.n_co[c][o] *= decay;
            }
        }
        for o in 0..outcome::N {
            self.n_out[o] *= decay;
        }
        let day_cues = self.hist_cues[0];
        let mut window: u8 = 0;
        for k in 0..=HORIZON {
            window |= self.hist_out[k];
        }
        self.n_days += 1.0;
        for o in 0..outcome::N {
            if window & (1 << o) != 0 {
                self.n_out[o] += 1.0;
            }
        }
        for c in 0..cue::N {
            if day_cues & (1 << c) != 0 {
                self.n_cue[c] += 1.0;
                for o in 0..outcome::N {
                    if window & (1 << o) != 0 {
                        self.n_co[c][o] += 1.0;
                    }
                }
            }
        }
    }

    /// ΔP = P(o|c) − P(o|¬c), with told evidence blended in. Returns (dp, p(o|c), n_c).
    pub fn delta_p(&self, c: usize, o: usize) -> (f32, f32, f32) {
        let mut nc = self.n_cue[c];
        let mut nco = self.n_co[c][o];
        for t in &self.told {
            if t.0 as usize == c && t.1 as usize == o {
                nc += t.2;
                nco += t.2 * 0.9;
            }
        }
        if nc < 0.5 {
            return (0.0, 0.0, nc);
        }
        let p_c = (nco / nc).min(1.0);
        let n_not = (self.n_days - self.n_cue[c]).max(0.5);
        let p_not = ((self.n_out[o] - self.n_co[c][o]).max(0.0) / n_not).min(1.0);
        (p_c - p_not, p_c, nc)
    }

    /// Expected outcomes given the current cues: max over active cues of P(o|c) where the rule holds.
    pub fn expect(&self, cues: u32, min_dp: f32, min_n: f32) -> [(f32, i8); outcome::N] {
        let mut e = [(0.0f32, -1i8); outcome::N];
        for c in 0..cue::N {
            if cues & (1 << c) == 0 {
                continue;
            }
            for o in 0..outcome::N {
                let (dp, p, n) = self.delta_p(c, o);
                if dp >= min_dp && n >= min_n && p > e[o].0 {
                    e[o] = (p, c as i8);
                }
            }
        }
        e
    }

    /// The strongest learned rules (for gossip, thought logs and metrics).
    pub fn rules(&self, min_dp: f32, min_n: f32) -> Vec<(usize, usize, f32)> {
        let mut v = Vec::new();
        for c in 0..cue::N {
            for o in 0..outcome::N {
                let (dp, _, n) = self.delta_p(c, o);
                if dp >= min_dp && n >= min_n {
                    v.push((c, o, dp));
                }
            }
        }
        v.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap().then(a.0.cmp(&b.0)).then(a.1.cmp(&b.1)));
        v
    }
}

/// Theory of mind: what I think of someone I know.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Model {
    pub who: u32,
    pub trust: f32,
    pub affinity: f32,
    pub fear: f32,
    pub respect: f32,
    /// Favours: positive = they owe me
    pub ledger: f32,
    pub grudge: f32,
    pub last_seen: u64,
    pub kin: u8,
    pub accepts: u16,
    pub refusals: u16,
    /// Observed role behaviour counters (decayed): see social::role
    pub seen: [f32; 8],
    pub followed: u16,
}

pub mod kin {
    pub const NONE: u8 = 0;
    pub const PARENT: u8 = 1;
    pub const CHILD: u8 = 2;
    pub const SIBLING: u8 = 3;
    pub const MATE: u8 = 4;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mind {
    pub drives: [f32; N_DRIVES],
    pub traits: [f32; N_TRAITS],
    pub base_traits: [f32; N_TRAITS],
    pub emo: [f32; N_EMO],
    pub episodes: Vec<Episode>,
    pub beliefs: Vec<Belief>,
    pub places: Vec<PlaceMem>,
    pub cont: Contingency,
    pub models: Vec<Model>,
    pub lex: Lexicon,
    pub comm_try: u32,
    pub comm_ok: u32,
    pub today_cues: u32,
    pub today_out: u8,
    pub expect: [(f32, i8); outcome::N],
}

impl Mind {
    pub fn new(traits: [f32; N_TRAITS]) -> Mind {
        Mind {
            drives: [0.2; N_DRIVES],
            traits,
            base_traits: traits,
            emo: [0.0; N_EMO],
            episodes: Vec::new(),
            beliefs: Vec::new(),
            places: vec![PlaceMem { food: 0.3, danger: 0.1, conf: 0.0, last_visit: 0, danger_src: 0, danger_distorted: false }; N_PLACES],
            cont: Contingency::default(),
            models: Vec::new(),
            lex: Lexicon::default(),
            comm_try: 0,
            comm_ok: 0,
            today_cues: 0,
            today_out: 0,
            expect: [(0.0, -1); outcome::N],
        }
    }

    pub fn model(&self, who: u32) -> Option<&Model> {
        self.models.iter().find(|m| m.who == who)
    }

    pub fn model_mut(&mut self, who: u32, tick: u64, cap: usize) -> &mut Model {
        if let Some(i) = self.models.iter().position(|m| m.who == who) {
            return &mut self.models[i];
        }
        if self.models.len() >= cap {
            // forget the least important acquaintance (never kin)
            if let Some((i, _)) = self
                .models
                .iter()
                .enumerate()
                .filter(|(_, m)| m.kin == kin::NONE)
                .map(|(i, m)| (i, m.affinity.abs() + m.trust.abs() * 0.5 + m.grudge + m.respect - (tick.saturating_sub(m.last_seen)) as f32 * 1e-5))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap().then(a.0.cmp(&b.0)))
            {
                self.models.remove(i);
            }
        }
        self.models.push(Model {
            who,
            trust: 0.3,
            affinity: 0.0,
            fear: 0.0,
            respect: 0.2,
            ledger: 0.0,
            grudge: 0.0,
            last_seen: tick,
            kin: kin::NONE,
            accepts: 0,
            refusals: 0,
            seen: [0.0; 8],
            followed: 0,
        });
        self.models.last_mut().unwrap()
    }

    pub fn remember(&mut self, ep: Episode, cap: usize) {
        if self.episodes.len() >= cap {
            if let Some((i, _)) = self
                .episodes
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.salience.partial_cmp(&b.1.salience).unwrap().then(a.0.cmp(&b.0)))
            {
                if self.episodes[i].salience < ep.salience {
                    self.episodes.remove(i);
                } else {
                    return;
                }
            }
        }
        self.episodes.push(ep);
    }

    pub fn belief(&self, subject: Concept, rel: Rel) -> f32 {
        self.beliefs.iter().find(|b| b.subject == subject && b.rel == rel).map_or(0.0, |b| b.conf)
    }

    /// Strengthen or create a belief. Returns true if it was newly formed.
    pub fn believe(&mut self, subject: Concept, rel: Rel, delta: f32, source: Option<u32>, origin: u32, tick: u64, distorted: bool, cap: usize) -> bool {
        if let Some(b) = self.beliefs.iter_mut().find(|b| b.subject == subject && b.rel == rel) {
            b.conf = (b.conf + delta).clamp(0.0, 1.0);
            return false;
        }
        if delta <= 0.0 {
            return false;
        }
        if self.beliefs.len() >= cap {
            if let Some((i, _)) = self
                .beliefs
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.conf.partial_cmp(&b.1.conf).unwrap().then(a.0.cmp(&b.0)))
            {
                self.beliefs.remove(i);
            }
        }
        self.beliefs.push(Belief { subject, rel, conf: delta.min(1.0), source, origin, formed: tick, distorted });
        true
    }

    pub fn feel(&mut self, e: usize, amount: f32) {
        self.emo[e] = (self.emo[e] + amount).clamp(0.0, 1.0);
    }

    pub fn top_emotion(&self) -> Option<(usize, f32)> {
        let (i, v) = self.emo.iter().enumerate().fold((0, 0.0f32), |a, (i, v)| if *v > a.1 { (i, *v) } else { a });
        if v > 0.15 {
            Some((i, v))
        } else {
            None
        }
    }
}
