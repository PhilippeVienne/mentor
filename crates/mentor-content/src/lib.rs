//! Compiler for the training catalogue.
//!
//! Port of `formation/training/content.py` (v1). The authoring format is described in
//! `catalogue/README.md`: one folder per course, one lesson per `NN-name.md` file, `:::lab` and
//! `:::quiz` blocks. The output of [`load_catalogue`] has the same shape as the JSON produced by v1's
//! `manage.py export_catalogue`; `tests/conformance.rs` compares both.
//!
//! Courses are distributed as packages: a directory with a `mentor.yml` manifest, compiled by
//! [`load_package`] with the same compiler (design in `doc/course-packages.md`).
//!
//! The catalogue format (front matter keys, directive names) and author-facing diagnostics are in English;
//! only the content of courses keeps its authors' language. What authors write is not trusted: the HTML
//! rendered from it is filtered (see `markdown`).

mod catalogue;
mod document;
mod environment;
mod error;
mod lab;
mod markdown;
mod package;
mod paths;
mod tree;
mod yaml;

pub use catalogue::{load_catalogue, Catalogue, Course, Exam, ExamQuestion, Lesson};
pub use document::{Quiz, QuizOption};
pub use error::ContentError;
pub use lab::{Check, Lab, Step};
pub use package::{load_manifest, load_package, load_package_with, Feature, Manifest, Package, MANIFEST_FILE, PACKAGE_FORMAT};
pub use paths::{load_paths, parse_paths, LearningPath, PathCourse, PathStage, PATHS_FILE};
pub use tree::{verify_tree, Limits, TreeSummary};
