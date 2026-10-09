# Validation: running Mentor labs in Atelier microVMs

> **Status: trial run on 6 October 2026, on Atelier's local development stack. Nothing was changed in Atelier's
> repository; nothing left the machine.** It is the "second trial" announced in
> [execution-plane.md §8](execution-plane.md#8-next-steps) and answers open points 4 and 5 of
> [architecture.md §10](architecture.md#10-decisions-taken-and-open-points).
>
> Every verdict below comes from a command that was run, unless it says "read in the code". One run of each
> measure: orders of magnitude, not benchmarks.

## 1. Verdict

**Mentor labs run in Atelier microVMs, verified by the server through Atelier's exec path, with no network:
13 labs (62 steps) of `git-basics` and `python` passed, and Docker works inside the guest. Not as they are
today, though.** No environment of the catalogue starts unmodified, each of Mentor's rules (learner account,
working folder, no network, time limit, bounded output) has to be rebuilt by Mentor around an exec made for
coding agents, and three gaps have no workaround on Mentor's side: the image is rebuilt for every session, the
session disk cannot be sized, and the web terminal runs as root on images without systemd. Mentor can build on
Atelier; it cannot rely on it yet.

| Labs replayed (checks must fail, solution, checks must hold; `tools/replay_labs.py` logic) | Result |
| --- | --- |
| `git-basics`, 7 labs, 31 steps, environment with 3 changes (§3) | **all passed**, 4.5 s to 12 s per lab |
| `python`, 6 labs, 31 steps, environment with 3 changes | **all passed**, 7 s to 11 s per lab |
| `docker-hello` lab 1, 3 steps, environment with 6 changes | **passed** |
| `docker-hello` labs 2 to 4 | **failed**: port 8080 is taken in the guest, and the disk is full (§5) |
| `git-basics`, environment unmodified | **never started**: stayed in `Provisioning` for more than 10 minutes |

## 2. What was run, and how

Machine and versions: the kind cluster `atelier-dev` left by the first trial (its pods, registry, port-forwards
and the pod-network route were still up), Atelier at `9eece24` (controller and API server from source; the
in-cluster images `atelier-*:dev` are those of the first trial), Mentor at `e0e78d0`. Guest kernel 5.10.223,
Firecracker under the jailer.

All files are in the scratch folder of the session, called `$V` here:
`the session scratch folder (atelier-validation/)`
(it is a temporary folder: copy it if it must be kept).

### 2.1 Two local patches, in a copy

Atelier's sources were copied to `$V/atelier-src` and patched there; the repository itself is untouched. The
build output was removed afterwards (`CARGO_TARGET_DIR=$V/target cargo build -p atelier-api-server -p
atelier-controller` rebuilds it in about a minute).

| Patch | File | Why |
| --- | --- | --- |
| `$V/api-listen-addr.patch` | `crates/api-server/src/main.rs` | The API server binds `0.0.0.0:8080`, fixed in the code; that port is taken on this machine by `portail-formation-keycloak`. The patch reads `ATELIER_API_LISTEN_ADDR` |
| `$V/controller-console-debug.patch` | `crates/controller/src/reconcile.rs` | The guest console is only logged at debug level and nothing lets a Workshop ask for it. The patch sets `RUST_LOG` on the supervisor of Workshops named `*-dbg`. Without the console, the Docker failures of §5 could not be diagnosed |

### 2.2 A git source the builder can reach, without publishing anything

Atelier builds from a Git URL and Mentor's repository is not published. What was done:

1. A scratch repository holding **only** the three environment folders of Mentor `e0e78d0`, plus their variants
   (`$V/git/work`, bare copy in `$V/git/srv/mentor/mentor-catalogue.git`).
2. A 60-line read-only server wrapping `git http-backend` (`$V/git/serve.py`), bound to the kind bridge only:
   `python3 serve.py 172.19.0.1 8765 $V/git/srv`. envbuilder needs the smart HTTP protocol, a static file
   server is not enough.
3. A Kubernetes `Service` without selector plus an `Endpoints` object pointing to `172.19.0.1:8765`
   (`mentor-git-src`), the pattern Atelier's stack already uses for its host-side API server.
4. The controller started with `ATELIER_GIT_HOST_SERVICE=mentor-git-src ATELIER_GIT_HOST_PORT=8765`: Atelier
   then routes the name `git.atelier.internal` to that Service for the build, **outside the egress allow-list**.
   Workshops use `repo: http://git.atelier.internal/mentor/mentor-catalogue.git`.

Using the Forgejo of Atelier's stack was the first idea; pushing the unpublished repository there was refused
by the session's permission rules, and the local server is the narrower solution anyway.

### 2.3 Starting the stack and driving it

```sh
source $V/env.sh        # Atelier's own env file, LLM proxy variables unset, git alias, listen address
DATABASE_URL="$ATELIER_DATABASE_URL_CONTROLLER" $V/target/debug/atelier-controller &
(cd ~/github.com/PhilippeVienne/atelier &&      # the service-account token path of the env file is relative
  DATABASE_URL="$ATELIER_DATABASE_URL_API_SERVER" $V/target/debug/atelier-api-server &)
python3 $V/create.py mv-git-b variants/git-basics/devcontainer.json "$(cat $V/allow-debian.txt)"
kubectl patch workshop mv-git-b --type merge -p '{"spec":{"egressAllowlist":[]}}'   # then suspend + resume
python3 $V/replay_atelier.py mv-git-b git-basics 01-introduction.md --reset
```

- `$V/atl.py`: client of the API (REST for the lifecycle, the MCP tool `exec_in_workshop` to start a command,
  the SSE stream to read its result).
- `$V/replay_atelier.py`: imports Mentor's `tools/replay_labs.py` and replaces only the transport: each setup
  file, setup command, solution and check is **one `exec_in_workshop` call**. Around each command it adds what
  Mentor's server would have to add (§4): `cd /workspace`, `HOME`, `USER`, the `containerEnv`, `timeout`,
  `</dev/null`, and files written through `base64 -d` since exec has no standard input.
- Logs of every run: `$V/lab-*.log`, `$V/create-*.log`, `$V/semantics*.log`, `$V/tenancy.log`,
  `$V/net-with-allowlist.txt`, `$V/console-docker*-dbg.txt`.

## 3. What had to change in the environments

The variants are in `$V/git/work/variants/`. Each line is a reason an unmodified environment fails.

| Change in the Dockerfile | Why | Observed without it |
| --- | --- | --- |
| Add `systemd-sysv` (or remove systemd entirely) | `openssh-server` pulls the `systemd` package without `/sbin/init`. Atelier sees the systemd binary, concludes systemd is the init and does not install its own; the kernel finds no init | Workshop stuck in `Provisioning`, no error anywhere (`git-basics` unmodified) |
| Add `curl` | Atelier's injected start scripts fetch the SSH key and the session password with `curl` | Read in the code (`inject_sshd`, `inject_terminal_and_ide`); `git-basics` has no `curl` |
| `RUN chown apprenant:apprenant /workspace` | Atelier mounts nothing on the working folder: it is the image's, owned by root | `/workspace` not writable by the learner |
| Docker only: an alias account and group `vscode` with uid and gid 1000, in the `docker` group | Exec logs in as `vscode`, always. Atelier adds that account when missing, with **no supplementary group** | Without the group alias, the terminal and VS Code units restart in a loop (40 restarts in 90 s) and the Workshop never reaches `Running` |
| Docker only: `update-alternatives --set iptables /usr/sbin/iptables-legacy` | The guest kernel has no `CONFIG_NF_TABLES` | `dockerd`: `failed to create NAT chain DOCKER: iptables: Failed to initialize nft: Protocol not supported` |
| Docker only: `Environment=DOCKER_INSECURE_NO_IPTABLES_RAW=1` for `docker.service` | The guest kernel has no `CONFIG_IP_NF_RAW` | `docker run`: `can't initialize iptables table 'raw'` (`--network none` and `--network host` did work) |

`python` was run with systemd removed instead (no `openssh-server`), to exercise Atelier's own init.

## 4. Mentor's rules, one by one

| Requirement | Verdict | Evidence |
| --- | --- | --- |
| A lab end to end through exec | **Works with workarounds** (§3, and the wrapper of §2.3) | 13 labs, 62 steps, 0 failure |
| Commands run as uid 1000 | **Works** | `id` → `uid=1000(apprenant) gid=1000(apprenant)`. The login name is `vscode`, `HOME=/home/vscode`, `USER=vscode`: Mentor must set them back |
| The learner cannot become root | **Works through exec; does not hold in the web terminal without systemd** | No `sudo` (exit 127), `su` fails, no capability (`CapEff: 0`). But `NoNewPrivs: 0` and the image's 13 setuid binaries (`su`, `mount`, `passwd`…) are live: the rule "no setuid bit" has to hold in the image. **On an image without systemd, `ttyd` (the web terminal) and `code-server` run as root** (`ps` in the `python` variant); with systemd they run as uid 1000 |
| No network at run time, network at build time | **Works with a workaround** | One allow-list serves both. With the build list, `deb.debian.org` stayed reachable from the lab. Patching `spec.egressAllowlist` to `[]` changes nothing on a running Workshop; after a **suspend and resume** it applies: every HTTP(S) request answered 403, no DNS, direct IP connections reset. Cost: about 60 s |
| What else the guest can reach | **Fine** | Other Workshops' pods, the Kubernetes API, the node, OpenBao, PostgreSQL, the registry, the host: all refused or timed out. Reachable: the pod's metadata server (the session's own SSH public key and terminal password) and the `identity-proxy` and `mcp-gateway` aliases |
| A build with a narrow allow-list | **Works** | Docker Hub hosts, `deb.debian.org`, `download.docker.com`, PyPI were enough; the git source and the internal registry need no entry. The first trial's `*` was not needed |
| Empty home and working folder | **Does not work** | The home is `/home/vscode` with Atelier's files; the working folder is the image's. **The whole git source is cloned into `/workspaces/<repository>` and readable by the learner**; exec starts there, and VS Code opens it. A catalogue repository holds the solutions |
| Writable, with a disk limit | **A limit exists, but cannot be chosen** | Root file system = image content + 512 MiB, written as one ext4 file: 352 MiB free (`git-basics`), 369 MiB (`python`), 321 MiB (`docker-hello`). `dd` stopped at 352 MiB with "no space left". `resources.disk` is never read by the controller (read in the code). With systemd, `/tmp` is a tmpfs of half the memory |
| Exit codes | **Works, with a caveat** | 0, 1, 2, 42, 127, 255 came back exact. A command killed by a signal returns **no exit code** (`null`), with status `Completed` |
| Standard output up to 64 KiB | **Works, but nothing bounds it** | 64 KiB, 1 MiB and 8 MiB came back intact (8 MiB in 4.5 s, all stored in PostgreSQL). Mentor must cut it itself. Invalid UTF-8 is replaced; **an output containing a NUL byte comes back empty**; `\r\n` becomes two line breaks |
| Standard error | **Unreliable** | Lost whenever standard output and standard error arrive in the same 300 ms poll (`echo out; echo err >&2` → `err` missing). Checks do not use it; error messages shown to authors would |
| A time limit | **Works with a workaround** | Atelier's own ceiling is 20 minutes and is not per call (set to 5 s for the test: exit 124 after 5.2 s). Mentor wraps each command in `timeout`: `timeout 2 sleep 30` → 124 after 2.2 s |
| Commands that wait or leave something behind | **Two traps** | Standard input is never closed: `cat` hung until it was killed (316 s). A background process that keeps the output open holds the exec open after the command ended, ceiling included: 602 s for a `sleep 602 &`. Mentor must add `</dev/null` and run its own client-side timeout |
| State between two execs | **Works** | Same VM: files persist, background processes survive (re-parented to init), environment variables and the current folder do not. After a suspend and resume, a file in `/tmp` and the processes were still there |
| Several checks at once | **Works** | 10 execs of `sleep 1` in parallel: 1.3 s in total |
| Docker in the guest | **Works with workarounds** (§5) | Daemon as root under systemd, learner through the `docker` group, mirror started, `docker run hello-world` offline |
| Terminal and VS Code tunnels | **Only reachability tested** | `GET /v1/workshops/<name>/terminal/` and `/vscode/` answered 200 for the owner, 404 for another group, 401 without a token. No interactive session was driven. The terminal's shell starts in `/` with `HOME=/home/vscode`, not in the working folder |
| `postStartCommand`, `containerEnv`, `workspaceFolder`, `remoteUser`, `hostRequirements` | **Ignored by Atelier** | None was applied; the replay tool applied them itself |

## 5. Docker inside the microVM

With the six changes of §3, in a 2 CPU, 2 GiB Workshop:

- `dockerd` 29.8.2 starts under systemd as root, two seconds after boot ("Daemon has completed
  initialization"), with the `overlayfs` containerd snapshotter, cgroup v2 and the systemd cgroup driver.
- The learner (uid 1000, group `docker` through the alias account) reaches `/var/run/docker.sock`.
- `mentor-docker` starts the local mirror and reports "Docker est prêt."; the mirror lists the four images.
- **With no network**, `docker run --rm hello-world` pulls from the mirror and prints "Hello from Docker!";
  `docker run -d -p 8081:80 nginx` then `curl` → HTTP 200; `docker pull busybox` fails, as it should.
- Lab 1 of `docker-hello` passes. Labs 2 to 4 fail for two reasons that are Atelier's:
  - **port 8080 is taken in the guest** by Atelier's `code-server` (`failed to bind host port 0.0.0.0:8080`);
    7681 (`ttyd`) and 2222 (`sshd`) are reserved the same way;
  - **the disk is full**: 321 MiB free is not enough to unpack `nginx` and `postgres` next to the mirror
    (`write /dev/stdout: no space left on device`).

The guest kernel has overlayfs, bridge, veth, the namespaces, cgroup v2 and legacy iptables with NAT. Missing,
in the kernel configuration used by Atelier's supervisor image: `CONFIG_NF_TABLES`, `CONFIG_IP_NF_RAW`,
`CONFIG_IP6_NF_RAW`, `CONFIG_NETFILTER_XT_MARK`, `CONFIG_NETFILTER_XT_MATCH_COMMENT`, `CONFIG_IP_SET`,
`CONFIG_IP_VS`, `CONFIG_VXLAN`. The first two are what Docker tripped on; the others would matter for
Kubernetes-in-the-guest courses and were not tested.

## 6. Measurements

| Measure | Result |
| --- | --- |
| First Workshop of `git-basics` (1 CPU, 768 MiB): creation → `Running` | 66 s and 69 s (build 49 s to 53 s, then 16 s) |
| **Second Workshop of the same source and commit** | **78 s: the image was rebuilt (62 s, a new digest), then 16 s** |
| First Workshop of `python` without systemd | 43 s (build 27 s) |
| First Workshop of `docker-hello` (2 CPU, 2 GiB) | 126 s to 142 s (build 110 s to 125 s) |
| Image ready → `Running`, and resume → `Running` | 15.5 s to 16.6 s, every time (12 measures) |
| Suspend | 41 s to 44 s (768 MiB), 57 s (2 GiB) |
| One exec round trip (`true`, 25 runs) | median 0.344 s, 0.337 s to 0.351 s; the call that starts it takes 0.031 s, the rest is the 300 ms polling of the result stream |
| One check inside a lab | median 0.34 s; 0.64 s when the command lasts longer than a poll |
| A whole lab (setup, then each step: checks, solution, checks) | 4.5 s to 12 s for 13 to 29 execs (`git-basics`); 7 s to 11 s (`python`) |
| Guest clock after a resume | 46 s behind the host: the time spent suspended (same defect as the first trial) |
| Memory of the three proxies of a pod | 11 MB to 15 MB together |
| Memory of the supervisor, microVM included | 268 MB to 379 MB (768 MiB guests), 969 MB (2 GiB guest running Docker) |
| Image cache used by the ten builds of this trial | about 17 GB; the eviction job of the development stack cannot pull its image and never runs |

Against [execution-plane.md §7](execution-plane.md#7-first-real-trial): the rebuild for a second sandbox, the
16 s of provisioning, the suspend and resume times, the exec round trip and the clock defect are **confirmed**.
The builds are faster here (smaller images). Two findings are **corrected**: an empty allow-list is not fatal
to a build once the git source goes through Atelier's git alias and the registries are listed; and group
isolation is weaker than reported (§7).

## 7. Tenancy

What isolates one organisation's Workshops from another's today:

- **The owner group, in the API.** A user of another group gets 404 on get, events, suspend, delete, exec, the
  exec result stream, the terminal and VS Code. An exec result asked under another Workshop's name is not
  found either.
- **But the list is not filtered by group**: `GET /v1/workshops` returns the Workshops the caller created, and
  **all Workshops of every group to anyone holding the realm role `admin`**, with their full specification
  (observed: the `admin` user of group `atelier-core` listed the four Workshops of `atelier-demo`). A tenant's
  administrator must never hold that role.
- **The microVM and its proxy, on the network.** From a guest, another Workshop's pod, the node and the cluster
  services did not answer.
- **Nothing at the Kubernetes level.** All Workshops live in the namespace `default`; there is no
  `NetworkPolicy` (`kubectl get networkpolicy -A`: none). The port-forward port of a Workshop's proxy answered
  a plain HTTP request from an unrelated pod; whether a tunnel to the guest can be opened from there without a
  credential was not tested (the controller's own probe sends none, read in the code).

## 8. Gaps, and what would close each

| # | Gap | Closed in | By |
| --- | --- | --- | --- |
| 1 | The image is rebuilt for every Workshop | Atelier | [atelier-image-reuse.md](atelier-image-reuse.md) (Atelier's spec 18) |
| 2 | 16 s from image to `Running`, 60 s to cut the network | Atelier | No copy of the root file system per VM; template snapshots; see 4 |
| 3 | The session disk is "image + 512 MiB", `resources.disk` is ignored | Atelier | A writable layer sized by `resources.disk` |
| 4 | One allow-list for build and run; no change without suspend and resume | Atelier | A build-time policy separate from the run-time one |
| 5 | Exec: fixed account `vscode`, fixed folder, no environment, no standard input, no per-call time limit, no output limit, polling at 300 ms, standard error lost, NUL bytes, no exit code after a signal, MCP only | Atelier (base API) | A plain "run a command" call: user, working folder, environment, timeout, output cap, exit code and signal, synchronous answer. Until then Mentor wraps every command, as `replay_atelier.py` does |
| 6 | Exec needs OpenBao (the SSH key of each Workshop is read from it); MCP sessions and access tokens expire after about 5 minutes | Atelier (base) | The vsock channel proposed in execution-plane.md §2; Mentor re-authenticates meanwhile |
| 7 | The git source is cloned into the image and shown to the learner | Atelier, or Mentor | An option to skip the workspace clone. Meanwhile Mentor must build environments from a repository that holds **only** environments, never the lessons |
| 8 | Web terminal and VS Code as root without systemd; 8080, 7681 and 2222 taken in the guest | Atelier | Drop privileges in its own init; configurable or unusual ports; an option not to inject the IDE |
| 9 | An image with systemd installed but no `/sbin/init` never starts, silently; `curl` and `bash` are required but not checked | Atelier, and Mentor | Atelier: test `/sbin/init`, fail the build with a message. Mentor: a compiler rule for environments (an init or none, `curl`, `bash`, `timeout`, working folder owned by the learner) |
| 10 | The guest console cannot be read | Atelier | Expose it per Workshop (this trial needed a patch) |
| 11 | Guest kernel without `nf_tables` and the iptables `raw` table | Atelier | Rebuild the guest kernel with them; Mentor's Docker environments carry the two workarounds meanwhile |
| 12 | Devcontainer fields ignored (`remoteUser`, `workspaceFolder`, `containerEnv`, `postStartCommand`, `hostRequirements`) | Mentor, or the base | Mentor's orchestration applies them (it reads the file anyway), or the base learns them |
| 13 | Home and working folder are not fresh | Atelier, or Mentor | A per-session volume; or one VM per lab session, never reused (Mentor's choice, costly until 1 and 2 are closed) |
| 14 | Tenancy stops at the API | Atelier (base) | A namespace and a `NetworkPolicy` per tenant; the list filtered by group |
| 15 | No idle or maximum lifetime; the clock drifts after resume | Atelier, or Mentor | Mentor can drive suspend and delete; the clock needs the base |
| 16 | The API port is fixed at 8080 | Atelier | A listen-address setting (the first patch of §2.1) |

## 9. Not tested

- An interactive terminal or VS Code session, and what a learner can do from VS Code.
- `catalogue/python/environnement` and `catalogue/docker-hello/environnement` unmodified (only `git-basics`
  was; the same cause applies to any image that installs `openssh-server`).
- A private git source with credentials; a build with a truly empty allow-list.
- Many concurrent Workshops, and a burst of sessions of one environment.
- Whether a pod of the cluster can tunnel into a guest through the port-forward port.
- Environments other than these three; Kubernetes inside the guest.
- A production install (Helm chart, controller in the cluster): this was the development stack.

## 10. Conclusion and next steps

> Since this was written, item 4 was done in Mentor: every environment of the catalogue carries `systemd-sysv`,
> `curl` and a working folder owned by the learner (and the Docker ones the three changes of §3), the compiler
> warns when a `Dockerfile` lacks one of them, and all labs were replayed with `tools/replay_labs.py`. Five of
> these environments were then started unmodified in Atelier: see [§11](#11-second-run-the-catalogues-environments-unmodified-6-october-2026).


Atelier's isolation, image builder and exec are real and did the job: a learner's command ran as uid 1000 in
a Firecracker microVM with no way out, and Mentor's server verified 62 lab steps through it. What is missing is
the layer between "a sandbox for one agent" and "a lab session for a learner", which is the base that
[execution-plane.md §2](execution-plane.md#2-where-the-line-could-be-drawn) proposes to extract. Until gaps
1, 3 and 8 are closed, Mentor cannot offer labs on it: a lab would open after one to two minutes, a Docker
course does not fit on the disk, and a learner of a course without systemd is root in the terminal.

1. **Bring gaps 1, 3, 4 and 8 to Atelier** with this document: image reuse (spec 18 is already proposed),
   `resources.disk`, a build-time egress policy, non-root services under its own init.
2. **Specify the base's "run a command" call** (gap 5) from the wrapper of `replay_atelier.py`, and decide the
   exec channel (gap 6).
3. **Report the defects found on the way**: silent non-start without `/sbin/init`, standard error lost in the
   result stream, NUL bytes, standard input never closed, the list endpoint and the `admin` role, the guest
   clock, the kernel options.
4. **In Mentor, now**: add the environment rules of gap 9 to the compiler and fix every environment of the
   catalogue (`systemd-sysv` or no systemd, `curl`, working folder owned by the learner); decide that
   environments are built from a repository without lessons (gap 7); move the lab ports away from 8080 or
   wait for gap 8.
5. **Turn `replay_atelier.py` into an option of `tools/replay_labs.py`**, so that the whole catalogue (107
   labs) can be replayed against Atelier after each of its releases.
6. **Trial 3**, once image reuse exists: thirty sessions of one environment started together, with the
   terminal driven for real.

## 11. Second run: the catalogue's environments, unmodified (6 October 2026)

Same machine, same method and same two local patches as §2, on Mentor `fe3e854`. The scratch git source was
rebuilt from the committed environment folders of five courses, **with no variant and no lesson**
(`$V/git2`); logs are in `$V/run2/`. Each Workshop was created with the CPU and memory its
`hostRequirements` ask for, its network was cut (allow-list emptied, suspend, resume), then every lab of the
course was replayed through `replay_atelier.py --commit fe3e854 --reset`.

### 11.1 Result

**All five environments start unmodified.** `git-basics`, `python` and `aws-cloud-practitioner` pass all their
labs; the two Docker courses start and Docker works, but most of their labs still fail, for three causes of
which one is new and Mentor's.

| Environment (guest) | Reaches `Running` | Labs replayed | Steps passed |
| --- | --- | --- | --- |
| `git-basics` (1 CPU, 512 MiB) | yes, 108 s ¹ | 7 of 7 pass | 31 of 31 |
| `python` (1 CPU, 512 MiB) | yes, 106 s ¹ | 6 of 6 pass | 31 of 31 |
| `aws-cloud-practitioner` (1 CPU, 512 MiB) | yes, 170 s ¹ | 12 of 12 pass | 59 of 59 |
| `docker-hello` (2 CPU, 2 GiB) | yes, 178 s ¹ | 1 of 4 passes | 8 of 19 |
| `docker-advanced` (2 CPU, 2 GiB) | yes, 140 s | 1 of 5 passes | 10 of 26 |

¹ The first four images were built at the same time; alone, a build takes about half of that (§6). Image
ready → `Running` took 16 s every time.

In all five (`ps`, `id`, `touch` through exec): systemd is the init; **the web terminal (`ttyd`) and
`code-server` run as uid 1000**; exec runs as uid 1000; **`/workspace` belongs to the learner and is
writable**. In the two Docker environments the learner has the `docker` group through exec, in the terminal
and in VS Code, `HOME` is `/home/apprenant`, `dockerd` 29.8.2 is up and `mentor-docker` answers "Docker est
prêt." with no network.

### 11.2 The AWS environment, first time in a microVM

- All 12 labs pass with no network, 18 s to 40 s per lab; a check takes about 1.25 s (the start-up time of
  the `aws` command), against 0.34 s elsewhere.
- **Nothing starts the emulator by itself**: Atelier ignores `postStartCommand` and no systemd unit exists
  for it. It was started by `demarrer-aws` through exec (once, as the platform would, and again by each lab's
  setup commands). It then runs **as uid 1000**, re-parented to init, and outlives the exec that started it.
- Memory: MiniStack 95 MB resident, plus 38 MB for a Lambda worker; 190 MB used and 295 MB available in the
  512 MiB guest after the twelve labs, Atelier's `code-server` included. 512 MiB is enough.
- It survives a suspend and resume **with its state**: a bucket created before was listed after.
- Its state is never reset between labs in one VM (it lives in memory): the twelve labs passed in a row on
  the same emulator, which is what one long session would do.

### 11.3 What still fails in the Docker courses, step by step

| Cause | Where it belongs | `docker-hello` | `docker-advanced` |
| --- | --- | --- | --- |
| **Port 8080 taken in the guest** by Atelier's `code-server` (`failed to bind host port 0.0.0.0:8080/tcp: address already in use`) | Atelier (gap 8), or the labs | Lab 2 step 2, and steps 3, 4, 5 which wait for it; lab 3 setup command, step 2, and steps 3 and 6 which wait for it | Lab 3 step 3, and steps 4 and 5 which wait for it |
| **Disk full**: 322 MiB and 276 MiB free at start (`no space left on device` while unpacking `postgres`, building, or writing a file) | Atelier (gap 3) | Lab 4 step 2 (`postgres` cannot be unpacked), steps 3 and 4 which wait for it, step 5 | Lab 1 step 5; lab 3 step 2; lab 4 steps 1 and 4, and steps 2, 5, 6 which wait for them; lab 5 step 2, and steps 3 and 4 which need its image |
| **Port 5000 taken in the guest by Debian's own `docker-registry.service`**: the `docker-registry` package ships an enabled unit, which systemd now starts (in a plain container nothing did). It answers 401 like the course's registry, so `mentor-docker registre` believes its registry is up and `mentor-docker sans-registre` cannot stop it | **Mentor's environment** | none (port 5000 is not used) | Lab 1 step 3 (`-p 5000:5000`: address already in use) and step 4 which waits for it; lab 5 step 5 (`docker login localhost:5000` → 401) and step 6 |

Passing: `docker-hello` lab 1, lab 2 step 1, lab 3 steps 1, 4, 5, lab 4 step 1; `docker-advanced` lab 2
entirely (volumes), lab 1 steps 1 and 2 (an image is built), lab 3 step 1, lab 4 step 3, lab 5 step 1.

### 11.4 What still has to change in the environments

1. **The three Docker environments** (`docker-hello`, `docker-advanced`, `_template/environnement-docker`):
   stop Debian's packaged registry from starting, for instance
   `RUN ln -s /dev/null /etc/systemd/system/docker-registry.service` after the package is installed. Not
   verified by a run. The same reasoning applies to `ssh.service` of `openssh-server`, which now starts in
   every environment (its start was seen on the guest console in the first run); it is harmless, since port 22
   is not reachable from outside the guest.
2. **Labs that publish port 8080** (`docker-hello` labs 2 and 3, `docker-advanced` lab 3): use another port,
   or wait for Atelier to free it. Port 8081 was checked in the first run (§5).
3. **Nothing for the disk**: no change in an environment gives a session more than "image + 512 MiB". The
   Docker courses cannot run fully until Atelier honours `resources.disk` (`docker-advanced` asks for 4 GB).
4. **Nothing in the AWS environment.** Mentor's server has to run `postStartCommand` itself when a session
   starts, there as everywhere.
5. Optional, all non-Docker environments: with exec and in the terminal, `HOME` is `/home/vscode` and the
   login name `vscode` (§4). The alias account the Docker environments already carry (`vscode`, uid and gid
   1000, home `/home/apprenant`) fixes both; it was only observed there.

Unchanged from the first run, and not environment matters: the image is rebuilt for every Workshop, the git
source is cloned into `/workspaces/` (here it held only environments), cutting the network costs a suspend and
resume (37 s to 82 s to suspend, 16 s to 18 s to resume, four at once).

## 12. Third run: disk sizing in a guest (9 October 2026)

Atelier at `e80d58d` (branch `feat/m14-1-disque-de-session`, local: tasks 14.1 "session disk" and 14.2
"build-time checks" of its spec 19), Mentor at `f3df7ce`. Same machine and method as §2. The controller and
the API server were built from a fresh copy of that branch with the two patches of §2.1; the in-cluster
images `atelier-vm-supervisor:dev` and `atelier-image-builder:dev` were rebuilt from the branch and loaded
into the kind cluster (the previous ones are kept on the host as `atelier-vm-supervisor:first-trial-aec2855`
and `atelier-image-builder:first-trial-5ea5d3f`). Guests are `git-basics` (1 CPU, 512 MiB) unless said
otherwise. Logs: `$V/run3/`.

### 12.1 Verdicts

| # | Point | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | Without `resources.disk`, nothing changes | **Holds** | No `ATELIER_VM_DISK_MIB` on the supervisor; `df -m /` → 1412 MiB, 352 MiB free, as in §4 |
| 2 | `disk: 4Gi` gives a 4 GiB disk | **Works** | Supervisor: `disque racine de la microVM agrandi from_mib=1471 to_mib=4096`; `df -h /` → `4.0G  971M  2.8G`; `/sys/block/vda/size` = 8388608 sectors. `dd` wrote 1 GiB (past the old 352 MiB), then 1.7 GiB more and stopped with the disk at 3776 of 3997 MiB (the 5 % ext4 keeps for root) |
| 2 | Cost of the resize | **Not measurable from outside** | Image ready → `Running`: 15.6 s without, 15.9 s with 4 GiB, 15.8 s and 15.9 s with 6 GiB (15.5 s to 16.6 s before). Inside the supervisor, restore took 2.5 s without and 5.2 s with a 4 GiB resize: about 3 s, hidden by the controller's probing |
| 3 | The cluster cap | **Works** | Controller started with `ATELIER_VM_MAX_DISK_MIB=3072`, `disk: 8Gi` → supervisor gets `3072`, log `to_mib=3072`, `df -h /` → `3.0G` |
| 4 | Smaller than the image | **Works** | `disk: 64Mi` → warning `taille de disque demandee inferieure a celle de l'image … requested_mib=64 image_mib=1471`; the VM boots, `df` as in point 1 |
| 5 | Suspend and resume with a sized disk | **Resumes, still 4 GiB, but the disk content is lost** (below) | Resume in 16.2 s, the resize runs again, `df` → 3997 MiB, writes succeed |
| 6 | The Docker courses with `disk: 6Gi` | **The disk failures are gone; port 8080 remains** (§12.3) | 5.9 GiB disks, 4.2 and 4.0 GiB free at start |
| 7 | Build-time checks (14.2) in the cluster | **Works, but the message stays in the job's logs** (§12.4) | `git-basics` of `e0e78d0` → Workshop `Failed` instead of `Provisioning` for ever |

### 12.2 A suspended Workshop loses everything written to its disk (existing defect, not caused by 14.1)

On resume the supervisor copies the root file system **again from the image cache** and loads the memory
snapshot on top of it: only memory is saved by a suspend. Observed identically on a Workshop without
`resources.disk` and on the 4 GiB one:

1. Before the suspend: a 200 MiB file of random bytes, a small file and a directory written under the home
   (on the root disk), then `sync`. Read through the page cache and with `O_DIRECT`, the big file has the same
   SHA-256: the data is on the device.
2. After suspend and resume: `ls` and `cat` still show everything, and `sha256sum` still matches, **from the
   guest's memory**. Read from the device (`dd iflag=direct`), the file's hash is that of 200 MiB of zeros
   (`72abf2ca…`): the blocks are no longer on the disk. The file system is mounted read-write and accepts new
   writes; the only sign is `systemd-journald: … Journal file corrupted, rotating`.
3. After memory pressure evicted the page cache (a process allocating 400 MiB): the big file reads as zeros,
   the two small files no longer show their content, `ls dir-before` fails with `Bad message`, and the kernel logs
   `EXT4-fs error (device vda): htree_dirblock_to_tree: … Directory block failed checksum`. The file system
   stays mounted read-write.

So the findings of §4 and of the first trial ("state survives a suspend") hold for memory, tmpfs and
processes only. A learner's work on disk silently disappears at the first suspend, and what comes back later
is a corrupted file system. It also undermines the "suspend and resume" workaround used in §4 and §11 to cut
the network: it is only safe on a Workshop nothing was written to, and Docker's own state under
`/var/lib/docker` is written at boot. Where it would be fixed in Atelier: the snapshot taken by the supervisor
(`snapshot_and_publish` in `crates/vm-supervisor/src/main.rs`) has to include the VM's root file system, and
`restore_persisted` (`crates/firecracker/src/vm.rs`) has to start from that copy rather than from the image
cache (the resize must then be skipped, or only grow).

### 12.3 The Docker courses on a 6 GiB disk

Mentor `f3df7ce` environments, unmodified, 2 CPU, 2 GiB, `disk: 6Gi`; both reach `Running` (152 s and 160 s,
built together). Debian's registry no longer starts (`docker-registry.service` → `/dev/null`, port 5000 does
not answer): **the port 5000 conflict of §11.3 is gone**. Because of §12.2 the network was **not** cut for
this replay: the build allow-list, which includes Docker Hub, was still in place.

| Course | Steps passed | Before (§11) | What still fails | Cause |
| --- | --- | --- | --- | --- |
| `docker-hello` | 12 of 19 | 8 of 19 | Lab 2 step 2, and steps 3, 4, 5 which wait for it; lab 3 setup command, step 2, and steps 3 and 6 which wait for it | Port 8080 taken by Atelier's `code-server` |
| `docker-advanced` | 23 of 27 | 10 of 26 | Lab 3 step 3, and steps 4 and 5 which wait for it; lab 4 step 5 (`curl localhost:8080` gets `code-server`'s login redirect, not nginx) | Port 8080 again |

Labs 1 and 4 of `docker-hello` and labs 1, 2 and 5 of `docker-advanced` pass entirely (image builds,
`postgres`, a multi-stage build, the team registry on port 5000). No step failed for lack of space: 3.4 GiB
were still free after the five labs of `docker-advanced`. Every remaining failure is port 8080.

### 12.4 Build-time checks (task 14.2)

`catalogue/git-basics/environnement` of Mentor `e0e78d0` (systemd without `/sbin/init`, no `curl`), which
stayed in `Provisioning` in §1:

- the build job now ends in error, with both causes named in the `image-builder` container's log: "systemd
  est installe mais `/sbin/init` n'existe pas … Installer le paquet qui fournit `/sbin/init` (`systemd-sysv`
  sur Debian et Ubuntu), ou retirer systemd de l'image" and "`curl` est absent de l'image …";
- the Workshop reaches **`phase: Failed`**, 3 min 17 s after its creation: the job ran the whole build three
  times (`backoffLimit: 2`) before giving up, although the failure is deterministic;
- **the message surfaces nowhere else**: `status.conditions` is empty, the pod's termination message is
  empty, the job only says `BackoffLimitExceeded`, and `GET /v1/workshops/<name>/events` lists pod events. A
  user of the API sees `Failed` without a reason.

### 12.5 What this asks of Atelier

1. **Suspend loses the disk** (§12.2): the serious one, and older than this change.
2. Task 14.2: put the refusal message in the Workshop's status (write it to the container's termination log
   and copy it into `status.conditions`), and do not retry a build refused by these checks.
3. Task 14.1: nothing to correct in what was observed. Two things to know: ext4 reserves 5 % for root, so a
   learner gets about 3.7 GiB of a 4 GiB disk (2.8 GiB free on this image); and the resize runs again at every
   resume (about 3 s), which the fix of point 1 will have to revisit.
4. Port 8080 (gap 8 of §8) is now the only thing that keeps the two Docker courses from passing.
