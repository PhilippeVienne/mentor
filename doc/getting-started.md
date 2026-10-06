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

A tenant is an organisation: its learners, their progress, its brand. It is served on the host names given
with `--host`; a request for a host name that belongs to no tenant gets a bare 404.

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

Open <http://localhost:8300>, choose « Connexion » and type any user name.

| Option | Environment variable | Default | Meaning |
| --- | --- | --- | --- |
| `--database-url` | `MENTOR_APP_DATABASE_URL` | none, required | Connection of the application role |
| `--listen` | `MENTOR_LISTEN` | `127.0.0.1:8300` | Address the server listens on |
| `--catalogue` | `MENTOR_CATALOGUE` | `catalogue` | Directory of the courses, compiled at start-up |
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
```

<http://acme.localhost:8300> now shows the same catalogue under another name, with its own learners. Colours
and texts are a JSON object in the `branding` column of the `tenant` table; the keys are read in
[`crates/mentor-web/src/brand.rs`](../crates/mentor-web/src/brand.rs). There is no command to edit it yet:

```shell
docker exec mentor-pg psql -U postgres -c \
  "UPDATE tenant SET branding = '{\"primary\": \"#0f766e\", \"tagline\": \"La formation maison\"}' WHERE slug = 'acme'"
```

## What you will not get

- **Labs.** Lessons show their lab as "to come": environments run as microVMs on a separate execution plane
  that the web server is not connected to yet. See [atelier-lab-validation.md](atelier-lab-validation.md).
- **One catalogue per tenant.** Every tenant sees the courses of `catalogue/`.

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
