# Course packages: distributing labs as Git repositories

> **Status: design with a first increment in code, 6 October 2026.** The request: labs must be distributable
> as Git repositories that are packages. Implemented: the manifest, the checks on the files of a package,
> `load_package`, `mentor package-check`, and the built-in catalogue loaded as a package (§10). Designed, not
> implemented: fetching a repository, installing for a tenant, storage, updates, the link with the execution
> plane. Nothing below was tried against Atelier. Decisions that belong to the product owner are in §11.

## 1. The unit: a package holds courses

| Candidate unit | Verdict | Why |
| --- | --- | --- |
| A lab alone | No | A lab is a block inside a lesson. It needs an environment (a folder of its course), and its result is stored under `(course, lesson)`. On its own it can be neither run nor recorded |
| A lesson alone | No | Same: its identity, its environment and its place in the progress rules come from its course |
| **One course** | The smallest unit that works | A course is self-contained: lessons, labs, environments, images, exam |
| **Several courses** | Allowed, same mechanism | Courses linked by `requires` must travel together (§7); the built-in catalogue is 19 of them |

**Decision: a package is a directory holding a `mentor.yml` manifest and one or more course folders, in a Git
repository pinned at a commit.** Usually the directory is the root of the repository; it may be a sub-folder
(a *path*), which is how the built-in `catalogue/` of this repository is a package.

"Distribute a lab" therefore means: a package of one course of one lesson holding one `:::lab`. A lesson
without a quiz is already allowed, so nothing new is needed for a stand-alone lab. Whether such a course
should be *presented* differently to learners ("a lab", outside the course list) is a product question (§11).

The course format does not change: everything in [`catalogue/README.md`](../catalogue/README.md) applies
inside a package, with two differences. `mentor.yml` replaces `catalogue.yml`. A package cannot declare
checks: they are code run by the server, so the table is the platform's (`_checks.yml` in a package is not
read).

## 2. A package repository on disk

```text
git-basics-course/            ← the repository; its name does not matter
├── mentor.yml                ← the manifest (§3)
├── README.md                 ← for people browsing the repository; not compiled
├── LICENSE
└── git-basics/               ← one folder per course, directly under the root; its name is the course slug
    ├── course.md
    ├── 01-introduction.md    ← lessons, with their :::lab and :::quiz blocks
    ├── 02-premier-commit.md
    ├── cheatsheet.md
    ├── exam.md
    ├── images/
    └── environnement/        ← the environment the labs run in
        ├── devcontainer.json
        └── Dockerfile
```

Courses are always sub-folders, even when there is only one: the slug stays the folder name, as today, and a
course moves between packages by moving its folder. A folder the manifest does not list is not compiled, but
it is still part of the package and checked like the rest (§6.2).

## 3. The manifest: `mentor.yml`

YAML, like every other file of the format. **Unknown keys are refused**, unlike front matter today: a
misspelt key must not be ignored silently, and a manifest written for a later format must not half-work.

| Key | Required | Rule | Role |
| --- | :---: | --- | --- |
| `format` | yes | The integer `1` | Version of the package format: these keys **and** the authoring format of courses (front matter, directives, check names). Read first; another value is refused with "unsupported package format" |
| `name` | yes | Lower-case letters, digits, single hyphens; 64 characters at most | Identity of the package for whoever installs it (§4) |
| `version` | yes | `MAJOR.MINOR.PATCH`, optional `-suffix` | For people. What is installed is a commit (§4) |
| `title` | yes | Plain text, 200 characters at most | Display name |
| `summary` | no | Plain text, 500 characters at most | One sentence |
| `license` | yes | Characters of an SPDX expression; not checked against the SPDX list | Licence the content is distributed under (`CC-BY-SA-4.0`, `LicenseRef-Proprietary`) |
| `authors` | no | List of plain texts | `Name <address>` |
| `courses` | yes | 1 to 200 course slugs, each a folder directly under the root, no duplicate | What the package contains, in display order |
| `features` | no | Closed list; today only `docker` | What the environments need that not every platform offers. `docker`: a Docker daemon inside the environment (`customizations.mentor.dockerInDocker`) |
| `build_egress` | no | Host names, `*.` prefix allowed; no scheme, port, path or address | Hosts the image builds reach besides what the platform allows every build. Run time has no network, whatever is written here |

```yaml
format: 1
name: git-basics
version: 1.2.0
title: "Git basics"
summary: "Sept leçons pour passer de zéro à une fusion sans conflit."
license: CC-BY-SA-4.0
authors:
  - "Camille Martin <camille@example.org>"
courses:
  - git-basics
features:
  - docker
build_egress:
  - registry-1.docker.io
  - "*.debian.org"
```

`features` and `build_egress` exist so that a platform can refuse or ask for consent **before** building
anything: "this package needs Docker in its environments, and its builds reach these hosts". The compiler
does not yet cross-check them against the `devcontainer.json` files (an environment asking for Docker in a
package that does not declare `docker` must become an error); that comes with the validation of
`devcontainer.json`, which v2 has not ported yet (architecture §6.1).

Not in format 1, on purpose: `depends` (§7), a content `locale` (architecture §10, open point 3: the place
for it is here), per-course metadata (it stays in `course.md`).

Training paths (`paths.yml`, being added to the compiler by another workstream while this was written) sit
next to the course folders, so a package can carry them at its root; `load_package` does not read that file
yet.

## 4. Identity and versioning

| Thing | Identity | Notes |
| --- | --- | --- |
| Package | `name` of the manifest, unique per tenant | Not the URL: a repository can move or be mirrored without becoming another package |
| Source | URL + commit + path | Where an installed package came from; recorded with it |
| Course | Its slug, **unique per tenant across all installed packages** | Unchanged: URLs (`/courses/<slug>/`), `requires`, progress and awards keep their keys. Installing a package whose course slug already exists in the tenant is refused |
| Lesson | Its `id`, unique in the course | Unchanged |
| Exam question | Digest of the question's rendered HTML | Unchanged (but see below) |

Namespacing courses by package (`<package>/<course>`) was considered and not chosen: it changes every URL
and every stored key for a collision that an install-time refusal handles. It remains possible later.

**Pinning.** An installation asks for a URL and a *ref*: a tag, a branch or a commit. The ref is resolved
**once**, at install time, to a commit identifier, and only that commit is stored, compiled and built. A tag
is a convenience for people, never an identity: tags can be moved. Nothing follows a branch: an update is
always a deliberate act. `version` is what people read in the management area; the platform does not
interpret it beyond showing "1.2.0 → 1.3.0".

**Update** = install another commit of the same `name` over the current one. The new commit is fetched,
verified and compiled next to the old one; nothing changes for learners until it is published. Before
publishing, the platform compares the two compiled packages and reports:

| Change found | Effect on stored data | Proposed rule |
| --- | --- | --- |
| Text, images, hints changed | None | Publish |
| Lesson added | A course that was complete is no longer (completion is derived from the current lesson list). The course award and badge already granted stay: awards are a journal | Publish; the report says how many learners lose "completed" |
| Lesson `id` removed, or course removed | Its `lesson_progress` rows and awards refer to nothing | **Refused unless confirmed.** Rows are kept, never deleted: XP stays, and the lesson comes back with its progress if the id returns. A removed course no longer satisfies `requires` |
| Lab steps added, removed or reordered | Nothing breaks: `tasks_done` holds step **identifiers** (the step's `id`, or a digest of its text) | Lessons already completed stay completed; for a lesson in progress, the steps still present keep their state, new or reworded ones are to do |
| Quiz questions added or removed | `quiz_best` is a count compared with the new total | Completed stays completed; in progress, the learner retakes the quiz |
| Exam pool changed | Open attempts refer to question ids that may be gone | Open attempts on that course are closed without penalty (no cooldown); finished attempts keep their score |
| Environment folder changed | Its image must be rebuilt | Build before publishing (§8); a failed build blocks the update |
| `name` changed | It is another package | Refused as an update |

Rolling back is the same operation with the previous commit; that is why the last published commit is kept.

**Exam question identifiers** digest the question's **source text** (the question and its answers, as
written), not its rendered HTML: a new version of the Markdown renderer does not change them.

## 5. Installing a package for a tenant

```text
URL + ref (+ path)  →  resolve ref to a commit  →  fetch that commit  →  verify the tree (§6.2)
  →  read mentor.yml  →  compile (same compiler)  →  diff with the installed version (§4)
  →  build environments (§8)  →  publish for the tenant
```

| Step | Where it runs | Result |
| --- | --- | --- |
| Resolve, fetch, verify | A job with outbound network and no access to the database or the cluster's internal network; never in `mentor-web` | A directory of plain files, or a refusal |
| Compile | The same job; the compiler needs no network | The compiled package (JSON) and its assets |
| Diff, publish | `mentor-web` or `mentor` (CLI), in one transaction for the tenant | The tenant's catalogue changes atomically |

Storage, to be added to `mentor-db` (not written yet):

| Table | Key | Holds |
| --- | --- | --- |
| `package` | `(tenant_id, name)` | Source URL and path, published commit, position among the tenant's packages, who installed it and when |
| `package_revision` | `(tenant_id, name, commit)` | Manifest, compiled courses (JSON), compiler version, state (`compiled`, `building`, `ready`, `published`, `retired`, `failed`), the diff report |

Both carry `tenant_id` and row-level security like every other table. Compiled courses fit in the database:
the 19 built-in courses compile to 2.7 MB of JSON. Images go to object storage under
`<tenant>/<package>/<commit>/<course>/images/` and stay served at `/static/catalogue/<course>/images/…`, the
address compiled lessons already use. `mentor-web` keeps one compiled catalogue per tenant in memory and
swaps it on publish, instead of one catalogue for everybody.

No sharing of compiled packages between tenants at first: two tenants installing the same commit compile it
twice. It costs a second of compilation and removes a whole class of questions about private repositories.
(Images are shared lower down, by the execution plane: §8.)

**Coexistence with `catalogue/`.** The built-in catalogue is now a package (`catalogue/mentor.yml`, name
`mentor-courses`): source = this repository, path = `catalogue`. The intended end state is that it is
installed like any other package and `catalogue.yml` disappears. Until per-tenant storage exists,
`mentor-web` keeps loading it from disk for every tenant, as today. Whether new tenants get it by default is
the open point 2 of the architecture (§11).

Commands, in the style of the existing ones (`package-check` exists; the others are designed):

```shell
mentor package-check [DIR]                                       # validate a package directory, no database
mentor package-install <url> --ref v1.2.0 --tenant acme [--path sub/folder]
mentor package-update  <name> --ref v1.3.0 --tenant acme [--confirm-removals]
mentor package-list    --tenant acme
mentor package-remove  <name> --tenant acme
```

## 6. Security: a package is untrusted input

The threat model already calls tenant authors untrusted (architecture §3). A package adds the repository
itself as an attack surface, at three moments.

### 6.1 Fetching (designed, not implemented)

**Git operation: the system `git` binary, run as a subprocess of the fetch job.** Not `git2` (libgit2, a C
library, in the address space of the server) and not `gix` (a large dependency tree for one operation). A
subprocess can be confined by the operating system and killed on a time limit; the binary is the reference
implementation and receives security fixes through the distribution. The options named below are from
Git's documentation and must be verified when this is written.

| Risk | Rule |
| --- | --- |
| Server-side request forgery, internal hosts | `https://` only; the host must resolve to public addresses (no loopback, private, link-local or cluster ranges), checked by the job's network policy and not only by the code; redirects are not followed (`http.followRedirects=false`); an operator allow-list of hosts is possible |
| Other transports (`file://`, `ssh://`, `ext::`) | Refused when the URL is parsed, and again by `GIT_ALLOW_PROTOCOL=https` |
| Credentials in the URL | A URL with user information (`https://user:token@…`) is refused: it would end up in logs and in the database. Private repositories use a per-tenant deploy token kept in the secret store and handed to Git through its environment, never written to disk or logged (phase 4) |
| Hooks, configuration, prompts | `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL=/dev/null`, `GIT_TERMINAL_PROMPT=0`, `core.hooksPath=/dev/null`; the fetch goes into a fresh bare repository; **no checkout is ever done** |
| Huge or slow repositories | One commit, no history (`--depth=1`, `--no-tags`); a wall-clock limit; a size-limited scratch volume and a file-size limit on the process, so a refusal does not depend on Git options; `transfer.fsckObjects=true` |
| Submodules, LFS, filters | Never fetched (`--no-recurse-submodules`); a tree holding a submodule entry, a `.gitmodules` or a `filter=` attribute is refused |
| Symbolic links, special files, traversal | The tree is listed (`git ls-tree -r -l`) and checked by the rules of §6.2 **before any file is written**. Files are then written one by one from their blobs (`git cat-file --batch`) under names the platform has just validated, with fixed permissions. Git never writes into a working tree, so no path, link or mode chosen by the author reaches the file system |
| Moving refs | The ref is resolved once; the commit fetched is checked to be the commit resolved |

### 6.2 The files of a package (implemented: `verify_tree`)

The whole directory is walked, without following links, before the compiler opens a file. The same rules
apply to an author's working directory and to a fetched tree.

| Rule | Default limit |
| --- | --- |
| Only ordinary files and folders: no symbolic link, device, socket or FIFO | — |
| Names hold only letters, digits, `.`, `_`, `-`, and do not start with `-` | 100 characters |
| No `.gitmodules`; no `.gitattributes` holding `filter=` | — |
| Files in the package | 5,000 |
| Size of one file | 8 MiB |
| Size of one file the compiler parses (`.md`, `.yml`, `.yaml`, `.json`) | 1 MiB |
| Size of the whole package | 64 MiB |
| Folder depth | 16 |

The built-in catalogue is 828 files and 2.6 MB; its largest file is 78 kB. The limits are a `Limits` value,
so a platform can set its own. The `.git` folder at the root of a working directory is skipped.

### 6.3 Compiling and serving

| Risk | Rule | State |
| --- | --- | --- |
| A manifest entry or an `environment` name that leaves the package (`../x`, `/etc`) | Course entries are slugs; an environment is a folder name, not a path | Implemented |
| A package that defines its own checks | Checks are the platform's table | Implemented |
| YAML alias bombs | Refused by the YAML parser's repetition limit | Verified by a test |
| Scripts in lesson HTML | Sanitised at compile time (existing) | Implemented |
| Compilation that never ends | A time and memory limit on the job; regular expressions use a linear-time engine | Designed |
| Scripts in SVG images | Images are served with `Content-Security-Policy: sandbox` and `X-Content-Type-Options: nosniff`; only image types are served | Designed |
| **Serving more than images** | Only `images/` of listed courses is published | **Not the case today**: see below |
| Malicious `Dockerfile` | Built in a disposable build microVM whose egress is the platform's list plus `build_egress` (§8) | Designed, depends on Atelier |
| Checks and solutions that attack the learner | They run as the learner, inside the learner's own microVM, without network | By design of the execution plane |

**Found while writing this, fixed since**: `mentor-web` used to serve the whole catalogue directory under
`/static/catalogue/`, so `exam.md` with its ticked answers, lesson sources with lab solutions and every
`Dockerfile` could be downloaded. It now serves only the pictures of the `images/` folder of known courses
(`crates/mentor-web/src/assets.rs`), and the compiler refuses a picture referenced outside that folder.

## 7. Dependencies between packages: not in format 1

In format 1, **`requires` in `course.md` can only name courses of the same package**; `load_package` refuses
anything else. Courses that depend on each other are distributed together, which is why a package may hold
several.

Why not yet: a prerequisite gates learners (a course stays locked until its prerequisites are complete). A
prerequisite on a package that is absent, removed later, or updated to a version without that course locks
a course for good, so cross-package `requires` needs a resolver, version constraints and protection against
removal. None of that is needed to distribute courses.

When it is needed, the likely shape is an explicit `depends` key in the manifest (package `name`, optional
source hint), `requires` entries resolved against the package itself and then its declared dependencies, an
install that refuses when a dependency is missing in the tenant, and a removal that refuses when something
depends on the package. Format 1 manifests stay valid: they depend on nothing.

## 8. Environments and the execution plane

Atelier builds an environment from a **Git source**: `repo`, `revision`, `configPath` (its `Workshop`
resource, `devcontainer` field). A package already is one, so the mapping is direct and Mentor needs no
artefact store of its own for images:

| Atelier field | Value for the environment `<env>` of course `<course>` |
| --- | --- |
| `repo` | Source URL of the package |
| `revision` | The pinned **commit** (never a branch or a tag) |
| `configPath` | `<path>/<course>/<env>/devcontainer.json` |
| `egressAllowlist` | Empty at run time; at build time, the platform's build list plus the manifest's `build_egress` |

With the source key proposed in [atelier-image-reuse.md](atelier-image-reuse.md) (URL, commit, config path,
builder version), every learner of a course at a given package commit shares one image, across tenants too.
Three things do not fit yet and have to be raised with Atelier; none was tried:

1. **Atelier copies the whole repository into the image.** Its image builder clones the source repository
   into `/workspaces/<name>` of the rootfs and refreshes it at boot (`ensure_workspace_clone` in
   `crates/image-builder`, read at its commit `9eece24`). For a course package that puts the lesson sources,
   **lab solutions and exam answers** in every learner's microVM. Mentor needs that clone to be optional.
   Until then a package repository must not be used as an Atelier source.
2. **A commit is too coarse a key.** Fixing a typo in a lesson changes the commit, hence the key of every
   environment of the package: 19 rebuilds for the built-in catalogue. Mentor can avoid it on its side: Git
   gives the tree identifier of an environment folder for free (`git rev-parse <commit>:<folder>`); when it
   is unchanged between two commits, Mentor keeps sending the **older commit** as `revision` for that
   environment. This is only sound if a build cannot read outside its folder, so `build.context` must stay
   inside the environment folder (a rule for the `devcontainer.json` validation). Better still would be for
   Atelier to key on that tree identifier.
3. **Build-time and run-time egress must be separate** (already noted in execution-plane.md §7), and
   private repositories need the tenant's credentials at build time without letting another tenant reuse the
   image (atelier-image-reuse.md §4, point 3).

Before a package is published, each of its environments is built once; publication waits for the builds
(§5), so a learner never opens a lab whose image fails to build.

## 9. Working on a package locally

```shell
mentor package-check                                   # in the package directory; or: mentor package-check path/
python tools/replay_labs.py --catalogue path/          # replays every lab of the package in its environment
python tools/replay_labs.py --catalogue path/ git-basics --only 03
```

`package-check` needs no database and no network. It validates the manifest, the files (§6.2) and every
course, stops at the first error naming the file, and prints one line per course (lessons, labs, exam,
environments). `replay_labs.py` reads the course list from `mentor.yml` when there is one. Together they
answer the two questions an author has before tagging a version: does it compile, and does every solution
pass its checks. A package repository can run both in its own CI.

Still missing for authors: a published binary of `mentor` (today: `cargo run -p mentor-cli -- package-check`),
a preview of the rendered lessons without a database, and a `package-init` that copies `_template/`.

## 10. Phases

| Phase | Deliverable | State |
| --- | --- | --- |
| **A. Format** | `mentor.yml`, strict validation, tree checks, `load_package`, `mentor package-check`, the built-in catalogue as a package, the replay tool reading a manifest | **Done** (this increment) |
| **B. One loader** | `mentor-web` and `import-v1` load `catalogue/` through `load_package`; `catalogue.yml` and `_checks.yml` removed; `/static/catalogue/` restricted to images; `features` cross-checked with `devcontainer.json` | Next; no owner decision needed |
| **C. Per-tenant catalogues** | `package` and `package_revision` tables, publish and roll back from a **local directory** (`mentor package-install ./dir --tenant acme`), one catalogue per tenant in `mentor-web` | Needs §11, points 1 and 2 |
| **D. Fetch** | The fetch job (§6.1), install from a public HTTPS URL, the update diff and its rules | Needs §11, points 3 and 4 |
| **E. Environments** | Package sources handed to the execution plane, builds before publication | Blocked on Atelier (§8) |
| **F. Later** | Private repositories, `depends`, a management page for packages, sharing compiled packages | — |

Phase C before D on purpose: installing from a directory exercises storage, publication, diff and rollback
without any network code, and an operator can already serve a tenant's package by cloning it by hand.

What this increment added:

| Where | What |
| --- | --- |
| `crates/mentor-content/src/package.rs` | `Manifest` (strict), `load_manifest`, `load_package`, `load_package_with` |
| `crates/mentor-content/src/tree.rs` | `Limits`, `verify_tree` |
| `crates/mentor-content/src/lab.rs` | `platform_checks`: the check table of the platform, kept identical to `catalogue/_checks.yml` by a test |
| `crates/mentor-content/src/catalogue.rs` | An `environment` is a folder name, never a path (also applies to the built-in catalogue) |
| `crates/mentor-cli` | `mentor package-check [DIR]`, which runs without a database |
| `catalogue/mentor.yml` | The built-in catalogue as the package `mentor-courses`; a test checks it compiles to exactly what `load_catalogue` produces |
| `tools/replay_labs.py` | `--catalogue` accepts a package directory |

## 11. Decisions for the product owner

| # | Question | Default taken here (reversible) | Recommendation |
| --- | --- | --- | --- |
| 1 | Do tenants get the built-in courses? | **Decided (9 October 2026): never by default.** A tenant sees only the packages installed for it; the built-in courses are a package that is offered, not installed. Until per-tenant installation exists (phase C), every tenant still sees them | Build phase C, then stop loading `catalogue/` for every tenant |
| 2 | Is a one-lab package shown as a course? | Yes: it is a course of one lesson | Keep it so until a real stand-alone lab exists to look at; a `kind` in `course.md` can change the presentation later without changing the format |
| 3 | Who may install a package: the platform operator only, or tenant administrators? | Not implemented | Operator only (CLI) through phase D; tenant administrators once builds have per-tenant quotas |
| 4 | When a lab changes under learners who are half-way through it | **Settled**: steps are followed by identifier, so the steps still present keep their state | Nothing more to decide, unless a reworded step should always keep its state (give it an `id`) |
| 5 | Licence of the built-in courses | `AGPL-3.0-or-later`, because that is the `LICENSE` of the repository they are in | Decide whether content gets a licence of its own (Creative Commons is usual for courses); the manifest key is ready |
| 6 | Must a package name be unique beyond a tenant? | No: there is no registry (a marketplace is out of scope, architecture §2) | Keep it per tenant |
| 7 | The key is spelt `license` | American spelling, as SPDX, Cargo and npm write it, although the format says `catalogue` | Keep `license`; changing it later means a format 2 |
