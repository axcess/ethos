//! Batch and scenario reports (markdown), including Phase 1 exit-criteria checks.
use crate::metrics::RunSummary;
use std::fmt::Write;

fn stats(v: &[f32]) -> (f32, f32, f32, f32) {
    if v.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mean = s.iter().sum::<f32>() / s.len() as f32;
    (mean, s[s.len() / 2], s[0], s[s.len() - 1])
}

pub fn frac(rs: &[RunSummary], f: impl Fn(&RunSummary) -> bool) -> f32 {
    if rs.is_empty() {
        return 0.0;
    }
    rs.iter().filter(|r| f(r)).count() as f32 / rs.len() as f32
}

pub fn batch_report(rs: &[RunSummary], pops: &[Vec<u32>], title: &str, cycles: u64) -> String {
    let mut o = String::new();
    let n = rs.len();
    writeln!(o, "# Batch report: {title}\n").unwrap();
    writeln!(o, "{n} tanks × {cycles} cycles. Engine {}.\n", crate::ENGINE_VERSION).unwrap();
    writeln!(o, "## Exit criteria (ROADMAP Phase 1)\n").unwrap();
    writeln!(o, "| Criterion | Target | Result | Pass |\n|---|---|---|---|").unwrap();
    let rows: Vec<(&str, f32, f32)> = vec![
        ("Groups form *and* split", 0.60, frac(rs, |r| r.groups_formed > 0 && r.group_splits > 0)),
        ("≥ 4 distinct roles emerge", 0.50, frac(rs, |r| r.max_roles >= 4)),
        ("A culture-specific *wrong* belief appears", 0.50, frac(rs, |r| r.culture_specific_false_max >= 1)),
        ("≥ 2 mutually unintelligible lexicons", 0.30, frac(rs, |r| r.max_languages >= 2)),
        ("Knowledge is lost at least once", 0.30, frac(rs, |r| r.knowledge_lost >= 1)),
    ];
    for (name, target, got) in rows {
        writeln!(o, "| {name} | ≥ {:.0}% of seeds | {:.0}% | {} |", target * 100.0, got * 100.0, if got >= target { "✅" } else { "❌" }).unwrap();
    }
    writeln!(o).unwrap();
    writeln!(o, "## Outcomes\n").unwrap();
    writeln!(o, "- Extinct: {:.0}% of tanks", frac(rs, |r| r.extinct) * 100.0).unwrap();
    writeln!(o, "- Any wrong belief held by someone: {:.0}%", frac(rs, |r| r.any_wrong_belief_frac_max > 0.0) * 100.0).unwrap();
    writeln!(o, "- A belief spread from one originator to ≥ 6 people (\"prophet\"): {:.0}%", frac(rs, |r| r.beliefs_spread > 0) * 100.0).unwrap();
    writeln!(o, "- An anti-theft norm took hold in a camp: {:.0}%", frac(rs, |r| r.norm_camps > 0) * 100.0).unwrap();
    writeln!(o, "- A lost discovery was rediscovered: {:.0}%\n", frac(rs, |r| r.rediscoveries > 0) * 100.0).unwrap();
    writeln!(o, "## Metric distributions across seeds\n").unwrap();
    writeln!(o, "| Metric | mean | median | min | max |\n|---|---|---|---|---|").unwrap();
    macro_rules! row {
        ($name:expr, $f:expr) => {
            let v: Vec<f32> = rs.iter().map($f).collect();
            let (a, b, c, d) = stats(&v);
            writeln!(o, "| {} | {:.2} | {:.2} | {:.2} | {:.2} |", $name, a, b, c, d).unwrap();
        };
    }
    row!("final population", |r: &RunSummary| r.final_pop as f32);
    row!("peak population", |r: &RunSummary| r.peak_pop as f32);
    row!("births", |r: &RunSummary| r.births as f32);
    row!("deaths: starvation", |r: &RunSummary| r.deaths_by[0] as f32);
    row!("deaths: gulpers", |r: &RunSummary| r.deaths_by[1] as f32);
    row!("deaths: fever", |r: &RunSummary| r.deaths_by[2] as f32);
    row!("deaths: old age", |r: &RunSummary| r.deaths_by[3] as f32);
    row!("deaths: cold", |r: &RunSummary| r.deaths_by[4] as f32);
    row!("cooperation rate (requests granted)", |r: &RunSummary| r.cooperation_rate);
    row!("shares per capita per cycle", |r: &RunSummary| r.share_per_capita);
    row!("groups formed", |r: &RunSummary| r.groups_formed as f32);
    row!("group splits", |r: &RunSummary| r.group_splits as f32);
    row!("camps founded", |r: &RunSummary| r.camps_founded as f32);
    row!("max distinct roles", |r: &RunSummary| r.max_roles as f32);
    row!("mean belief diversity (Jaccard)", |r: &RunSummary| r.mean_belief_diversity);
    row!("culture-specific wrong beliefs (max)", |r: &RunSummary| r.culture_specific_false_max as f32);
    row!("inventions", |r: &RunSummary| r.inventions as f32);
    row!("distinct discoveries", |r: &RunSummary| r.distinct_discoveries as f32);
    row!("discoveries per generation", |r: &RunSummary| r.discoveries_per_generation);
    row!("knowledge lost", |r: &RunSummary| r.knowledge_lost as f32);
    row!("languages (max)", |r: &RunSummary| r.max_languages as f32);
    row!("min intelligibility", |r: &RunSummary| r.min_intelligibility);
    row!("final lexicon coherence", |r: &RunSummary| r.final_coherence);
    row!("grammar tier (proxy, max)", |r: &RunSummary| r.max_grammar_tier as f32);
    row!("behavior entropy (per person)", |r: &RunSummary| r.mean_behavior_entropy);
    row!("behavior divergence (between people)", |r: &RunSummary| r.mean_behavior_divergence);
    row!("change index (mean)", |r: &RunSummary| r.mean_change_index);
    row!("change index (min, 2nd half)", |r: &RunSummary| r.min_change_index_late);
    row!("thefts", |r: &RunSummary| r.thefts as f32);
    row!("punishments", |r: &RunSummary| r.punishments as f32);
    row!("decisions / sec", |r: &RunSummary| r.decisions_per_sec as f32);
    row!("wall seconds", |r: &RunSummary| r.wall_secs as f32);
    writeln!(o).unwrap();
    // survival curve
    if !pops.is_empty() {
        let maxc = pops.iter().map(|p| p.len()).max().unwrap_or(0);
        writeln!(o, "## Survival curve\n\n| cycle | tanks alive | median pop |\n|---|---|---|").unwrap();
        let step = (maxc / 10).max(1);
        for c in (0..maxc).step_by(step).chain(std::iter::once(maxc.saturating_sub(1))) {
            let mut v: Vec<u32> = pops.iter().map(|p| *p.get(c).unwrap_or(&0)).collect();
            v.sort();
            let alive = v.iter().filter(|x| **x > 0).count();
            writeln!(o, "| {c} | {alive}/{} | {} |", pops.len(), v[v.len() / 2]).unwrap();
        }
        writeln!(o).unwrap();
    }
    writeln!(o, "## Per-seed\n\n| seed | pop | splits | roles | languages | inventions | lost | culture-false | extinct |\n|---|---|---|---|---|---|---|---|---|").unwrap();
    for r in rs {
        writeln!(o, "| {} | {} | {} | {} | {} | {} | {} | {} | {} |", r.seed, r.final_pop, r.group_splits, r.max_roles, r.max_languages, r.inventions, r.knowledge_lost, r.culture_specific_false_max, r.extinct).unwrap();
    }
    o
}

pub fn scenario_report(rows: &[(u64, String, RunSummary)]) -> String {
    let mut o = String::new();
    writeln!(o, "# Scenario comparison\n").unwrap();
    writeln!(o, "Each scenario is run on the same seeds as the baseline; values are means across seeds, with the change from baseline.\n").unwrap();
    let base: Vec<&RunSummary> = rows.iter().filter(|r| r.1 == "baseline").map(|r| &r.2).collect();
    let metrics: Vec<(&str, fn(&RunSummary) -> f32)> = vec![
        ("final pop", |r| r.final_pop as f32),
        ("deaths", |r| r.deaths as f32),
        ("starvation", |r| r.deaths_by[0] as f32),
        ("gulper deaths", |r| r.deaths_by[1] as f32),
        ("fever deaths", |r| r.deaths_by[2] as f32),
        ("cold deaths", |r| r.deaths_by[4] as f32),
        ("coop rate", |r| r.cooperation_rate),
        ("shares/cap", |r| r.share_per_capita),
        ("splits", |r| r.group_splits as f32),
        ("camps founded", |r| r.camps_founded as f32),
        ("inventions", |r| r.inventions as f32),
        ("thefts", |r| r.thefts as f32),
        ("punish", |r| r.punishments as f32),
        ("belief div", |r| r.mean_belief_diversity),
        ("change", |r| r.mean_change_index),
    ];
    let mean = |v: &[&RunSummary], f: fn(&RunSummary) -> f32| if v.is_empty() { 0.0 } else { v.iter().map(|r| f(r)).sum::<f32>() / v.len() as f32 };
    let mut header = "| scenario |".to_string();
    let mut sep = "|---|".to_string();
    for (n, _) in &metrics {
        header += &format!(" {n} |");
        sep += "---|";
    }
    writeln!(o, "{header}\n{sep}").unwrap();
    let mut names: Vec<String> = rows.iter().map(|r| r.1.clone()).collect();
    names.dedup();
    let mut seen = Vec::new();
    for nme in names {
        if seen.contains(&nme) {
            continue;
        }
        seen.push(nme.clone());
        let v: Vec<&RunSummary> = rows.iter().filter(|r| r.1 == nme).map(|r| &r.2).collect();
        let mut line = format!("| {nme} |");
        for (_, f) in &metrics {
            let m = mean(&v, *f);
            if nme == "baseline" {
                line += &format!(" {m:.2} |");
            } else {
                let b = mean(&base, *f);
                line += &format!(" {m:.2} ({:+.2}) |", m - b);
            }
        }
        writeln!(o, "{line}").unwrap();
    }
    o
}
