//! THE TANK: headless simulation engine (Phase 1: Minds Engine).
//! Deterministic from (seed, config, engine version, climate dataset). See docs/DESIGN.md.
pub mod climate;
pub mod concepts;
pub mod config;
pub mod decide;
pub mod discovery;
pub mod events;
pub mod evo;
pub mod export;
pub mod genome;
pub mod language;
pub mod metrics;
pub mod mind;
pub mod person;
pub mod report;
pub mod rng;
pub mod scenario;
pub mod sim;
pub mod social;
pub mod thought;
pub mod world;

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
