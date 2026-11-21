//! Thought log (DESIGN §7.12): compact structured records of meaningful decisions,
//! rendered to text on demand.
use crate::person::Act;
use serde::{Deserialize, Serialize};

pub const NONE: u8 = 255;

/// Reasons ("because") codes.
pub mod why {
    pub const HUNGER: u8 = 0;
    pub const SAFETY: u8 = 1;
    pub const REST: u8 = 2;
    pub const BELONGING: u8 = 3;
    pub const STATUS: u8 = 4;
    pub const CURIOSITY: u8 = 5;
    pub const MATING: u8 = 6;
    pub const CARE: u8 = 7;
    pub const EXPECT_HUNGER: u8 = 8;
    pub const EXPECT_ATTACK: u8 = 9;
    pub const EXPECT_COLD: u8 = 10;
    pub const EXPECT_SICK: u8 = 11;
    pub const TRUST: u8 = 12;
    pub const KIN: u8 = 13;
    pub const OWES_ME: u8 = 14;
    pub const GRUDGE: u8 = 15;
    pub const NORM: u8 = 16;
    pub const FEAR: u8 = 17;
    pub const ANGER: u8 = 18;
    pub const GRIEF: u8 = 19;
    pub const NEED: u8 = 20;
    pub const PLACE_FOOD: u8 = 21;
    pub const PLACE_DANGER: u8 = 22;
    pub const RESPECT: u8 = 23;
    pub const NIGHT: u8 = 24;
    pub const DESPERATE: u8 = 25;
    pub const CROWDED: u8 = 26;
    pub const STRANGER: u8 = 27;
    pub const STOCKPILE: u8 = 28;
    pub const HABIT: u8 = 29;
    pub const NAMES: [&str; 30] = [
        "hunger", "safety", "tired", "lonely", "status", "curious", "wants-a-mate", "kin-needs-care", "expects-hunger",
        "expects-attack", "expects-cold", "expects-sickness", "trusts", "kin", "they-owe-me", "grudge", "norm",
        "fear", "anger", "grief", "need", "food-there", "danger-there", "respect", "night", "desperate", "crowded",
        "stranger", "stockpiling", "habit",
    ];
}

/// Things perceived.
pub mod saw {
    pub const GULPER: u8 = 0;
    pub const HUNGRY: u8 = 1;
    pub const SICK: u8 = 2;
    pub const THIEF: u8 = 3;
    pub const STRANGER: u8 = 4;
    pub const HERD: u8 = 5;
    pub const PHANTOM: u8 = 6;
    pub const FOOD_LOW: u8 = 7;
    pub const NAMES: [&str; 8] = ["gulper", "hungry", "sick", "thief", "stranger", "drifters", "shadow(gulper?)", "food-low"];
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Thought {
    pub tick: u64,
    pub who: u32,
    pub felt: (u8, f32),
    pub drive: (u8, f32),
    /// (outcome, cue, probability)
    pub expect: (u8, u8, f32),
    pub saw: [(u8, u32); 2],
    pub chose: Act,
    pub why: [(u8, f32); 3],
    pub interrupt: bool,
    pub result: u8,
}

pub mod result {
    pub const NONE: u8 = 0;
    pub const ACCEPTED: u8 = 1;
    pub const REFUSED: u8 = 2;
}
