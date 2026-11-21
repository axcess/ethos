//! Emergent language (DESIGN §9): per-culture phonology, word coining, the naming game,
//! and regular sound change that spreads word by word (lexical diffusion).
use crate::concepts::Concept;
use crate::rng::Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Master phoneme inventory: (romanization, IPA, articulator 0=lips 1=tongue 2=throat 3=vowel)
pub const PHONEMES: &[(&str, &str, u8)] = &[
    ("p", "p", 0), ("b", "b", 0), ("m", "m", 0), ("f", "f", 0), ("v", "v", 0), ("w", "w", 0),
    ("t", "t", 1), ("d", "d", 1), ("n", "n", 1), ("s", "s", 1), ("z", "z", 1), ("l", "l", 1),
    ("r", "r", 1), ("sh", "ʃ", 1), ("ch", "tʃ", 1), ("y", "j", 1), ("th", "θ", 1),
    ("k", "k", 2), ("g", "g", 2), ("ng", "ŋ", 2), ("kh", "x", 2), ("h", "h", 2), ("'", "ʔ", 2), ("q", "q", 2),
    ("gh", "ɣ", 2),
    // vowels
    ("a", "a", 3), ("e", "e", 3), ("i", "i", 3), ("o", "o", 3), ("u", "u", 3), ("aa", "aː", 3), ("ii", "iː", 3),
    ("uu", "uː", 3), ("ai", "ai", 3), ("au", "au", 3),
];
pub const FIRST_VOWEL: u8 = 25;
pub fn is_vowel(p: u8) -> bool {
    p >= FIRST_VOWEL
}
fn front(p: u8) -> bool {
    matches!(PHONEMES[p as usize].0, "e" | "i" | "ii" | "ai")
}

pub type Form = Vec<u8>;

pub fn romanize(f: &[u8]) -> String {
    f.iter().map(|p| PHONEMES[*p as usize].0).collect()
}
pub fn ipa(f: &[u8]) -> String {
    f.iter().map(|p| PHONEMES[*p as usize].1).collect()
}
pub fn name_case(f: &[u8]) -> String {
    let s = romanize(f);
    let mut c = s.chars();
    match c.next() {
        Some(x) => x.to_uppercase().collect::<String>() + c.as_str(),
        None => s,
    }
}

pub fn edit_distance(a: &[u8], b: &[u8]) -> usize {
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Env {
    Any,
    Intervocalic,
    Initial,
    Final,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SoundLaw {
    Sub { from: u8, to: u8, env: Env },
    DropFinalVowel,
    DropInitialConsonant { which: u8 },
}

impl SoundLaw {
    pub fn apply(&self, w: &[u8]) -> Form {
        let mut out: Form = Vec::with_capacity(w.len());
        match *self {
            SoundLaw::Sub { from, to, env } => {
                for (i, &p) in w.iter().enumerate() {
                    let ok = p == from
                        && match env {
                            Env::Any => true,
                            Env::Initial => i == 0,
                            Env::Final => i + 1 == w.len(),
                            Env::Intervocalic => i > 0 && i + 1 < w.len() && is_vowel(w[i - 1]) && is_vowel(w[i + 1]),
                        };
                    out.push(if ok { to } else { p });
                }
            }
            SoundLaw::DropFinalVowel => {
                out.extend_from_slice(w);
                if w.len() >= 4 && is_vowel(*w.last().unwrap()) {
                    out.pop();
                }
            }
            SoundLaw::DropInitialConsonant { which } => {
                out.extend_from_slice(w);
                if w.len() >= 3 && w[0] == which && is_vowel(w[1]) {
                    out.remove(0);
                }
            }
        }
        if !out.iter().any(|p| is_vowel(*p)) {
            return w.to_vec();
        }
        out
    }
    pub fn describe(&self) -> String {
        match *self {
            SoundLaw::Sub { from, to, env } => format!(
                "{} > {} ({:?})",
                PHONEMES[from as usize].0,
                PHONEMES[to as usize].0,
                env
            ),
            SoundLaw::DropFinalVowel => "final vowel lost".into(),
            SoundLaw::DropInitialConsonant { which } => format!("initial {} lost", PHONEMES[which as usize].0),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Phonology {
    pub consonants: Vec<u8>,
    pub vowels: Vec<u8>,
    /// Weights for syllable templates V, CV, CVC, CCV, VC
    pub templates: [f32; 5],
    /// Weights for 1, 2, 3 syllables
    pub lengths: [f32; 3],
    pub harmony: bool,
}

impl Phonology {
    /// A fresh phonology constrained by the speakers' voice genes (lips/tongue/throat ability).
    pub fn generate(r: &mut Rng, lips: f32, tongue: f32, throat: f32) -> Phonology {
        let mut consonants = Vec::new();
        for (i, (_, _, art)) in PHONEMES.iter().enumerate() {
            let ability = match art {
                0 => lips,
                1 => tongue,
                2 => throat,
                _ => continue,
            };
            if r.chance(0.15 + 0.6 * ability) {
                consonants.push(i as u8);
            }
        }
        while consonants.len() < 6 {
            let c = r.below(FIRST_VOWEL as u32) as u8;
            if !consonants.contains(&c) {
                consonants.push(c);
            }
        }
        consonants.sort();
        let mut vowels = vec![FIRST_VOWEL, FIRST_VOWEL + 2, FIRST_VOWEL + 4]; // a i u
        for v in [FIRST_VOWEL + 1, FIRST_VOWEL + 3, FIRST_VOWEL + 5, FIRST_VOWEL + 6, FIRST_VOWEL + 7, FIRST_VOWEL + 8, FIRST_VOWEL + 9] {
            if r.chance(0.35) {
                vowels.push(v);
            }
        }
        vowels.sort();
        let mut templates = [0.0; 5];
        for t in templates.iter_mut() {
            *t = r.f32() * r.f32();
        }
        templates[1] += 0.5; // CV is near-universal
        Phonology {
            consonants,
            vowels,
            templates,
            lengths: [r.range(0.1, 0.6), r.range(0.3, 1.0), r.range(0.05, 0.5)],
            harmony: r.chance(0.3),
        }
    }

    pub fn coin(&self, r: &mut Rng) -> Form {
        let mut w = self.coin_raw(r);
        w.truncate(8);
        if !w.iter().any(|p| is_vowel(*p)) {
            w.push(self.vowels[0]);
        }
        w
    }

    fn coin_raw(&self, r: &mut Rng) -> Form {
        let nsyl = r.weighted(&self.lengths).unwrap_or(1) + 1;
        let mut w = Vec::new();
        let front_word = r.chance(0.5);
        let vowel = |r: &mut Rng| -> u8 {
            if self.harmony {
                let set: Vec<u8> = self.vowels.iter().copied().filter(|v| front(*v) == front_word).collect();
                if !set.is_empty() {
                    return *r.pick(&set).unwrap();
                }
            }
            *r.pick(&self.vowels).unwrap()
        };
        for _ in 0..nsyl {
            let c = |r: &mut Rng| *r.pick(&self.consonants).unwrap();
            if w.last().map_or(false, |p| is_vowel(*p)) && r.chance(0.85) {
                w.push(c(r));
            }
            match r.weighted(&self.templates).unwrap_or(1) {
                0 => w.push(vowel(r)),
                1 => {
                    w.push(c(r));
                    w.push(vowel(r))
                }
                2 => {
                    w.push(c(r));
                    w.push(vowel(r));
                    w.push(c(r))
                }
                3 => {
                    w.push(c(r));
                    w.push(c(r));
                    w.push(vowel(r))
                }
                _ => {
                    w.push(vowel(r));
                    w.push(c(r))
                }
            }
        }
        w
    }

    /// A plausible sound change for this phonology: shift to a neighbouring sound.
    pub fn random_law(&self, r: &mut Rng) -> SoundLaw {
        match r.below(10) {
            0 => SoundLaw::DropFinalVowel,
            1 => SoundLaw::DropInitialConsonant { which: *r.pick(&self.consonants).unwrap() },
            _ => {
                let from = *r.pick(&self.consonants).unwrap();
                let art = PHONEMES[from as usize].2;
                let same: Vec<u8> = (0..FIRST_VOWEL).filter(|p| PHONEMES[*p as usize].2 == art && *p != from).collect();
                let to = *r.pick(&same).unwrap_or(&from);
                let env = [Env::Any, Env::Intervocalic, Env::Initial, Env::Final][r.below(4) as usize];
                SoundLaw::Sub { from, to, env }
            }
        }
    }
}

/// One mind's vocabulary: concept → competing word forms with scores.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Lexicon {
    pub entries: BTreeMap<Concept, Vec<(Form, f32)>>,
}

impl Lexicon {
    pub fn best(&self, c: &Concept) -> Option<&Form> {
        self.entries.get(c).and_then(|v| {
            v.iter().fold(None, |acc: Option<&(Form, f32)>, x| match acc {
                Some(a) if a.1 >= x.1 => Some(a),
                _ => Some(x),
            })
        }).map(|x| &x.0)
    }
    pub fn has_form(&self, c: &Concept, f: &[u8]) -> bool {
        self.entries.get(c).map_or(false, |v| v.iter().any(|x| x.0 == f))
    }
    /// Reinforce `f` for `c` and inhibit competitors (naming-game success).
    pub fn reinforce(&mut self, c: Concept, f: &[u8], d: f32) {
        if let Some(v) = self.entries.get_mut(&c) {
            for x in v.iter_mut() {
                if x.0 == f {
                    x.1 = (x.1 + d).min(1.0);
                } else {
                    x.1 -= d;
                }
            }
            v.retain(|x| x.1 > 0.0);
        }
    }
    pub fn adopt(&mut self, c: Concept, f: Form, score: f32) {
        let v = self.entries.entry(c).or_default();
        if let Some(x) = v.iter_mut().find(|x| x.0 == f) {
            x.1 = x.1.max(score);
        } else {
            v.push((f, score));
            if v.len() > 3 {
                v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                v.truncate(3);
            }
        }
    }
    pub fn weaken(&mut self, c: Concept, f: &[u8], d: f32) {
        if let Some(v) = self.entries.get_mut(&c) {
            for x in v.iter_mut() {
                if x.0 == f {
                    x.1 -= d;
                }
            }
            v.retain(|x| x.1 > 0.0);
            if v.is_empty() {
                self.entries.remove(&c);
            }
        }
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    /// Keep the lexicon bounded. Forget, in order: names of the dead, names of people one
    /// doesn't know, other names, camp and invention words; basic vocabulary goes last.
    pub fn prune(&mut self, cap: usize, alive: impl Fn(u32) -> bool, known: impl Fn(u32) -> bool) {
        if self.entries.len() <= cap {
            return;
        }
        self.entries.retain(|c, _| match c {
            Concept::Person(p) => alive(*p),
            _ => true,
        });
        if self.entries.len() <= cap {
            return;
        }
        let mut order: Vec<(u8, f32, Concept)> = self
            .entries
            .iter()
            .map(|(c, v)| {
                let tier = match c {
                    Concept::Person(p) if !known(*p) => 0,
                    Concept::Person(_) => 1,
                    Concept::Camp(_) | Concept::Node(_) => 2,
                    Concept::Place(_) => 3,
                    Concept::Kind(_) => 4,
                };
                (tier, v.iter().map(|x| x.1).fold(0.0f32, f32::max), *c)
            })
            .collect();
        order.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.partial_cmp(&b.1).unwrap()).then(a.2.cmp(&b.2)));
        let excess = self.entries.len() - cap;
        for (_, _, c) in order.into_iter().take(excess) {
            self.entries.remove(&c);
        }
    }
}
