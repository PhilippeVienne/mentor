//! Compiler for the training catalogue.
//!
//! Port of `formation/training/content.py` (v1). The authoring format is described in
//! `catalogue/README.md`: one folder per course, one lesson per `NN-name.md` file, `:::labo` and
//! `:::quiz` blocks. The output of [`load_catalogue`] has the same shape as the JSON produced by v1's
//! `manage.py export_catalogue`; `tests/conformance.rs` compares both.
//!
//! The catalogue format (front matter keys, directive names) and author-facing diagnostics are in French:
//! they are part of the product, not of the code.

mod catalogue;
mod document;
mod error;
mod lab;
mod markdown;
mod yaml;

pub use catalogue::{load_catalogue, Catalogue, Course, Exam, ExamQuestion, Lesson};
pub use document::{Quiz, QuizOption};
pub use error::ContentError;
pub use lab::{Check, Effect, Lab, Step};
