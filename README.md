# Mentor

White-label interactive training platform. Learners follow short courses, practise in **real isolated
environments** verified by the server, answer quizzes, pass validation exams, and earn XP, levels and badges.
Courses are plain Markdown files; several organisations can share one instance, each under its own brand.

![The map of a training path, with the learner's progress](doc/screenshots/path-map.png)

| | |
| --- | --- |
| ![Dashboard](doc/screenshots/dashboard.png) | ![A course](doc/screenshots/course.png) |
| ![A lesson quiz](doc/screenshots/quiz.png) | ![Badges and levels, dark theme](doc/screenshots/badges.png) |

More pictures, page by page, in the [tour](doc/tour.md).

This repository is **Mentor v2**, a rewrite in Rust of a Python/Django application (v1), which stays the
reference while v2 is built. The interface and the courses shipped here are in French; code, documentation and
the course format are in English.

## What works today

- **Catalogue**: 22 courses (Git, Docker, Linux, Python, SQL, web front ends, CI, Kubernetes, Terraform, three
  AWS certification courses…), 140 lessons each with a lab and a quiz, compiled from Markdown. What authors
  write is not trusted: the HTML rendered from it is filtered.
- **Learning**: sign-in (development only, see below), quizzes, lesson and course completion, prerequisites
  between courses, validation exams drawn, timed and graded on the server, XP, levels, badges, a dashboard.
- **Training paths**: ordered sets of courses drawn as a map, with the learner's advancement and next step.
- **Multi-tenancy**: one instance, several organisations resolved by host name, each with its brand; data is
  isolated by PostgreSQL row-level security.
- **Labs proven by replay**: every lab of the catalogue is replayed in its environment, without network, by
  [`tools/replay_labs.py`](tools/replay_labs.py): each check must fail before the solution and hold after it.
- **Course packages**: a course, or a set of courses, as a Git repository with a `mentor.yml` manifest, which
  can be validated and replayed on its own. Each organisation has its own catalogue, made of the packages
  installed for it, from a directory or straight from a repository at a pinned commit; none is offered by
  default. An update says what it changes for learners before it is applied, and can be rolled back.

## What does not work yet

- **Labs cannot be run from the web interface.** Environments are meant to run as Firecracker microVMs
  provided by the [Atelier](https://github.com/PhilippeVienne/atelier) project. The catalogue's environments
  start there and labs were verified through it, but the web server is not connected to it; until then a
  lesson is its text and its quiz. See [doc/atelier-lab-validation.md](doc/atelier-lab-validation.md).
- **Real sign-in.** There is no OIDC yet, only a password-less development sign-in (`--dev-login`) that must
  never be enabled on a reachable deployment.
- Private package repositories, the management area for tutors (only the page of course packages exists),
  the help pages, and the catalogue linter of v1.

## Quick start

You need Rust (stable) and Docker. Full details in [doc/getting-started.md](doc/getting-started.md).

```shell
docker run -d --name mentor-pg -e POSTGRES_PASSWORD=mentor -p 127.0.0.1:5432:5432 postgres:16-alpine

export MENTOR_DATABASE_URL=postgres://postgres:mentor@127.0.0.1:5432/postgres
cargo run -p mentor-cli -- migrate
cargo run -p mentor-cli -- tenant-create demo "Mentor" --host localhost --host 127.0.0.1
cargo run -p mentor-cli -- package-install catalogue --tenant demo    # the courses of this repository
docker exec mentor-pg psql -U postgres -c "CREATE ROLE mentor_web LOGIN PASSWORD 'mentor' IN ROLE mentor_app"

MENTOR_APP_DATABASE_URL=postgres://mentor_web:mentor@127.0.0.1:5432/postgres \
    cargo run -p mentor-web -- --dev-login
```

Then open <http://localhost:8300> and sign in with any user name.

## Documentation

| Document | What it covers |
| --- | --- |
| [doc/tour.md](doc/tour.md) | The product, page by page, with screenshots |
| [doc/getting-started.md](doc/getting-started.md) | Running an instance: database, roles, tenants, brand, sessions |
| [doc/development.md](doc/development.md) | Working on the code: layout, tests, conformance with v1, replaying labs, screenshots |
| [catalogue/README.md](catalogue/README.md) | Writing courses: lessons, labs and their checks, quizzes, exams, environments, training paths |
| [doc/course-packages.md](doc/course-packages.md) | Courses distributed as Git repositories: design and what exists |
| [doc/architecture.md](doc/architecture.md) | Direction, threat model, migration plan from v1, decisions taken |
| [doc/execution-plane.md](doc/execution-plane.md) | How labs are to run on a base shared with Atelier |
| [doc/atelier-lab-validation.md](doc/atelier-lab-validation.md) | What was actually run on Atelier, and what is missing |

## Licence

[GNU AGPL v3](LICENSE) or later.
