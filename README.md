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

| [`mentor-db`](crates/mentor-db) | PostgreSQL storage: tenants, learners, progress, XP, badges, cohorts, exam attempts, and the import of v1 data. Tenant isolation is enforced by row-level security |
| [`mentor-cli`](crates/mentor-cli) | The `mentor` command: database migrations, tenant creation, import of a v1 export |
| [`mentor-web`](crates/mentor-web) | The web server: resolves the tenant from the host name, serves home, catalogue, course and lesson pages with that tenant's brand, and records quiz scores and lesson progress for a signed-in learner (development sign-in only, OIDC is not there yet) |

Not built yet: sign-in, progress and interactive labs and quizzes in the web server; the catalogue linter; lab orchestration.

Real labs run in Firecracker microVMs, but Mentor does not implement that isolation: it will come from a base
shared with the [Atelier](https://github.com/PhilippeVienne/atelier) project, on Kubernetes. The integration
study is in [doc/execution-plane.md](doc/execution-plane.md).

## Layout

```text
crates/          Rust workspace
static/          style sheets, scripts and default brand images served by mentor-web (copied from v1)
catalogue/       the 19 courses, converted from v1 (French content, English format)
tools/           one-off tools (v1 catalogue migration)
conformance/     reference output exported from v1, and the v1 commit it comes from
doc/             architecture and decisions
```

## Build and test

```shell
cargo test                                         # unit tests and conformance with v1
cargo run -p mentor-content --example export       # compile catalogue/ and print it as JSON
cargo clippy --all-targets && cargo fmt --check
```

The database tests need a real PostgreSQL and are skipped without it:

```shell
docker run -d --name mentor-test-pg -e POSTGRES_PASSWORD=mentor-test -p 127.0.0.1:55439:5432 postgres:16-alpine
MENTOR_TEST_DATABASE_URL=postgres://postgres:mentor-test@127.0.0.1:55439/postgres cargo test -p mentor-db
```

The superuser of that URL only creates a database and two ordinary roles per test. Migrations and platform
operations then run as a non-superuser owner, and the application side as a role that is only a member of
`mentor_app`: a superuser would bypass row-level security and the isolation tests would prove nothing.
PostgreSQL 16 or later is required.

The conformance test compiles `catalogue/` and compares it with `conformance/v1-catalogue.json`: structure must be
strictly equal, and HTML fragments must have the same text.

## Running the web server

```shell
export MENTOR_DATABASE_URL=postgres://owner:…@host/mentor          # the owning role
mentor migrate
mentor tenant-create demo "Mentor" --host localhost --host 127.0.0.1
MENTOR_APP_DATABASE_URL=postgres://app:…@host/mentor cargo run -p mentor-web   # a member of mentor_app
```

Then open <http://localhost:8300>. A request whose host name belongs to no tenant gets a bare 404. A second
tenant on `acme.localhost` shows the same catalogue under its own name and colours: brand settings are a JSON
object in `tenant.branding` (see `crates/mentor-web/src/brand.rs` for the keys); there is no command to edit it
yet.

Sessions are a cookie signed with `--session-secret` (`MENTOR_SESSION_SECRET`); without one a random secret is
drawn at start-up, so sessions do not survive a restart. Until OIDC is implemented the only way to sign in is
`--dev-login`, which mounts `/dev/login`: a user name, no password. **Never enable it on a reachable
deployment**: anyone could sign in as anyone. Requests that change state are accepted only from the site's own
origin.

Limits of this first version: one catalogue shared by every tenant; lesson HTML is inserted as compiled, without
sanitising, which is only acceptable while the operator writes the catalogue; fonts are the system's; the
interface texts are French, like the catalogue.

## Moving a v1 portal into a tenant

```shell
# on the v1 side: Django's own export, no change to v1 needed
python manage.py dumpdata auth.user training.lesson training.lessonprogress training.xpevent \
    training.userbadge training.cohort training.cohortmembership training.examattempt -o v1.json

# on the v2 side, with the role that owns the database
export MENTOR_DATABASE_URL=postgres://owner:…@host/mentor
mentor migrate
mentor tenant-create acme "Acme" --host acme.mentor.example
mentor import-v1 v1.json --tenant acme --v1-catalogue conformance/v1-catalogue.json --catalogue catalogue
```

The import runs in one transaction and can be replayed. Dates are kept. Tried on v1's demonstration data (5
users, 210 XP events): XP totals, badges and completed lessons came out identical for every user. Imported
accounts get a placeholder identity (`v1:<username>`) until their first login, which the web server will have
to reconcile; that part does not exist yet.

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
