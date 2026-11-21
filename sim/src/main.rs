//! `tank`: command-line interface to the headless engine.
//!
//!   tank run --seed 7 --cycles 40 [--scenario famine@6] [--viewer] [--out runs/x]
//!   tank batch --seeds 1..100 --cycles 40 [--jobs 4] [--scenario S] [--out runs/b]
//!   tank scenarios --seeds 1..20 --cycles 30 [--jobs 4]
//!   tank determinism --seed 3 --cycles 6
//!   tank bench [--pops 100,200,400,800] [--cycles 2]
//!   tank inspect --run runs/x --agent 12
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tank_sim::climate::Dataset;
use tank_sim::config::Config;
use tank_sim::export::{render_thought, write_run, Viewer};
use tank_sim::metrics::{summarize, RunSummary};
use tank_sim::sim::{Sim, ThoughtMode};

const VIEWER_HTML: &str = include_str!("../../tools/viewer/index.html");

struct Args(Vec<String>);
impl Args {
    fn get(&self, k: &str) -> Option<String> {
        self.0.iter().position(|a| a == k).and_then(|i| self.0.get(i + 1).cloned())
    }
    fn has(&self, k: &str) -> bool {
        self.0.iter().any(|a| a == k)
    }
    fn num(&self, k: &str, d: u64) -> u64 {
        self.get(k).and_then(|v| v.parse().ok()).unwrap_or(d)
    }
    fn range(&self, k: &str, d: (u64, u64)) -> (u64, u64) {
        match self.get(k) {
            Some(v) => match v.split_once("..") {
                Some((a, b)) => (a.parse().unwrap_or(d.0), b.parse().unwrap_or(d.1)),
                None => {
                    let x = v.parse().unwrap_or(d.0);
                    (x, x)
                }
            },
            None => d,
        }
    }
}

fn find_file(rel: &str) -> String {
    if std::path::Path::new(rel).exists() {
        return rel.to_string();
    }
    let mut roots = vec![std::env::current_dir().unwrap()];
    if let Ok(exe) = std::env::current_exe() {
        roots.push(exe);
    }
    for r in roots {
        let mut d = r.as_path();
        loop {
            let cand = d.join(rel);
            if cand.exists() {
                return cand.to_string_lossy().to_string();
            }
            match d.parent() {
                Some(p) => d = p,
                None => break,
            }
        }
    }
    rel.to_string()
}

fn load(args: &Args) -> (Config, Arc<Dataset>) {
    let cfg = match args.get("--config") {
        Some(p) => Config::load(&find_file(&p)).unwrap_or_else(|e| fail(&e)),
        None => Config::default(),
    };
    let ds = Dataset::load(&find_file(&cfg.climate.dataset)).unwrap_or_else(|e| fail(&e));
    (cfg, Arc::new(ds))
}

fn fail(msg: &str) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(2)
}

fn make_sim(seed: u64, cfg: &Config, ds: &Arc<Dataset>, scenario: &str) -> Sim {
    let mut sim = Sim::new(seed, cfg.clone(), ds.clone());
    if !scenario.is_empty() && scenario != "baseline" {
        for s in scenario.split(',') {
            let inj = tank_sim::scenario::parse(s, cfg.time.days_per_cycle()).unwrap_or_else(|e| fail(&e));
            sim.scenario.push(inj);
        }
    }
    sim
}

fn max_rss_mb() -> f64 {
    #[cfg(unix)]
    unsafe {
        let mut ru: libc::rusage = std::mem::zeroed();
        libc::getrusage(libc::RUSAGE_SELF, &mut ru);
        let v = ru.ru_maxrss as f64;
        if cfg!(target_os = "macos") {
            v / 1024.0 / 1024.0
        } else {
            v / 1024.0
        }
    }
    #[cfg(not(unix))]
    0.0
}

fn cmd_run(args: &Args) {
    let (cfg, ds) = load(args);
    let seed = args.num("--seed", 1);
    let cycles = args.num("--cycles", 20);
    let scenario = args.get("--scenario").unwrap_or_default();
    let out = args.get("--out").unwrap_or_else(|| format!("runs/seed-{seed}{}", if scenario.is_empty() { String::new() } else { format!("-{}", scenario.replace([',', '@'], "_")) }));
    let mut sim = make_sim(seed, &cfg, &ds, &scenario);
    sim.thought_mode = match args.get("--thoughts").as_deref() {
        Some("all") => ThoughtMode::All,
        Some("none") => ThoughtMode::None,
        _ => ThoughtMode::Tracked,
    };
    let viewer_on = args.has("--viewer");
    let (vf, vt) = args.range("--viewer-cycles", (0, cycles.min(12)));
    let stride = args.num("--viewer-stride", 128);
    let mut viewer = if viewer_on { Some(Viewer::new(&sim)) } else { None };
    std::fs::create_dir_all(&out).unwrap();
    let mut thoughts = std::io::BufWriter::new(std::fs::File::create(format!("{out}/thoughts.txt")).unwrap());
    let t0 = Instant::now();
    let end = sim.tick0 + cycles * sim.tpc();
    let vfrom = sim.tick0 + vf * sim.tpc();
    let vto = sim.tick0 + vt * sim.tpc();
    let mut last_report = sim.cycle();
    while sim.tick < end && !sim.extinct {
        sim.step();
        for th in std::mem::take(&mut sim.thought_out) {
            writeln!(thoughts, "{}", render_thought(&sim, &th)).unwrap();
        }
        if let Some(v) = viewer.as_mut() {
            if sim.tick >= vfrom && sim.tick <= vto && sim.tick % stride == 0 {
                v.capture(&sim);
            }
        }
        if sim.cycle() != last_report {
            last_report = sim.cycle();
            if let Some(s) = sim.stats.last() {
                eprintln!(
                    "cycle {:>3} (Chicago {}) pop {:>4} camps {:>2} groups {:>2} known {:>2} lost {:>2} langs {} intel {:.2} roles {:>2} change {:.2}",
                    s.cycle, s.real_year, s.pop, s.camps, s.groups, s.known_nodes, s.knowledge_lost_total, s.languages, s.min_intelligibility, s.roles_distinct, s.change_index
                );
            }
        }
    }
    let wall = t0.elapsed().as_secs_f64();
    drop(thoughts);
    let summary = summarize(&sim, if scenario.is_empty() { "baseline" } else { &scenario }, wall);
    write_run(&sim, &out, &summary).unwrap();
    if let Some(mut v) = viewer {
        v.finish(&sim, vfrom, vto);
        let json = serde_json::to_string(&v).unwrap();
        // A double-clickable page: the viewer plus the data as a script (works from file://)
        std::fs::write(format!("{out}/viewer-data.js"), format!("window.TANK_DATA = {json};\n")).unwrap();
        let html = VIEWER_HTML.replacen("<script>\n(() =>", "<script src=\"viewer-data.js\"></script>\n<script>\n(() =>", 1);
        std::fs::write(format!("{out}/view.html"), html).unwrap();
        std::fs::write(format!("{out}/viewer.json"), json).unwrap();
    }
    if args.has("--snapshot") {
        std::fs::write(format!("{out}/final.snap"), sim.snapshot()).unwrap();
    }
    eprintln!(
        "done: {} cycles in {:.1}s ({:.0} decisions/s), final pop {}, hash {}, max RSS {:.0} MB → {out}",
        sim.stats.len(),
        wall,
        summary.decisions_per_sec,
        summary.final_pop,
        summary.state_hash,
        max_rss_mb()
    );
}

fn run_batch(cfg: &Config, ds: &Arc<Dataset>, jobs: Vec<(u64, String)>, cycles: u64, threads: usize) -> Vec<(u64, String, RunSummary, Vec<u32>)> {
    let queue = Arc::new(Mutex::new(jobs.into_iter().enumerate().collect::<Vec<_>>()));
    let results = Arc::new(Mutex::new(Vec::new()));
    std::thread::scope(|s| {
        for _ in 0..threads {
            let q = queue.clone();
            let res = results.clone();
            s.spawn(move || loop {
                let job = q.lock().unwrap().pop();
                let Some((k, (seed, scen))) = job else { break };
                let t0 = Instant::now();
                let mut sim = make_sim(seed, cfg, ds, &scen);
                sim.thought_mode = ThoughtMode::None;
                sim.run_cycles(cycles);
                let wall = t0.elapsed().as_secs_f64();
                let sum = summarize(&sim, &scen, wall);
                let pops: Vec<u32> = sim.stats.iter().map(|c| c.pop).collect();
                eprintln!("  seed {seed:>4} {scen:<16} pop {:>4} splits {:>2} roles {:>2} langs {} inv {:>3} lost {} ({:.0}s)", sum.final_pop, sum.group_splits, sum.max_roles, sum.max_languages, sum.inventions, sum.knowledge_lost, wall);
                res.lock().unwrap().push((k, seed, scen, sum, pops));
            });
        }
    });
    let mut r = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
    r.sort_by_key(|x| x.0);
    r.into_iter().map(|x| (x.1, x.2, x.3, x.4)).collect()
}

fn cmd_batch(args: &Args) {
    let (cfg, ds) = load(args);
    let (a, b) = args.range("--seeds", (1, 20));
    let cycles = args.num("--cycles", 30);
    let threads = args.num("--jobs", std::thread::available_parallelism().map(|n| n.get() as u64).unwrap_or(2)) as usize;
    let scen = args.get("--scenario").unwrap_or_else(|| "baseline".into());
    let out = args.get("--out").unwrap_or_else(|| format!("runs/batch-{a}-{b}-c{cycles}"));
    let jobs: Vec<(u64, String)> = (a..=b).map(|s| (s, scen.clone())).collect();
    let t0 = Instant::now();
    let res = run_batch(&cfg, &ds, jobs, cycles, threads);
    std::fs::create_dir_all(&out).unwrap();
    let sums: Vec<RunSummary> = res.iter().map(|r| r.2.clone()).collect();
    let pops: Vec<Vec<u32>> = res.iter().map(|r| r.3.clone()).collect();
    let title = format!("seeds {a}..{b}, config '{}', scenario '{scen}'", cfg.name);
    let md = tank_sim::report::batch_report(&sums, &pops, &title, cycles);
    std::fs::write(format!("{out}/report.md"), &md).unwrap();
    std::fs::write(format!("{out}/summaries.json"), serde_json::to_string_pretty(&sums).unwrap()).unwrap();
    eprintln!("batch done in {:.0}s → {out}/report.md", t0.elapsed().as_secs_f64());
    println!("{md}");
}

fn cmd_scenarios(args: &Args) {
    let (cfg, ds) = load(args);
    let (a, b) = args.range("--seeds", (1, 10));
    let cycles = args.num("--cycles", 24);
    let threads = args.num("--jobs", std::thread::available_parallelism().map(|n| n.get() as u64).unwrap_or(2)) as usize;
    let at = args.num("--at", 6);
    let out = args.get("--out").unwrap_or_else(|| format!("runs/scenarios-{a}-{b}-c{cycles}"));
    let mut jobs = Vec::new();
    for s in a..=b {
        jobs.push((s, "baseline".to_string()));
        for k in tank_sim::scenario::ALL {
            jobs.push((s, format!("{k}@{at}")));
        }
    }
    let res = run_batch(&cfg, &ds, jobs, cycles, threads);
    let rows: Vec<(u64, String, RunSummary)> = res.into_iter().map(|r| (r.0, r.1, r.2)).collect();
    let md = tank_sim::report::scenario_report(&rows);
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(format!("{out}/scenarios.md"), &md).unwrap();
    println!("{md}");
}

fn cmd_determinism(args: &Args) {
    let (cfg, ds) = load(args);
    let seed = args.num("--seed", 3);
    let cycles = args.num("--cycles", 6);
    let half = cycles / 2;
    let run = |c: u64| {
        let mut s = Sim::new(seed, cfg.clone(), ds.clone());
        s.thought_mode = ThoughtMode::None;
        s.run_cycles(c);
        s
    };
    let t0 = Instant::now();
    let a = run(cycles);
    let wall = t0.elapsed().as_secs_f64();
    let b = run(cycles);
    let ha = a.state_hash();
    let hb = b.state_hash();
    // snapshot at half, restore, continue
    let mid = run(half);
    let snap = mid.snapshot();
    let mut restored = Sim::restore(&snap, ds.clone());
    restored.run_cycles(cycles);
    let hc = restored.state_hash();
    println!("seed {seed}, {cycles} cycles ({:.1}s per run, {} people ever, {} alive)", wall, a.people.len(), a.alive.len());
    println!("run A          {ha:016x}");
    println!("run B          {hb:016x}  {}", if ha == hb { "MATCH" } else { "MISMATCH" });
    println!("snapshot@{half}+resume {hc:016x}  {}", if ha == hc { "MATCH" } else { "MISMATCH" });
    println!("snapshot size  {} KB", snap.len() / 1024);
    if ha != hb || ha != hc {
        std::process::exit(1);
    }
}

fn cmd_bench(args: &Args) {
    let (mut cfg, ds) = load(args);
    let pops: Vec<u32> = args.get("--pops").unwrap_or_else(|| "100,200,400,800".into()).split(',').filter_map(|s| s.parse().ok()).collect();
    let cycles = args.num("--cycles", 2);
    println!("| initial pop | cycles | wall s | decisions | decisions/s | µs/decision | sim days/s | peak RSS MB |");
    println!("|---|---|---|---|---|---|---|---|");
    for p in pops {
        cfg.people.initial = p;
        cfg.people.bands = (p / 30).max(2);
        let mut s = Sim::new(42, cfg.clone(), ds.clone());
        s.thought_mode = ThoughtMode::None;
        let t0 = Instant::now();
        s.run_cycles(cycles);
        let w = t0.elapsed().as_secs_f64();
        let days = (cycles * cfg.time.days_per_cycle()) as f64;
        println!("| {p} | {cycles} | {w:.2} | {} | {:.0} | {:.1} | {:.1} | {:.0} |", s.decisions, s.decisions as f64 / w, w * 1e6 / s.decisions.max(1) as f64, days / w, max_rss_mb());
    }
}

fn cmd_inspect(args: &Args) {
    let dir = args.get("--run").unwrap_or_else(|| fail("--run DIR required"));
    let id: u32 = args.num("--agent", 0) as u32;
    let text = std::fs::read_to_string(format!("{dir}/people.json")).unwrap_or_else(|e| fail(&e.to_string()));
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    match v.as_array().and_then(|a| a.iter().find(|p| p["id"] == id)) {
        Some(p) => println!("{}", serde_json::to_string_pretty(p).unwrap()),
        None => fail("agent not found"),
    }
}

fn main() {
    let args = Args(std::env::args().collect());
    match args.0.get(1).map(|s| s.as_str()) {
        Some("run") => cmd_run(&args),
        Some("batch") => cmd_batch(&args),
        Some("scenarios") => cmd_scenarios(&args),
        Some("determinism") => cmd_determinism(&args),
        Some("bench") => cmd_bench(&args),
        Some("inspect") => cmd_inspect(&args),
        _ => {
            eprintln!("usage: tank <run|batch|scenarios|determinism|bench|inspect> [options]\n  see sim/src/main.rs header for options");
            std::process::exit(1);
        }
    }
}
