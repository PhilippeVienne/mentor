# Getting started

How to run an instance of Mentor on your machine. Each command was run while writing this page, with other
names for the container and the database.

## Requirements

- Rust, stable toolchain.
- PostgreSQL 16 or later. The examples start one with Docker.

## 1. A database

```shell
docker run -d --name mentor-pg -e POSTGRES_PASSWORD=mentor -p 127.0.0.1:5432:5432 postgres:16-alpine
```

## 2. The schema and a first tenant

The `mentor` command administers a platform. It connects as the role that owns the database:

```shell
export MENTOR_DATABASE_URL=postgres://postgres:mentor@127.0.0.1:5432/postgres
cargo run -p mentor-cli -- migrate
cargo run -p mentor-cli -- tenant-create demo "Mentor" --host localhost --host 127.0.0.1
```

A tenant is an organisation: its learners, their progress, its brand, its courses. It is served on the host
names given with `--host`; a request for a host name that belongs to no tenant gets a bare 404.

A new tenant has **no course**: nothing is offered by default. Install a course package for it, here the
courses of this repository:

```shell
cargo run -p mentor-cli -- package-install catalogue --tenant demo
```

The package is validated and compiled first, then stored for the tenant; a running server shows it at the next
request. Installing again replaces it (a new version, an edited lesson). See [Course packages](#course-packages)
below.

## 3. A role for the web server

The web server must **not** connect as the owner: it connects as an ordinary role that is a member of
`mentor_app`, which the migrations created. That role is subject to row-level security, which is what keeps
one tenant's data out of another's reach; a superuser would bypass it.

```shell
docker exec mentor-pg psql -U postgres -c "CREATE ROLE mentor_web LOGIN PASSWORD 'mentor' IN ROLE mentor_app"
```

## 4. The web server

```shell
MENTOR_APP_DATABASE_URL=postgres://mentor_web:mentor@127.0.0.1:5432/postgres \
    cargo run -p mentor-web -- --dev-login
```

Open <http://localhost:8300>, choose « Connexion » and type any user name. Tick « Administrateur·rice » to
also get the management page of the organisation (`/manage/packages/`).

| Option | Environment variable | Default | Meaning |
| --- | --- | --- | --- |
| `--database-url` | `MENTOR_APP_DATABASE_URL` | none, required | Connection of the application role |
| `--listen` | `MENTOR_LISTEN` | `127.0.0.1:8300` | Address the server listens on |
| `--static-dir` | `MENTOR_STATIC` | `static` | Style sheets, scripts and default brand images |
| `--session-secret` | `MENTOR_SESSION_SECRET` | random | Key that signs session cookies, 32 characters at least |
| `--dev-login` | | off | Mounts `/dev/login`: a user name, no password |

`cargo run -p mentor-web -- --help` prints the exact list.

**`--dev-login` is for development only.** Anyone who reaches the server can sign in as anyone. There is no
other way to sign in yet: OIDC per tenant is not implemented.

Without `--session-secret` a random key is drawn at start-up, so sessions end when the server restarts.
Requests that change something are accepted only from the site's own origin.

## A second tenant, with its brand

```shell
cargo run -p mentor-cli -- tenant-create acme "Acme Academy" --host acme.localhost
cargo run -p mentor-cli -- package-install catalogue --tenant acme
```

<http://acme.localhost:8300> now shows these courses under another name, with its own learners. Colours
and texts are a JSON object in the `branding` column of the `tenant` table; the keys are read in
[`crates/mentor-web/src/brand.rs`](../crates/mentor-web/src/brand.rs). There is no command to edit it yet:

```shell
docker exec mentor-pg psql -U postgres -c \
  "UPDATE tenant SET branding = '{\"primary\": \"#0f766e\", \"tagline\": \"La formation maison\"}' WHERE slug = 'acme'"
```

## What you will not get

- **Labs.** Lessons show their lab as "to come": environments run as microVMs on a separate execution plane
  that the web server is not connected to yet. See [atelier-lab-validation.md](atelier-lab-validation.md).
- **Private repositories.** A package is fetched from a public `https://` repository, or installed from a
  directory you cloned yourself.

## Course packages

```shell
mentor package-check ./my-courses                    # validate and compile, without a database
mentor package-install ./my-courses --tenant acme    # install, or replace the installed package of that name
mentor package-install https://github.com/acme/courses --ref v1.2.0 --tenant acme   # from a repository
mentor package-install https://github.com/acme/all --ref main --path courses/git --tenant acme
mentor package-install ./my-courses --tenant acme --dry-run     # only say what it would change for learners
mentor package-rollback my-courses --tenant acme     # bring back the version it had before the last update
mentor package-list --tenant acme
mentor package-remove my-courses --tenant acme       # learners' progress on its courses is kept
```

From a repository, `--ref` is a tag, a branch or a commit identifier (the default branch without it) and
`--path` the folder of the package when it is not at the root. The reference is resolved **once**, to a
commit, which is what gets installed and recorded: nothing follows a branch afterwards, an update is always a
command you run. Only public `https://` repositories are accepted, the host must have public addresses, and
the repository is never checked out: its files are validated, then written one by one
([course-packages.md §6.1](course-packages.md#61-fetching)). It needs `git` on the machine that runs the
command.

Replacing an installed package is an update that learners live through. The command first says what changes
for them: courses and lessons added or removed, lab steps and quizzes that changed, how many learners did
something in what is removed, how many had completed a course that gains lessons. An update that **takes away
a course or a lesson is refused** unless you pass `--confirm-removals`; nothing a learner did is ever deleted,
and it shows again if the lesson comes back. The replaced version is kept, one version back, for
`package-rollback`.

A package is a directory holding a `mentor.yml` manifest, one folder per course and, optionally, a
`paths.yml` of training paths: see [course-packages.md](course-packages.md) and the
[authoring guide](../catalogue/README.md). Two packages of a tenant cannot bring a course or a training path
of the same name. Only the compiled courses and the pictures of their `images/` folders are stored: lesson
sources, lab solutions and exam answers are never served.

## Bringing the data of a v1 portal

```shell
# on the v1 side: Django's own export
python manage.py dumpdata auth.user training.lesson training.lessonprogress training.xpevent \
    training.userbadge training.cohort training.cohortmembership training.examattempt -o v1.json

# on the v2 side, as the owner of the database
cargo run -p mentor-cli -- import-v1 v1.json --tenant acme \
    --v1-catalogue conformance/v1-catalogue.json --catalogue catalogue
```

The import runs in one transaction and can be replayed; dates are kept. On v1's demonstration data (5 users,
210 XP events) XP totals, badges and completed lessons came out identical. Imported accounts get a placeholder
identity (`v1:<username>`) that a real sign-in will have to reconcile; that part does not exist yet.
