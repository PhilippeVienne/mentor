//! What can be said of an environment by reading its `Dockerfile`, before anything is built.
//!
//! The execution plane (see `doc/atelier-lab-validation.md`) starts an image as a microVM, and three things
//! make that fail or misbehave without any message: an image that carries systemd but no `/sbin/init` never
//! boots, the start scripts injected in the guest need `curl`, and nothing is mounted on the working folder,
//! which therefore belongs to whoever created it in the image. These are found by reading the text, so they
//! are warnings, not errors: a base image may already provide what the `Dockerfile` does not name.

use std::sync::LazyLock;

use regex::Regex;

/// Working folder of every environment so far; `devcontainer.json` is not parsed yet.
const WORKING_FOLDER: &str = "/workspace";

static SYSTEMD_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(^|[\s=])(openssh-server|systemd)([\s\\]|$)").unwrap());
static CURL_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(^|[\s=/])curl([\s\\]|$)").unwrap());
static CHOWN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"chown\s+(-\S+\s+)*\S+\s+(\S+\s+)*/workspace(\s|$)").unwrap());

/// Warnings about a `Dockerfile`, in the order an author would fix them.
pub fn dockerfile_warnings(dockerfile: &str) -> Vec<String> {
    // Comments explain; they install nothing.
    let instructions: String = dockerfile.lines().filter(|line| !line.trim_start().starts_with('#')).collect::<Vec<_>>().join("\n");
    let mut warnings = Vec::new();
    if SYSTEMD_RE.is_match(&instructions) && !instructions.contains("systemd-sysv") {
        warnings.push(
            "systemd is installed (directly or by `openssh-server`) without `systemd-sysv`: the image has no `/sbin/init` and \
             would never start as a microVM; install `systemd-sysv`"
                .to_string(),
        );
    }
    if !CURL_RE.is_match(&instructions) {
        warnings.push("`curl` is not installed: the platform needs it in the guest to start a session".to_string());
    }
    if !CHOWN_RE.is_match(&instructions) {
        warnings.push(format!(
            "the working folder `{WORKING_FOLDER}` is not given to the learner (`RUN chown <user>:<user> {WORKING_FOLDER}`): \
             nothing is mounted on it, so it would belong to root"
        ));
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    const READY: &str = "FROM debian:trixie-slim\nRUN apt-get update \\\n    && apt-get install -y curl openssh-server systemd-sysv \\\n    && useradd --uid 1000 apprenant\nWORKDIR /workspace\nRUN chown apprenant:apprenant /workspace\nUSER apprenant\n";

    #[test]
    fn a_ready_dockerfile_has_no_warning() {
        assert!(dockerfile_warnings(READY).is_empty(), "{:?}", dockerfile_warnings(READY));
        // No systemd at all is fine too: the platform then brings its own init.
        assert!(dockerfile_warnings(&READY.replace(" openssh-server systemd-sysv", "")).is_empty());
    }

    #[test]
    fn each_missing_piece_is_named() {
        let warnings = dockerfile_warnings(&READY.replace(" systemd-sysv", ""));
        assert!(warnings.len() == 1 && warnings[0].contains("`systemd-sysv`"), "{warnings:?}");
        let warnings = dockerfile_warnings(&READY.replace("curl ", ""));
        assert!(warnings.len() == 1 && warnings[0].starts_with("`curl` is not installed"), "{warnings:?}");
        let warnings = dockerfile_warnings(&READY.replace("RUN chown apprenant:apprenant /workspace\n", ""));
        assert!(warnings.len() == 1 && warnings[0].contains("is not given to the learner"), "{warnings:?}");
    }

    #[test]
    fn comments_install_nothing() {
        let commented = READY.replace("curl ", "").replace("USER apprenant", "# curl and systemd-sysv would be nice\nUSER apprenant");
        assert_eq!(dockerfile_warnings(&commented).len(), 1);
        // A package whose name merely contains the word is not the tool.
        assert_eq!(dockerfile_warnings(&READY.replace("curl ", "libcurl4 ")).len(), 1);
    }
}
