//! Deterministic RNG (SplitMix64). Every consumer owns its own stream so that adding an
//! agent or a system never shifts the random numbers seen by unrelated parts of the world.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rng {
    s: u64,
}

pub fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Stream tags, so streams never collide.
pub mod tag {
    pub const WORLD: u64 = 1;
    pub const MAP: u64 = 2;
    pub const PERSON: u64 = 3;
    pub const PRED: u64 = 4;
    pub const CLIMATE: u64 = 5;
    pub const SCENARIO: u64 = 6;
    pub const OBSERVER: u64 = 7;
    pub const DISEASE: u64 = 8;
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut r = Rng { s: mix(seed ^ 0x9E37_79B9_7F4A_7C15) };
        r.next_u64();
        r
    }
    pub fn stream(seed: u64, tag: u64, id: u64) -> Self {
        Rng::new(mix(seed.wrapping_add(mix(tag.wrapping_mul(0x1000_0001) ^ mix(id)))))
    }
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.s = self.s.wrapping_add(0x9E37_79B9_7F4A_7C15);
        mix(self.s)
    }
    /// Uniform in [0,1).
    #[inline]
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
    #[inline]
    pub fn below(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        (((self.next_u64() >> 32) * n as u64) >> 32) as u32
    }
    #[inline]
    pub fn chance(&mut self, p: f32) -> bool {
        self.f32() < p
    }
    #[inline]
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f32()
    }
    pub fn normal(&mut self) -> f32 {
        let u1 = self.f32().max(1e-7);
        let u2 = self.f32();
        libm::sqrtf(-2.0 * libm::logf(u1)) * libm::cosf(2.0 * core::f32::consts::PI * u2)
    }
    pub fn pick<'a, T>(&mut self, v: &'a [T]) -> Option<&'a T> {
        if v.is_empty() {
            None
        } else {
            Some(&v[self.below(v.len() as u32) as usize])
        }
    }
    /// Index chosen proportionally to non-negative weights.
    pub fn weighted(&mut self, w: &[f32]) -> Option<usize> {
        let total: f32 = w.iter().map(|x| x.max(0.0)).sum();
        if total <= 0.0 {
            return None;
        }
        let mut r = self.f32() * total;
        for (i, x) in w.iter().enumerate() {
            let x = x.max(0.0);
            if r < x {
                return Some(i);
            }
            r -= x;
        }
        Some(w.len() - 1)
    }
}

/// FNV-1a 64-bit, used for state hashes (determinism checks) and keys.
pub fn fnv64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    h
}
