//! The shared ontology minds think and talk about (DESIGN §7.6). Words attach to concepts.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Concept {
    Kind(Kind),
    Place(u8),
    Person(u32),
    Camp(u16),
    Node(u8),
}

macro_rules! kinds {
    ($($k:ident),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub enum Kind { $($k),* }
        pub const ALL_KINDS: &[Kind] = &[$(Kind::$k),*];
        impl Kind {
            pub fn gloss(self) -> &'static str {
                match self { $(Kind::$k => stringify!($k)),* }
            }
        }
    };
}

kinds!(
    // things in the world
    Algae, Shellfish, Drifter, Gulper, Food, Water, Lamp, Glass, Vent, Stone, Shell, Kelp, Clay, Sponge,
    Glowworm, Sand,
    // states and weather
    Cold, Warm, Dim, Bright, Storm, Ice, Sickness, Death, Hunger, Plenty, Danger, Night, Day,
    // people & social
    Child, Mother, Father, Mate, Friend, Stranger, Elder, Us, Them, Ancestors,
    // actions
    Eat, Go, Give, Take, Fight, Flee, Help, Sleep, Make, Teach, Hunt, Steal,
    // evaluation
    Good, Bad, Many, Few, Forbidden,
);

impl Concept {
    /// Abstract concepts count toward the language "abstract vocabulary" measure.
    pub fn is_abstract(&self) -> bool {
        matches!(
            self,
            Concept::Kind(
                Kind::Ancestors | Kind::Forbidden | Kind::Good | Kind::Bad | Kind::Death | Kind::Us | Kind::Them | Kind::Lamp
            ) | Concept::Node(_)
        )
    }
    pub fn key(&self) -> u64 {
        match *self {
            Concept::Kind(k) => k as u64,
            Concept::Place(p) => 1_000 + p as u64,
            Concept::Camp(c) => 10_000 + c as u64,
            Concept::Node(n) => 20_000 + n as u64,
            Concept::Person(p) => 1_000_000 + p as u64,
        }
    }
}

/// Cues and outcomes for contingency learning (DESIGN §7.7).
pub mod cue {
    pub const SEASON0: usize = 0; // 0..4 seasons
    pub const SEASON_LATE: usize = 4; // last third of a season (needs a calendar to perceive sharply)
    pub const LAMP_SHORTENING: usize = 5;
    pub const LAMP_LENGTHENING: usize = 6;
    pub const TEMP_FALLING: usize = 7;
    pub const TEMP_RISING: usize = 8;
    pub const COLD: usize = 9;
    pub const WARM: usize = 10;
    pub const STORM: usize = 11;
    pub const ICE: usize = 12;
    pub const PREDATOR_SEEN: usize = 13;
    pub const SICK_NEARBY: usize = 14;
    pub const STASH_LOW: usize = 15;
    pub const CROWDED: usize = 16;
    pub const STRANGER: usize = 17;
    pub const DIM: usize = 18;
    pub const N: usize = 19;
    pub const NAMES: [&str; N] = [
        "early-season", "growing-season", "late-season", "dark-season", "season-ending", "light-shortening",
        "light-lengthening", "water-cooling", "water-warming", "cold", "warm", "storm", "ice-overhead",
        "gulper-seen", "sick-nearby", "stash-low", "crowded-camp", "stranger-seen", "dim-light",
    ];
}

pub mod outcome {
    pub const HUNGER: usize = 0;
    pub const ATTACK: usize = 1;
    pub const SICKNESS: usize = 2;
    pub const COLD_STRESS: usize = 3;
    pub const PLENTY: usize = 4;
    pub const DEATH: usize = 5;
    pub const N: usize = 6;
    pub const NAMES: [&str; N] = ["hunger", "gulper-attack", "sickness", "cold-suffering", "plenty", "a-death"];
}
