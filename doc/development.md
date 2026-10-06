# Development

## Layout

```text
crates/
  mentor-core/     business rules, free of I/O: XP and levels, progress, unlocking, badges, exams
  mentor-content/  catalogue compiler: Markdown, labs, quizzes, exams, training paths, course packages
  mentor-db/       PostgreSQL storage, migrations, row-level security, import of v1 data
  mentor-cli/      the `mentor` command: migrate, tenant-create, import-v1, package-check
  mentor-web/      the web server (axum, askama templates)
static/            style sheets, scripts and default brand images served by mentor-web
catalogue/         the courses, their environments, the training paths, the authoring guide
conformance/       the catalogue as v1 compiled it, and the v1 → v2 name table
tools/             replay of labs, screenshots, migration of a v1 catalogue
doc/               this documentation
```

Everything is in English: code, comments, tests, documentation, compiler diagnostics and the catalogue
*format*. Course *content* keeps its authors' language, French for the courses shipped here, and so do the
interface labels.

## Checks to run before a commit

```shell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The continuous integration runs the same three, plus a syntax check of the browser scripts.

### Database tests

They need a real PostgreSQL, and **skip themselves** when `MENTOR_TEST_DATABASE_URL` is not set:

```shell
docker run -d --name mentor-test-pg -e POSTGRES_PASSWORD=mentor-test -p 127.0.0.1:55439:5432 postgres:16-alpine
MENTOR_TEST_DATABASE_URL=postgres://postgres:mentor-test@127.0.0.1:55439/postgres cargo test --workspace
```

The superuser of that URL only creates one database and two ordinary roles per test. Migrations then run as a
non-superuser owner and the application side as a member of `mentor_app`: a superuser would bypass row-level
security and the isolation tests would prove nothing.

### Conformance with v1

`crates/mentor-content/tests/conformance.rs` compiles `catalogue/` and compares it with
`conformance/v1-catalogue.json`, the output of v1 for the same courses: the structure must be equal and each
HTML fragment must have the same text. Deliberate differences are listed in the test, each with its reason:
fragments corrected since v1, the two courses rewritten when simulated labs were removed, and the courses
added since.

## Proving labs

A lab is only known to work once it has been replayed:

```shell
python tools/replay_labs.py                 # every course
python tools/replay_labs.py git-basics --only 03-
```

For each lab the tool builds the image of its environment, starts it **without network** as the learner, runs
the setup, then for each step checks that its checks fail, runs its solution, and checks that they hold. It
needs Docker and PyYAML; the Docker courses need a privileged container. A full run builds about twenty images.

A course package is checked without building anything by:

```shell
cargo run -p mentor-cli -- package-check catalogue
```

It validates the manifest and the files, compiles the courses, and prints warnings, among them what would
keep an environment from starting as a microVM.

## Looking at the result

```shell
cargo run -p mentor-content --example export      # the compiled catalogue, as JSON
python tools/screenshots.py --url http://localhost:8300
```

`tools/screenshots.py` drives a headless Chrome against a server started with `--dev-login`, signs in as a
demonstration learner, records some progress and writes the pictures of [screenshots/](screenshots/). Run it
after a change of the interface and look at the pictures before committing them.

## Adding a course

Follow [the authoring guide](../catalogue/README.md). A new course must be listed in `catalogue/catalogue.yml`
and `catalogue/mentor.yml`, and, since v1 never had it, in `ADDED_SINCE_V1` of the conformance test.
