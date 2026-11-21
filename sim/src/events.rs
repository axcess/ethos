//! Structured event log (DESIGN §13.1). Each event carries a short human summary for
//! reports; the Chronicle/news layers will use the structured fields.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvKind {
    Birth,
    Death,
    Discovery,
    Rediscovery,
    KnowledgeLost,
    CampFounded,
    CampAbandoned,
    Joined,
    GroupFormed,
    GroupSplit,
    GroupMerged,
    GroupDissolved,
    PairBond,
    Theft,
    Punishment,
    NormSpread,
    HuntWon,
    PredatorAttack,
    PredatorKilled,
    Outbreak,
    StructureBuilt,
    LanguageSplit,
    RoleEmerged,
    BeliefSpread,
    Scenario,
    ClimateExtreme,
    Famine,
    Extinction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub tick: u64,
    pub cycle: u64,
    pub kind: EvKind,
    pub who: Vec<u32>,
    pub place: u8,
    pub text: String,
    pub importance: f32,
}

impl EvKind {
    pub fn base_importance(self) -> f32 {
        use EvKind::*;
        match self {
            Birth => 0.05,
            Death => 0.2,
            Discovery => 0.7,
            Rediscovery => 0.6,
            KnowledgeLost => 0.7,
            CampFounded => 0.6,
            CampAbandoned => 0.6,
            Joined => 0.2,
            GroupFormed => 0.3,
            GroupSplit => 0.6,
            GroupMerged => 0.4,
            GroupDissolved => 0.4,
            PairBond => 0.1,
            Theft => 0.25,
            Punishment => 0.35,
            NormSpread => 0.6,
            HuntWon => 0.15,
            PredatorAttack => 0.4,
            PredatorKilled => 0.6,
            Outbreak => 0.6,
            StructureBuilt => 0.4,
            LanguageSplit => 0.8,
            RoleEmerged => 0.4,
            BeliefSpread => 0.5,
            Scenario => 0.5,
            ClimateExtreme => 0.5,
            Famine => 0.7,
            Extinction => 1.0,
        }
    }
}
