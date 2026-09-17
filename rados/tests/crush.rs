//! Hermetic CRUSH compatibility tests; no Ceph tools or cluster required.

#[path = "crush/golden.rs"]
mod golden;

#[path = "crush/functional.rs"]
mod functional;

#[path = "crush/regressions.rs"]
mod regressions;

#[path = "crush/weights.rs"]
mod weights;

#[path = "crush/choose_args.rs"]
mod choose_args;

#[path = "crush/hierarchy.rs"]
mod hierarchy;
