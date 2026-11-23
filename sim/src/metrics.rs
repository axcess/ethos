//! Per-cycle metrics (ROADMAP Phase 1) and run summaries.
use crate::social::role;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CycleCounters {
    pub births: u32,
    pub deaths: u32,
    pub deaths_starvation: u32,
    pub deaths_predator: u32,
    pub deaths_disease: u32,
    pub deaths_old: u32,
    pub deaths_cold: u32,
    pub deaths_violence: u32,
    pub attacks: u32,
    pub infections: u32,
    pub proposals: u32,
    pub proposal_asks: u32,
    pub proposal_accepts: u32,
    pub asks: u32,
    pub asks_granted: u32,
    pub shares: u32,
    pub conversations: u32,
    pub hunts_won: u32,
    pub hunts_lost: u32,
    pub unfair_shares: u32,
    pub deserted: u32,
    pub thefts: u32,
    pub punishments: u32,
    pub heals: u32,
    pub taught: u32,
    pub imitated: u32,
    pub food_gathered: f32,
    pub food_hunted: f32,
    pub food_eaten: f32,
    pub food_spoiled: f32,
    pub algae_grown: f32,
    pub shell_grown: f32,
    pub inventions: u32,
    pub knowledge_lost: u32,
    pub structures: u32,
    pub camps_founded: u32,
    pub groups_formed: u32,
    pub group_splits: u32,
    pub group_merges: u32,
    pub groups_dissolved: u32,
    pub bonds: u32,
    pub predators_killed: u32,
    pub distortions: u32,
    pub false_beliefs_formed: u32,
}

impl CycleCounters {
    pub fn record_death(&mut self, cause: &str) {
        self.deaths += 1;
        if cause.contains("cold") {
            self.deaths_cold += 1;
        } else if cause.contains("starv") {
            self.deaths_starvation += 1;
        } else if cause.contains("gulper") {
            self.deaths_predator += 1;
        } else if cause.contains("fever") {
            self.deaths_disease += 1;
        } else if cause.contains("old") {
            self.deaths_old += 1;
        } else if cause.contains("cold") {
            self.deaths_cold += 1;
        } else {
            self.deaths_violence += 1;
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CycleStats {
    pub cycle: u64,
    pub pop: u32,
    pub adults: u32,
    pub camps: u32,
    pub groups: u32,
    pub predators: u32,
    pub herd_animals: u32,
    pub food_total: f32,
    pub real_year: u32,
    pub surface_t: f32,
    pub known_nodes: u32,
    pub knowledge_lost_total: u32,
    pub languages: u32,
    pub min_intelligibility: f32,
    pub coherence: f32,
    pub lexicon_mean: f32,
    pub max_grammar_tier: u8,
    pub comm_success: f32,
    pub roles_distinct: u32,
    pub role_counts: [u32; role::N],
    pub belief_diversity: f32,
    pub distinct_beliefs: u32,
    pub false_belief_frac: f32,
    pub culture_specific_false: u32,
    pub behavior_entropy: f32,
    pub behavior_divergence: f32,
    pub change_index: f32,
    pub believed_groups: u32,
    pub mean_identity: f32,
    pub identity_agreement: f32,
    pub mean_fatigue: f32,
    pub max_foul: f32,
    pub decisions: u64,
    pub c: CycleCounters,
}

/// Summary of one tank run, used by batch reports.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RunSummary {
    pub seed: u64,
    pub scenario: String,
    pub cycles: u64,
    pub extinct: bool,
    pub extinct_cycle: Option<u64>,
    pub final_pop: u32,
    pub peak_pop: u32,
    pub min_pop: u32,
    pub births: u32,
    pub deaths: u32,
    pub deaths_by: [u32; 6],
    pub cooperation_rate: f32,
    pub share_per_capita: f32,
    pub groups_formed: u32,
    pub group_splits: u32,
    pub group_merges: u32,
    pub camps_founded: u32,
    pub max_roles: u32,
    pub roles_ever: Vec<String>,
    pub max_belief_diversity: f32,
    pub mean_belief_diversity: f32,
    pub culture_specific_false_max: u32,
    pub any_wrong_belief_frac_max: f32,
    pub inventions: u32,
    pub distinct_discoveries: u32,
    pub discoveries_per_generation: f32,
    pub knowledge_lost: u32,
    pub rediscoveries: u32,
    pub max_languages: u32,
    pub min_intelligibility: f32,
    pub final_coherence: f32,
    pub max_grammar_tier: u8,
    pub mean_behavior_entropy: f32,
    pub mean_behavior_divergence: f32,
    pub mean_change_index: f32,
    pub min_change_index_late: f32,
    pub thefts: u32,
    pub punishments: u32,
    pub norm_camps: u32,
    pub beliefs_spread: u32,
    pub decisions: u64,
    pub wall_secs: f64,
    pub decisions_per_sec: f64,
    pub state_hash: String,
    pub real_years: Vec<u32>,
    pub activity_share: Vec<(String, f32)>,
    /// none | crash | crash_recovered | escaped | sagged
    pub max_believed_groups: u32,
    pub mean_identity_agreement: f32,
    pub boom_outcome: String,
    pub boom_cycle: Option<u64>,
    pub crash_cycle: Option<u64>,
    pub crash_depth: f32,
}

/// Classify what happened after a population boom (≥ 1.5× the early level).
pub fn classify_boom(pops: &[u32]) -> (String, Option<u64>, Option<u64>, f32) {
    if pops.len() < 12 {
        return ("none".into(), None, None, 0.0);
    }
    let mut early: Vec<u32> = pops[..8].to_vec();
    early.sort();
    // the early level: founding population or the early median, whichever is higher
    let base = early[early.len() / 2].max(pops[0]).max(1) as f32;
    let Some(b) = pops.iter().position(|&p| p as f32 >= 1.5 * base) else {
        return ("none".into(), None, None, 0.0);
    };
    let mut peak = 0u32;
    let mut crash: Option<(usize, f32, u32)> = None;
    for (k, &p) in pops.iter().enumerate().skip(b) {
        if crash.is_none() {
            peak = peak.max(p);
            if (p as f32) <= 0.7 * peak as f32 {
                crash = Some((k, 1.0 - p as f32 / peak as f32, peak));
            }
        } else if let Some((_, d, pk)) = crash {
            let depth = 1.0 - p as f32 / pk as f32;
            if depth > d {
                crash = Some((crash.unwrap().0, depth, pk));
            }
        }
    }
    let last = *pops.last().unwrap();
    match crash {
        Some((k, d, pk)) => {
            let recovered = last >= pk;
            ((if recovered { "crash_recovered" } else { "crash" }).into(), Some(b as u64), Some(k as u64), d)
        }
        None => ((if last as f32 >= 0.9 * peak as f32 { "escaped" } else { "sagged" }).into(), Some(b as u64), None, 0.0),
    }
}

pub fn summarize(sim: &crate::sim::Sim, scenario: &str, wall: f64) -> RunSummary {
    let s = &sim.stats;
    let mut r = RunSummary { seed: sim.seed, scenario: scenario.into(), cycles: s.len() as u64, ..Default::default() };
    r.extinct = sim.extinct;
    r.extinct_cycle = if sim.extinct { Some(sim.cycle()) } else { None };
    r.final_pop = sim.alive.len() as u32;
    r.peak_pop = s.iter().map(|c| c.pop).max().unwrap_or(0);
    r.min_pop = s.iter().map(|c| c.pop).min().unwrap_or(0);
    let mut asks = 0;
    let mut accepts = 0;
    let mut shares = 0;
    let mut pop_sum = 0u64;
    for c in s {
        r.births += c.c.births;
        r.deaths += c.c.deaths;
        r.deaths_by[0] += c.c.deaths_starvation;
        r.deaths_by[1] += c.c.deaths_predator;
        r.deaths_by[2] += c.c.deaths_disease;
        r.deaths_by[3] += c.c.deaths_old;
        r.deaths_by[4] += c.c.deaths_cold;
        r.deaths_by[5] += c.c.deaths_violence;
        asks += c.c.proposal_asks + c.c.asks;
        accepts += c.c.proposal_accepts + c.c.asks_granted;
        shares += c.c.shares + c.c.asks_granted;
        pop_sum += c.pop as u64;
        r.groups_formed += c.c.groups_formed;
        r.group_splits += c.c.group_splits;
        r.group_merges += c.c.group_merges;
        r.camps_founded += c.c.camps_founded;
        r.inventions += c.c.inventions;
        r.thefts += c.c.thefts;
        r.punishments += c.c.punishments;
        r.max_roles = r.max_roles.max(c.roles_distinct);
        r.max_belief_diversity = r.max_belief_diversity.max(c.belief_diversity);
        r.culture_specific_false_max = r.culture_specific_false_max.max(c.culture_specific_false);
        r.any_wrong_belief_frac_max = r.any_wrong_belief_frac_max.max(c.false_belief_frac);
        r.max_languages = r.max_languages.max(c.languages);
        r.max_grammar_tier = r.max_grammar_tier.max(c.max_grammar_tier);
        if !r.real_years.contains(&c.real_year) {
            r.real_years.push(c.real_year);
        }
    }
    r.min_intelligibility = s.iter().map(|c| c.min_intelligibility).fold(1.0, f32::min);
    let n = s.len().max(1) as f32;
    r.mean_belief_diversity = s.iter().map(|c| c.belief_diversity).sum::<f32>() / n;
    r.mean_behavior_entropy = s.iter().map(|c| c.behavior_entropy).sum::<f32>() / n;
    r.mean_behavior_divergence = s.iter().map(|c| c.behavior_divergence).sum::<f32>() / n;
    r.mean_change_index = s.iter().map(|c| c.change_index).sum::<f32>() / n;
    r.min_change_index_late = s.iter().skip(s.len() / 2).map(|c| c.change_index).fold(f32::MAX, f32::min);
    if r.min_change_index_late == f32::MAX {
        r.min_change_index_late = 0.0;
    }
    r.final_coherence = s.last().map_or(0.0, |c| c.coherence);
    r.cooperation_rate = if asks > 0 { accepts as f32 / asks as f32 } else { 0.0 };
    r.share_per_capita = shares as f32 / (pop_sum.max(1) as f32 / n) / n;
    r.distinct_discoveries = sim.nodes.iter().filter(|x| x.first_tick.is_some()).count() as u32;
    r.knowledge_lost = sim.nodes.iter().map(|x| x.lost.len() as u32).sum();
    r.rediscoveries = sim.nodes.iter().map(|x| x.rediscovered.len() as u32).sum();
    let gens = (s.len() as f32 / 20.0).max(0.25);
    r.discoveries_per_generation = r.inventions as f32 / gens;
    r.roles_ever = (0..role::N).filter(|k| sim.obs.roles_ever & (1 << k) != 0).map(|k| role::NAMES[k].to_string()).collect();
    r.norm_camps = sim.obs.norm_camps.len() as u32;
    r.beliefs_spread = sim.obs.spread_beliefs.len() as u32;
    r.decisions = sim.decisions;
    r.wall_secs = wall;
    r.decisions_per_sec = sim.decisions as f64 / wall.max(1e-6);
    r.state_hash = format!("{:016x}", sim.state_hash());
    let mut tot = [0u64; crate::person::N_ACT];
    for p in &sim.people {
        for k in 0..crate::person::N_ACT {
            tot[k] += p.activity_total[k] as u64;
        }
    }
    let sum: u64 = tot.iter().sum::<u64>().max(1);
    let mut v: Vec<(String, f32)> = (0..crate::person::N_ACT).map(|k| (crate::person::ACT_NAMES[k].to_string(), (tot[k] as f32 / sum as f32 * 1000.0).round() / 10.0)).collect();
    v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    r.activity_share = v;
    r.max_believed_groups = s.iter().map(|c| c.believed_groups).max().unwrap_or(0);
    let late: Vec<f32> = s.iter().skip(s.len() / 2).map(|c| c.identity_agreement).collect();
    r.mean_identity_agreement = late.iter().sum::<f32>() / late.len().max(1) as f32;
    let pops: Vec<u32> = s.iter().map(|c| c.pop).collect();
    let (o, bc, cc, d) = classify_boom(&pops);
    r.boom_outcome = o;
    r.boom_cycle = bc;
    r.crash_cycle = cc;
    r.crash_depth = d;
    r
}
