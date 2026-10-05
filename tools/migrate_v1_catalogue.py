#!/usr/bin/env python3
"""Migrate a v1 catalogue (French format) to the v2 format (English keys, directives and file names).

Usage:
    python tools/migrate_v1_catalogue.py <catalogue-dir> [--names conformance/v1-names.json]

The catalogue is rewritten in place. Course content (lesson prose, titles, check documentation) is not
translated: only the format is. YAML is edited with ruamel.yaml in round-trip mode, so comments, quoting
and block scalars are preserved. Running the tool twice is harmless.

Since v2 dropped simulated labs, the result only compiles for courses whose labs are all real (`moteur: reel`):
labs of the simulated `git` and `docker` engines are converted in name, then refused by the compiler, and have
to be rewritten as real labs by hand.

Requires: ruamel.yaml (pip install ruamel.yaml).
"""

import argparse
import io
import json
import re
import sys
from pathlib import Path

from ruamel.yaml import YAML
from ruamel.yaml.comments import CommentedMap, CommentedSeq

FENCE_RE = re.compile(r"^(```+|~~~+)[ \t]*(.*?)[ \t]*$")
DIRECTIVE_RE = re.compile(r"^:::(\w+)([ \t]*.*)$")
LESSON_FILE_RE = re.compile(r"^\d+-.+\.md$")


def make_yaml():
    yaml = YAML()
    yaml.preserve_quotes = True
    yaml.width = 100000
    yaml.indent(mapping=2, sequence=4, offset=2)
    return yaml


def dump(yaml, data):
    buffer = io.StringIO()
    yaml.dump(data, buffer)
    return buffer.getvalue()


def rename_keys(mapping, names):
    """Rename keys of a round-trip mapping in place, keeping their position and comments."""
    if not isinstance(mapping, CommentedMap):
        return
    for old in [key for key in mapping if key in names and names[key] != key]:
        position = list(mapping.keys()).index(old)
        comment = mapping.ca.items.pop(old, None)
        value = mapping.pop(old)
        mapping.insert(position, names[old], value)
        if comment is not None:
            mapping.ca.items[names[old]] = comment


def as_list(value):
    return value if isinstance(value, CommentedSeq) else [value]


class Migrator:
    def __init__(self, names):
        self.n = names
        self.yaml = make_yaml()

    # ── Structures shared by labs and sandbox scenarios ─────────────────────
    def commit(self, commit):
        rename_keys(commit, self.n["commit"])

    def server(self, servers):
        for server in as_list(servers) if servers else []:
            if isinstance(server, CommentedMap):
                for commit in server.get("commits") or []:
                    self.commit(commit)

    def checks(self, checks):
        for item in as_list(checks):
            rename_keys(item, self.n["checks"])

    def lab(self, lab):
        if not isinstance(lab, CommentedMap):
            return
        rename_keys(lab, self.n["lab"])
        if lab.get("engine") in self.n["engines"]:
            lab["engine"] = self.n["engines"][lab["engine"]]
        self.server(lab.get("server"))
        for step in lab.get("steps") or []:
            if not isinstance(step, CommentedMap):
                continue
            rename_keys(step, self.n["step"])
            if "checks" in step:
                self.checks(step["checks"])
            effect = step.get("effect")
            if isinstance(effect, CommentedMap):
                rename_keys(effect, self.n["effects"])
                for args in effect.values():
                    self.commit(args)
            if "solution" in step:
                for action in as_list(step["solution"]):
                    rename_keys(action, self.n["action"])

    # ── Files ────────────────────────────────────────────────────────────────
    def front_matter(self, text, names):
        if not text.startswith("---\n"):
            return text
        end = text.find("\n---", 4)
        if end < 0:
            return text
        meta = self.yaml.load(text[4 : end + 1])
        if not isinstance(meta, CommentedMap):
            return text
        rename_keys(meta, names)
        return "---\n" + dump(self.yaml, meta) + text[end + 1 :]

    def body(self, text):
        """Rename directives and migrate the YAML of lab blocks, leaving code fences untouched."""
        out, lab, fence = [], None, None
        for line in text.split("\n"):
            match = FENCE_RE.match(line)
            if match:
                marker = match.group(1)
                if fence is None:
                    fence = marker
                elif not match.group(2) and marker[0] == fence[0] and len(marker) >= len(fence):
                    fence = None
            if fence is None and not match:
                directive = DIRECTIVE_RE.match(line)
                if directive and lab is None:
                    name = self.n["directives"].get(directive.group(1), directive.group(1))
                    out.append(f":::{name}{directive.group(2)}")
                    if name == "lab":
                        lab = []
                    continue
                if line.strip() == ":::" and lab is not None:
                    data = self.yaml.load("\n".join(lab))
                    self.lab(data)
                    out.append(dump(self.yaml, data).rstrip("\n") if data is not None else "\n".join(lab))
                    out.append(line)
                    lab = None
                    continue
            (lab if lab is not None else out).append(line)
        if lab is not None:
            raise ValueError("unclosed :::lab block")
        return "\n".join(out)

    def markdown(self, path, names):
        text = path.read_text(encoding="utf-8")
        new = self.body(self.front_matter(text, names) if names is not None else text)
        if new != text:
            path.write_text(new, encoding="utf-8")

    def yaml_file(self, path, transform):
        data = self.yaml.load(path.read_text(encoding="utf-8"))
        transform(data)
        path.write_text(dump(self.yaml, data), encoding="utf-8")

    def checks_file(self, data):
        rename_keys(data, {"verifications": "checks", "effets": "effects"})
        for section, names in (("checks", self.n["checks"]), ("effects", self.n["effects"])):
            specs = data.get(section)
            rename_keys(specs, names)
            for spec in (specs or {}).values():
                rename_keys(spec, self.n["check_spec"])
                for key in ("arguments", "optional"):
                    for i, arg in enumerate(spec.get(key) or []):
                        spec[key][i] = self.n["arguments"].get(arg, arg)
                for i, engine in enumerate(spec.get("engines") or []):
                    spec["engines"][i] = self.n["engines"].get(engine, engine)

    def sandbox(self, data):
        for scenario in (data.get("scenarios") or {}).values():
            rename_keys(scenario, self.n["scenario"])
            self.server(scenario.get("server") if isinstance(scenario, CommentedMap) else None)

    def run(self, root):
        for old, new in self.n["files"].items():
            for path in sorted(root.rglob(old), reverse=True):
                path.rename(path.with_name(new))
        self.yaml_file(root / "catalogue.yml", lambda data: rename_keys(data, {"parcours": "courses"}))
        self.yaml_file(root / "_checks.yml", self.checks_file)
        for course in sorted(p for p in root.iterdir() if p.is_dir()):
            if (course / "sandbox.yml").exists():
                self.yaml_file(course / "sandbox.yml", self.sandbox)
            for path in sorted(course.glob("*.md")):
                if path.name == "course.md":
                    names = self.n["course"]
                elif path.name == "exam.md":
                    names = self.n["exam"]
                elif LESSON_FILE_RE.match(path.name):
                    names = self.n["lesson"]
                else:
                    names = None
                try:
                    self.markdown(path, names)
                except Exception as exc:  # report the file, then stop: a half-migrated catalogue is worse
                    sys.exit(f"{path}: {exc}")


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("catalogue", type=Path)
    parser.add_argument("--names", type=Path, default=Path(__file__).resolve().parent.parent / "conformance" / "v1-names.json")
    args = parser.parse_args()
    Migrator(json.loads(args.names.read_text(encoding="utf-8"))).run(args.catalogue)


if __name__ == "__main__":
    main()
