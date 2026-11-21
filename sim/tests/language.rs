use tank_sim::concepts::{Concept, Kind};
use tank_sim::language::{Lexicon, Phonology};
use tank_sim::rng::Rng;

#[test]
fn coined_words_are_bounded_and_have_vowels() {
    let mut r = Rng::new(1);
    let ph = Phonology::generate(&mut r, 0.5, 0.5, 0.5);
    for _ in 0..500 {
        let w = ph.coin(&mut r);
        assert!(!w.is_empty() && w.len() <= 8);
        assert!(w.iter().any(|p| tank_sim::language::is_vowel(*p)));
    }
}

#[test]
fn pruning_keeps_basic_vocabulary() {
    let mut r = Rng::new(2);
    let ph = Phonology::generate(&mut r, 0.5, 0.5, 0.5);
    let mut lex = Lexicon::default();
    lex.adopt(Concept::Kind(Kind::Food), ph.coin(&mut r), 0.2);
    for p in 0..50u32 {
        lex.adopt(Concept::Person(p), ph.coin(&mut r), 0.9);
    }
    lex.prune(10, |_| true, |p| p < 3);
    assert_eq!(lex.len(), 10);
    assert!(lex.best(&Concept::Kind(Kind::Food)).is_some(), "basic words are forgotten last");
    assert!(lex.best(&Concept::Person(1)).is_some(), "known people's names survive before strangers'");
}
