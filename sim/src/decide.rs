//! Decision making and action effects (DESIGN §7.9, §8): utility scoring over candidate
//! actions, anticipatory intents, requests and responses, the naming game, gossip,
//! teaching, experimentation, building, theft and punishment.
use crate::concepts::{outcome, Concept, Kind};
use crate::discovery::{mat, n, need, KnowSource, NODES};
use crate::events::EvKind;
use crate::language::edit_distance;
use crate::mind::{drv, emo, kin, EpKind, Rel};
use crate::person::{Act, Intent};
use crate::sim::{pair_mut, Sim, Task};
use crate::social::role;
use crate::thought::{result, saw, why, Thought, NONE};
use crate::world::{dist, place_of, Habitat, N_PLACES, PW};

const SPEED: f32 = 1.2;

#[derive(Clone, Copy)]
struct Cand {
    act: Act,
    u: f32,
    why: [(u8, f32); 3],
    saw: (u8, u32),
}

fn c(act: Act, u: f32, w: &[(u8, f32)]) -> Cand {
    let mut why = [(NONE, 0.0); 3];
    let mut v: Vec<(u8, f32)> = w.iter().copied().map(|x| (x.0, x.1.min(1.0))).filter(|x| x.1 > 0.05).collect();
    v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
    for (k, x) in v.into_iter().take(3).enumerate() {
        why[k] = x;
    }
    Cand { act, u, why, saw: (NONE, 0) }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Req {
    Food,
    Hunt,
    Court,
    Migrate,
}

impl Sim {
    fn kin_of(&self, i: usize, j: u32) -> u8 {
        self.people[i].mind.model(j).map_or(kin::NONE, |m| m.kin)
    }

    fn carry_cap(&self, i: usize) -> f32 {
        self.cfg.people.carry_cap * if self.people[i].knows(n::BASKET) { 2.0 } else { 1.0 }
    }

    fn home_pos(&self, i: usize) -> Option<(f32, f32)> {
        self.people[i].home.map(|h| (self.world.camps[h as usize].x, self.world.camps[h as usize].y))
    }

    /// Start an action: move from current position to `to`, finishing after travel + work.
    fn start(&mut self, i: usize, act: Act, to: (f32, f32), work: u64) {
        let tick = self.tick;
        let p = &mut self.people[i];
        let from = p.pos(tick);
        let d = self.world.travel_dist(from.0, from.1, to.0, to.1);
        let travel = (d / SPEED) as u64;
        p.from = from;
        p.to = to;
        p.act = act;
        p.act_start = tick;
        p.next_decision = tick + (travel + work).max(8);
        let k = act.kind();
        p.activity[k] += 1.0;
        p.activity_total[k] += 1;
    }

    // ================================================================ deciding

    pub fn decide(&mut self, i: usize, threat: Option<u32>) {
        self.decisions += 1;
        self.update_energy(i);
        let tick = self.tick;
        let tpc = self.tpc();
        let age = self.people[i].age_cycles(tick, tpc);
        let adult = age >= self.cfg.people.adult_cycles;
        let (x, y) = self.people[i].pos(tick);
        let place = place_of(x, y);

        // infants stay with their mother
        if age < 2.0 {
            let target = self.people[i].mother.filter(|m| self.people[*m as usize].alive).or(self.people[i].father.filter(|f| self.people[*f as usize].alive));
            let to = match target {
                Some(m) => self.people[m as usize].pos(tick),
                None => self.home_pos(i).unwrap_or((x, y)),
            };
            let act = target.map(|w| Act::Follow { who: w }).unwrap_or(Act::Rest);
            self.start(i, act, to, 96);
            return;
        }

        let night = self.is_night(tick);
        let at_home = self.at_home(i);
        let home = self.home_pos(i);
        let noise = self.cfg.mind.noise;
        let mut cands: Vec<Cand> = Vec::with_capacity(24);
        let mut saw_list: Vec<(u8, u32)> = Vec::new();

        // ---------- threats (interrupt) and phantom fears
        let mut phantom = false;
        if threat.is_none() && night && !at_home && self.people[i].mind.places[place as usize].danger > 0.5 {
            let pr = 0.04 * (1.2 - self.people[i].mind.traits[0]);
            phantom = self.people[i].rng.chance(pr);
        }
        if threat.is_some() || phantom {
            let (tx, ty) = match threat.and_then(|pid| self.world.predators.iter().find(|q| q.id == pid)) {
                Some(q) => (q.x, q.y),
                None => (x + 10.0, y + 10.0),
            };
            let p = &self.people[i];
            let m = &p.mind;
            let fear = m.emo[emo::FEAR];
            let bold = m.traits[0];
            let spear = if p.knows(n::SPEAR) { 1.0 } else { 0.0 };
            let others = self.near(x, y, 30.0);
            let kin_near = others.iter().filter(|&&o| o != p.id && self.kin_of(i, o) != kin::NONE && !self.adult(o as usize)).count() as f32;
            let defenders = others.len().saturating_sub(1).min(6) as f32;
            let u_flee = 0.6 + fear * 0.6 + (1.0 - bold) * 0.5 + m.expect[outcome::ATTACK].0 * 0.2;
            let hunting_kin = threat
                .and_then(|pid| self.world.predators.iter().find(|q| q.id == pid))
                .and_then(|q| q.target)
                .map_or(false, |t| t == p.id || self.kin_of(i, t) != kin::NONE);
            let u_def = if adult && !phantom {
                bold * 0.7 + m.traits[3] * 0.25 + kin_near.min(2.0) * 0.3 + spear * 0.45 + defenders * 0.04 - fear * 0.4 + if hunting_kin { 0.35 } else { -0.3 }
            } else {
                -1.0
            };
            let s = if phantom { saw::PHANTOM } else { saw::GULPER };
            let pid = threat.unwrap_or(0);
            let mut cf = c(Act::Flee, u_flee, &[(why::FEAR, fear + 0.3), (why::SAFETY, 0.5), (why::EXPECT_ATTACK, m.expect[outcome::ATTACK].0)]);
            cf.saw = (s, pid);
            let mut cd = c(Act::Defend { pred: pid }, u_def, &[(why::KIN, kin_near * 0.4), (why::ANGER, m.traits[3]), (why::STATUS, bold * 0.5)]);
            cd.saw = (s, pid);
            let noise1 = self.people[i].rng.normal() * noise;
            let choice = if u_def + noise1 > u_flee { cd } else { cf };
            let (to, work) = match choice.act {
                Act::Flee => {
                    let (hx, hy) = home.unwrap_or((x, y));
                    let (dx, dy) = (x - tx, y - ty);
                    let d = (dx * dx + dy * dy).sqrt().max(1.0);
                    let away = ((x + dx / d * 40.0).clamp(4.0, crate::world::TANK_W - 4.0), (y + dy / d * 40.0).clamp(4.0, crate::world::TANK_H - 4.0));
                    if home.is_some() && dist(hx, hy, tx, ty) > 30.0 {
                        ((hx, hy), 24)
                    } else {
                        (away, 24)
                    }
                }
                _ => ((tx, ty), 16),
            };
            if phantom {
                let p = &mut self.people[i];
                p.mind.feel(emo::FEAR, 0.3);
                self.episode(i, EpKind::FalseAlarm, None, -0.2, 0.2);
            }
            self.record(i, &choice, true, result::NONE);
            self.start(i, choice.act, to, work);
            return;
        }

        // ---------- ordinary decisions
        let rand_places = {
            let r = &mut self.people[i].rng;
            (r.below(N_PLACES as u32) as u8, r.below(N_PLACES as u32) as u8, r.below(N_PLACES as u32) as u8)
        };
        let others: Vec<u32> = self.near(x, y, 26.0).into_iter().filter(|&o| o != i as u32).collect();
        let p = &self.people[i];
        let m = &p.mind;
        let tr = m.traits;
        let energy = p.energy;
        let hunger = 1.0 - energy;
        let fear = m.emo[emo::FEAR];
        let anger = m.emo[emo::ANGER];
        let grief = m.emo[emo::GRIEF];
        let exp_h = m.expect[outcome::HUNGER].0;
        let exp_cold = m.expect[outcome::COLD_STRESS].0;
        let exp_sick = m.expect[outcome::SICKNESS].0;
        let carried = p.carried;
        let stash = p.stash;
        let cap = self.carry_cap(i);
        let food_have = carried + if at_home { stash } else { 0.0 };
        let stockpiling = matches!(p.intent, Some(Intent::Stockpile { target, until }) if until > tick && stash < target);
        let believes_spread = m.belief(Concept::Kind(Kind::Sickness), Rel::Spreads) > 0.5 || p.knows(n::QUARANTINE);
        let norm_steal = m.belief(Concept::Kind(Kind::Steal), Rel::Forbidden);

        // Eat
        if food_have > 0.2 {
            let u = hunger * 1.5 + if hunger > 0.6 { 0.3 } else { 0.0 } - 0.35;
            cands.push(c(Act::Eat, u, &[(why::HUNGER, hunger)]));
        }
        // Forage: evaluate a handful of places
        {
            let mut places: Vec<u8> = vec![place];
            if let Some(h) = p.home {
                places.push(self.world.camps[h as usize].place);
            }
            if adult {
                let mut best = (0u8, f32::MIN);
                for (k, pm) in m.places.iter().enumerate() {
                    let s = pm.food * (pm.conf + 0.2) - pm.danger * 0.5;
                    if s > best.1 && self.world.places[k].passable {
                        best = (k as u8, s);
                    }
                }
                places.push(best.0);
                places.push(rand_places.0);
                places.push(rand_places.1);
            }
            places.sort();
            places.dedup();
            let base = hunger * 0.9 + exp_h * 0.5 + if stockpiling { 0.45 } else { 0.0 } + if stash < 2.0 { 0.25 } else { 0.0 } + 0.15 - (carried / cap) * 0.9 - if night { if p.knows(n::GLOWLAMP) { 0.1 } else { 0.45 } } else { 0.0 };
            for pl in places {
                if !self.world.places[pl as usize].passable {
                    continue;
                }
                let pm = &m.places[pl as usize];
                let (cx, cy) = (self.world.places[pl as usize].cx, self.world.places[pl as usize].cy);
                let d = self.world.travel_dist(x, y, cx, cy);
                if d > 220.0 {
                    continue;
                }
                let danger_w = 0.4 + fear + (1.0 - tr[0]) * 0.5;
                let s = (pm.food + 0.15) * (1.0 - pm.danger * danger_w).max(0.0) - d / 400.0;
                let u = base * (0.6 + s);
                cands.push(c(
                    Act::Forage { place: pl },
                    u,
                    &[
                        (why::HUNGER, hunger),
                        (why::EXPECT_HUNGER, exp_h),
                        (why::STOCKPILE, if stockpiling { 0.5 } else { 0.0 }),
                        (why::PLACE_FOOD, pm.food),
                        (why::PLACE_DANGER, -pm.danger),
                    ],
                ));
            }
        }
        // Store at home
        if carried > 0.8 && home.is_some() {
            let u = 0.2 + (carried / cap) * 0.6 + if night { 0.3 } else { 0.0 } + exp_h * 0.3 + if stockpiling { 0.3 } else { 0.0 } - hunger * 0.3;
            cands.push(c(Act::Store, u, &[(why::EXPECT_HUNGER, exp_h), (why::STOCKPILE, if stockpiling { 0.4 } else { 0.0 }), (why::NIGHT, if night { 0.3 } else { 0.0 })]));
        }
        // Sleep / rest
        {
            let rest = m.drives[drv::REST];
            let u = if night { 0.45 + rest * 0.9 } else { rest * 0.3 - 0.25 } - hunger * 0.35 + exp_cold * 0.1;
            cands.push(c(Act::Sleep, u, &[(why::NIGHT, if night { 0.6 } else { 0.0 }), (why::REST, rest)]));
            if p.health < 0.6 {
                cands.push(c(Act::Rest, (1.0 - p.health) * 0.8 - hunger * 0.3, &[(why::REST, 1.0 - p.health)]));
            }
        }
        // Social candidates
        let belonging = m.drives[drv::BELONGING];
        let mut best_social: Option<(u32, f32, u8)> = None;
        let mut hungry_other: Option<(u32, f32, f32)> = None;
        let mut sick_other: Option<(u32, f32)> = None;
        let mut thief_here: Option<(u32, f32)> = None;
        let mut mate_cand: Option<(u32, f32)> = None;
        let mut rich_other: Option<(u32, f32)> = None;
        let mut learner: Option<(u32, u8, f32)> = None;
        let mut share_why = (0.0f32, 0.0f32);
        for &o in &others {
            let q = &self.people[o as usize];
            let mo = m.model(o);
            let (aff, trust, grudge, k) = mo.map_or((0.0, 0.2, 0.0, kin::NONE), |mm| (mm.affinity, mm.trust, mm.grudge, mm.kin));
            let stranger = (q.culture != p.culture || q.stranger) && mo.map_or(true, |mm| mm.affinity < 0.3 && mm.kin == kin::NONE);
            if stranger {
                saw_list.push((saw::STRANGER, o));
            }
            let q_sick = q.infected.is_some();
            let mut s = aff * 0.6 + if k != kin::NONE { 0.4 } else { 0.0 } + trust * 0.2 - grudge * 0.8;
            if stranger {
                s += tr[2] * 0.4 - tr[5] * 0.3;
            }
            if q_sick && believes_spread {
                s -= 0.6;
            }
            if best_social.map_or(true, |b| s > b.1) {
                best_social = Some((o, s, if stranger { 1 } else { 0 }));
            }
            let q_h = 1.0 - q.energy;
            if q_h > 0.55 {
                let kin_w = if k != kin::NONE { p.genome.kin_bias } else { 0.0 };
                let deserve = kin_w + aff.clamp(-1.0, 1.0) * 0.5 + p.genome.cooperation * 0.4 - grudge;
                if hungry_other.map_or(true, |h| deserve > h.1) {
                    hungry_other = Some((o, deserve, q_h));
                    share_why = (kin_w, aff.clamp(0.0, 1.0));
                }
            }
            if q_sick {
                saw_list.push((saw::SICK, o));
                let w = aff + if k != kin::NONE { 0.6 } else { 0.0 } + p.genome.cooperation * 0.3;
                if sick_other.map_or(true, |s2| w > s2.1) {
                    sick_other = Some((o, w));
                }
            }
            let thief = m.belief(Concept::Person(o), Rel::Thief);
            let recently = m.episodes.iter().any(|e| e.other == Some(o) && matches!(e.kind, EpKind::Punished | EpKind::StolenFrom) && tick.saturating_sub(e.tick) < 2 * self.tpd());
            let punished_lately = p.thoughts.iter().rev().take(24).any(|t| t.chose == Act::Punish { who: o });
            if (thief > 0.4 || grudge > 0.5) && !punished_lately && (recently || thief > 0.6 || tr[3] > 0.6) {
                saw_list.push((saw::THIEF, o));
                let w = thief * (norm_steal + 0.2) + grudge;
                if thief_here.map_or(true, |t| w > t.1) {
                    thief_here = Some((o, w));
                }
            }
            if adult && p.mate.is_none() && q.mate.is_none() && q.female != p.female && self.adult(o as usize) && k == kin::NONE {
                let sig: f32 = (0..3).map(|z| (p.genome.mating_sig[z] - q.genome.mating_sig[z]).abs()).sum::<f32>();
                let w = aff * 0.5 + 0.3 - sig * 0.3;
                if mate_cand.map_or(true, |mc| w > mc.1) {
                    mate_cand = Some((o, w));
                }
            }
            let q_food = q.carried + if q.home == p.home && q.home.is_some() { q.stash } else { 0.0 };
            if q_food > 1.5 && q.energy > 0.45 {
                let (acc, refu) = mo.map_or((0, 0), |mm| (mm.accepts, mm.refusals));
                let generosity = (acc as f32 + 1.0) / (acc as f32 + refu as f32 + 2.0);
                let w = (trust + if k != kin::NONE { 0.5 } else { 0.0 } + aff * 0.3) * generosity * 1.6;
                if rich_other.map_or(true, |r| w > r.1) {
                    rich_other = Some((o, w));
                }
            }
            if (k != kin::NONE || aff > 0.4) && learner.is_none() {
                for kn in &p.knowledge {
                    if kn.prof > 0.4 && !q.knows(kn.node) {
                        let w = if k == kin::CHILD { 0.5 } else { 0.2 } + aff * 0.3;
                        learner = Some((o, kn.node, w));
                        break;
                    }
                }
            }
        }
        if let Some((o, s, stranger)) = best_social {
            let u = belonging * 0.75 + tr[1] * 0.35 + s * 0.3 - hunger * 0.45 + if night { 0.05 } else { 0.0 } - 0.15;
            let mut cc = c(Act::Socialize { who: o }, u, &[(why::BELONGING, belonging), (why::TRUST, s), (why::STRANGER, stranger as f32 * tr[2])]);
            if stranger == 1 {
                cc.saw = (saw::STRANGER, o);
            }
            cands.push(cc);
        }
        if let Some((o, deserve, qh)) = hungry_other {
            if food_have > 1.5 {
                let norm = if p.knows(n::SHARING) { 0.35 } else { 0.0 };
                let holding_back = if share_why.0 == 0.0 { m.drives[drv::CARE] * 0.6 } else { 0.0 };
                let u = qh * (deserve + norm + 0.1) - hunger * 0.8 - exp_h * 0.3 * (1.0 - p.genome.cooperation) - holding_back - 0.1;
                let mut cc = c(Act::Share { who: o }, u, &[(why::KIN, share_why.0), (why::NORM, norm), (why::TRUST, share_why.1), (why::HUNGER, -hunger)]);
                cc.saw = (saw::HUNGRY, o);
                cands.push(cc);
            }
        }
        if let Some((o, w)) = rich_other {
            if hunger > 0.5 && food_have < 0.5 {
                let u = hunger * 0.9 * (0.3 + w) - tr[5] * 0.1;
                cands.push(c(Act::AskHelp { who: o }, u, &[(why::HUNGER, hunger), (why::TRUST, w)]));
                // theft as the desperate alternative
                let q = &self.people[o as usize];
                let k = self.kin_of(i, o);
                if k == kin::NONE {
                    let aff = m.model(o).map_or(0.0, |mm| mm.affinity);
                    let refused = p.mind.episodes.iter().filter(|e| e.kind == EpKind::WasRefused && tick.saturating_sub(e.tick) < 4 * self.tpd()).count() as f32;
                    let grudge_o = m.model(o).map_or(0.0, |mm| mm.grudge);
                    let u = hunger * 0.95 + tr[3] * 0.35 + refused.min(3.0) * 0.15 + grudge_o * 0.3 - p.genome.cooperation * 0.3 - norm_steal * (0.4 + tr[5] * 0.8) - aff.max(0.0) * 0.5 - (others.len() as f32 * 0.02) - q.genome.dominance * 0.15 - 0.3;
                    cands.push(c(Act::Steal { who: o }, u, &[(why::DESPERATE, hunger + refused * 0.15), (why::GRUDGE, grudge_o), (why::NORM, -norm_steal)]));
                }
            }
        }
        if let Some((o, w)) = sick_other {
            if p.knows(n::REMEDY) && adult {
                let healer_habit = (p.activity[22] * 0.05).min(0.3);
                let u = 0.3 + w * 0.5 + healer_habit - hunger * 0.4;
                let mut cc = c(Act::Heal { who: o }, u, &[(why::CARE, w), (why::HABIT, healer_habit)]);
                cc.saw = (saw::SICK, o);
                cands.push(cc);
            }
        }
        if let Some((o, w)) = thief_here {
            if adult {
                let them = &self.people[o as usize];
                let fear_them = m.model(o).map_or(0.0, |mm| mm.fear);
                let u = anger * 0.6 + w * 0.5 + tr[3] * 0.3 + tr[5] * norm_steal * 0.2 - them.genome.dominance * 0.25 - fear * 0.2 - fear_them * 0.4 - tr[4] * 0.2 - 0.3;
                let mut cc = c(Act::Punish { who: o }, u, &[(why::GRUDGE, w), (why::ANGER, anger), (why::NORM, norm_steal)]);
                cc.saw = (saw::THIEF, o);
                cands.push(cc);
            }
        }
        if let Some((o, w)) = mate_cand {
            let rep = m.drives[drv::REPRO];
            cands.push(c(Act::Court { who: o }, rep * 0.7 + w * 0.4 - hunger * 0.4 - 0.15, &[(why::MATING, rep)]));
        }
        if let Some((o, node, w)) = learner {
            let u = 0.15 + w * 0.6 + tr[4] * 0.2 + m.drives[drv::CARE] * 0.2 - hunger * 0.5;
            cands.push(c(Act::Teach { who: o, node }, u, &[(why::KIN, w), (why::CARE, tr[4])]));
        }
        // Joint hunt: lead a hunt for drifters
        if adult && others.len() >= 1 {
            let mut best_h = None;
            let mut bd = 140.0;
            for h in &self.world.herds {
                if h.count >= 1.0 && self.world.on_same_side(x, h.x) {
                    let d = dist(x, y, h.x, h.y);
                    if d < bd {
                        bd = d;
                        best_h = Some(h.id);
                    }
                }
            }
            if let Some(hid) = best_h {
                let lure = if p.knows(n::LURE) { 0.15 } else { 0.0 };
                let spear = if p.knows(n::SPEAR) { 0.2 } else { 0.0 };
                let u = hunger * 0.5 + exp_h * 0.4 + m.drives[drv::STATUS] * 0.25 + tr[0] * 0.3 + p.skill_hunt * 0.3 + lure + spear - bd / 400.0 - p.needs[need::HUNT_FAIL] * 0.05 - 0.35 - if night { 0.4 } else { 0.0 };
                let mut cc = c(Act::Hunt { task: hid }, u, &[(why::HUNGER, hunger), (why::EXPECT_HUNGER, exp_h), (why::STATUS, m.drives[drv::STATUS])]);
                cc.saw = (saw::HERD, hid);
                cands.push(cc);
            }
        }
        // Experiment / play
        {
            let need_max = p.needs.iter().fold(0.0f32, |a, b| a.max(*b)).min(3.0) / 3.0;
            let cur = m.drives[drv::CURIOSITY];
            if adult {
                let u = cur * 0.5 * (0.3 + tr[2]) + need_max * 0.25 - hunger * 0.8 - fear * 0.3 - 0.2;
                cands.push(c(Act::Experiment, u, &[(why::CURIOSITY, cur * tr[2]), (why::NEED, need_max)]));
            } else {
                let u = p.genome.play * 0.6 + cur * 0.3 - hunger * 0.5;
                cands.push(c(Act::Play, u, &[(why::CURIOSITY, cur)]));
            }
            // Explore
            if adult {
                let target = rand_places.2;
                let pm = &m.places[target as usize];
                if self.world.places[target as usize].passable {
                    let td = self.world.travel_dist(x, y, self.world.places[target as usize].cx, self.world.places[target as usize].cy);
                    let u = cur * 0.45 * tr[2] + tr[0] * 0.15 + (1.0 - pm.conf) * 0.2 - pm.danger * 0.4 - hunger * 0.4 - if night { 0.4 } else { 0.0 } - 0.15 - td / 500.0;
                    cands.push(c(Act::Explore { place: target }, u, &[(why::CURIOSITY, cur)]));
                }
            }
        }
        // Build at home
        if adult && at_home {
            if let Some(hc) = p.home {
                let camp = &self.world.camps[hc as usize];
                let pmats = self.world.places[camp.place as usize].mats;
                for &node in &[n::CACHE, n::BURROW, n::HALL, n::WALL, n::GARDEN, n::SHELL_BED] {
                    if !p.knows(node) {
                        continue;
                    }
                    let def = &NODES[node as usize];
                    if !def.materials.is_empty() && !def.materials.iter().any(|mm| pmats & mm != 0) {
                        continue;
                    }
                    let existing = camp.structures.iter().find(|s| s.node == node).map_or(0.0, |s| s.condition);
                    let relevance: f32 = def.needs.iter().map(|nn| p.needs[*nn]).sum::<f32>().min(2.0) / 2.0;
                    let garden = matches!(node, n::GARDEN | n::SHELL_BED);
                    let lacking = if garden { 0.6 } else { 1.0 - existing };
                    let u = lacking * 0.5 + relevance * 0.5 + exp_h * if garden { 0.3 } else { 0.0 } + exp_cold * if node == n::HALL || node == n::BURROW { 0.3 } else { 0.0 } - hunger * 0.5 - 0.2;
                    cands.push(c(Act::Build { node }, u, &[(why::NEED, relevance), (why::EXPECT_HUNGER, exp_h * garden as u8 as f32), (why::EXPECT_COLD, exp_cold)]));
                }
            }
        }
        // Mourn
        if grief > 0.3 {
            cands.push(c(Act::Mourn, grief * 0.8 - hunger * 0.3, &[(why::GRIEF, grief)]));
        }
        // Migration (considered every few days)
        if adult && self.day() >= p.consider_move_day {
            if let Some((act, u, w)) = self.migration_option(i) {
                cands.push(c(act, u, &w));
            }
        }
        cands.push(c(Act::Idle, 0.02, &[]));

        // anticipatory intent: stockpile before expected scarcity
        let form_stockpile = exp_h > 0.45 && !stockpiling && adult && stash < 6.0;
        let avoid_sick = exp_sick > 0.4;

        // choose (with a habit term: practiced activities feel natural, which breeds specialists)
        let act_tot: f32 = self.people[i].activity.iter().sum::<f32>().max(1.0);
        let mut best = 0;
        let mut bu = f32::MIN;
        for (k, cd) in cands.iter().enumerate() {
            let share = self.people[i].activity[cd.act.kind()] / act_tot;
            let habit = if matches!(cd.act, Act::Eat | Act::Sleep | Act::Idle | Act::Store | Act::Follow { .. }) { 0.0 } else { share * 0.6 };
            let u = cd.u + habit + self.people[i].rng.normal() * noise;
            if u > bu {
                bu = u;
                best = k;
            }
        }
        let mut choice = cands[best];
        if choice.saw.0 == NONE {
            if let Some(s) = saw_list.first() {
                choice.saw = *s;
            }
        }
        if self.day() >= self.people[i].consider_move_day {
            self.people[i].consider_move_day = self.day() + 5 + self.people[i].rng.below(4) as u64;
        }
        if form_stockpile {
            let target = 6.0 + 6.0 * self.people[i].mind.traits[4];
            let until = tick + 6 * self.tpd();
            self.people[i].intent = Some(Intent::Stockpile { target, until });
            let th = Thought {
                tick,
                who: i as u32,
                felt: self.people[i].mind.top_emotion().map_or((NONE, 0.0), |(e, v)| (e as u8, v)),
                drive: (drv::HUNGER as u8, hunger),
                expect: (outcome::HUNGER as u8, self.people[i].mind.expect[outcome::HUNGER].1 as u8, exp_h),
                saw: [(saw::FOOD_LOW, 0), (NONE, 0)],
                chose: Act::Forage { place },
                why: [(why::EXPECT_HUNGER, exp_h), (why::STOCKPILE, 1.0), (NONE, 0.0)],
                interrupt: false,
                result: result::NONE,
            };
            self.log_thought(i, th);
        }
        let _ = avoid_sick;
        self.execute(i, choice);
    }

    fn migration_option(&self, i: usize) -> Option<(Act, f32, Vec<(u8, f32)>)> {
        let p = &self.people[i];
        let m = &p.mind;
        let (x, y) = p.pos(self.tick);
        let Some(h) = p.home else {
            // homeless: join the camp here, or found one
            let here = self.world.camps.iter().filter(|c| c.abandoned.is_none()).min_by(|a, b| {
                dist(x, y, a.x, a.y).partial_cmp(&dist(x, y, b.x, b.y)).unwrap().then(a.id.cmp(&b.id))
            });
            if let Some(c) = here {
                if dist(x, y, c.x, c.y) < 120.0 {
                    return Some((Act::Migrate { camp: c.id }, 0.8, vec![(why::BELONGING, 0.8)]));
                }
            }
            return Some((Act::Found { place: place_of(x, y) }, 0.6, vec![(why::BELONGING, 0.6)]));
        };
        let camp = &self.world.camps[h as usize];
        let crowd = ((camp.members as f32 - 22.0) / 22.0).clamp(0.0, 1.0);
        let home_food = m.places[camp.place as usize].food;
        let scarcity = (1.0 - home_food * 1.6).clamp(0.0, 1.0);
        let mut conflict = 0.0f32;
        let mut ties = 0.0f32;
        for md in &m.models {
            let q = &self.people[md.who as usize];
            if q.alive && q.home == Some(h) {
                conflict = conflict.max(md.grudge);
                ties += md.affinity.max(0.0) * if md.kin != kin::NONE { 1.5 } else { 0.4 };
            }
        }
        let danger = m.places[camp.place as usize].danger;
        let d = 0.45 * crowd + 0.35 * scarcity + 0.5 * conflict + 0.25 * danger - (ties * 0.05).min(0.5);
        let u = d * 1.3 + m.traits[0] * 0.25 - 0.55;
        if u < 0.0 {
            return None;
        }
        // best known place without a camp nearby
        let mut best = None;
        let mut bs = f32::MIN;
        for (k, pm) in m.places.iter().enumerate() {
            if k == camp.place as usize || !self.world.places[k].passable {
                continue;
            }
            let occupied = self.world.camps.iter().any(|c| c.abandoned.is_none() && c.place as usize == k);
            let td = self.world.travel_dist(x, y, self.world.places[k].cx, self.world.places[k].cy);
            if td > 300.0 {
                continue;
            }
            let s = pm.food * (0.3 + pm.conf) - pm.danger * 0.8 - if occupied { 0.5 } else { 0.0 } - td / 300.0;
            if s > bs {
                bs = s;
                best = Some(k as u8);
            }
        }
        best.map(|pl| (Act::Found { place: pl }, u, vec![(why::CROWDED, crowd), (why::HUNGER, scarcity), (why::GRUDGE, conflict), (why::PLACE_DANGER, danger)]))
    }

    fn record(&mut self, i: usize, cd: &Cand, interrupt: bool, res: u8) {
        let p = &self.people[i];
        let m = &p.mind;
        let felt = m.top_emotion().map_or((NONE, 0.0), |(e, v)| (e as u8, v));
        let (dk, dv) = m.drives.iter().enumerate().fold((0usize, 0.0f32), |a, (k, v)| if *v > a.1 { (k, *v) } else { a });
        let mut expect = (NONE, NONE, 0.0);
        for (o, (pr, cue)) in m.expect.iter().enumerate() {
            if *pr > expect.2 && *pr > 0.3 {
                expect = (o as u8, *cue as u8, *pr);
            }
        }
        let th = Thought {
            tick: self.tick,
            who: i as u32,
            felt,
            drive: (dk as u8, dv),
            expect,
            saw: [cd.saw, (NONE, 0)],
            chose: cd.act,
            why: cd.why,
            interrupt,
            result: res,
        };
        self.log_thought(i, th);
    }

    fn meaningful(prev: &Act, a: &Act) -> bool {
        !matches!(a, Act::Eat | Act::Sleep | Act::Idle | Act::Store | Act::Rest | Act::Follow { .. })
            && !(matches!(a, Act::Forage { .. }) && matches!(prev, Act::Forage { .. } | Act::Store | Act::Eat))
    }

    /// Turn a chosen candidate into an action (handling requests that need a partner's consent).
    fn execute(&mut self, i: usize, cd: Cand) {
        let tick = self.tick;
        let prev = self.people[i].act;
        let (x, y) = self.people[i].pos(tick);
        let meaningful = Self::meaningful(&prev, &cd.act);
        let mut res = result::NONE;
        match cd.act {
            Act::Eat => {
                self.start(i, Act::Eat, (x, y), 16);
            }
            Act::Forage { place } => {
                let pl = &self.world.places[place as usize];
                let (tx, ty) = (pl.cx + self.people[i].rng.range(-18.0, 18.0), pl.cy + self.people[i].rng.range(-12.0, 12.0));
                self.start(i, cd.act, (tx, ty), 64);
            }
            Act::Store => {
                let h = self.home_pos(i).unwrap_or((x, y));
                self.start(i, cd.act, h, 16);
            }
            Act::Sleep => {
                let h = self.home_pos(i).unwrap_or((x, y));
                let tod = self.tick % self.tpd();
                let until_dawn = (self.tpd() - tod).max(128);
                self.people[i].mind.drives[drv::REST] = 0.0;
                self.start(i, cd.act, h, until_dawn.min(600));
            }
            Act::Rest => {
                self.start(i, cd.act, (x, y), 96);
            }
            Act::Socialize { who } | Act::Teach { who, .. } | Act::Heal { who } | Act::Punish { who } | Act::Steal { who } => {
                let to = self.people[who as usize].pos(tick);
                let work = match cd.act {
                    Act::Socialize { .. } => 48,
                    Act::Teach { .. } => 96,
                    Act::Heal { .. } => 64,
                    Act::Punish { .. } => 32,
                    _ => 24,
                };
                self.start(i, cd.act, (to.0 + 1.5, to.1), work);
            }
            Act::Share { who } => {
                let to = self.people[who as usize].pos(tick);
                self.start(i, cd.act, to, 16);
            }
            Act::AskHelp { who } => {
                let ok = self.respond(who as usize, i, Req::Food);
                res = if ok { result::ACCEPTED } else { result::REFUSED };
                let to = self.people[who as usize].pos(tick);
                self.start(i, cd.act, to, 24);
            }
            Act::Court { who } => {
                let ok = self.respond(who as usize, i, Req::Court);
                res = if ok { result::ACCEPTED } else { result::REFUSED };
                if ok {
                    self.bond(i, who as usize);
                }
                let to = self.people[who as usize].pos(tick);
                self.start(i, cd.act, to, 48);
            }
            Act::Hunt { task: herd } => {
                res = self.propose_hunt(i, herd);
                if res == result::REFUSED {
                    // nobody came; forage instead
                    let pl = place_of(x, y);
                    self.start(i, Act::Forage { place: pl }, (x, y), 64);
                }
            }
            Act::Defend { pred } => {
                let to = self.world.predators.iter().find(|q| q.id == pred).map_or((x, y), |q| (q.x, q.y));
                self.start(i, cd.act, to, 16);
            }
            Act::Flee => {
                self.start(i, cd.act, (x, y), 24);
            }
            Act::Experiment | Act::Play | Act::Mourn => {
                let w = if matches!(cd.act, Act::Mourn) { 96 } else { 128 };
                self.start(i, cd.act, (x, y), w);
            }
            Act::Build { .. } => {
                let h = self.home_pos(i).unwrap_or((x, y));
                self.start(i, cd.act, h, 192);
            }
            Act::Explore { place } => {
                let pl = &self.world.places[place as usize];
                let to = (pl.cx, pl.cy);
                self.start(i, cd.act, to, 48);
            }
            Act::Migrate { camp } => {
                let ok = self.join_camp(i, camp);
                res = if ok { result::ACCEPTED } else { result::REFUSED };
                if ok {
                    let c = &self.world.camps[camp as usize];
                    let to = (c.x, c.y);
                    self.start(i, cd.act, to, 16);
                } else {
                    self.start(i, Act::Idle, (x, y), 48);
                }
            }
            Act::Found { place } => {
                res = self.found_camp(i, place);
            }
            Act::Idle | Act::Follow { .. } => {
                self.start(i, Act::Idle, (x, y), 48);
            }
        }
        if meaningful || res != result::NONE {
            self.record(i, &cd, false, res);
        }
    }

    // ================================================================ requests

    /// Partner `j` decides whether to grant `i`'s request. This is a real decision by j's mind.
    pub fn respond(&mut self, j: usize, i: usize, req: Req) -> bool {
        if !self.people[j].alive {
            return false;
        }
        let tick = self.tick;
        let iid = i as u32;
        let q = &self.people[j];
        let mq = &q.mind;
        let md = mq.model(iid);
        let (trust, aff, ledger, respect, k, acc, refu) = md.map_or((0.2, 0.0, 0.0, 0.2, kin::NONE, 0, 0), |m| (m.trust, m.affinity, m.ledger, m.respect, m.kin, m.accepts, m.refusals));
        let reciprocity = if acc + refu > 0 { acc as f32 / (acc + refu) as f32 - 0.5 } else { 0.0 };
        let mut u = trust * 0.45 + aff * 0.35 + if k != kin::NONE { 0.45 } else { 0.0 } + ledger.max(-3.0) * -0.08 + respect * 0.3 + q.genome.cooperation * 0.3 + reciprocity * 0.2 - 0.62;
        let hunger_j = 1.0 - q.energy;
        let why_v;
        match req {
            Req::Food => {
                let have = q.carried + q.stash;
                u += if have > 3.0 { 0.25 } else { -0.3 } - hunger_j * 0.6 - mq.expect[outcome::HUNGER].0 * 0.3 * (1.0 - q.genome.cooperation);
                if q.knows(n::SHARING) {
                    u += 0.3;
                }
                if hunger_j > 0.65 && have < 4.0 && k != kin::CHILD {
                    u -= 1.2;
                }
                why_v = [(why::TRUST, trust), (why::KIN, if k != kin::NONE { 0.5 } else { 0.0 }), (why::HUNGER, -hunger_j)];
            }
            Req::Hunt => {
                u += hunger_j * 0.35 + mq.traits[0] * 0.2 - mq.emo[emo::FEAR] * 0.3 + mq.expect[outcome::HUNGER].0 * 0.25 + q.skill_hunt * 0.1 - mq.drives[drv::CARE] * 0.3;
                if matches!(q.act, Act::Sleep | Act::Flee | Act::Defend { .. }) && q.next_decision > tick {
                    u -= 0.6;
                }
                why_v = [(why::RESPECT, respect), (why::HUNGER, hunger_j), (why::TRUST, trust)];
            }
            Req::Court => {
                let pp = &self.people[i];
                let sig: f32 = (0..3).map(|z| (pp.genome.mating_sig[z] - q.genome.mating_sig[z]).abs()).sum::<f32>();
                u += mq.drives[drv::REPRO] * 0.6 - sig * 0.5 + aff * 0.3 + 0.2;
                if q.mate.is_some() {
                    u -= 2.0;
                }
                why_v = [(why::MATING, mq.drives[drv::REPRO]), (why::TRUST, aff), (NONE, 0.0)];
            }
            Req::Migrate => {
                u += aff * 0.4 + if k == kin::MATE || k == kin::PARENT { 0.6 } else { 0.0 } + mq.traits[0] * 0.15 - 0.1;
                why_v = [(why::TRUST, aff), (why::KIN, if k != kin::NONE { 0.6 } else { 0.0 }), (why::RESPECT, respect)];
            }
        }
        let u = u + self.people[j].rng.normal() * self.cfg.mind.noise;
        let ok = u > 0.0;
        let cap = self.cfg.mind.model_cap;
        {
            let (pi, pj) = pair_mut(&mut self.people, i, j);
            let mi = pi.mind.model_mut(j as u32, tick, cap);
            if ok {
                mi.accepts += 1;
                mi.trust = (mi.trust + 0.03).min(1.0);
            } else {
                mi.refusals += 1;
                mi.trust = (mi.trust - 0.02).max(0.0);
            }
            pj.mind.model_mut(iid, tick, cap).last_seen = tick;
        }
        match req {
            Req::Food if ok => {
                let (pi, pj) = pair_mut(&mut self.people, i, j);
                let give = (pj.carried + pj.stash).min(2.0);
                let from_c = give.min(pj.carried);
                pj.carried -= from_c;
                pj.stash -= give - from_c;
                pi.carried += give;
                pj.mind.model_mut(iid, tick, cap).ledger += give;
                pi.mind.model_mut(j as u32, tick, cap).ledger -= give;
                pi.mind.model_mut(j as u32, tick, cap).affinity += 0.08;
                pi.mind.feel(emo::TRUST, 0.3);
                if let Some(b) = pi.mind.beliefs.iter_mut().find(|b| b.subject == Concept::Person(j as u32) && b.rel == Rel::Thief) {
                    b.conf -= 0.15;
                }
                self.counters.asks_granted += 1;
                self.episode(i, EpKind::WasShared, Some(j as u32), 0.6, 0.35);
            }
            Req::Food => {
                self.people[i].mind.model_mut(j as u32, tick, cap).affinity -= 0.05;
                self.episode(i, EpKind::WasRefused, Some(j as u32), -0.4, 0.3);
            }
            _ => {}
        }
        if req == Req::Food {
            self.counters.asks += 1;
        }
        // j's side of the thought log
        let act = match req {
            Req::Food => Act::Share { who: iid },
            Req::Hunt => Act::Hunt { task: 0 },
            Req::Court => Act::Court { who: iid },
            Req::Migrate => Act::Follow { who: iid },
        };
        let mut cd = c(act, u, &why_v);
        cd.why = why_v;
        self.record(j, &cd, false, if ok { result::ACCEPTED } else { result::REFUSED });
        ok
    }

    fn bond(&mut self, i: usize, j: usize) {
        let tick = self.tick;
        let cap = self.cfg.mind.model_cap;
        let (pi, pj) = pair_mut(&mut self.people, i, j);
        pi.mate = Some(j as u32);
        pj.mate = Some(i as u32);
        let a = pi.mind.model_mut(j as u32, tick, cap);
        a.kin = kin::MATE;
        a.affinity = (a.affinity + 0.4).min(1.0);
        let b = pj.mind.model_mut(i as u32, tick, cap);
        b.kin = kin::MATE;
        b.affinity = (b.affinity + 0.4).min(1.0);
        if pi.home != pj.home {
            if pj.home.is_none() {
                pj.home = pi.home;
            } else if pi.home.is_none() {
                pi.home = pj.home;
            } else if pi.female {
                pj.home = pi.home; // matrilocal by default; arbitrary but fixed
            } else {
                pi.home = pj.home;
            }
        }
        let (ni, nj) = (pi.display_name(), pj.display_name());
        pi.life_note(tick, format!("bonded with {nj}"));
        pj.life_note(tick, format!("bonded with {ni}"));
        self.counters.bonds += 1;
        self.episode(i, EpKind::Bonded, Some(j as u32), 0.8, 0.5);
        self.episode(j, EpKind::Bonded, Some(i as u32), 0.8, 0.5);
        let pl = self.place_of_person(i);
        self.emit(EvKind::PairBond, vec![i as u32, j as u32], pl, format!("{ni} and {nj} bond"));
    }

    fn propose_hunt(&mut self, i: usize, herd: u32) -> u8 {
        let tick = self.tick;
        let (x, y) = self.people[i].pos(tick);
        let mut cands: Vec<(u32, f32)> = self
            .near(x, y, 30.0)
            .into_iter()
            .filter(|&o| o != i as u32 && self.adult(o as usize))
            .map(|o| {
                let m = self.people[i].mind.model(o);
                (o, m.map_or(0.0, |mm| mm.trust + mm.affinity * 0.5 + mm.seen[role::HUNTER] * 0.05))
            })
            .collect();
        cands.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
        self.counters.proposals += 1;
        let mut members = vec![i as u32];
        for (o, _) in cands.into_iter().take(5) {
            self.counters.proposal_asks += 1;
            if self.respond(o as usize, i, Req::Hunt) {
                members.push(o);
                self.counters.proposal_accepts += 1;
            }
        }
        if members.len() < 2 {
            return result::REFUSED;
        }
        let Some(h) = self.world.herds.iter().find(|h| h.id == herd) else { return result::REFUSED };
        let (hx, hy) = (h.x, h.y);
        let id = self.next_task;
        self.next_task += 1;
        let d = self.world.travel_dist(x, y, hx, hy);
        let end = tick + (d / SPEED) as u64 + 140;
        for &mbr in &members {
            let mi = mbr as usize;
            let from = self.people[mi].pos(tick);
            let p = &mut self.people[mi];
            p.from = from;
            p.to = (hx + p.rng.range(-4.0, 4.0), hy + p.rng.range(-4.0, 4.0));
            p.act = Act::Hunt { task: id };
            p.act_start = tick;
            p.next_decision = end;
            p.activity[10] += 1.0;
            p.activity_total[10] += 1;
        }
        let place = place_of(hx, hy);
        self.tasks.push(Task { id, leader: i as u32, members, herd, end, place });
        result::ACCEPTED
    }

    pub fn resolve_tasks(&mut self) {
        let tick = self.tick;
        let due: Vec<Task> = self.tasks.iter().filter(|t| t.end <= tick && t.end + self.cfg.time.step_ticks > tick).cloned().collect();
        for t in due {
            let present: Vec<u32> = t.members.iter().copied().filter(|&m| self.people[m as usize].alive && self.people[m as usize].act == Act::Hunt { task: t.id }).collect();
            let deserters: Vec<u32> = t.members.iter().copied().filter(|m| !present.contains(m) && self.people[*m as usize].alive).collect();
            if present.is_empty() {
                continue;
            }
            let nn = present.len() as f32;
            let spear = present.iter().filter(|&&m| self.people[m as usize].knows(n::SPEAR)).count() as f32 / nn;
            let lure = present.iter().any(|&m| self.people[m as usize].knows(n::LURE));
            let skill: f32 = present.iter().map(|&m| self.people[m as usize].skill_hunt).sum::<f32>() / nn;
            let herd_ok = self.world.herds.iter().any(|h| h.id == t.herd && h.count >= 1.0);
            let p_succ = if herd_ok { 1.0 - libm::expf(-0.22 * nn * (1.0 + 0.8 * spear + if lure { 0.4 } else { 0.0 }) * (0.6 + skill)) } else { 0.0 };
            let success = self.rng.chance(p_succ);
            let leader = t.leader;
            let lname = self.name(leader);
            let cap = self.cfg.mind.model_cap;
            if success {
                if let Some(h) = self.world.herds.iter_mut().find(|h| h.id == t.herd) {
                    h.count -= 1.0;
                }
                let food = 14.0 + 3.0 * nn;
                let lp = &self.people[leader as usize];
                let greed = if present.contains(&leader) { ((lp.genome.dominance + lp.mind.traits[3] - lp.genome.cooperation) * 0.5).clamp(0.0, 0.8) } else { 0.0 };
                let fair = food / nn;
                let lead_share = if present.contains(&leader) { (fair * (1.0 + greed)).min(food) } else { 0.0 };
                let rest = (food - lead_share) / ((nn - if present.contains(&leader) { 1.0 } else { 0.0 }).max(1.0));
                for &m in &present {
                    let mi = m as usize;
                    let share = if m == leader { lead_share } else { rest };
                    self.people[mi].carried += share;
                    self.people[mi].skill_hunt = (self.people[mi].skill_hunt + 0.04).min(1.0);
                    self.use_know(mi, n::SPEAR);
                    self.use_know(mi, n::LURE);
                    self.people[mi].needs[need::HUNT_FAIL] *= 0.5;
                    if m != leader {
                        let unfair = share < fair * 0.8;
                        let (r, followed) = {
                            let pm = &mut self.people[mi];
                            let md = pm.mind.model_mut(leader, tick, cap);
                            md.followed += 1;
                            md.seen[role::HUNTER] += 1.0;
                            if unfair {
                                md.trust -= 0.12;
                                md.grudge = (md.grudge + 0.25).min(1.0);
                            } else {
                                md.trust = (md.trust + 0.06).min(1.0);
                                md.respect = (md.respect + 0.07).min(1.0);
                                md.affinity += 0.04;
                            }
                            let out = (md.respect, md.followed);
                            if unfair {
                                pm.mind.feel(emo::ANGER, 0.45);
                            }
                            out
                        };
                        if unfair {
                            self.episode(mi, EpKind::UnfairShare, Some(leader), -0.5, 0.5);
                        } else {
                            self.episode(mi, EpKind::HuntWon, Some(leader), 0.6, 0.35);
                            if r > 0.7 && followed >= 3 {
                                let cb = self.cfg.mind.belief_cap;
                                self.people[mi].mind.believe(Concept::Person(leader), Rel::Leads, 0.15, None, m, tick, false, cb);
                            }
                        }
                    } else {
                        self.episode(mi, EpKind::HuntWon, None, 0.7, 0.4);
                        self.people[mi].mind.drives[drv::STATUS] *= 0.5;
                    }
                    for &o in &present {
                        if o != m {
                            let pm = &mut self.people[mi];
                            let md = pm.mind.model_mut(o, tick, cap);
                            md.affinity += 0.03;
                            md.trust = (md.trust + 0.02).min(1.0);
                        }
                    }
                }
                self.counters.hunts_won += 1;
                if greed > 0.4 {
                    self.counters.unfair_shares += 1;
                }
                self.emit(EvKind::HuntWon, present.clone(), t.place, format!("{lname} leads {} in bringing down a drifter", present.len() - 1));
            } else {
                for &m in &present {
                    let mi = m as usize;
                    self.people[mi].needs[need::HUNT_FAIL] += 1.0;
                    self.people[mi].skill_hunt = (self.people[mi].skill_hunt + 0.01).min(1.0);
                    if m != leader {
                        let md = self.people[mi].mind.model_mut(leader, tick, cap);
                        md.respect = (md.respect - 0.02).max(0.0);
                    }
                    self.episode(mi, EpKind::HuntLost, None, -0.3, 0.25);
                }
                self.counters.hunts_lost += 1;
            }
            // deserters are remembered
            for &dsr in &deserters {
                for &m in &present {
                    let md = self.people[m as usize].mind.model_mut(dsr, tick, cap);
                    md.trust -= 0.1;
                    md.grudge = (md.grudge + 0.1).min(1.0);
                }
                if present.contains(&leader) {
                    self.episode(leader as usize, EpKind::Abandoned, Some(dsr), -0.4, 0.4);
                }
                self.counters.deserted += 1;
            }
        }
    }

    fn join_camp(&mut self, i: usize, camp: u16) -> bool {
        if self.world.camps[camp as usize].abandoned.is_some() && self.world.camps[camp as usize].members > 0 {
            return false;
        }
        // a member decides whether to accept the newcomer
        let member = self.alive.iter().copied().find(|&o| o != i as u32 && self.people[o as usize].home == Some(camp));
        let accepted = match member {
            None => true,
            Some(o) => {
                let q = &self.people[o as usize];
                let m = q.mind.model(i as u32);
                let u = m.map_or(0.0, |mm| mm.trust + mm.affinity) + q.mind.traits[1] * 0.4 - q.mind.traits[5] * 0.3 * self.people[i].stranger as u8 as f32 - (q.mind.belief(Concept::Person(i as u32), Rel::Thief)) + 0.1;
                u + self.people[o as usize].rng.normal() * 0.1 > 0.0
            }
        };
        if accepted {
            let old = self.people[i].home;
            self.people[i].home = Some(camp);
            let tick = self.tick;
            let cname = self.camp_name(camp);
            self.people[i].life_note(tick, format!("joined the camp of {cname}"));
            if old != Some(camp) {
                let place = self.world.camps[camp as usize].place;
                let name = self.name(i as u32);
                self.emit(EvKind::Joined, vec![i as u32], place, format!("{name} joins the camp of {cname}"));
                self.episode(i, EpKind::Migrated, None, 0.3, 0.4);
            }
        }
        accepted
    }

    fn found_camp(&mut self, i: usize, place: u8) -> u8 {
        let tick = self.tick;
        let (x, y) = self.people[i].pos(tick);
        let home = self.people[i].home;
        // recruit followers: mate, children, friends from the same camp
        let mut followers = Vec::new();
        if let Some(h) = home {
            let mut cands: Vec<(u32, f32)> = self
                .people[i]
                .mind
                .models
                .iter()
                .filter(|m| self.people[m.who as usize].alive && self.people[m.who as usize].home == Some(h) && (m.affinity > 0.3 || m.kin != kin::NONE))
                .map(|m| (m.who, m.affinity + if m.kin != kin::NONE { 1.0 } else { 0.0 }))
                .collect();
            cands.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
            for (o, _) in cands.into_iter().take(12) {
                let oi = o as usize;
                if !self.adult(oi) {
                    if self.people[oi].mother == Some(i as u32) || self.people[oi].father == Some(i as u32) {
                        followers.push(o);
                    }
                    continue;
                }
                if self.respond(oi, i, Req::Migrate) {
                    followers.push(o);
                }
            }
            if followers.iter().filter(|&&f| self.adult(f as usize)).count() < 2 && self.people[i].mind.traits[0] < 0.75 {
                self.start(i, Act::Idle, (x, y), 32);
                return result::REFUSED;
            }
        }
        let culture = self.people[i].culture;
        let cid = self.new_camp(place, i as u32, culture);
        let cname = self.camp_name(cid);
        let cform = self.world.camps[cid as usize].name.clone();
        let (cx, cy) = (self.world.camps[cid as usize].x, self.world.camps[cid as usize].y);
        let mut all = vec![i as u32];
        all.extend(followers.iter().copied());
        for &m in &all {
            let mi = m as usize;
            self.people[mi].home = Some(cid);
            self.people[mi].mind.lex.adopt(Concept::Camp(cid), cform.clone(), 0.7);
            let to = (cx + self.people[mi].rng.range(-5.0, 5.0), cy + self.people[mi].rng.range(-5.0, 5.0));
            let act = if m == i as u32 { Act::Found { place } } else { Act::Migrate { camp: cid } };
            self.start(mi, act, to, 32);
            self.episode(mi, EpKind::Migrated, Some(i as u32), 0.3, 0.5);
            if m != i as u32 {
                let md = self.people[mi].mind.model_mut(i as u32, tick, self.cfg.mind.model_cap);
                md.followed += 1;
                md.respect = (md.respect + 0.1).min(1.0);
            }
        }
        self.people[i].founded_camps += 1;
        self.people[i].life_note(tick, format!("founded the camp of {cname} with {} others", followers.len()));
        let from = home.map(|h| self.camp_name(h)).unwrap_or_else(|| "nowhere".into());
        let name = self.name(i as u32);
        self.counters.camps_founded += 1;
        self.emit(EvKind::CampFounded, all, place, format!("{name} leads {} away from {from} to found {cname} at the {}", followers.len(), self.place_name(place)));
        result::ACCEPTED
    }

    // ================================================================ completing

    pub fn complete(&mut self, i: usize) {
        let act = self.people[i].act;
        let tick = self.tick;
        self.update_energy(i);
        let (x, y) = self.people[i].to;
        let place = place_of(x, y);
        let cap = self.cfg.mind.model_cap;
        let bcap = self.cfg.mind.belief_cap;
        // visiting a place updates the cognitive map with what is actually there
        if !matches!(act, Act::Sleep | Act::Rest | Act::Idle | Act::Follow { .. }) {
            let food = (self.world.place_food(place) / 160.0).min(1.0);
            let pm = &mut self.people[i].mind.places[place as usize];
            pm.food = pm.food * 0.4 + food * 0.6;
            pm.conf = (pm.conf + 0.3).min(1.0);
            pm.last_visit = tick;
        }
        match act {
            Act::Eat => {
                let at_home = self.at_home(i);
                let value = if self.people[i].knows(n::VENT_COOK) { 0.31 } else { 0.25 };
                let p = &mut self.people[i];
                let want = ((1.0 - p.energy) / value).max(0.0);
                let a = want.min(p.carried);
                p.carried -= a;
                let mut b = 0.0;
                if at_home {
                    b = (want - a).min(p.stash);
                    p.stash -= b;
                }
                p.energy = (p.energy + (a + b) * value).min(1.0);
                if value > 0.25 {
                    self.use_know(i, n::VENT_COOK);
                }
            }
            Act::Forage { .. } => {
                let night = self.is_night(tick);
                let cap_c = self.carry_cap(i);
                let p = &self.people[i];
                let mut eff = 2.4 * (0.7 + p.skill_forage * 0.6) * if night { 0.6 } else { 1.0 };
                if p.knows(n::NET) {
                    eff *= 1.4;
                }
                let scraper = p.knows(n::SCRAPER);
                let tends = p.home.map_or(false, |h| self.world.camps[h as usize].place == place);
                let garden = tends && p.knows(n::GARDEN);
                let bed = tends && p.knows(n::SHELL_BED);
                let cells = self.world.best_cells(place, 3);
                let mut got = 0.0;
                for ci in cells {
                    let cell = &mut self.world.cells[ci];
                    // thin patches are hard to gather: yield falls with density, leaving refuges
                    let da = if cell.algae_cap > 0.0 { (5.0 * cell.algae / cell.algae_cap).min(1.0) } else { 0.0 };
                    let ds = if cell.shell_cap > 0.0 { (5.0 * cell.shell / cell.shell_cap).min(1.0) } else { 0.0 };
                    let take_a = (eff * 0.5 * da).min(cell.algae);
                    cell.algae -= take_a;
                    let take_s = (eff * 0.25 * ds * if scraper { 1.6 } else { 1.0 }).min(cell.shell);
                    cell.shell -= take_s;
                    got += take_a + take_s * 1.5;
                    if garden {
                        cell.garden = (cell.garden + 0.05).min(1.0);
                    }
                    if bed && cell.hab == Habitat::Reef {
                        cell.bed = true;
                    }
                }
                for node in [n::NET, n::SCRAPER, n::BASKET] {
                    self.use_know(i, node);
                }
                if got > 1.0 {
                    let pm = self.world.places[place as usize].mats;
                    self.disconfirm_curse(i, pm);
                }
                if night {
                    self.use_know(i, n::GLOWLAMP);
                }
                if garden {
                    self.use_know(i, n::GARDEN);
                }
                if bed {
                    self.use_know(i, n::SHELL_BED);
                }
                let p = &mut self.people[i];
                let room = (cap_c - p.carried).max(0.0);
                if got > room {
                    p.needs[need::CARRY] += 0.5;
                }
                p.carried += got.min(room);
                p.skill_forage = (p.skill_forage + 0.005).min(1.0);
                if got < 1.0 {
                    p.needs[need::HUNGER] += 0.2;
                }
                let pm = &mut p.mind.places[place as usize];
                pm.food = pm.food * 0.5 + (got / 6.0).min(1.0) * 0.5;
            }
            Act::Store => {
                if self.at_home(i) {
                    let p = &mut self.people[i];
                    p.stash += p.carried;
                    p.carried = 0.0;
                }
            }
            Act::Rest => {
                let p = &mut self.people[i];
                p.health = (p.health + 0.05).min(1.0);
                p.mind.drives[drv::REST] *= 0.5;
            }
            Act::Sleep => {
                self.people[i].mind.drives[drv::REST] = 0.0;
            }
            Act::Socialize { who } => {
                let j = who as usize;
                if self.people[j].alive && self.close(i, j) {
                    self.converse(i, j);
                }
            }
            Act::Share { who } => {
                let j = who as usize;
                if self.people[j].alive && self.close(i, j) {
                    let (pi, pj) = pair_mut(&mut self.people, i, j);
                    let give = (pi.carried + pi.stash).min(2.0);
                    let from_c = give.min(pi.carried);
                    pi.carried -= from_c;
                    pi.stash -= give - from_c;
                    pj.carried += give;
                    pi.mind.model_mut(who, tick, cap).ledger += give;
                    let mj = pj.mind.model_mut(i as u32, tick, cap);
                    mj.ledger -= give;
                    mj.affinity += 0.1;
                    mj.trust = (mj.trust + 0.06).min(1.0);
                    pj.mind.feel(emo::JOY, 0.3);
                    pj.mind.feel(emo::TRUST, 0.3);
                    self.counters.shares += 1;
                    self.use_know(i, n::SHARING);
                    self.episode(j, EpKind::WasShared, Some(i as u32), 0.6, 0.35);
                    self.episode(i, EpKind::Helped, Some(who), 0.4, 0.2);
                }
            }
            Act::Teach { who, node } => {
                let j = who as usize;
                if self.people[j].alive && self.close(i, j) {
                    self.teach(i, j, node);
                }
            }
            Act::Heal { who } => {
                let j = who as usize;
                if self.people[j].alive && self.close(i, j) && self.people[j].infected.is_some() {
                    self.people[j].healed_today = true;
                    let mj = self.people[j].mind.model_mut(i as u32, tick, cap);
                    mj.trust = (mj.trust + 0.12).min(1.0);
                    mj.affinity += 0.1;
                    mj.seen[role::HEALER] += 1.0;
                    let seen = mj.seen[role::HEALER];
                    if seen >= 2.0 {
                        self.people[j].mind.believe(Concept::Person(i as u32), Rel::Heals, 0.25, None, j as u32, tick, false, bcap);
                    }
                    self.episode(j, EpKind::Helped, Some(i as u32), 0.6, 0.4);
                    self.observe_role(i, role::HEALER, Some(Rel::Heals));
                    self.use_know(i, n::REMEDY);
                    self.counters.heals += 1;
                }
            }
            Act::Steal { who } => {
                let j = who as usize;
                if self.people[j].alive {
                    self.steal(i, j);
                }
            }
            Act::Punish { who } => {
                let j = who as usize;
                if self.people[j].alive && self.close(i, j) {
                    self.punish(i, j);
                }
            }
            Act::Defend { pred } => {
                self.defend(i, pred);
            }
            Act::Experiment => {
                self.experiment(i, 1.0);
            }
            Act::Play => {
                self.people[i].skill_forage = (self.people[i].skill_forage + 0.01).min(1.0);
                self.people[i].mind.drives[drv::CURIOSITY] *= 0.6;
                self.experiment(i, 0.3);
            }
            Act::Build { node } => {
                self.build(i, node);
            }
            Act::Explore { place: pl } => {
                let p = &mut self.people[i];
                p.mind.drives[drv::CURIOSITY] *= 0.5;
                p.mind.places[pl as usize].conf = 1.0;
            }
            Act::Mourn => {
                self.use_know(i, n::BURIAL);
                self.use_know(i, n::ANCESTOR);
                let knows_burial = self.people[i].knows(n::BURIAL);
                let rites = self.people[i].knows(n::ANCESTOR);
                let p = &mut self.people[i];
                p.mind.emo[emo::GRIEF] *= if knows_burial { 0.4 } else { 0.7 };
                p.needs[need::GRIEF] *= 0.7;
                if rites {
                    // mourning together binds the camp
                    let home = p.home;
                    let near: Vec<u32> = self.near(x, y, 20.0).into_iter().filter(|&o| o != i as u32 && self.people[o as usize].home == home).collect();
                    for o in near {
                        let md = self.people[i].mind.model_mut(o, tick, cap);
                        md.affinity += 0.05;
                    }
                }
            }
            Act::Flee => {
                self.people[i].mind.drives[drv::SAFETY] *= 0.7;
            }
            Act::Hunt { .. } | Act::AskHelp { .. } | Act::Court { .. } | Act::Migrate { .. } | Act::Found { .. } | Act::Idle | Act::Follow { .. } => {}
        }
    }

    fn close(&self, i: usize, j: usize) -> bool {
        let (a, b) = (self.people[i].pos(self.tick), self.people[j].pos(self.tick));
        dist(a.0, a.1, b.0, b.1) < 40.0
    }

    // ================================================================ language & gossip

    /// One naming-game round about `topic`, speaker `s`, hearer `h`. Returns understood.
    pub fn talk(&mut self, s: usize, h: usize, topic: Concept) -> bool {
        let lc = self.cfg.language.clone();
        let tpc = self.tpc();
        let h_child = self.people[h].age_cycles(self.tick, tpc) < self.cfg.people.adult_cycles;
        let (sp, hp) = pair_mut(&mut self.people, s, h);
        let form = match sp.mind.lex.best(&topic) {
            Some(f) => f.clone(),
            None => {
                // lacking a word, the speaker points and the hearer supplies theirs
                if let Some(hf) = hp.mind.lex.best(&topic).cloned() {
                    let learn = lc.adopt_score * (0.5 + sp.genome.learning);
                    sp.mind.lex.adopt(topic, hf, learn.min(0.9));
                    sp.mind.comm_try += 1;
                    hp.mind.comm_try += 1;
                    sp.mind.comm_ok += 1;
                    hp.mind.comm_ok += 1;
                    return true;
                }
                let w = if topic == Concept::Person(sp.id) { sp.name.clone() } else { sp.phon.coin(&mut sp.rng) };
                sp.mind.lex.adopt(topic, w.clone(), 0.5);
                w
            }
        };
        let mut form = form;
        let entrenched = sp.mind.lex.entries.get(&topic).map_or(0.0, |v| v.iter().map(|x| x.1).fold(0.0, f32::max));
        if !matches!(topic, Concept::Person(_) | Concept::Camp(_)) && sp.rng.chance(lc.synonym_rate * (1.15 - entrenched).max(0.05)) {
            let mut fresh = if sp.rng.chance(0.5) && form.len() >= 3 {
                // clipped compound: keep the start of the old word, add a fresh syllable
                let mut f = form.clone();
                f.truncate(3);
                let tail = sp.phon.coin(&mut sp.rng);
                f.extend(tail.iter().take(2));
                f
            } else {
                sp.phon.coin(&mut sp.rng)
            };
            fresh.truncate(7);
            let s0 = sp.mind.lex.entries.get(&topic).map_or(0.5, |v| v.iter().map(|x| x.1).fold(0.0, f32::max));
            sp.mind.lex.adopt(topic, fresh.clone(), (s0 + 0.05).min(1.0));
            form = fresh;
        }
        if let Some(law) = sp.innovation {
            if sp.rng.chance(lc.innovation_use) {
                let f2 = law.apply(&form);
                if f2 != form {
                    let s0 = sp.mind.lex.entries.get(&topic).map_or(0.5, |v| v.iter().map(|x| x.1).fold(0.0, f32::max));
                    sp.mind.lex.adopt(topic, f2.clone(), (s0 + 0.05).min(1.0));
                    form = f2;
                }
            }
        }
        sp.mind.comm_try += 1;
        hp.mind.comm_try += 1;
        if hp.mind.lex.has_form(&topic, &form) {
            hp.mind.lex.reinforce(topic, &form, lc.reinforce);
            sp.mind.lex.reinforce(topic, &form, lc.reinforce);
            sp.mind.comm_ok += 1;
            hp.mind.comm_ok += 1;
            true
        } else {
            let close = hp.mind.lex.best(&topic).map_or(false, |b| edit_distance(b, &form) <= 1);
            let learn = lc.adopt_score * if h_child { 1.6 } else { 1.0 } * (0.5 + hp.genome.learning);
            let mut heard = form;
            if h_child && hp.rng.chance(0.02) && heard.len() >= 2 {
                // a child mishears one sound, swapping it for a similar one it can make
                let k = hp.rng.below(heard.len() as u32) as usize;
                let old = heard[k];
                let art = crate::language::PHONEMES[old as usize].2;
                let pool: Vec<u8> = if crate::language::is_vowel(old) { hp.phon.vowels.clone() } else { hp.phon.consonants.iter().copied().filter(|c| crate::language::PHONEMES[*c as usize].2 == art).collect() };
                if let Some(&nw) = hp.rng.pick(&pool) {
                    heard[k] = nw;
                }
            }
            hp.mind.lex.adopt(topic, heard, learn.min(0.9));
            // occasionally the hearer picks up the speaker's sound change itself
            if hp.innovation.is_none() && sp.innovation.is_some() && hp.rng.chance(0.05) {
                hp.innovation = sp.innovation;
            }
            if close {
                sp.mind.comm_ok += 1;
                hp.mind.comm_ok += 1;
            }
            close
        }
    }

    fn converse(&mut self, s: usize, h: usize) {
        let tick = self.tick;
        let cap = self.cfg.mind.model_cap;
        let tpc = self.tpc();
        let sid = s as u32;
        let hid = h as u32;
        // meeting: learn each other's names
        let knew = self.people[h].mind.model(sid).is_some();
        if !knew {
            let sname = self.people[s].name.clone();
            let _ = sname;
            self.talk(s, h, Concept::Person(sid));
            self.talk(h, s, Concept::Person(hid));
            let stranger = self.people[s].culture != self.people[h].culture || self.people[s].stranger;
            let conform = self.people[h].mind.traits[5];
            let md = self.people[h].mind.model_mut(sid, tick, cap);
            md.trust = if stranger { 0.25 - conform * 0.15 } else { 0.35 };
            if stranger {
                self.episode(h, EpKind::MetStranger, Some(sid), 0.0, 0.5);
            }
            let md2 = self.people[s].mind.model_mut(hid, tick, cap);
            md2.trust = md2.trust.max(0.25);
        }
        // children learn many words at once
        let h_child = self.people[h].age_cycles(tick, tpc) < self.cfg.people.adult_cycles;
        let rounds = if h_child { 4 } else { 1 };
        for _ in 0..rounds {
            let topic = {
                let sp = &mut self.people[s];
                let n = sp.mind.lex.entries.len();
                if n == 0 {
                    Concept::Kind(Kind::Food)
                } else {
                    let k = sp.rng.below(n as u32) as usize;
                    *sp.mind.lex.entries.keys().nth(k).unwrap()
                }
            };
            self.talk(s, h, topic);
        }
        self.gossip(s, h);
        // relationship
        let soc = self.people[s].mind.traits[1];
        for (a, b) in [(s, h), (h, s)] {
            let pa = &mut self.people[a];
            let md = pa.mind.model_mut(b as u32, tick, cap);
            md.affinity = (md.affinity + 0.04 + soc * 0.03).min(1.0);
            md.trust = (md.trust + 0.01).min(1.0);
            md.last_seen = tick;
            pa.mind.drives[drv::BELONGING] *= 0.5;
        }
        self.counters.conversations += 1;
    }

    fn gossip(&mut self, s: usize, h: usize) {
        let tick = self.tick;
        let bcap = self.cfg.mind.belief_cap;
        let distortion = self.cfg.mind.distortion;
        let sid = s as u32;
        // what's on the speaker's mind?
        #[derive(Clone, Copy)]
        enum Topic {
            Danger(u8, f32, bool),
            Food(u8, f32),
            Rep(u32, Rel, f32, u32),
            Rule(u8, u8, f32),
            Belief(Concept, Rel, f32, u32),
            Skill(u8),
        }
        let sp = &self.people[s];
        let mut opts: Vec<(Topic, f32)> = Vec::new();
        let (mut dmax, mut dp) = (0.0, 0u8);
        let (mut fmax, mut fp) = (0.0, 0u8);
        for (k, pm) in sp.mind.places.iter().enumerate() {
            if pm.danger > dmax && pm.conf > 0.1 {
                dmax = pm.danger;
                dp = k as u8;
            }
            if pm.food * pm.conf > fmax {
                fmax = pm.food * pm.conf;
                fp = k as u8;
            }
        }
        if dmax > 0.4 {
            opts.push((Topic::Danger(dp, dmax, sp.mind.places[dp as usize].danger_distorted), dmax));
        }
        opts.push((Topic::Food(fp, sp.mind.places[fp as usize].food), fmax * 0.5));
        for b in &sp.mind.beliefs {
            if b.conf < 0.3 {
                continue;
            }
            match (b.subject, b.rel) {
                (Concept::Person(z), r) if z != h as u32 => opts.push((Topic::Rep(z, r, b.conf, b.origin), b.conf * 0.8)),
                (subj, r) if !matches!(subj, Concept::Person(_)) => opts.push((Topic::Belief(subj, r, b.conf, b.origin), b.conf * 0.7)),
                _ => {}
            }
        }
        if let Some(&(c0, o0, dpv)) = sp.mind.cont.rules(self.cfg.mind.rule_min_dp, self.cfg.mind.rule_min_n).first() {
            opts.push((Topic::Rule(c0 as u8, o0 as u8, dpv), dpv * 0.6));
        }
        if let Some(k) = sp.knowledge.iter().find(|k| k.prof > 0.5 && !self.people[h].knows(k.node)) {
            opts.push((Topic::Skill(k.node), 0.15));
        }
        let weights: Vec<f32> = opts.iter().map(|o| o.1).collect();
        let Some(pick) = self.people[s].rng.weighted(&weights) else { return };
        let topic = opts[pick].0;
        let (subj, pred) = match topic {
            Topic::Danger(p, _, _) => (Concept::Place(p), Concept::Kind(Kind::Danger)),
            Topic::Food(p, _) => (Concept::Place(p), Concept::Kind(Kind::Food)),
            Topic::Rep(z, r, _, _) => (
                Concept::Person(z),
                Concept::Kind(match r {
                    Rel::Thief => Kind::Steal,
                    Rel::Heals => Kind::Help,
                    Rel::Leads => Kind::Elder,
                    Rel::Teaches => Kind::Teach,
                    _ => Kind::Bad,
                }),
            ),
            Topic::Rule(_, o, _) => (
                Concept::Kind(Kind::Dim),
                Concept::Kind(match o as usize {
                    outcome::HUNGER => Kind::Hunger,
                    outcome::ATTACK => Kind::Gulper,
                    outcome::SICKNESS => Kind::Sickness,
                    outcome::COLD_STRESS => Kind::Cold,
                    outcome::DEATH => Kind::Death,
                    _ => Kind::Plenty,
                }),
            ),
            Topic::Belief(subj, r, _, _) => (subj, Concept::Kind(if r == Rel::Forbidden { Kind::Forbidden } else { Kind::Good })),
            Topic::Skill(node) => (Concept::Node(node), Concept::Kind(Kind::Make)),
        };
        let ok1 = self.talk(s, h, subj);
        let ok2 = self.talk(s, h, pred);
        if !(ok1 && ok2) {
            return;
        }
        let cap = self.cfg.mind.model_cap;
        let trust = self.people[h].mind.model(sid).map_or(0.2, |m| m.trust);
        let conform = self.people[h].mind.traits[5];
        let w = (trust * (0.5 + conform * 0.5)).clamp(0.0, 1.0);
        let distort = self.people[h].rng.chance(distortion);
        match topic {
            Topic::Danger(p, d, was_distorted) => {
                let mut target = p;
                if distort {
                    let (px, py) = ((p as usize % PW) as i32, (p as usize / PW) as i32);
                    let nx = (px + if self.people[h].rng.chance(0.5) { 1 } else { -1 }).clamp(0, PW as i32 - 1);
                    target = (py * PW as i32 + nx) as u8;
                }
                let pm = &mut self.people[h].mind.places[target as usize];
                pm.danger = (pm.danger + (d - pm.danger) * w * 0.6).clamp(0.0, 1.0);
                pm.danger_src = sid + 1;
                pm.danger_distorted = distort || was_distorted;
            }
            Topic::Food(p, f) => {
                let pm = &mut self.people[h].mind.places[p as usize];
                pm.food = pm.food + (f - pm.food) * w * 0.5;
                pm.conf = (pm.conf + 0.1 * w).min(1.0);
            }
            Topic::Rep(z, r, conf, origin) => {
                let mut subject = z;
                if distort {
                    let others: Vec<u32> = self.people[h].mind.models.iter().map(|m| m.who).filter(|&o| o != z && o != sid).collect();
                    if let Some(&o) = self.people[h].rng.pick(&others) {
                        subject = o;
                    }
                }
                let cur = self.people[h].mind.belief(Concept::Person(subject), r);
                let target = (conf * 0.85).min(cur + conf * w * 0.4);
                let newly = target > cur && self.people[h].mind.believe(Concept::Person(subject), r, target - cur, Some(sid), origin, tick, distort, bcap);
                let md = self.people[h].mind.model_mut(subject, tick, cap);
                match r {
                    Rel::Thief => md.trust -= 0.1 * w,
                    Rel::Heals | Rel::Teaches => md.trust = (md.trust + 0.05 * w).min(1.0),
                    Rel::Leads => md.respect = (md.respect + 0.1 * w).min(1.0),
                    _ => {}
                }
                if newly && distort {
                    self.counters.distortions += 1;
                }
            }
            Topic::Rule(c0, o0, dpv) => {
                let told = &mut self.people[h].mind.cont.told;
                if told.len() < 8 {
                    told.push((c0, o0, 4.0 * w * dpv.max(0.3), sid));
                }
            }
            Topic::Belief(subj, r, conf, origin) => {
                let cur = self.people[h].mind.belief(subj, r);
                let target = (conf * 0.85).min(cur + conf * w * 0.4);
                if target > cur {
                    self.people[h].mind.believe(subj, r, target - cur, Some(sid), origin, tick, false, bcap);
                }
            }
            Topic::Skill(node) => {
                if self.people[h].rng.chance(0.15 * self.cfg.discovery.teach_fidelity) {
                    let mat_ = self.people[s].know(node).map_or(0, |k| k.material);
                    self.learn(h, node, mat_, KnowSource::Taught(sid));
                }
            }
        }
    }

    fn teach(&mut self, s: usize, h: usize, node: u8) {
        let tick = self.tick;
        let cap = self.cfg.mind.model_cap;
        let ok = self.talk(s, h, Concept::Node(node));
        let parent = self.people[h].mother == Some(s as u32) || self.people[h].father == Some(s as u32);
        let fid = self.cfg.discovery.teach_fidelity * (0.5 + self.people[h].genome.learning) * if ok { 1.0 } else { 0.4 } * if parent { 1.3 } else { 1.0 };
        if self.people[h].rng.chance(fid) {
            let mat_ = self.people[s].know(node).map_or(0, |k| k.material);
            if self.learn(h, node, mat_, KnowSource::Taught(s as u32)) {
                self.episode(h, EpKind::Learned, Some(s as u32), 0.4, 0.3);
                self.counters.taught += 1;
            }
        }
        let md = self.people[h].mind.model_mut(s as u32, tick, cap);
        md.respect = (md.respect + 0.04).min(1.0);
        md.seen[role::TEACHER] += 1.0;
        self.people[s].mind.drives[drv::STATUS] *= 0.7;
        self.observe_role(s, role::TEACHER, Some(Rel::Teaches));
        // tally marks: teachers who know them leave a record at camp
        if self.people[s].knows(n::TALLY) && self.at_home(s) {
            self.use_know(s, n::TALLY);
            if let Some(hc) = self.people[s].home {
                let c = &mut self.world.camps[hc as usize];
                if !c.records.contains(&node) {
                    c.records.push(node);
                }
            }
        }
    }

    /// Bystanders note role-revealing behaviour; repeated observation becomes a role belief.
    fn observe_role(&mut self, actor: usize, r: usize, belief: Option<Rel>) {
        let tick = self.tick;
        let (x, y) = self.people[actor].pos(tick);
        let cap = self.cfg.mind.model_cap;
        let bcap = self.cfg.mind.belief_cap;
        let aid = actor as u32;
        for o in self.near(x, y, 25.0) {
            if o == aid {
                continue;
            }
            let oi = o as usize;
            let md = self.people[oi].mind.model_mut(aid, tick, cap);
            md.seen[r] += 1.0;
            let seen = md.seen[r];
            if let Some(rel) = belief {
                if seen >= 3.0 {
                    self.people[oi].mind.believe(Concept::Person(aid), rel, 0.12, None, o, tick, false, bcap);
                }
            }
        }
    }

    fn steal(&mut self, i: usize, j: usize) {
        let tick = self.tick;
        let cap = self.cfg.mind.model_cap;
        let bcap = self.cfg.mind.belief_cap;
        let (x, y) = self.people[i].pos(tick);
        let victim_present = self.close(i, j);
        let cache = self.people[j].knows(n::CACHE);
        let (pi, pj) = pair_mut(&mut self.people, i, j);
        let avail = pj.carried + if cache { pj.stash * 0.3 } else { pj.stash };
        let take = avail.min(2.0);
        if take <= 0.1 {
            return;
        }
        let from_c = take.min(pj.carried);
        pj.carried -= from_c;
        pj.stash = (pj.stash - (take - from_c)).max(0.0);
        pi.carried += take;
        pj.needs[need::STOLEN] += 1.0;
        self.counters.thefts += 1;
        let witnesses: Vec<u32> = self.near(x, y, 25.0).into_iter().filter(|&o| o != i as u32).collect();
        let mut caught = victim_present && self.people[j].rng.chance(0.7);
        let mut seen_by = Vec::new();
        for &o in &witnesses {
            let vis = self.people[o as usize].genome.vision;
            if self.people[o as usize].rng.chance(0.35 * (0.5 + vis)) {
                seen_by.push(o);
                caught = true;
            }
        }
        if !caught {
            return;
        }
        let iid = i as u32;
        let shame = self.people[i].mind.traits[5];
        self.people[i].mind.feel(emo::SHAME, 0.3 + shame * 0.4);
        self.episode(i, EpKind::CaughtStealing, Some(j as u32), -0.4, 0.5);
        if !seen_by.contains(&(j as u32)) {
            seen_by.push(j as u32);
        }
        for o in seen_by {
            let oi = o as usize;
            let md = self.people[oi].mind.model_mut(iid, tick, cap);
            md.trust = (md.trust - 0.2).max(0.0);
            md.seen[role::THIEF] += 1.0;
            if oi == j {
                md.grudge = (md.grudge + 0.4).min(1.0);
                self.people[oi].mind.feel(emo::ANGER, 0.6);
                self.episode(oi, EpKind::StolenFrom, Some(iid), -0.6, 0.6);
            }
            self.people[oi].mind.believe(Concept::Person(iid), Rel::Thief, 0.4, None, o, tick, false, bcap);
        }
        let place = place_of(x, y);
        let (ni, nj) = (self.name(iid), self.name(j as u32));
        self.emit(EvKind::Theft, vec![iid, j as u32], place, format!("{ni} is caught stealing food from {nj}"));
    }

    fn punish(&mut self, i: usize, j: usize) {
        let tick = self.tick;
        let cap = self.cfg.mind.model_cap;
        let bcap = self.cfg.mind.belief_cap;
        let iid = i as u32;
        let jid = j as u32;
        let (x, y) = self.people[i].pos(tick);
        let restitution = self.people[i].knows(n::RESTITUTION);
        if restitution {
            self.use_know(i, n::RESTITUTION);
            let (pi, pj) = pair_mut(&mut self.people, i, j);
            let pay = (pj.carried + pj.stash).min(2.0);
            let from_c = pay.min(pj.carried);
            pj.carried -= from_c;
            pj.stash -= pay - from_c;
            pi.carried += pay;
            pi.mind.model_mut(jid, tick, cap).grudge *= 0.3;
            pj.mind.feel(emo::SHAME, 0.4);
        } else {
            let power_i = self.people[i].genome.dominance + self.people[i].mind.traits[3] + self.people[i].genome.size;
            let power_j = self.people[j].genome.dominance + self.people[j].mind.traits[3] + self.people[j].genome.size;
            let win = self.rng.chance(0.5 + (power_i - power_j) * 0.3);
            let (pi, pj) = pair_mut(&mut self.people, i, j);
            if win {
                pj.health -= 0.15;
                pj.mind.feel(emo::SHAME, 0.3 + pj.mind.traits[5] * 0.3);
                pj.mind.feel(emo::ANGER, 0.3);
                pi.mind.model_mut(jid, tick, cap).grudge *= 0.4;
            } else {
                pi.health -= 0.1;
                pi.mind.feel(emo::FEAR, 0.3);
            }
            let mj = pj.mind.model_mut(iid, tick, cap);
            mj.grudge = (mj.grudge + 0.15 + 0.2 * pj.genome.traits[3]).min(1.0);
            mj.fear = (mj.fear + 0.25).min(1.0);
            pj.needs[need::CONFLICT] += 1.0;
            pi.needs[need::CONFLICT] += 0.5;
        }
        self.episode(j, EpKind::Punished, Some(iid), -0.6, 0.6);
        self.counters.punishments += 1;
        // witnesses who think the target is a thief learn that stealing is punished: a norm forms
        let thief_belief_i = self.people[i].mind.belief(Concept::Person(jid), Rel::Thief);
        for o in self.near(x, y, 25.0) {
            if o == iid || o == jid {
                continue;
            }
            let oi = o as usize;
            let tb = self.people[oi].mind.belief(Concept::Person(jid), Rel::Thief);
            if tb > 0.3 || thief_belief_i > 0.5 {
                let d = 0.15 * (0.5 + self.people[oi].mind.traits[5]);
                self.people[oi].mind.believe(Concept::Kind(Kind::Steal), Rel::Forbidden, d, Some(iid), iid, tick, false, bcap);
                let md = self.people[oi].mind.model_mut(iid, tick, cap);
                md.respect = (md.respect + 0.05).min(1.0);
            } else {
                // punishing without known cause looks like bullying
                let md = self.people[oi].mind.model_mut(iid, tick, cap);
                md.trust -= 0.05;
                md.fear = (md.fear + 0.1).min(1.0);
            }
        }
        if thief_belief_i > 0.3 {
            self.people[i].mind.believe(Concept::Kind(Kind::Steal), Rel::Forbidden, 0.2, None, iid, tick, false, bcap);
        }
        let place = place_of(x, y);
        let (ni, nj) = (self.name(iid), self.name(jid));
        let how = if restitution { "makes {nj} repay what was taken" } else { "beats {nj}" };
        self.emit(EvKind::Punishment, vec![iid, jid], place, format!("{ni} {}", how.replace("{nj}", &nj)));
    }

    fn defend(&mut self, i: usize, pred: u32) {
        let tick = self.tick;
        let (x, y) = self.people[i].pos(tick);
        let Some(k) = self.world.predators.iter().position(|q| q.id == pred && q.alive) else { return };
        let (px, py) = (self.world.predators[k].x, self.world.predators[k].y);
        if dist(x, y, px, py) > 25.0 {
            return;
        }
        let allies = self.people.iter().filter(|q| q.alive && q.act == (Act::Defend { pred }) && q.id != i as u32).count() as f32;
        let spear = if self.people[i].knows(n::SPEAR) { 0.3 } else { 0.0 };
        self.use_know(i, n::SPEAR);
        let p_kill = 0.008 + spear * 0.35 + allies.min(4.0) * 0.01 + self.people[i].genome.size * 0.006;
        if self.rng.chance(p_kill) {
            self.world.predators[k].alive = false;
            self.people[i].predators_killed += 1;
            self.people[i].mind.drives[drv::STATUS] = 0.0;
            self.people[i].mind.feel(emo::JOY, 0.6);
            self.people[i].needs[need::PREDATOR] *= 0.5;
            let name = self.name(i as u32);
            let place = place_of(x, y);
            self.people[i].life_note(tick, "killed a gulper".into());
            self.episode(i, EpKind::PredatorKilled, None, 0.9, 0.8);
            self.emit(EvKind::PredatorKilled, vec![i as u32], place, format!("{name} kills a gulper"));
            self.observe_role(i, role::GUARD, Some(Rel::Leads));
            self.counters.predators_killed += 1;
        } else {
            // driven off, sometimes at a cost; sometimes it turns on the defender
            let turn = self.rng.chance(0.25);
            let pr = &mut self.world.predators[k];
            if turn {
                pr.target = Some(i as u32);
                return;
            }
            pr.target = None;
            pr.cooldown = tick + 96;
            pr.tx = (pr.x + (pr.x - x) * 3.0).clamp(5.0, crate::world::TANK_W - 5.0);
            pr.ty = (pr.y + (pr.y - y) * 3.0).clamp(5.0, crate::world::TANK_H - 5.0);
            if self.rng.chance(0.2) {
                self.people[i].health -= 0.25;
                if self.people[i].health <= 0.0 {
                    self.die(i, "killed fighting a gulper");
                    return;
                }
            }
            self.observe_role(i, role::GUARD, None);
        }
    }

    fn build(&mut self, i: usize, node: u8) {
        if !self.at_home(i) {
            return;
        }
        let Some(hc) = self.people[i].home else { return };
        let tick = self.tick;
        let material = self.people[i].know(node).map_or(0, |k| k.material);
        let camp_place = self.world.camps[hc as usize].place;
        match node {
            n::GARDEN | n::SHELL_BED => {
                for ci in self.world.best_cells(camp_place, 8) {
                    let cell = &mut self.world.cells[ci];
                    if node == n::GARDEN {
                        cell.garden = (cell.garden + 0.25).min(1.0);
                    } else if cell.hab == Habitat::Reef || cell.shell_cap > 5.0 {
                        cell.bed = true;
                        cell.shell = (cell.shell + 2.0).min(cell.shell_cap);
                    }
                }
            }
            _ => {
                let camp = &mut self.world.camps[hc as usize];
                if let Some(s) = camp.structures.iter_mut().find(|s| s.node == node) {
                    s.condition = (s.condition + 0.5).min(1.0);
                } else {
                    camp.structures.push(crate::world::Structure { node, material, condition: 0.6, built: tick, builder: i as u32 });
                    let cname = self.camp_name(hc);
                    let name = self.name(i as u32);
                    let vname = crate::discovery::Registry::variant_name(node, material);
                    self.emit(EvKind::StructureBuilt, vec![i as u32], camp_place, format!("{name} raises a {vname} at {cname}"));
                    self.counters.structures += 1;
                }
            }
        }
        self.observe_role(i, role::BUILDER, None);
        self.use_know(i, node);
    }

    /// Need-driven experimentation (DESIGN §10.2).
    fn experiment(&mut self, i: usize, scale: f32) {
        let tick = self.tick;
        let (x, y) = self.people[i].pos(tick);
        let place = place_of(x, y);
        let mut mats = self.world.places[place as usize].mats;
        if let Some(h) = self.people[i].home {
            mats |= self.world.places[self.world.camps[h as usize].place as usize].mats;
        }
        let group = self.people[i].home.map_or(1, |h| self.world.camps[h as usize].members);
        let r_canon = self.people[i].rng.f32();
        let r_pick = self.people[i].rng.next_u64();
        let p = &self.people[i];
        // reading old tally marks
        if p.knows(n::TALLY) {
            if let Some(h) = p.home {
                let rec: Vec<u8> = self.world.camps[h as usize].records.iter().copied().filter(|r| !p.knows(*r)).collect();
                if let Some(&r) = rec.first() {
                    self.learn(i, r, 0, KnowSource::Record);
                    return;
                }
            }
        }
        let mut options: Vec<(u8, u16, f32)> = Vec::new();
        for d in NODES.iter() {
            if p.knows(d.id) || d.min_group as u32 > group || !d.prereq.iter().all(|q| p.knows(*q)) {
                continue;
            }
            let avail: Vec<u16> = d
                .materials
                .iter()
                .copied()
                .filter(|m| mats & m != 0 && p.mind.belief(Concept::Kind(mat_kind(*m)), Rel::Cursed) * (0.4 + p.mind.traits[5] + p.mind.emo[emo::FEAR]) < 0.6)
                .collect();
            if !d.materials.is_empty() && avail.is_empty() {
                continue;
            }
            let need_score: f32 = d.needs.iter().map(|nn| p.needs[*nn]).sum();
            let w = 0.15 + need_score.min(4.0) + p.mind.traits[2] * 0.3;
            let material = if avail.is_empty() {
                0
            } else if avail[0] == d.materials[0] && r_canon < 0.75 {
                avail[0]
            } else {
                avail[(r_pick % avail.len() as u64) as usize]
            };
            options.push((d.id, material, w));
        }
        let weights: Vec<f32> = options.iter().map(|o| o.2).collect();
        let Some(k) = self.people[i].rng.weighted(&weights) else { return };
        let (node, material, w) = options[k];
        let d = &NODES[node as usize];
        let p = &self.people[i];
        let chance = self.cfg.discovery.base_rate * scale * (0.4 + w * 0.5) * (0.5 + p.mind.traits[2]) * p.genome.capacity / d.difficulty;
        self.people[i].mind.drives[drv::CURIOSITY] *= 0.7;
        if self.people[i].rng.chance(chance) {
            self.learn(i, node, material, KnowSource::Invented);
            self.disconfirm_curse(i, material);
            let p = &mut self.people[i];
            p.inventions += 1;
            p.mind.drives[drv::STATUS] *= 0.3;
            p.mind.feel(emo::JOY, 0.7);
            for nn in d.needs {
                p.needs[*nn] *= 0.5;
            }
            let vname = crate::discovery::Registry::variant_name(node, material);
            p.life_note(tick, format!("invented the {vname}"));
            // the inventor coins a word for it
            let w = p.phon.coin(&mut p.rng);
            p.mind.lex.adopt(Concept::Node(node), w, 0.8);
            self.episode(i, EpKind::Discovered, None, 0.8, 0.8);
            self.observe_role(i, role::INVENTOR, None);
        } else if self.people[i].rng.chance(self.cfg.discovery.false_belief_on_fail * scale) && material != 0 {
            // a failed experiment blamed on the material: a false belief that can spread
            let bcap = self.cfg.mind.belief_cap;
            let id = i as u32;
            self.people[i].mind.believe(Concept::Kind(mat_kind(material)), Rel::Cursed, 0.45, None, id, tick, false, bcap);
            self.counters.false_beliefs_formed += 1;
        }
    }
}

pub fn mat_kind(m: u16) -> Kind {
    match m {
        mat::SHELL => Kind::Shell,
        mat::STONE => Kind::Stone,
        mat::KELP => Kind::Kelp,
        mat::CLAY => Kind::Clay,
        mat::SPONGE => Kind::Sponge,
        mat::GLOW => Kind::Glowworm,
        mat::VENT | mat::VENTGLASS => Kind::Vent,
        _ => Kind::Sand,
    }
}
