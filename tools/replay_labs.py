#!/usr/bin/env python3
"""Replay the labs of the catalogue in their environments, to prove that each solution passes its checks.

Usage:
    python tools/replay_labs.py [course ...] [--only TEXT] [--jobs N] [--no-build] [--catalogue DIR]

`--catalogue` names a catalogue directory or a course package (a directory with a `mentor.yml` manifest, see
`doc/course-packages.md`). Without a course, every course it lists that has labs is replayed. For each lab the tool builds the
image of its environment, starts a fresh container without network, runs the setup (`files`, `commands`), then
for each step, in order:

1. evaluates the checks: a step that already holds before its solution proves nothing, and is a failure.
   The one exception is reported as a warning: a step that waits for earlier steps (`after`) and holds as
   soon as they are done, such as "run the whole test suite", is implied by them. It checks nothing new,
   which may be what the author wants for a closing step;
2. runs the `solution`, as the learner;
3. evaluates the checks again: all of them must hold.

The exit code is 0 when no step failed; warnings do not change it.

The container stands in for the learner's microVM, which the tool cannot start: no network, the learner's
account, no capability, an empty home and working folder, the memory and processors the environment asks for.
An environment that declares `customizations.mentor.dockerInDocker` gets what the platform is expected to give
it instead: a Docker daemon started as root, which requires a PRIVILEGED container on the machine running this
tool. What passes here can still fail in a real microVM (another kernel, another init, a disk quota).

Checks are evaluated as the server is specified to do it (`catalogue/_checks.yml`): by `sh -c`, as the learner,
in the working folder, regular expressions searched with `^` and `$` matching at each line.

Requires: Docker, PyYAML (pip install pyyaml).
"""

import argparse
import json
import re
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import yaml

LESSON_FILE_RE = re.compile(r"^\d+-.+\.md$")
LAB_RE = re.compile(r"^:::lab[ \t]*\n(.*?)^:::[ \t]*$", re.S | re.M)
FRONT_MATTER_RE = re.compile(r"\A---\n(.*?)\n---\n", re.S)

CHECK_TIMEOUT = 30
COMMAND_TIMEOUT = 300
# A service started by a step may need a moment before its check holds.
SETTLE_SECONDS = 20
OUTPUT_LIMIT = 65536


def run(args, timeout=None, text_input=None):
    """Runs a command of the host; a timeout is reported as exit code 124."""
    try:
        return subprocess.run(
            args, capture_output=True, text=True, timeout=timeout, input=text_input, stdin=None if text_input is not None else subprocess.DEVNULL
        )
    except subprocess.TimeoutExpired:
        return subprocess.CompletedProcess(args, 124, "", "timed out")


def strip_json_comments(text):
    """`devcontainer.json` is JSON with `//` comments and trailing commas."""
    out, i, in_string = [], 0, False
    while i < len(text):
        char = text[i]
        if in_string:
            out.append(char)
            if char == "\\":
                out.append(text[i + 1])
                i += 1
            elif char == '"':
                in_string = False
        elif char == '"':
            in_string = True
            out.append(char)
        elif text.startswith("//", i):
            i = text.find("\n", i)
            if i < 0:
                break
            continue
        else:
            out.append(char)
        i += 1
    return re.sub(r",(\s*[}\]])", r"\1", "".join(out))


def front_matter(path):
    match = FRONT_MATTER_RE.match(path.read_text())
    return (yaml.safe_load(match.group(1)) or {}) if match else {}


def labs_of(course_dir):
    """Yields (lesson file, lab, environment folder name) for each lab of a course."""
    course_environment = front_matter(course_dir / "course.md").get("environment") or ""
    for path in sorted(course_dir.iterdir()):
        if not LESSON_FILE_RE.match(path.name):
            continue
        match = LAB_RE.search(path.read_text())
        if not match:
            continue
        lab = yaml.safe_load(match.group(1)) or {}
        environment = lab.get("environment") or front_matter(path).get("environment") or course_environment
        yield path, lab, environment


class Environment:
    """A started environment: one container, thrown away after the lab."""

    def __init__(self, name, image, spec):
        self.name, self.image, self.spec = name, image, spec
        self.user = spec.get("containerUser") or spec.get("remoteUser") or "root"
        self.workspace = spec.get("workspaceFolder") or "/workspace"
        self.docker_inside = bool(spec.get("customizations", {}).get("mentor", {}).get("dockerInDocker"))

    def _exec(self, argv, user=None, timeout=COMMAND_TIMEOUT, text_input=None):
        flags = ["-i"] if text_input is not None else []
        return run(["docker", "exec", *flags, "-u", user or self.user, "-w", self.workspace, self.name, *argv], timeout, text_input)

    def shell(self, command, timeout=COMMAND_TIMEOUT, shell="sh"):
        return self._exec([shell, "-c", command], timeout=timeout)

    def start(self):
        run(["docker", "rm", "-f", "-v", self.name])
        identity = run(["docker", "run", "--rm", "--network", "none", "--user", self.user, self.image, "sh", "-c", 'echo "$(id -u):$(id -g):$HOME"'])
        if identity.returncode:
            raise RuntimeError(f"the image does not run as `{self.user}`: {identity.stderr.strip()}")
        uid, gid, home = identity.stdout.strip().split(":", 2)
        if uid == "0":
            raise RuntimeError("the environment runs as root: `containerUser` must be an ordinary account")
        args = ["docker", "run", "-d", "--name", self.name, "--network", "none", "--user", "root"]
        # Home and working folder are empty when an environment starts: nothing of the image survives there.
        for folder in {home, self.workspace}:
            args += ["--tmpfs", f"{folder}:exec,uid={uid},gid={gid},mode=0755"]
        requirements = self.spec.get("hostRequirements", {})
        if requirements.get("memory"):
            args += ["--memory", str(requirements["memory"]).lower().replace("gb", "g").replace("mb", "m")]
        if requirements.get("cpus"):
            args += ["--cpus", str(requirements["cpus"])]
        for key, value in (self.spec.get("containerEnv") or {}).items():
            args += ["-e", f"{key}={value}"]
        if self.docker_inside:
            # What the platform provides in the microVM: a Docker daemon run by root.
            args += ["--privileged", "-v", "/var/lib/docker"]
        else:
            args += ["--cap-drop", "ALL", "--security-opt", "no-new-privileges"]
        started = run([*args, self.image, "sleep", "infinity"])
        if started.returncode:
            raise RuntimeError(f"the environment did not start: {started.stderr.strip()}")
        if self.docker_inside:
            run(["docker", "exec", "-d", "-u", "root", self.name, "sh", "-c", "dockerd > /var/log/dockerd.log 2>&1"])
        self.shell("cp -a /etc/skel/. \"$HOME\"/ 2>/dev/null || true")
        for hook in ("postCreateCommand", "postStartCommand"):
            command = self.spec.get(hook)
            if not command:
                continue
            result = self.shell(command if isinstance(command, str) else " ".join(command))
            if result.returncode:
                raise RuntimeError(f"`{hook}` failed: {(result.stderr or result.stdout).strip()[-300:]}")

    def stop(self):
        run(["docker", "rm", "-f", "-v", self.name])

    def write(self, name, content):
        result = self._exec(["sh", "-c", 'mkdir -p "$(dirname "$1")" && cat > "$1"', "sh", name], text_input=content)
        if result.returncode:
            raise RuntimeError(f"cannot write `{name}`: {result.stderr.strip()}")

    def check(self, item):
        """Evaluates one check as `catalogue/_checks.yml` specifies it."""
        ((name, value),) = item.items()
        args = [str(arg) for arg in (value if isinstance(value, list) else [value])]
        if name == "command-succeeds":
            return self.shell(args[0], CHECK_TIMEOUT).returncode == 0
        if name == "command-fails":
            return self.shell(args[0], CHECK_TIMEOUT).returncode not in (0, 124, 137)
        if name == "output-contains":
            result = self.shell(args[0], CHECK_TIMEOUT)
            return result.returncode not in (124, 137) and re.search(args[1], result.stdout[:OUTPUT_LIMIT], re.M) is not None
        if name == "env-file-exists":
            return self._exec(["test", "-e", args[0]], timeout=CHECK_TIMEOUT).returncode == 0
        if name == "env-file-absent":
            return self._exec(["test", "!", "-e", args[0]], timeout=CHECK_TIMEOUT).returncode == 0
        if name == "env-file-contains":
            result = self._exec(["head", "-c", str(OUTPUT_LIMIT), "--", args[0]], timeout=CHECK_TIMEOUT)
            return result.returncode == 0 and re.search(args[1], result.stdout, re.M) is not None
        raise KeyError(f"unknown check `{name}`")


def replay_lab(environment, lab, report):
    """Replays one lab; returns the numbers of failures and of warnings."""
    failures, warnings = 0, 0
    for name, content in (lab.get("files") or {}).items():
        environment.write(name, content)
    for command in lab.get("commands") or []:
        result = environment.shell(command)
        if result.returncode:
            failures += 1
            report(f"   SETUP FAILED `{command[:70]}` (exit {result.returncode}): {(result.stderr or result.stdout).strip()[-200:]}")
    learner_shell = "bash" if environment.shell("command -v bash").returncode == 0 else "sh"
    done = set()
    for number, step in enumerate(lab.get("steps") or [], 1):
        after = step.get("after") or []
        ready = all(earlier in done for earlier in (after if isinstance(after, list) else [after]))
        checks = step["checks"] if isinstance(step["checks"], list) else [step["checks"]]
        solution = step["solution"] if isinstance(step["solution"], list) else [step["solution"]]
        notes = []
        implied = False
        if ready and all(environment.check(check) for check in checks):
            if after:
                implied = True
                warnings += 1
            else:
                failures += 1
                notes.append("ALREADY VALID before its solution")
        for action in solution:
            if isinstance(action, dict):
                for name, content in action["write"].items():
                    environment.write(name, content)
                continue
            result = environment.shell(action, shell=learner_shell)
            if result.returncode:
                notes.append(f"`{action[:60]}` exit {result.returncode}: {(result.stderr or result.stdout).strip()[-160:]}")
        deadline = time.monotonic() + SETTLE_SECONDS
        while True:
            results = [environment.check(check) for check in checks]
            if all(results) or time.monotonic() > deadline:
                break
            time.sleep(1)
        if all(results) and ready:
            done.add(number)
            verdict = "ok (warning: IMPLIED by the steps it follows, its checks add nothing)" if implied else "ok"
        else:
            failures += 1
            failing = [next(iter(check)) for check, held in zip(checks, results) if not held]
            verdict = "FAILED" + (f" checks {failing}" if failing else "") + ("" if ready else " (earlier steps it waits for are not done)")
        # A solution may hold commands that fail on purpose (an error to observe): only shown when useful.
        shown = notes if not verdict.startswith("ok") or any(note.startswith("ALREADY") for note in notes) else []
        report(f"   step {number}: {verdict}" + "".join(f"\n      {note}" for note in shown))
    return failures, warnings


def replay_course(catalogue, course, options):
    """Replays the labs of a course; returns (text of the report, failures, warnings)."""
    lines, failures, warnings, images = [f"== {course}"], 0, 0, {}
    course_dir = catalogue / course
    for path, lab, folder in labs_of(course_dir):
        if options.only and options.only not in path.name:
            continue
        lines.append(f" = {path.name}")
        if not folder:
            failures += 1
            lines.append("   FAILED: the lab has no environment")
            continue
        try:
            if folder not in images:
                context = course_dir / folder
                spec = json.loads(strip_json_comments((context / "devcontainer.json").read_text()))
                # An image name starts with a letter or a digit (`_template` does not).
                image = f"mentor-replay/{course.lstrip('_.-')}--{folder}".lower()
                if not options.no_build:
                    build = spec.get("build") or {}
                    built = run(
                        ["docker", "build", "-q", "-t", image, "-f", str(context / build.get("dockerfile", "Dockerfile")), str(context / build.get("context", "."))],
                        timeout=3600,
                    )
                    if built.returncode:
                        raise RuntimeError(f"the image does not build: {built.stderr.strip()[-600:]}")
                images[folder] = (image, spec)
            image, spec = images[folder]
            environment = Environment(f"mentor-replay-{course.lstrip('_.-')}", image, spec)
            try:
                environment.start()
                failed, warned = replay_lab(environment, lab, lines.append)
                failures, warnings = failures + failed, warnings + warned
            finally:
                environment.stop()
        except (RuntimeError, KeyError, OSError, ValueError) as error:
            failures += 1
            lines.append(f"   FAILED: {error}")
    return "\n".join(lines), failures, warnings


def listed_courses(directory):
    """The course folders of a package (`mentor.yml`), or of a catalogue without a manifest (`catalogue.yml`)."""
    for name in ("mentor.yml", "catalogue.yml"):
        if (directory / name).is_file():
            return yaml.safe_load((directory / name).read_text())["courses"]
    sys.exit(f"{directory}: neither mentor.yml nor catalogue.yml: not a course package or a catalogue")


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("courses", nargs="*", help="course folders (default: every course of mentor.yml, or of catalogue.yml)")
    parser.add_argument("--catalogue", default="catalogue", type=Path, help="catalogue or course package directory")
    parser.add_argument("--only", help="replay only the lessons whose file name contains this text")
    parser.add_argument("--jobs", type=int, default=1, help="courses replayed at the same time")
    parser.add_argument("--no-build", action="store_true", help="reuse the images built by a previous run")
    options = parser.parse_args()

    courses = options.courses or listed_courses(options.catalogue)
    courses = [course for course in courses if any(True for _ in labs_of(options.catalogue / course))]
    total, warned = 0, 0
    with ThreadPoolExecutor(max_workers=max(1, options.jobs)) as pool:
        for text, failures, warnings in pool.map(lambda course: replay_course(options.catalogue, course, options), courses):
            print(text, flush=True)
            total, warned = total + failures, warned + warnings
    print(f"FAILURES: {total}, WARNINGS: {warned}")
    sys.exit(1 if total else 0)


if __name__ == "__main__":
    main()
