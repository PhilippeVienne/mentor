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
| [`mentor-core`](crates/mentor-core) | Business rules, free of I/O: XP and levels, lesson progress and idempotent awards, course unlocking, streaks, badge rules, exam drawing and grading |
| [`mentor-content`](crates/mentor-content) | Catalogue compiler: front matter, Markdown, code blocks, callouts, quizzes, labs, exams |

| [`mentor-guest`](crates/mentor-guest) | **Prototype.** Init process and agent of a microVM: runs commands and interactive terminals requested over vsock |
| [`mentor-host`](crates/mentor-host) | **Prototype.** Starts Firecracker microVMs and talks to their guest agent |

Not built yet: the web server, the database layer, the scheduler, the catalogue linter. The execution plane is
a prototype: kernel → read-only rootfs → guest agent → vsock, with per-session disks and snapshots, but without
the jailer or networking yet.
The details of what is and is not ported are in [doc/architecture.md §6.1](doc/architecture.md#61-phase-0-conformance-what-matches-v1-means).

## Layout

```text
crates/          Rust workspace
catalogue/       the 19 courses, converted from v1 (French content, English format)
tools/           development tools (microVM setup, rootfs build, v1 catalogue migration)
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

## Running a microVM

Requirements: Linux on x86_64 with KVM accessible to your user, Docker (to export a base image), `fakeroot`,
`mke2fs`, and the Rust target `x86_64-unknown-linux-musl` (`rustup target add x86_64-unknown-linux-musl`).

```shell
tools/setup-dev.sh        # downloads Firecracker (checksum verified) and a guest kernel into .dev/
tools/build-rootfs.sh     # builds .dev/rootfs.ext4 from alpine, with mentor-guest as init
cargo run -p mentor-host -- exec -- sh -c 'uname -a; id'
cargo run -p mentor-host -- shell
cargo run -p mentor-host -- snapshot .dev/snapshot                    # save a booted microVM
cargo run -p mentor-host -- exec --from .dev/snapshot -- date         # start from the snapshot
cargo test -p mentor-host # end-to-end tests; skipped when KVM or .dev/ is missing
```

On the development machine the guest agent answers about 550 ms after a cold boot and about 6 ms after a
restore from a snapshot. The workspace is a per-session disk (`--disk <MiB>`, 256 by default) whose size is a hard
quota.

## Language and catalogue format

Everything is in English: code, comments, tests, documentation, compiler diagnostics and the **catalogue format**
(file names, front matter keys, lab keys, directive and check names). Course *content* stays in the language of
its authors; the 19 courses shipped here are written in French, and so are the default callout titles and the
labels of generated buttons.

The catalogue was converted from the French v1 format with
[`tools/migrate_v1_catalogue.py`](tools/migrate_v1_catalogue.py), driven by the name table
[`conformance/v1-names.json`](conformance/v1-names.json). The same table lets the conformance test compare v2's
output with v1's export.

`catalogue/README.md`, the authoring guide, still describes the French v1 names and has to be rewritten.

## Licence

[GNU AGPL v3](LICENSE) or later.
