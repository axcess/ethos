//! The observer (DESIGN §8.3–8.4, §9.3): read-only analysis that detects groups, roles,
//! languages, norms and belief patterns. It never changes what minds believe or do.
use crate::concepts::{Concept, Kind, ALL_KINDS};
use crate::events::EvKind;
use crate::language::{edit_distance, Form};
use crate::metrics::CycleStats;
use crate::mind::Rel;
use crate::person::N_ACT;
use crate::rng::{fnv64, Rng};
use crate::sim::Sim;
use crate::world::N_PLACES;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub mod role {
    // roles also observed by minds (indices into Model::seen)
    pub const GATHERER: usize = 0;
    pub const HUNTER: usize = 1;
    pub const GUARD: usize = 2;
    pub const TEACHER: usize = 3;
    pub const HEALER: usize = 4;
    pub const BUILDER: usize = 5;
    pub const INVENTOR: usize = 6;
    pub const THIEF: usize = 7;
    // observer-only
    pub const LEADER: usize = 8;
    pub const OUTCAST: usize = 9;
    pub const PROVIDER: usize = 10;
    pub const PROPHET: usize = 11;
    pub const STORYTELLER: usize = 12;
    pub const LAWGIVER: usize = 13;
    pub const N: usize = 14;
    pub const NAMES: [&str; N] = [
        "gatherer", "hunter", "guard", "teacher", "healer", "builder", "inventor", "thief", "leader", "outcast",
        "provider", "prophet", "storyteller", "lawgiver",
    ];
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Group {
    pub id: u32,
    pub members: Vec<u32>,
    pub formed: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Observer {
    pub groups: Vec<Group>,
    pub next_group: u32,
    pub membership: BTreeMap<u32, u32>,
    pub prev_cycle_membership: BTreeMap<u32, u32>,
    pub prev_roles: BTreeMap<u32, u16>,
    pub roles_ever: u16,
    pub norm_camps: Vec<u16>,
    pub max_languages: u32,
    pub spread_beliefs: Vec<u64>,
    pub prev_pop: u32,
    pub languages: Vec<LanguageInfo>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub communities: Vec<u32>,
    pub speakers: u32,
    pub shared_lexicon: u32,
    pub coherence: f32,
    pub grammar_tier: u8,
    pub sample: Vec<(String, String)>,
}

/// Weighted label propagation over the mutual-affinity graph, then matching to previous groups.
pub fn detect_groups(sim: &mut Sim) {
    let min_group = sim.cfg.observer.min_group;
    let adults: Vec<u32> = sim.alive.iter().copied().filter(|&i| sim.adult(i as usize)).collect();
    let idx: BTreeMap<u32, usize> = adults.iter().enumerate().map(|(k, &id)| (id, k)).collect();
    let mut adj: Vec<Vec<(usize, f32)>> = vec![Vec::new(); adults.len()];
    for (a, &ida) in adults.iter().enumerate() {
        for m in &sim.people[ida as usize].mind.models {
            let Some(&b) = idx.get(&m.who) else { continue };
            if b <= a {
                continue;
            }
            let back = sim.people[m.who as usize].mind.model(ida).map_or(0.0, |x| x.affinity);
            let w = (m.affinity + back) * 0.5;
            if w > 0.25 {
                adj[a].push((b, w));
                adj[b].push((a, w));
            }
        }
    }
    let mut label: Vec<usize> = (0..adults.len()).collect();
    for _ in 0..10 {
        let mut changed = false;
        for a in 0..adults.len() {
            if adj[a].is_empty() {
                continue;
            }
            let mut score: BTreeMap<usize, f32> = BTreeMap::new();
            for &(b, w) in &adj[a] {
                *score.entry(label[b]).or_default() += w;
            }
            let best = score.iter().fold((label[a], f32::MIN), |acc, (l, s)| if *s > acc.1 + 1e-6 { (*l, *s) } else { acc }).0;
            if best != label[a] {
                label[a] = best;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let mut clusters: BTreeMap<usize, Vec<u32>> = BTreeMap::new();
    for (a, &l) in label.iter().enumerate() {
        clusters.entry(l).or_default().push(adults[a]);
    }
    let clusters: Vec<Vec<u32>> = clusters.into_values().filter(|c| c.len() >= min_group).collect();
    // match to previous groups
    let prev_sizes: BTreeMap<u32, usize> = sim.obs.groups.iter().map(|g| (g.id, g.members.len())).collect();
    let mut overlaps: Vec<BTreeMap<u32, usize>> = Vec::new();
    for c in &clusters {
        let mut o: BTreeMap<u32, usize> = BTreeMap::new();
        for m in c {
            if let Some(&g) = sim.obs.membership.get(m) {
                *o.entry(g).or_default() += 1;
            }
        }
        overlaps.push(o);
    }
    let mut inherit: BTreeMap<u32, (usize, usize)> = BTreeMap::new();
    for (ci, o) in overlaps.iter().enumerate() {
        for (&g, &n) in o {
            let e = inherit.entry(g).or_insert((ci, n));
            if n > e.1 {
                *e = (ci, n);
            }
        }
    }
    let mut new_groups: Vec<Group> = Vec::new();
    let mut events: Vec<(EvKind, Vec<u32>, String)> = Vec::new();
    let mut used_prev: BTreeSet<u32> = BTreeSet::new();
    let tick = sim.tick;
    for (ci, c) in clusters.iter().enumerate() {
        let o = &overlaps[ci];
        let inherited: Option<u32> = inherit.iter().find(|(_, v)| v.0 == ci && v.1 >= 2).map(|(g, _)| *g);
        let id = match inherited {
            Some(g) => {
                used_prev.insert(g);
                // merges: a large share from a second previous group that has no other heir
                for (&g2, &n2) in o {
                    if g2 != g && n2 >= 3 && inherit.get(&g2).map_or(false, |v| v.0 == ci) {
                        used_prev.insert(g2);
                        events.push((EvKind::GroupMerged, c.clone(), format!("Group {g2} merges into group {g} ({} members)", c.len())));
                    }
                }
                g
            }
            None => {
                let id = sim.obs.next_group;
                sim.obs.next_group += 1;
                let split_from = o.iter().filter(|(g, n)| **n >= 3 && (**n as f32) >= 0.2 * *prev_sizes.get(g).unwrap_or(&1) as f32).max_by_key(|(_, n)| **n).map(|(g, _)| *g);
                match split_from {
                    Some(g) => events.push((EvKind::GroupSplit, c.clone(), format!("Group {id} ({} members) splits off from group {g}", c.len()))),
                    None => events.push((EvKind::GroupFormed, c.clone(), format!("A new group ({id}) forms with {} members", c.len()))),
                }
                id
            }
        };
        let formed = sim.obs.groups.iter().find(|g| g.id == id).map_or(tick, |g| g.formed);
        new_groups.push(Group { id, members: c.clone(), formed });
    }
    for g in &sim.obs.groups {
        if !used_prev.contains(&g.id) && !new_groups.iter().any(|ng| ng.id == g.id) {
            events.push((EvKind::GroupDissolved, vec![], format!("Group {} dissolves", g.id)));
        }
    }
    sim.obs.membership.clear();
    for g in &new_groups {
        for &m in &g.members {
            sim.obs.membership.insert(m, g.id);
        }
    }
    sim.obs.groups = new_groups;
    for (k, who, text) in events {
        match k {
            EvKind::GroupFormed => sim.counters.groups_formed += 1,
            EvKind::GroupSplit => sim.counters.group_splits += 1,
            EvKind::GroupMerged => sim.counters.group_merges += 1,
            EvKind::GroupDissolved => sim.counters.groups_dissolved += 1,
            _ => {}
        }
        let place = who.first().map_or(0, |&w| sim.place_of_person(w as usize));
        let who_short: Vec<u32> = who.into_iter().take(8).collect();
        sim.emit(k, who_short, place, text);
    }
}

fn entropy(v: &[f32]) -> f32 {
    let s: f32 = v.iter().sum();
    if s <= 0.0 {
        return 0.0;
    }
    let mut h = 0.0;
    for x in v {
        if *x > 0.0 {
            let p = x / s;
            h -= p * libm::logf(p);
        }
    }
    h / libm::logf(v.len() as f32)
}

/// Roles as the observer sees them (bitmask over role::*).
fn detect_roles(sim: &Sim, id: u32, respecters: u32, prophet: bool, lawgiver: bool) -> u16 {
    let p = &sim.people[id as usize];
    let tot: f32 = p.activity.iter().sum::<f32>().max(1.0);
    let share = |k: usize| p.activity[k] / tot;
    let mut r = 0u16;
    if share(1) > 0.35 {
        r |= 1 << role::GATHERER;
    }
    if share(10) > 0.10 {
        r |= 1 << role::HUNTER;
    }
    if p.activity_total[11] >= 3 || p.predators_killed > 0 {
        r |= 1 << role::GUARD;
    }
    if share(12) > 0.04 {
        r |= 1 << role::TEACHER;
    }
    if share(22) > 0.03 {
        r |= 1 << role::HEALER;
    }
    if share(14) > 0.05 {
        r |= 1 << role::BUILDER;
    }
    if p.inventions > 0 {
        r |= 1 << role::INVENTOR;
    }
    if p.activity_total[15] >= 3 {
        r |= 1 << role::THIEF;
    }
    if respecters >= 4 {
        r |= 1 << role::LEADER;
    }
    if share(8) > 0.05 {
        r |= 1 << role::PROVIDER;
    }
    if share(7) > 0.3 {
        r |= 1 << role::STORYTELLER;
    }
    if prophet {
        r |= 1 << role::PROPHET;
    }
    if lawgiver {
        r |= 1 << role::LAWGIVER;
    }
    // outcast: camp-mates distrust or resent them
    if let Some(h) = p.home {
        let mut n = 0;
        let mut bad = 0;
        for &o in &sim.alive {
            let q = &sim.people[o as usize];
            if o == id || q.home != Some(h) {
                continue;
            }
            if let Some(m) = q.mind.model(id) {
                n += 1;
                if m.trust < 0.15 || m.grudge > 0.4 {
                    bad += 1;
                }
            }
        }
        if n >= 4 && bad * 2 >= n {
            r |= 1 << role::OUTCAST;
        }
    } else if sim.adult(id as usize) {
        r |= 1 << role::OUTCAST;
    }
    r
}

fn modal_form(forms: &BTreeMap<Form, u32>) -> Option<(&Form, u32)> {
    forms.iter().fold(None, |acc: Option<(&Form, u32)>, (f, n)| match acc {
        Some((_, m)) if m >= *n => acc,
        _ => Some((f, *n)),
    })
}

pub fn cycle_observe(sim: &mut Sim, cycle: u64) {
    let tick = sim.tick;
    let adults: Vec<u32> = sim.alive.iter().copied().filter(|&i| sim.adult(i as usize)).collect();
    let pop = sim.alive.len() as u32;

    // ---------- beliefs: origins (prophets), diversity, false beliefs
    let mut origin_counts: BTreeMap<(u32, u64, u8), u32> = BTreeMap::new();
    let mut signatures: Vec<BTreeSet<u64>> = Vec::new();
    let mut false_keys: Vec<BTreeSet<u64>> = Vec::new();
    let mut all_beliefs: BTreeSet<u64> = BTreeSet::new();
    let mut wrong_people = 0u32;
    for &id in &adults {
        let p = &sim.people[id as usize];
        let mut sig = BTreeSet::new();
        let mut fk = BTreeSet::new();
        for b in &p.mind.beliefs {
            if b.conf < 0.35 {
                continue;
            }
            let key = b.subject.key() * 32 + b.rel as u64;
            sig.insert(key);
            if !matches!(b.subject, Concept::Person(_)) {
                *origin_counts.entry((b.origin, b.subject.key(), b.rel as u8)).or_default() += 1;
            }
            let wrong = b.rel == Rel::Cursed
                || b.distorted
                || (b.rel == Rel::Thief && matches!(b.subject, Concept::Person(z) if sim.people[z as usize].activity_total[15] == 0));
            if wrong {
                fk.insert(key);
            }
        }
        for (k, pm) in p.mind.places.iter().enumerate() {
            if pm.danger > 0.5 {
                let key = 7_000_000 + k as u64;
                sig.insert(key);
                let real = sim.world.attacks_by_place[k] + sim.world.attacks_by_place_prev[k];
                if pm.danger_distorted || (pm.danger_src != 0 && real == 0 && pm.danger > 0.6) {
                    fk.insert(key);
                }
            }
        }
        for (c, o, _) in p.mind.cont.rules(sim.cfg.mind.rule_min_dp, sim.cfg.mind.rule_min_n).into_iter().take(4) {
            sig.insert(8_000_000 + (c * 16 + o) as u64);
        }
        if !fk.is_empty() {
            wrong_people += 1;
        }
        all_beliefs.extend(sig.iter().copied());
        signatures.push(sig);
        false_keys.push(fk);
    }
    let mut obs_rng = Rng::stream(sim.seed, crate::rng::tag::OBSERVER, cycle);
    let mut jd = 0.0;
    let mut pairs = 0;
    if signatures.len() >= 2 {
        for _ in 0..300 {
            let a = obs_rng.below(signatures.len() as u32) as usize;
            let b = obs_rng.below(signatures.len() as u32) as usize;
            if a == b {
                continue;
            }
            let inter = signatures[a].intersection(&signatures[b]).count() as f32;
            let uni = signatures[a].union(&signatures[b]).count() as f32;
            if uni > 0.0 {
                jd += 1.0 - inter / uni;
                pairs += 1;
            }
        }
    }
    let belief_diversity = if pairs > 0 { jd / pairs as f32 } else { 0.0 };
    // culture-specific false beliefs: common in one group, rare elsewhere
    let mut culture_false = 0u32;
    {
        let gid: Vec<Option<u32>> = adults.iter().map(|a| sim.obs.membership.get(a).copied()).collect();
        let mut keys: BTreeSet<u64> = BTreeSet::new();
        for f in &false_keys {
            keys.extend(f.iter().copied());
        }
        for k in keys {
            let mut by_g: BTreeMap<u32, (u32, u32)> = BTreeMap::new();
            for (ai, g) in gid.iter().enumerate() {
                if let Some(g) = g {
                    let e = by_g.entry(*g).or_default();
                    e.1 += 1;
                    if false_keys[ai].contains(&k) {
                        e.0 += 1;
                    }
                }
            }
            let fr: Vec<f32> = by_g.values().filter(|v| v.1 >= 4).map(|v| v.0 as f32 / v.1 as f32).collect();
            if fr.len() >= 2 && fr.iter().any(|x| *x >= 0.3) && fr.iter().filter(|x| **x <= 0.05).count() >= 1 {
                culture_false += 1;
            }
        }
    }
    // prophets & spreading beliefs
    let mut prophets: BTreeSet<u32> = BTreeSet::new();
    let mut lawgivers: BTreeSet<u32> = BTreeSet::new();
    let mut spread_events = Vec::new();
    for (&(origin, subj, rel), &cnt) in &origin_counts {
        if cnt >= 6 {
            if rel == Rel::Forbidden as u8 {
                lawgivers.insert(origin);
            } else if matches!(rel, x if x == Rel::Cursed as u8 || x == Rel::Watch as u8 || x == Rel::Governs as u8 || x == Rel::Sacred as u8) {
                prophets.insert(origin);
            }
            let key = fnv64(&[origin.to_le_bytes().as_slice(), subj.to_le_bytes().as_slice(), &[rel]].concat());
            if !sim.obs.spread_beliefs.contains(&key) {
                sim.obs.spread_beliefs.push(key);
                spread_events.push((origin, subj, rel, cnt));
            }
        }
    }
    for (origin, subj, rel, cnt) in spread_events {
        let rel_name = match rel {
            x if x == Rel::Cursed as u8 => "is cursed",
            x if x == Rel::Forbidden as u8 => "is forbidden",
            x if x == Rel::Spreads as u8 => "spreads from the sick",
            x if x == Rel::Watch as u8 => "watch over the living",
            x if x == Rel::Governs as u8 => "governs the seasons",
            _ => "matters",
        };
        let subj_name = ALL_KINDS.iter().find(|k| **k as u64 == subj).map(|k| k.gloss().to_lowercase()).unwrap_or_else(|| format!("concept {subj}"));
        let oname = sim.name(origin);
        let place = sim.place_of_person(origin as usize);
        sim.emit(EvKind::BeliefSpread, vec![origin], place, format!("The belief that {subj_name} {rel_name}, first held by {oname}, is now shared by {cnt} people"));
    }

    // ---------- roles
    let mut respecters: BTreeMap<u32, u32> = BTreeMap::new();
    for &o in &sim.alive {
        for m in &sim.people[o as usize].mind.models {
            if m.respect > 0.6 && m.followed >= 2 {
                *respecters.entry(m.who).or_default() += 1;
            }
        }
    }
    let mut role_counts = [0u32; role::N];
    let mut role_churn = 0u32;
    let mut new_roles: Vec<(u32, u16)> = Vec::new();
    for &id in &adults {
        let r = detect_roles(sim, id, *respecters.get(&id).unwrap_or(&0), prophets.contains(&id), lawgivers.contains(&id));
        for k in 0..role::N {
            if r & (1 << k) != 0 {
                role_counts[k] += 1;
            }
        }
        if sim.obs.prev_roles.get(&id).copied().unwrap_or(0) != r {
            role_churn += 1;
        }
        new_roles.push((id, r));
    }
    let mut first_roles = Vec::new();
    for (id, r) in &new_roles {
        let p = &mut sim.people[*id as usize];
        let gained = *r & !p.roles;
        p.roles = *r;
        for k in 0..role::N {
            if gained & (1 << k) != 0 && sim.obs.roles_ever & (1 << k) == 0 {
                sim.obs.roles_ever |= 1 << k;
                if k != role::GATHERER {
                    first_roles.push((*id, k));
                }
            }
            if gained & (1 << k) != 0 && matches!(k, role::LEADER | role::HEALER | role::PROPHET | role::OUTCAST | role::TEACHER | role::LAWGIVER) {
                let text = format!("became known as a {}", role::NAMES[k]);
                p.life_note(tick, text);
            }
        }
        sim.obs.prev_roles.insert(*id, *r);
    }
    for (id, k) in first_roles {
        let name = sim.name(id);
        let place = sim.place_of_person(id as usize);
        sim.emit(EvKind::RoleEmerged, vec![id], place, format!("For the first time, someone acts as a {}: {name}", role::NAMES[k]));
    }
    let roles_distinct = role_counts.iter().filter(|c| **c > 0).count() as u32;

    // ---------- norms
    let camps: Vec<u16> = sim.world.camps.iter().filter(|c| c.abandoned.is_none()).map(|c| c.id).collect();
    for c in camps {
        if sim.obs.norm_camps.contains(&c) {
            continue;
        }
        let members: Vec<u32> = adults.iter().copied().filter(|&a| sim.people[a as usize].home == Some(c)).collect();
        if members.len() < 5 {
            continue;
        }
        let holders = members.iter().filter(|&&a| sim.people[a as usize].mind.belief(Concept::Kind(Kind::Steal), Rel::Forbidden) > 0.4).count();
        if holders * 10 >= members.len() * 4 {
            sim.obs.norm_camps.push(c);
            let cname = sim.camp_name(c);
            let place = sim.world.camps[c as usize].place;
            sim.emit(EvKind::NormSpread, vec![], place, format!("At {cname}, most now hold that stealing food is wrong ({holders}/{})", members.len()));
        }
    }

    // ---------- language
    let mut comms: Vec<(u32, Vec<u32>)> = sim.obs.groups.iter().filter(|g| g.members.len() >= 5).map(|g| (g.id, g.members.clone())).collect();
    if comms.is_empty() {
        for c in &sim.world.camps {
            let m: Vec<u32> = adults.iter().copied().filter(|&a| sim.people[a as usize].home == Some(c.id)).collect();
            if m.len() >= 5 {
                comms.push((10_000 + c.id as u32, m));
            }
        }
    }
    let base: Vec<Concept> = ALL_KINDS.iter().map(|k| Concept::Kind(*k)).chain((0..N_PLACES as u8).map(Concept::Place)).collect();
    let mut modal: Vec<BTreeMap<Concept, Form>> = Vec::new();
    let mut coh_sum = 0.0;
    let mut lex_sum = 0.0;
    let mut info: Vec<LanguageInfo> = Vec::new();
    for (_, members) in &comms {
        let mut mm = BTreeMap::new();
        let mut coh = 0.0;
        let mut nconc = 0;
        let mut shared = 0u32;
        let mut abstract_n = 0u32;
        for c in &base {
            let mut forms: BTreeMap<Form, u32> = BTreeMap::new();
            let mut knowers = 0;
            for &m in members {
                if let Some(f) = sim.people[m as usize].mind.lex.best(c) {
                    *forms.entry(f.clone()).or_default() += 1;
                    knowers += 1;
                }
            }
            if knowers * 2 < members.len() {
                continue;
            }
            if let Some((f, n)) = modal_form(&forms) {
                let share = n as f32 / knowers as f32;
                coh += share;
                nconc += 1;
                if share >= 0.5 {
                    shared += 1;
                    if c.is_abstract() {
                        abstract_n += 1;
                    }
                }
                mm.insert(*c, f.clone());
            }
        }
        let coherence = if nconc > 0 { coh / nconc as f32 } else { 0.0 };
        coh_sum += coherence;
        lex_sum += members.iter().map(|&m| sim.people[m as usize].mind.lex.len() as f32).sum::<f32>() / members.len() as f32;
        let tier = if shared < 20 {
            0
        } else if shared < 45 {
            1
        } else if coherence < 0.6 || shared < 60 {
            2
        } else if abstract_n >= 6 {
            3
        } else {
            2
        };
        let mut sample = Vec::new();
        for k in [Kind::Food, Kind::Gulper, Kind::Lamp, Kind::Danger, Kind::Death, Kind::Ancestors] {
            if let Some(f) = mm.get(&Concept::Kind(k)) {
                sample.push((k.gloss().to_lowercase(), crate::language::romanize(f)));
            }
        }
        info.push(LanguageInfo { communities: vec![], speakers: members.len() as u32, shared_lexicon: shared, coherence, grammar_tier: tier, sample });
        modal.push(mm);
    }
    // mutual intelligibility → languages (average linkage: a community belongs to a language
    // if, on average, it understands that language's communities)
    let nc = modal.len();
    let mut intel = vec![vec![1.0f32; nc]; nc];
    let mut min_intel: f32 = 1.0;
    // how well do community b's speakers recognise community a's usual words (any form they
    // know for the concept, within one sound)?
    let understand = |a: usize, b: usize| -> f32 {
        // only words the listener has *some* word for count: a gap in vocabulary is not a
        // different language, a different word is
        let members = &comms[b].1;
        let mut ok = 0u32;
        let mut n = 0u32;
        for (c, fa) in &modal[a] {
            for &m in members {
                if let Some(v) = sim.people[m as usize].mind.lex.entries.get(c) {
                    n += 1;
                    if v.iter().any(|(f, _)| edit_distance(f, fa) <= 1) {
                        ok += 1;
                    }
                }
            }
        }
        if n > 0 { ok as f32 / n as f32 } else { 0.0 }
    };
    for a in 0..nc {
        for b in (a + 1)..nc {
            let x = 0.5 * (understand(a, b) + understand(b, a));
            intel[a][b] = x;
            intel[b][a] = x;
            min_intel = min_intel.min(x);
        }
    }
    let mut order: Vec<usize> = (0..nc).collect();
    order.sort_by(|a, b| info[*b].speakers.cmp(&info[*a].speakers).then(a.cmp(b)));
    let mut clusters: Vec<Vec<usize>> = Vec::new();
    for a in order {
        match clusters.iter_mut().find(|cl| cl.iter().map(|&b| intel[a][b]).sum::<f32>() / cl.len() as f32 >= 0.5) {
            Some(cl) => cl.push(a),
            None => clusters.push(vec![a]),
        }
    }
    let mut langs_out = Vec::new();
    for cl in &clusters {
        let mut li = info[cl[0]].clone();
        li.speakers = cl.iter().map(|&a| info[a].speakers).sum();
        li.communities = cl.iter().map(|&a| comms[a].0).collect();
        langs_out.push(li);
    }
    let languages = clusters.len() as u32;
    sim.obs.languages = langs_out;
    if languages > sim.obs.max_languages.max(1) {
        sim.emit(EvKind::LanguageSplit, vec![], 0, format!("The people now speak {languages} mutually unintelligible tongues"));
    }
    sim.obs.max_languages = sim.obs.max_languages.max(languages);

    // ---------- behaviour diversity
    let mut hs = 0.0;
    let mut mean = [0.0f32; N_ACT];
    for &id in &adults {
        let a = &sim.people[id as usize].activity;
        hs += entropy(a);
        let s: f32 = a.iter().sum::<f32>().max(1e-6);
        for k in 0..N_ACT {
            mean[k] += a[k] / s;
        }
    }
    let na = adults.len().max(1) as f32;
    let h_mean = hs / na;
    let divergence = (entropy(&mean) - h_mean).max(0.0);

    // ---------- notability
    for &id in &sim.alive.clone() {
        let p = &sim.people[id as usize];
        let kids = p.children.iter().filter(|&&c| sim.people[c as usize].alive).count() as f32;
        let r = p.roles;
        let score = p.inventions as f32 * 3.0
            + p.founded_camps as f32 * 4.0
            + p.predators_killed as f32 * 2.0
            + if r & (1 << role::LEADER) != 0 { 4.0 } else { 0.0 }
            + if r & (1 << role::PROPHET) != 0 { 4.0 } else { 0.0 }
            + if r & (1 << role::LAWGIVER) != 0 { 3.0 } else { 0.0 }
            + if r & (1 << role::HEALER) != 0 { 1.5 } else { 0.0 }
            + kids * 0.3
            + *respecters.get(&id).unwrap_or(&0) as f32 * 0.3;
        sim.people[id as usize].notability = score;
    }

    // ---------- change index
    let mut churn = 0u32;
    for &id in &adults {
        if sim.obs.membership.get(&id) != sim.obs.prev_cycle_membership.get(&id) {
            churn += 1;
        }
    }
    sim.obs.prev_cycle_membership = sim.obs.membership.clone();
    let prev_pop = sim.obs.prev_pop.max(1);
    let d_pop = (pop as f32 - prev_pop as f32).abs() / prev_pop as f32;
    let k_churn = ((sim.counters.inventions + sim.counters.taught) as f32 / pop.max(1) as f32).min(1.0);
    let change = (d_pop.min(1.0) + churn as f32 / na + k_churn + role_churn as f32 / na) / 4.0;
    sim.obs.prev_pop = pop;

    // ---------- assemble
    let known: u32 = sim.nodes.iter().filter(|s| s.holders > 0).count() as u32;
    let total_food: f32 = sim.world.cells.iter().map(|c| c.algae + c.shell).sum();
    let cnt = std::mem::take(&mut sim.counters);
    let comm: (u32, u32) = sim.alive.iter().fold((0, 0), |a, &id| {
        let m = &sim.people[id as usize].mind;
        (a.0 + m.comm_ok, a.1 + m.comm_try)
    });
    let st = CycleStats {
        cycle,
        pop,
        adults: adults.len() as u32,
        camps: sim.world.camps.iter().filter(|c| c.abandoned.is_none()).count() as u32,
        groups: sim.obs.groups.len() as u32,
        predators: sim.world.predators.iter().filter(|p| p.alive).count() as u32,
        herd_animals: sim.world.herds.iter().map(|h| h.count).sum::<f32>() as u32,
        food_total: total_food,
        real_year: sim.world.climate.real_year,
        surface_t: sim.world.climate.surface_t,
        known_nodes: known,
        knowledge_lost_total: sim.nodes.iter().map(|s| s.lost.len() as u32).sum(),
        languages,
        min_intelligibility: if nc >= 2 { min_intel } else { 1.0 },
        coherence: if nc > 0 { coh_sum / nc as f32 } else { 0.0 },
        lexicon_mean: if nc > 0 { lex_sum / nc as f32 } else { 0.0 },
        max_grammar_tier: sim.obs.languages.iter().map(|l| l.grammar_tier).max().unwrap_or(0),
        comm_success: if comm.1 > 0 { comm.0 as f32 / comm.1 as f32 } else { 0.0 },
        roles_distinct,
        role_counts,
        belief_diversity,
        distinct_beliefs: all_beliefs.len() as u32,
        false_belief_frac: wrong_people as f32 / na,
        culture_specific_false: culture_false,
        behavior_entropy: h_mean,
        behavior_divergence: divergence,
        change_index: change,
        mean_fatigue: {
            let v: Vec<f32> = sim.world.cells.iter().filter(|c| c.algae_cap > 0.0).map(|c| c.fatigue).collect();
            v.iter().sum::<f32>() / v.len().max(1) as f32
        },
        max_foul: sim.world.foul.iter().fold(0.0f32, |a, b| a.max(*b)),
        decisions: sim.decisions,
        c: cnt,
    };
    sim.stats.push(st);
}
