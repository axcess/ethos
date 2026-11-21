//! A sapient individual: body state, mind, knowledge, life story, current action.
use crate::discovery::{need, Know};
use crate::genome::Genome;
use crate::language::{Form, Phonology, SoundLaw};
use crate::mind::Mind;
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum Act {
    Idle,
    Forage { place: u8 },
    Store,
    Eat,
    Rest,
    Sleep,
    Flee,
    Socialize { who: u32 },
    Share { who: u32 },
    AskHelp { who: u32 },
    Hunt { task: u32 },
    Defend { pred: u32 },
    Teach { who: u32, node: u8 },
    Experiment,
    Build { node: u8 },
    Steal { who: u32 },
    Punish { who: u32 },
    Court { who: u32 },
    Explore { place: u8 },
    Migrate { camp: u16 },
    Found { place: u8 },
    Mourn,
    Heal { who: u32 },
    Play,
    Follow { who: u32 },
}

pub const N_ACT: usize = 25;
pub const ACT_NAMES: [&str; N_ACT] = [
    "idle", "forage", "store", "eat", "rest", "sleep", "flee", "socialize", "share", "ask-help", "hunt", "defend",
    "teach", "experiment", "build", "steal", "punish", "court", "explore", "migrate", "found-camp", "mourn", "heal",
    "play", "follow",
];

impl Act {
    pub fn kind(&self) -> usize {
        match self {
            Act::Idle => 0,
            Act::Forage { .. } => 1,
            Act::Store => 2,
            Act::Eat => 3,
            Act::Rest => 4,
            Act::Sleep => 5,
            Act::Flee => 6,
            Act::Socialize { .. } => 7,
            Act::Share { .. } => 8,
            Act::AskHelp { .. } => 9,
            Act::Hunt { .. } => 10,
            Act::Defend { .. } => 11,
            Act::Teach { .. } => 12,
            Act::Experiment => 13,
            Act::Build { .. } => 14,
            Act::Steal { .. } => 15,
            Act::Punish { .. } => 16,
            Act::Court { .. } => 17,
            Act::Explore { .. } => 18,
            Act::Migrate { .. } => 19,
            Act::Found { .. } => 20,
            Act::Mourn => 21,
            Act::Heal { .. } => 22,
            Act::Play => 23,
            Act::Follow { .. } => 24,
        }
    }
    pub fn name(&self) -> &'static str {
        ACT_NAMES[self.kind()]
    }
    pub fn target(&self) -> Option<u32> {
        match *self {
            Act::Socialize { who } | Act::Share { who } | Act::AskHelp { who } | Act::Teach { who, .. } | Act::Steal { who }
            | Act::Punish { who } | Act::Court { who } | Act::Heal { who } | Act::Follow { who } => Some(who),
            _ => None,
        }
    }
}

/// An intention spanning several actions (anticipatory plans).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum Intent {
    /// Gather extra food until the stash reaches `target` (anticipating scarcity).
    Stockpile { target: f32, until: u64 },
    /// Avoid a place believed dangerous until `until`.
    Avoid { place: u8, until: u64 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LifeEntry {
    pub tick: u64,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Person {
    pub id: u32,
    pub alive: bool,
    pub name: Form,
    pub name_gloss: String,
    pub female: bool,
    pub born: u64,
    pub died: Option<u64>,
    pub death_cause: Option<String>,
    pub mother: Option<u32>,
    pub father: Option<u32>,
    pub mate: Option<u32>,
    pub children: Vec<u32>,
    pub genome: Genome,
    pub culture: u16,
    pub phon: Phonology,
    pub innovation: Option<SoundLaw>,
    pub home: Option<u16>,
    pub energy: f32,
    pub health: f32,
    pub carried: f32,
    pub stash: f32,
    pub last_upd: u64,
    pub infected: Option<u64>,
    pub immune_until: u64,
    pub healed_today: bool,
    pub pregnant: Option<(u64, u32)>,
    pub act: Act,
    pub act_start: u64,
    pub next_decision: u64,
    pub from: (f32, f32),
    pub to: (f32, f32),
    pub intent: Option<Intent>,
    pub mind: Mind,
    pub knowledge: Vec<Know>,
    pub skill_forage: f32,
    pub skill_hunt: f32,
    pub activity: [f32; N_ACT],
    pub activity_total: [u32; N_ACT],
    pub needs: [f32; need::N],
    pub rng: Rng,
    pub saw_predator_today: bool,
    pub alarm_until: u64,
    pub life: Vec<LifeEntry>,
    pub thoughts: Vec<crate::thought::Thought>,
    pub roles: u16,
    pub notability: f32,
    pub inventions: u32,
    pub stranger: bool,
    pub predators_killed: u32,
    pub founded_camps: u32,
    pub consider_move_day: u64,
}

impl Person {
    pub fn knows(&self, node: u8) -> bool {
        self.knowledge.iter().any(|k| k.node == node)
    }
    pub fn know(&self, node: u8) -> Option<&crate::discovery::Know> {
        self.knowledge.iter().find(|k| k.node == node)
    }
    pub fn age_ticks(&self, now: u64) -> u64 {
        now.saturating_sub(self.born)
    }
    pub fn age_cycles(&self, now: u64, tpc: u64) -> f32 {
        self.age_ticks(now) as f32 / tpc as f32
    }
    pub fn pos(&self, now: u64) -> (f32, f32) {
        if now >= self.next_decision || self.next_decision <= self.act_start {
            return self.to;
        }
        let t = (now - self.act_start) as f32 / (self.next_decision - self.act_start) as f32;
        let t = (t * 1.6).min(1.0); // travel happens in the first part of an action
        (self.from.0 + (self.to.0 - self.from.0) * t, self.from.1 + (self.to.1 - self.from.1) * t)
    }
    pub fn life_note(&mut self, tick: u64, text: String) {
        if self.life.len() >= 60 {
            self.life.remove(1);
        }
        self.life.push(LifeEntry { tick, text });
    }
    pub fn display_name(&self) -> String {
        crate::language::name_case(&self.name)
    }
}
