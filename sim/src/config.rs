//! All tunable parameters. Preset TOML files only need to list the values they override.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub name: String,
    pub time: TimeCfg,
    pub world: WorldCfg,
    pub climate: ClimateCfg,
    pub people: PeopleCfg,
    pub predators: PredCfg,
    pub disease: DiseaseCfg,
    pub mind: MindCfg,
    pub discovery: DiscoveryCfg,
    pub language: LanguageCfg,
    pub observer: ObserverCfg,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            name: "default".into(),
            time: TimeCfg::default(),
            world: WorldCfg::default(),
            climate: ClimateCfg::default(),
            people: PeopleCfg::default(),
            predators: PredCfg::default(),
            disease: DiseaseCfg::default(),
            mind: MindCfg::default(),
            discovery: DiscoveryCfg::default(),
            language: LanguageCfg::default(),
            observer: ObserverCfg::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TimeCfg {
    pub ticks_per_day: u64,
    pub days_per_season: u64,
    pub step_ticks: u64,
}
impl Default for TimeCfg {
    fn default() -> Self {
        TimeCfg { ticks_per_day: 1024, days_per_season: 12, step_ticks: 8 }
    }
}
impl TimeCfg {
    pub fn days_per_cycle(&self) -> u64 {
        self.days_per_season * 4
    }
    pub fn ticks_per_cycle(&self) -> u64 {
        self.ticks_per_day * self.days_per_cycle()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldCfg {
    pub ridge: bool,
    pub vents: u32,
    pub algae_regrow: f32,
    pub algae_cap_scale: f32,
    pub shell_regrow: f32,
    pub shell_cap_scale: f32,
    pub shell_allee: f32,
    pub spoil_rate: f32,
    pub herds: u32,
    pub herd_growth: f32,
    pub herd_max: f32,
    pub storm_bloom: f32,
    pub fatigue_per_harvest: f32,
    pub fatigue_effect: f32,
    pub fatigue_rest: f32,
    pub foul_per_person: f32,
    pub foul_clear: f32,
    pub crowd_disease: f32,
}
impl Default for WorldCfg {
    fn default() -> Self {
        WorldCfg {
            ridge: true,
            vents: 2,
            algae_regrow: 0.14,
            algae_cap_scale: 1.0,
            shell_regrow: 0.04,
            shell_cap_scale: 1.0,
            shell_allee: 0.10,
            spoil_rate: 0.05,
            herds: 6,
            herd_growth: 0.012,
            herd_max: 30.0,
            storm_bloom: 0.6,
            fatigue_per_harvest: 0.08,
            fatigue_effect: 0.8,
            fatigue_rest: 0.02,
            foul_per_person: 0.0006,
            foul_clear: 0.04,
            crowd_disease: 1.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ClimateCfg {
    pub dataset: String,
    pub block_min: u32,
    pub block_max: u32,
    pub amplitude: f32,
    /// If set, the year sequence starts at this real year (blocks still shuffled after it).
    pub start_year: Option<u32>,
}
impl Default for ClimateCfg {
    fn default() -> Self {
        ClimateCfg {
            dataset: "data/climate/chicago_daily.csv".into(),
            block_min: 3,
            block_max: 10,
            amplitude: 1.0,
            start_year: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PeopleCfg {
    pub initial: u32,
    pub bands: u32,
    pub adult_cycles: f32,
    pub max_age_min: f32,
    pub max_age_max: f32,
    pub gestation_days: u64,
    pub conceive_per_day: f32,
    pub hunger_per_day: f32,
    pub carry_cap: f32,
    pub capacity_min: f32,
    pub capacity_max: f32,
    pub temp_comfort: f32,
    pub cold_damage: f32,
    pub starve_damage: f32,
    pub max_population: u32,
}
impl Default for PeopleCfg {
    fn default() -> Self {
        PeopleCfg {
            initial: 80,
            bands: 4,
            adult_cycles: 8.0,
            max_age_min: 30.0,
            max_age_max: 50.0,
            gestation_days: 36,
            conceive_per_day: 0.05,
            hunger_per_day: 0.55,
            carry_cap: 4.0,
            capacity_min: 0.78,
            capacity_max: 0.95,
            temp_comfort: 12.0,
            cold_damage: 0.07,
            starve_damage: 0.12,
            max_population: 2000,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PredCfg {
    pub initial: u32,
    pub max: u32,
    pub sense: f32,
    pub speed: f32,
    pub kill_chance: f32,
    pub lifespan_cycles: f32,
    pub energy_per_day: f32,
    pub breed_energy: f32,
    pub immigration_per_cycle: f32,
}
impl Default for PredCfg {
    fn default() -> Self {
        PredCfg {
            initial: 4,
            max: 6,
            sense: 36.0,
            speed: 0.9,
            kill_chance: 0.12,
            lifespan_cycles: 10.0,
            energy_per_day: 0.08,
            breed_energy: 2.6,
            immigration_per_cycle: 0.5,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DiseaseCfg {
    pub spontaneous_per_cycle: f32,
    pub beta: f32,
    pub duration_days: u64,
    pub mortality_per_day: f32,
    pub immunity_cycles: f32,
}
impl Default for DiseaseCfg {
    fn default() -> Self {
        DiseaseCfg {
            spontaneous_per_cycle: 0.15,
            beta: 0.08,
            duration_days: 8,
            mortality_per_day: 0.015,
            immunity_cycles: 8.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct MindCfg {
    pub heartbeat: u64,
    pub noise: f32,
    pub episodic_cap: usize,
    pub belief_cap: usize,
    pub model_cap: usize,
    pub lexicon_cap: usize,
    pub emotion_decay: f32,
    pub contingency_decay: f32,
    pub rule_min_dp: f32,
    pub rule_min_n: f32,
    pub distortion: f32,
    pub thought_ring: usize,
}
impl Default for MindCfg {
    fn default() -> Self {
        MindCfg {
            heartbeat: 96,
            noise: 0.08,
            episodic_cap: 64,
            belief_cap: 96,
            model_cap: 60,
            lexicon_cap: 320,
            emotion_decay: 0.75,
            contingency_decay: 0.996,
            rule_min_dp: 0.2,
            rule_min_n: 4.0,
            distortion: 0.04,
            thought_ring: 64,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DiscoveryCfg {
    pub base_rate: f32,
    pub imitation: f32,
    pub teach_fidelity: f32,
    pub false_belief_on_fail: f32,
}
impl Default for DiscoveryCfg {
    fn default() -> Self {
        DiscoveryCfg { base_rate: 0.0015, imitation: 0.04, teach_fidelity: 0.5, false_belief_on_fail: 0.05 }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LanguageCfg {
    pub compound_chance: f32,
    pub innovator_chance: f32,
    pub innovation_use: f32,
    pub adopt_score: f32,
    pub reinforce: f32,
    pub synonym_rate: f32,
}
impl Default for LanguageCfg {
    fn default() -> Self {
        LanguageCfg {
            compound_chance: 0.3,
            innovator_chance: 0.12,
            innovation_use: 0.15,
            adopt_score: 0.4,
            reinforce: 0.12,
            synonym_rate: 0.0006,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ObserverCfg {
    pub group_every_days: u64,
    pub min_group: usize,
}
impl Default for ObserverCfg {
    fn default() -> Self {
        ObserverCfg { group_every_days: 12, min_group: 4 }
    }
}

impl Config {
    pub fn load(path: &str) -> Result<Config, String> {
        let s = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        toml::from_str(&s).map_err(|e| format!("{path}: {e}"))
    }
}
