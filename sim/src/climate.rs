//! Climate driver: one tank cycle replays one real Chicago year (DESIGN §4.1).
//! The dataset is bundled and checksummed; the simulation never touches the network.
use crate::config::ClimateCfg;
use crate::rng::{fnv64, Rng};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub struct RealDay {
    pub tmax: f32,
    pub tmin: f32,
    pub prcp: f32,
    pub snwd: f32,
    pub wind: f32,
}

#[derive(Clone, Debug)]
pub struct Dataset {
    pub first_year: u32,
    pub years: Vec<[RealDay; 365]>,
    /// Day-of-year climatology of mean temperature (°C).
    pub clim_tmean: Vec<f32>,
    pub hash: u64,
}

impl Dataset {
    pub fn load(path: &str) -> Result<Dataset, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("climate dataset {path}: {e}"))?;
        let hash = fnv64(text.as_bytes());
        let mut first_year = 0;
        let mut years: Vec<[RealDay; 365]> = Vec::new();
        let blank = RealDay { tmax: 0.0, tmin: 0.0, prcp: 0.0, snwd: 0.0, wind: 0.0 };
        for (i, line) in text.lines().enumerate() {
            if i == 0 {
                continue;
            }
            let f: Vec<&str> = line.split(',').collect();
            if f.len() < 7 {
                continue;
            }
            let y: u32 = f[0].parse().map_err(|_| format!("bad year line {i}"))?;
            let doy: usize = f[1].parse().map_err(|_| format!("bad doy line {i}"))?;
            if first_year == 0 {
                first_year = y;
            }
            let yi = (y - first_year) as usize;
            while years.len() <= yi {
                years.push([blank; 365]);
            }
            let p = |s: &str| s.parse::<f32>().unwrap_or(0.0);
            years[yi][doy] = RealDay { tmax: p(f[2]), tmin: p(f[3]), prcp: p(f[4]), snwd: p(f[5]), wind: p(f[6]) };
        }
        let mut clim = vec![0.0f32; 365];
        for d in 0..365 {
            let s: f32 = years.iter().map(|y| (y[d].tmax + y[d].tmin) * 0.5).sum();
            clim[d] = s / years.len() as f32;
        }
        Ok(Dataset { first_year, years, clim_tmean: clim, hash })
    }
    pub fn last_year(&self) -> u32 {
        self.first_year + self.years.len() as u32 - 1
    }
}

/// What one tank day's weather looks like, in tank terms plus the real-world source.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DayClimate {
    pub real_year: u32,
    pub real_doy: u32,
    pub real_tmean_c: f32,
    pub real_prcp_mm: f32,
    /// Surface water temperature in tank °C (≈4..32).
    pub surface_t: f32,
    /// Fraction of the day the Lamp is on.
    pub lamp_frac: f32,
    /// Lamp brightness 0..1 (clouds).
    pub brightness: f32,
    /// Surface ice cover 0..1.
    pub ice: f32,
    /// Heavy-rain runoff pulse.
    pub storm: bool,
    pub wind: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClimateDriver {
    /// Real year for each tank cycle (index = cycle), extended on demand.
    pub year_seq: Vec<u32>,
    rng: Rng,
    block_min: u32,
    block_max: u32,
    pub amplitude: f32,
    /// Scenario overrides: (cycle, real_year).
    pub overrides: Vec<(u64, u32)>,
}

pub const LATITUDE: f32 = 41.88;

impl ClimateDriver {
    pub fn new(seed: u64, cfg: &ClimateCfg, ds: &Dataset) -> Self {
        let mut d = ClimateDriver {
            year_seq: Vec::new(),
            rng: Rng::stream(seed, crate::rng::tag::CLIMATE, 0),
            block_min: cfg.block_min.max(1),
            block_max: cfg.block_max.max(cfg.block_min.max(1)),
            amplitude: cfg.amplitude,
            overrides: Vec::new(),
        };
        if let Some(y) = cfg.start_year {
            let y = y.clamp(ds.first_year, ds.last_year());
            let len = d.block_max;
            for k in 0..len {
                d.year_seq.push((y + k).min(ds.last_year()));
            }
        }
        d
    }

    fn year_for_cycle(&mut self, cycle: u64, ds: &Dataset) -> u32 {
        if let Some(&(_, y)) = self.overrides.iter().rev().find(|(c, _)| *c == cycle) {
            return y;
        }
        while self.year_seq.len() as u64 <= cycle {
            let n = ds.years.len() as u32;
            let len = self.block_min + self.rng.below(self.block_max - self.block_min + 1);
            let start = ds.first_year + self.rng.below(n.saturating_sub(len).max(1));
            for k in 0..len {
                self.year_seq.push(start + k);
            }
        }
        self.year_seq[cycle as usize]
    }

    /// Compute the climate for a tank day. `day_in_cycle` in 0..days_per_cycle.
    pub fn day(&mut self, cycle: u64, day_in_cycle: u64, days_per_cycle: u64, ds: &Dataset) -> DayClimate {
        let year = self.year_for_cycle(cycle, ds);
        let yi = (year - ds.first_year) as usize;
        let a = (day_in_cycle * 365 / days_per_cycle) as usize;
        let b = (((day_in_cycle + 1) * 365 / days_per_cycle) as usize).max(a + 1).min(365);
        let (mut tsum, mut clim, mut prcp, mut snow, mut wind, mut rng_t) = (0.0, 0.0, 0.0f32, 0.0f32, 0.0, 0.0f32);
        let mut tmin_lo = f32::MAX;
        let mut tmax_hi = f32::MIN;
        for d in a..b {
            let r = ds.years[yi][d];
            tsum += (r.tmax + r.tmin) * 0.5;
            clim += ds.clim_tmean[d];
            prcp += r.prcp;
            snow = snow.max(r.snwd);
            wind += r.wind;
            rng_t += r.tmax - r.tmin;
            tmin_lo = tmin_lo.min(r.tmin);
            tmax_hi = tmax_hi.max(r.tmax);
        }
        let n = (b - a) as f32;
        let mut tmean = tsum / n;
        // Preserve extremes: lean toward the most extreme day in the block (§4.1).
        let clim_mean = clim / n;
        if tmin_lo < clim_mean - 15.0 {
            tmean = 0.7 * tmean + 0.3 * (tmin_lo + 5.0);
        }
        if tmax_hi > clim_mean + 15.0 {
            tmean = 0.7 * tmean + 0.3 * (tmax_hi - 5.0);
        }
        let to_tank = |c: f32| 4.0 + (c + 29.0) / 67.0 * 28.0;
        let tank_clim = to_tank(clim_mean);
        let surface_t = (tank_clim + self.amplitude * (to_tank(tmean) - tank_clim)).clamp(1.0, 34.0);
        let doy_mid = ((a + b) / 2) as f32;
        let daylen = day_length_hours(doy_mid);
        let wet_days_frac = (prcp / n / 4.0).min(1.0);
        let cloudy = (1.0 - (rng_t / n) / 12.0).clamp(0.0, 1.0) * 0.5 + wet_days_frac * 0.5;
        let ice = if tmean < -4.0 { ((-4.0 - tmean) / 8.0 + snow / 300.0).clamp(0.0, 1.0) } else { 0.0 };
        DayClimate {
            real_year: year,
            real_doy: a as u32,
            real_tmean_c: tmean,
            real_prcp_mm: prcp,
            surface_t,
            lamp_frac: daylen / 24.0,
            brightness: (1.0 - 0.45 * cloudy).clamp(0.3, 1.0),
            ice,
            storm: prcp > 30.0 || (prcp / n) > 12.0,
            wind: wind / n,
        }
    }
}

/// Astronomical day length (hours) at Chicago's latitude for a day of year.
pub fn day_length_hours(doy: f32) -> f32 {
    let decl = 23.44f32.to_radians() * libm::sinf(2.0 * core::f32::consts::PI * (284.0 + doy) / 365.0);
    let lat = LATITUDE.to_radians();
    let x = (-libm::tanf(lat) * libm::tanf(decl)).clamp(-1.0, 1.0);
    2.0 * libm::acosf(x).to_degrees() / 15.0
}
