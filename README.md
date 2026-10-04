# Mentor

White-label interactive training platform: learners practise in simulated terminals or in real isolated
environments, validate quizzes, earn XP, level up and unlock badges. Courses are plain Markdown files.

This repository is **Mentor v2**: a rewrite in Rust, with Firecracker microVMs for real labs and multi-tenancy.
v1 is a Python/Django application and stays the reference implementation while v2 is built.
The direction, threat model and migration plan are in [doc/architecture.md](doc/architecture.md).

## Status

Phase 0 (foundations) is in progress. What exists today:

| Crate | What it does |
| --- | --- |
| [`mentor-core`](crates/mentor-core) | XP constants and level computation, free of I/O |
| [`mentor-content`](crates/mentor-content) | Catalogue compiler: front matter, Markdown, code blocks, callouts, quizzes, labs, exams |

Not built yet: the web server, the database layer, the execution plane (Firecracker), the catalogue linter.
The details of what is and is not ported are in [doc/architecture.md §6.1](doc/architecture.md#61-phase-0-conformance-what-matches-v1-means).

## Layout

```text
crates/          Rust workspace
catalogue/       the 19 courses, copied from v1 (French content, format described in catalogue/README.md)
conformance/     reference output exported from v1, and the v1 commit it comes from
doc/             architecture and decisions
```

## Build and test

```shell
cargo test                                         # unit tests and conformance with v1
cargo run -p mentor-content --example export       # compile catalogue/ and print it as JSON
cargo clippy --all-targets && cargo fmt --check
```

The conformance test compiles `catalogue/` and compares it with `conformance/v1-catalogue.json`: structure must be
strictly equal, and HTML fragments must have the same text.

## Language

Code, comments, tests and documentation are in English. The catalogue (course content, front matter keys,
directive names) and the diagnostics shown to catalogue authors are French: they are product content.

## Licence

[GNU AGPL v3](LICENSE) or later.
