//! Phase 1 discovery subgraph (~26 aquatic nodes) and the Discovery Registry (DESIGN §10, §13.2).
//! Discoveries come from need + experimentation + chance + local materials. No research points.
use serde::{Deserialize, Serialize};

pub mod mat {
    pub const SHELL: u16 = 1;
    pub const STONE: u16 = 2;
    pub const KELP: u16 = 4;
    pub const CLAY: u16 = 8;
    pub const SPONGE: u16 = 16;
    pub const GLOW: u16 = 32;
    pub const VENT: u16 = 64;
    pub const SAND: u16 = 128;
    pub const VENTGLASS: u16 = 256;
    pub const NAMES: [(u16, &str); 9] = [
        (SHELL, "shell"), (STONE, "stone"), (KELP, "kelp"), (CLAY, "clay"), (SPONGE, "sponge"),
        (GLOW, "glowworm"), (VENT, "vent-mineral"), (SAND, "sand"), (VENTGLASS, "vent-glass"),
    ];
    pub fn name(m: u16) -> &'static str {
        NAMES.iter().find(|x| x.0 == m).map(|x| x.1).unwrap_or("?")
    }
    pub fn list(bits: u16) -> impl Iterator<Item = u16> {
        NAMES.iter().map(|x| x.0).filter(move |m| bits & m != 0)
    }
}

/// Need signals tracked per mind (decayed counters). Discoveries are biased toward needs.
pub mod need {
    pub const HUNGER: usize = 0;
    pub const SPOILED: usize = 1;
    pub const STOLEN: usize = 2;
    pub const PREDATOR: usize = 3;
    pub const COLD: usize = 4;
    pub const SICK: usize = 5;
    pub const CARRY: usize = 6;
    pub const HUNT_FAIL: usize = 7;
    pub const GRIEF: usize = 8;
    pub const CONFLICT: usize = 9;
    pub const DARK: usize = 10;
    pub const N: usize = 11;
    pub const NAMES: [&str; N] = [
        "hunger", "spoiled-food", "theft", "predators", "cold", "sickness", "carrying", "failed-hunts", "grief",
        "conflict", "darkness",
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Tool,
    Structure,
    Technique,
    Practice,
    Belief,
    Institution,
}

pub struct NodeDef {
    pub id: u8,
    pub name: &'static str,
    pub kind: NodeKind,
    pub function: &'static str,
    /// Any of these materials (canonical first); others in the list are variants.
    pub materials: &'static [u16],
    pub prereq: &'static [u8],
    pub needs: &'static [usize],
    pub min_group: u8,
    pub difficulty: f32,
    pub desc: &'static str,
}

pub mod n {
    pub const SCRAPER: u8 = 0;
    pub const CORDAGE: u8 = 1;
    pub const BASKET: u8 = 2;
    pub const NET: u8 = 3;
    pub const BLADE: u8 = 4;
    pub const SPEAR: u8 = 5;
    pub const CACHE: u8 = 6;
    pub const JAR: u8 = 7;
    pub const BURROW: u8 = 8;
    pub const HALL: u8 = 9;
    pub const WALL: u8 = 10;
    pub const VENT_COOK: u8 = 11;
    pub const GARDEN: u8 = 12;
    pub const SHELL_BED: u8 = 13;
    pub const REMEDY: u8 = 14;
    pub const QUARANTINE: u8 = 15;
    pub const BURIAL: u8 = 16;
    pub const ANCESTOR: u8 = 17;
    pub const LAMPWATCH: u8 = 18;
    pub const CALENDAR: u8 = 19;
    pub const TALLY: u8 = 20;
    pub const GLOWLAMP: u8 = 21;
    pub const ALARM: u8 = 22;
    pub const SHARING: u8 = 23;
    pub const RESTITUTION: u8 = 24;
    pub const LURE: u8 = 25;
    pub const COUNT: usize = 26;
}

use mat::*;
use need::*;
use NodeKind::*;

pub static NODES: [NodeDef; n::COUNT] = [
    NodeDef { id: 0, name: "shell scraper", kind: Tool, function: "cut", materials: &[SHELL, STONE], prereq: &[], needs: &[HUNGER], min_group: 1, difficulty: 1.0, desc: "pries shellfish loose; more food per dive" },
    NodeDef { id: 1, name: "kelp cordage", kind: Technique, function: "bind", materials: &[KELP], prereq: &[], needs: &[CARRY, HUNT_FAIL], min_group: 1, difficulty: 1.2, desc: "twisted kelp fibre" },
    NodeDef { id: 2, name: "carry basket", kind: Tool, function: "carry", materials: &[KELP], prereq: &[1], needs: &[CARRY, HUNGER], min_group: 1, difficulty: 1.4, desc: "doubles what can be carried home" },
    NodeDef { id: 3, name: "woven net", kind: Tool, function: "catch", materials: &[KELP], prereq: &[1], needs: &[HUNGER], min_group: 1, difficulty: 1.8, desc: "sweeps algae and small prey; more food" },
    NodeDef { id: 4, name: "blade", kind: Tool, function: "cut", materials: &[STONE, SHELL, VENTGLASS], prereq: &[0], needs: &[HUNT_FAIL, PREDATOR], min_group: 1, difficulty: 1.6, desc: "a sharpened edge" },
    NodeDef { id: 5, name: "hafted spear", kind: Tool, function: "pierce", materials: &[STONE, SHELL, VENTGLASS], prereq: &[4, 1], needs: &[HUNT_FAIL, PREDATOR], min_group: 1, difficulty: 2.2, desc: "blade bound to a stalk; hunting and defence" },
    NodeDef { id: 6, name: "food cache", kind: Structure, function: "contain", materials: &[STONE, CLAY, SHELL], prereq: &[], needs: &[SPOILED, STOLEN], min_group: 1, difficulty: 1.2, desc: "a covered store; slows spoilage and theft" },
    NodeDef { id: 7, name: "sealed jar", kind: Tool, function: "contain", materials: &[CLAY], prereq: &[6], needs: &[SPOILED], min_group: 1, difficulty: 2.4, desc: "clay pot sealed with sand; food keeps" },
    NodeDef { id: 8, name: "sand burrow", kind: Structure, function: "shelter", materials: &[SAND], prereq: &[], needs: &[COLD, PREDATOR], min_group: 1, difficulty: 1.0, desc: "a dug hollow; warmer rest" },
    NodeDef { id: 9, name: "woven hall", kind: Structure, function: "shelter", materials: &[KELP, STONE], prereq: &[1, 8], needs: &[COLD], min_group: 6, difficulty: 2.6, desc: "a shared woven shelter; much warmer" },
    NodeDef { id: 10, name: "stone ring-wall", kind: Structure, function: "defend", materials: &[STONE, SHELL], prereq: &[], needs: &[PREDATOR], min_group: 5, difficulty: 2.0, desc: "piled stones around the camp" },
    NodeDef { id: 11, name: "vent cooking", kind: Technique, function: "heat", materials: &[VENT], prereq: &[], needs: &[SICK, HUNGER, COLD], min_group: 1, difficulty: 1.6, desc: "food softened at the hot vents; more nourishing, less sickness" },
    NodeDef { id: 12, name: "algae garden", kind: Technique, function: "cultivate", materials: &[SAND, CLAY], prereq: &[], needs: &[HUNGER], min_group: 4, difficulty: 2.4, desc: "tended algae beds near camp" },
    NodeDef { id: 13, name: "shellfish bed", kind: Technique, function: "cultivate", materials: &[SHELL], prereq: &[12], needs: &[HUNGER], min_group: 4, difficulty: 2.8, desc: "reseeded shellfish beds; they no longer crash" },
    NodeDef { id: 14, name: "bitter-sponge remedy", kind: Technique, function: "heal", materials: &[SPONGE], prereq: &[], needs: &[SICK], min_group: 1, difficulty: 1.8, desc: "chewed sponge eases fever" },
    NodeDef { id: 15, name: "keeping-apart", kind: Practice, function: "protect", materials: &[], prereq: &[], needs: &[SICK], min_group: 3, difficulty: 2.0, desc: "the sick are kept apart from the well" },
    NodeDef { id: 16, name: "burial", kind: Practice, function: "ritual", materials: &[SAND], prereq: &[], needs: &[GRIEF], min_group: 2, difficulty: 1.0, desc: "the dead are covered in sand" },
    NodeDef { id: 17, name: "ancestor rites", kind: Belief, function: "worship", materials: &[], prereq: &[16], needs: &[GRIEF, CONFLICT], min_group: 5, difficulty: 2.0, desc: "the dead are remembered together; the camp holds closer" },
    NodeDef { id: 18, name: "lamp-watching", kind: Belief, function: "measure", materials: &[], prereq: &[], needs: &[COLD, HUNGER], min_group: 1, difficulty: 2.2, desc: "the Lamp's lengthening and shortening are counted" },
    NodeDef { id: 19, name: "season-count", kind: Institution, function: "record", materials: &[STONE, SHELL], prereq: &[18], needs: &[HUNGER, COLD], min_group: 4, difficulty: 2.6, desc: "a calendar; the dark season is foreseen" },
    NodeDef { id: 20, name: "tally marks", kind: Technique, function: "record", materials: &[STONE, SHELL, CLAY], prereq: &[], needs: &[CONFLICT, STOLEN], min_group: 3, difficulty: 2.4, desc: "scratched marks that keep count and memory" },
    NodeDef { id: 21, name: "glowlamp", kind: Tool, function: "light", materials: &[GLOW], prereq: &[], needs: &[DARK, PREDATOR], min_group: 1, difficulty: 2.0, desc: "a captive glowworm in a shell" },
    NodeDef { id: 22, name: "alarm call", kind: Practice, function: "signal", materials: &[], prereq: &[], needs: &[PREDATOR], min_group: 3, difficulty: 1.2, desc: "a shared cry that sends everyone to cover" },
    NodeDef { id: 23, name: "sharing custom", kind: Institution, function: "exchange", materials: &[], prereq: &[], needs: &[HUNGER, CONFLICT], min_group: 6, difficulty: 2.2, desc: "those with plenty feed the hungry" },
    NodeDef { id: 24, name: "restitution law", kind: Institution, function: "govern", materials: &[], prereq: &[], needs: &[STOLEN, CONFLICT], min_group: 8, difficulty: 2.8, desc: "a thief repays rather than being beaten" },
    NodeDef { id: 25, name: "drifter lure", kind: Technique, function: "hunt", materials: &[KELP, GLOW], prereq: &[3], needs: &[HUNT_FAIL], min_group: 1, difficulty: 2.2, desc: "drifters are lured into ambush" },
];

/// What a mind holds about one discovery.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Know {
    pub node: u8,
    /// Material variant actually used (0 = none / immaterial).
    pub material: u16,
    pub prof: f32,
    pub from: KnowSource,
    pub tick: u64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum KnowSource {
    Invented,
    Taught(u32),
    Imitated(u32),
    Record,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Via {
    Invented,
    Taught,
    Imitated,
    Record,
    Stranger,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub node: u8,
    pub material: u16,
    pub name: String,
    pub function: String,
    pub first_tick: u64,
    pub first_by: u32,
    pub first_culture: u16,
    pub inventions: u32,
    pub spread: [u32; 5],
    pub holders: u32,
    pub peak_holders: u32,
    pub lost: Vec<u64>,
    pub rediscovered: Vec<u64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Registry {
    pub entries: Vec<RegistryEntry>,
}

impl Registry {
    pub fn key_index(&self, node: u8, material: u16) -> Option<usize> {
        self.entries.iter().position(|e| e.node == node && e.material == material)
    }
    pub fn variant_name(node: u8, material: u16) -> String {
        let d = &NODES[node as usize];
        if material == 0 || d.materials.first() == Some(&material) || d.materials.is_empty() {
            d.name.to_string()
        } else {
            format!("{} {}", mat::name(material), d.name)
        }
    }
}
