//! Mentor business rules, free of I/O: everything here is pure and testable without a database or network.
//!
//! - [`gamification`]: XP amounts and levels;
//! - [`progress`]: what a learner event (a validated lab step, a quiz score) changes and awards;
//! - [`badges`]: badge rules and the statistics they read;
//! - [`exam`]: drawing, timing and grading of a course's validation exam.

pub mod badges;
pub mod exam;
pub mod gamification;
pub mod progress;
