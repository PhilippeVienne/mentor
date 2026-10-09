//! Fetches a course package from a Git repository, at one commit, into a directory of plain files.
//!
//! A repository named by a tenant is untrusted input, and so is the host it names. The rules of
//! `doc/course-packages.md` §6.1 are applied here:
//!
//! - **`https://` only**, without credentials in the URL, to a host whose addresses are all public: no
//!   loopback, private, link-local or carrier-grade range. Redirects are not followed.
//! - **The system `git`, as a confined subprocess**: no system or user configuration, no prompt, no hook, no
//!   other protocol, a time limit on every call. One commit is fetched into a fresh bare repository, without
//!   history, tags or submodules.
//! - **No checkout is ever done.** The tree of the commit is listed and checked first (regular files only,
//!   plain names, bounded count and sizes); files are then written one by one from their blobs, under names
//!   this module has just validated and with fixed permissions. No path, link or mode chosen by the author
//!   reaches the file system.
//! - **The ref is resolved once**; what is fetched is checked to be the commit resolved.
//!
//! The directory obtained is then handled like any package directory: its tree is verified again and it is
//! compiled by `mentor_content::load_package`.
//!
//! Limits that remain: the host is resolved here and again by Git, so the address check does not replace a
//! network policy around the process; and the size of what is downloaded is bounded by the time limit and
//! checked afterwards, not capped while it arrives.

use std::io::Read;
use std::net::{IpAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use mentor_content::Limits;

/// Time one Git command may take.
const GIT_TIME_LIMIT: Duration = Duration::from_secs(180);

/// Where a package comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub url: String,
    /// A tag, a branch, a full commit identifier, or `HEAD`.
    pub reference: String,
    /// Folder of the package in the repository; empty for its root.
    pub path: String,
}

/// A package fetched at a commit.
#[derive(Debug)]
pub struct Fetched {
    /// The commit the reference resolved to: what identifies what was installed.
    pub commit: String,
    /// The directory holding the package. It lives as long as this value.
    pub directory: PathBuf,
    _scratch: tempfile::TempDir,
}

/// Which hosts may be reached. Tests fetch from a repository on disk; nothing else ever uses `LocalFiles`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    PublicHttps,
    #[cfg_attr(not(test), allow(dead_code))]
    LocalFiles,
}

/// An address that designates a machine on the public Internet: not this machine, not a private network, not
/// a link-local or metadata address, not a range reserved for something else.
pub fn is_public(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(v4) => {
            let [a, b, ..] = v4.octets();
            !(v4.is_unspecified()
                || v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_multicast()
                || v4.is_documentation()
                // Carrier-grade NAT (100.64.0.0/10), benchmarking (198.18.0.0/15), reserved (240.0.0.0/4).
                || (a == 100 && (64..128).contains(&b))
                || (a == 198 && (b == 18 || b == 19))
                || a >= 240
                || a == 0)
        }
        IpAddr::V6(v6) => {
            if let Some(mapped) = v6.to_ipv4_mapped() {
                return is_public(IpAddr::V4(mapped));
            }
            let first = v6.segments()[0];
            !(v6.is_unspecified()
                || v6.is_loopback()
                || v6.is_multicast()
                // Unique local (fc00::/7), link-local (fe80::/10), documentation (2001:db8::/32).
                || (first & 0xfe00) == 0xfc00
                || (first & 0xffc0) == 0xfe80
                || (first == 0x2001 && v6.segments()[1] == 0x0db8))
        }
    }
}

/// Checks the form of a repository URL and returns its host. Nothing is resolved here.
pub fn check_url(url: &str) -> Result<&str, String> {
    let Some(rest) = url.strip_prefix("https://") else {
        return Err("only `https://` repository addresses are accepted".into());
    };
    if url.chars().any(|c| c.is_whitespace() || c.is_control()) || url.len() > 2000 {
        return Err("the repository address contains spaces or control characters, or is too long".into());
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    if authority.contains('@') {
        return Err("the repository address must not contain credentials: they would end up in logs and in the database".into());
    }
    let host = authority
        .rsplit_once(':')
        .map_or(authority, |(host, port)| if port.chars().all(|c| c.is_ascii_digit()) { host } else { authority });
    if host.is_empty() || host.starts_with('-') {
        return Err("the repository address has no host".into());
    }
    Ok(host)
}

fn check_host_is_public(url: &str) -> Result<(), String> {
    let host = check_url(url)?;
    let lookup = host.trim_start_matches('[').trim_end_matches(']');
    let addresses: Vec<IpAddr> = (lookup, 443)
        .to_socket_addrs()
        .map_err(|err| format!("the host `{host}` cannot be resolved: {err}"))?
        .map(|address| address.ip())
        .collect();
    if addresses.is_empty() {
        return Err(format!("the host `{host}` has no address"));
    }
    match addresses.iter().find(|address| !is_public(**address)) {
        Some(address) => Err(format!("the host `{host}` resolves to {address}, which is not a public address")),
        None => Ok(()),
    }
}

/// A Git command that reads no configuration but ours, asks nothing, runs no hook and speaks one protocol.
fn git(repository: &Path, transport: Transport) -> Command {
    let mut command = Command::new("git");
    command
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("HOME", repository)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_ASKPASS", "/bin/false")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_ALLOW_PROTOCOL", if transport == Transport::LocalFiles { "file" } else { "https" })
        .env("LC_ALL", "C")
        .arg("--git-dir")
        .arg(repository)
        .args(["-c", "core.hooksPath=/dev/null", "-c", "http.followRedirects=false", "-c", "transfer.fsckObjects=true"])
        .args(["-c", "credential.helper=", "-c", "protocol.version=2", "-c", "gc.auto=0"]);
    command
}

/// Runs a command to its end, or kills it at the time limit. Returns its standard output, at most `cap` bytes.
fn run(command: Command, what: &str, cap: u64) -> Result<Vec<u8>, String> {
    run_with_input(command, what, cap, None)
}

/// Same, feeding `input` to the command's standard input.
fn run_with_input(mut command: Command, what: &str, cap: u64, input: Option<Vec<u8>>) -> Result<Vec<u8>, String> {
    command.stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() }).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|err| format!("git cannot be run ({what}): {err}"))?;
    if let (Some(input), Some(mut stdin)) = (input, child.stdin.take()) {
        // Written from its own thread: the command answers while it is being fed.
        std::thread::spawn(move || {
            use std::io::Write;
            stdin.write_all(&input).ok();
        });
    }
    let (stdout, stderr) = (child.stdout.take().expect("piped"), child.stderr.take().expect("piped"));
    let read = |stream: Box<dyn Read + Send>, cap: u64| {
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let mut stream = stream.take(cap + 1);
            stream.read_to_end(&mut bytes).ok();
            // Whatever is left is read and dropped, so that the command never blocks on a full pipe.
            std::io::copy(&mut stream.into_inner(), &mut std::io::sink()).ok();
            bytes
        })
    };
    let (output, errors) = (read(Box::new(stdout), cap), read(Box::new(stderr), 64 * 1024));
    let deadline = Instant::now() + GIT_TIME_LIMIT;
    let status = loop {
        match child.try_wait().map_err(|err| format!("{what}: {err}"))? {
            Some(status) => break status,
            None if Instant::now() > deadline => {
                child.kill().ok();
                child.wait().ok();
                return Err(format!("{what} took more than {} seconds and was stopped", GIT_TIME_LIMIT.as_secs()));
            }
            None => std::thread::sleep(Duration::from_millis(20)),
        }
    };
    let (output, errors) = (output.join().unwrap_or_default(), errors.join().unwrap_or_default());
    if !status.success() {
        let message = String::from_utf8_lossy(&errors);
        let last = message.lines().rev().find(|line| !line.trim().is_empty()).unwrap_or("no message");
        return Err(format!("{what} failed: {}", last.trim()));
    }
    if output.len() as u64 > cap {
        return Err(format!("{what}: more output than expected"));
    }
    Ok(output)
}

fn is_commit_id(text: &str) -> bool {
    text.len() == 40 && text.chars().all(|c| c.is_ascii_hexdigit())
}

/// Resolves a reference of the remote repository to a commit, once.
fn resolve(repository: &Path, transport: Transport, source: &Source) -> Result<String, String> {
    let reference = source.reference.as_str();
    if is_commit_id(reference) {
        return Ok(reference.to_lowercase());
    }
    if reference.is_empty()
        || reference.starts_with('-')
        || reference.chars().any(|c| c.is_whitespace() || c.is_control() || "~^:?*[\\".contains(c))
    {
        return Err(format!("`{reference}` is not a tag, a branch or a commit identifier"));
    }
    let mut command = git(repository, transport);
    command.args(["ls-remote", "--", &source.url]);
    let listing = String::from_utf8_lossy(&run(command, "listing the references of the repository", 16 << 20)?).into_owned();
    let find = |name: &str| {
        listing.lines().find_map(|line| line.split_once('\t').filter(|(_, listed)| *listed == name).map(|(id, _)| id.to_string()))
    };
    if reference == "HEAD" {
        return find("HEAD").ok_or_else(|| "the repository has no default branch".to_string());
    }
    // An annotated tag is listed twice: the tag object, then (`^{}`) the commit it points to.
    let tag = find(&format!("refs/tags/{reference}^{{}}")).or_else(|| find(&format!("refs/tags/{reference}")));
    let branch = find(&format!("refs/heads/{reference}"));
    match (tag, branch) {
        (Some(_), Some(_)) => Err(format!("`{reference}` is both a tag and a branch of the repository: give the commit identifier")),
        (Some(commit), None) | (None, Some(commit)) => Ok(commit),
        (None, None) => Err(format!("the repository has no tag or branch named `{reference}`")),
    }
}

/// One file of the tree of a commit.
#[derive(Debug, PartialEq, Eq)]
struct Entry {
    object: String,
    size: u64,
    /// Path under the package folder, with `/` separators.
    path: String,
}

/// A name the platform accepts to write: no separator, no parent, nothing a shell or a file system reinterprets.
fn plain_component(name: &str, limits: &Limits) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && name.chars().count() <= limits.name_chars
        && !name.chars().any(|c| c.is_control() || c == '\\' || c == '/' || c == ':')
}

/// Reads the output of `git ls-tree -r -l -z` and refuses anything that is not a regular file with a plain
/// path, or that exceeds the limits. Nothing has been written when this fails.
fn check_tree(listing: &[u8], limits: &Limits) -> Result<Vec<Entry>, String> {
    let mut entries = Vec::new();
    let mut total = 0u64;
    for record in listing.split(|byte| *byte == 0).filter(|record| !record.is_empty()) {
        let record = std::str::from_utf8(record).map_err(|_| "a file name of the repository is not valid UTF-8".to_string())?;
        let (head, path) = record.split_once('\t').ok_or("unexpected output of git ls-tree")?;
        let fields: Vec<&str> = head.split_whitespace().collect();
        let [mode, kind, object, size] = fields[..] else { return Err("unexpected output of git ls-tree".into()) };
        match (mode, kind) {
            ("100644" | "100755", "blob") => {}
            ("120000", _) => return Err(format!("`{path}` is a symbolic link: a package holds regular files only")),
            ("160000", _) => return Err(format!("`{path}` is a submodule: a package holds its files itself")),
            _ => return Err(format!("`{path}` is not a regular file (mode {mode})")),
        }
        let components: Vec<&str> = path.split('/').collect();
        if components.len() > limits.depth + 1 || !components.iter().all(|name| plain_component(name, limits)) {
            return Err(format!("`{path}`: this path is not accepted (too deep, too long, or with unusual characters)"));
        }
        if !is_commit_id(object) {
            return Err("unexpected output of git ls-tree".into());
        }
        let size: u64 = size.parse().map_err(|_| format!("`{path}`: unknown size"))?;
        if size > limits.file_bytes {
            return Err(format!("`{path}` is {size} bytes: more than the {} allowed for one file", limits.file_bytes));
        }
        total += size;
        entries.push(Entry { object: object.to_string(), size, path: path.to_string() });
        if entries.len() > limits.files {
            return Err(format!("the package holds more than {} files", limits.files));
        }
        if total > limits.total_bytes {
            return Err(format!("the package holds more than {} bytes", limits.total_bytes));
        }
    }
    if entries.is_empty() {
        return Err("no file was found there".into());
    }
    Ok(entries)
}

fn folder_size(folder: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(folder) else { return 0 };
    entries
        .filter_map(|entry| entry.ok())
        .map(|entry| match entry.metadata() {
            Ok(metadata) if metadata.is_dir() => folder_size(&entry.path()),
            Ok(metadata) => metadata.len(),
            Err(_) => 0,
        })
        .sum()
}

/// Fetches the package of `source` at the commit its reference designates.
pub fn fetch(source: &Source, limits: &Limits, transport: Transport) -> Result<Fetched, String> {
    if transport == Transport::PublicHttps {
        check_host_is_public(&source.url)?;
    }
    let path = source.path.trim_matches('/');
    if !path.is_empty() && !path.split('/').all(|name| plain_component(name, limits)) {
        return Err(format!("`{}` is not a folder path inside a repository", source.path));
    }
    let scratch = tempfile::Builder::new().prefix("mentor-fetch-").tempdir().map_err(|err| format!("no scratch directory: {err}"))?;
    let repository = scratch.path().join("repository.git");
    let mut init = Command::new("git");
    init.env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .args(["init", "--quiet", "--bare", "--template="])
        .arg(&repository);
    run(init, "preparing the scratch repository", 1 << 20)?;

    let commit = resolve(&repository, transport, source)?;
    let mut command = git(&repository, transport);
    command.args(["fetch", "--quiet", "--depth=1", "--no-tags", "--no-recurse-submodules", "--", &source.url, &commit]);
    run(command, "fetching the commit", 1 << 20)?;
    // Twice the content allowed, compressed or not, is more than a package can legitimately need.
    if folder_size(&repository) > 2 * limits.total_bytes {
        return Err(format!("the repository sends more than {} bytes for this commit", 2 * limits.total_bytes));
    }
    let mut command = git(&repository, transport);
    command.args(["rev-parse", "--verify", "--quiet", "FETCH_HEAD^{commit}"]);
    let fetched = String::from_utf8_lossy(&run(command, "reading the fetched commit", 1024)?).trim().to_string();
    if fetched != commit {
        return Err(format!("the repository answered with commit {fetched}, not the commit {commit} that was asked"));
    }

    let tree = if path.is_empty() { format!("{commit}^{{tree}}") } else { format!("{commit}:{path}") };
    let mut command = git(&repository, transport);
    command.args(["ls-tree", "-r", "-l", "-z", "--full-tree", &tree]);
    let listing = run(command, "listing the files of the package", 64 << 20).map_err(|err| {
        if path.is_empty() {
            err
        } else {
            format!("the folder `{path}` cannot be read at commit {commit}: {err}")
        }
    })?;
    let entries = check_tree(&listing, limits)?;

    // Every blob is asked for in one call: `<object> blob <size>`, a line break, the content, a line break.
    let mut command = git(&repository, transport);
    command.args(["cat-file", "--batch"]);
    let wanted: Vec<u8> = entries.iter().flat_map(|entry| entry.object.bytes().chain(std::iter::once(b'\n'))).collect();
    let expected: u64 = entries.iter().map(|entry| entry.size + 64).sum();
    let contents = run_with_input(command, "reading the files of the package", expected, Some(wanted))?;

    let directory = scratch.path().join("package");
    let mut rest = contents.as_slice();
    for entry in &entries {
        let unexpected = || format!("`{}`: unexpected answer of git cat-file", entry.path);
        let line_end = rest.iter().position(|byte| *byte == b'\n').ok_or_else(unexpected)?;
        let header = std::str::from_utf8(&rest[..line_end]).map_err(|_| unexpected())?;
        if header != format!("{} blob {}", entry.object, entry.size) {
            return Err(format!("`{}` is not the file the repository announced ({header})", entry.path));
        }
        let size = entry.size as usize;
        let content = rest.get(line_end + 1..line_end + 1 + size).ok_or_else(unexpected)?;
        if rest.get(line_end + 1 + size) != Some(&b'\n') {
            return Err(unexpected());
        }
        rest = &rest[line_end + 2 + size..];
        let destination = directory.join(&entry.path);
        std::fs::create_dir_all(destination.parent().expect("under the package directory"))
            .map_err(|err| format!("{}: {err}", entry.path))?;
        std::fs::write(&destination, content).map_err(|err| format!("{}: {err}", entry.path))?;
    }
    Ok(Fetched { commit, directory, _scratch: scratch })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_public_addresses_pass() {
        for address in ["140.82.121.3", "2606:50c0:8000::153", "1.1.1.1"] {
            assert!(is_public(address.parse().unwrap()), "{address}");
        }
        for address in [
            "127.0.0.1",
            "10.1.2.3",
            "172.19.0.1",
            "192.168.1.1",
            "169.254.169.254",
            "100.64.0.1",
            "0.0.0.0",
            "255.255.255.255",
            "224.0.0.1",
            "198.18.0.1",
            "240.0.0.1",
            "::1",
            "::",
            "fe80::1",
            "fd00::1",
            "::ffff:10.0.0.1",
            "::ffff:127.0.0.1",
            "2001:db8::1",
            "ff02::1",
        ] {
            assert!(!is_public(address.parse().unwrap()), "{address}");
        }
    }

    #[test]
    fn a_repository_address_is_https_without_credentials() {
        assert_eq!(check_url("https://github.com/acme/courses.git"), Ok("github.com"));
        assert_eq!(check_url("https://git.example.org:8443/x"), Ok("git.example.org"));
        for url in [
            "http://github.com/acme/courses",
            "git@github.com:acme/courses.git",
            "ssh://git@github.com/acme/courses",
            "file:///etc",
            "ext::sh -c id",
            "https://user:token@github.com/acme/courses",
            "https://token@github.com/acme/courses",
            "https:///nowhere",
            "https://github.com/a b",
            "https://--upload-pack=x/y",
            "/srv/git/courses",
        ] {
            assert!(check_url(url).is_err(), "{url}");
        }
    }

    #[test]
    fn hosts_that_are_not_public_are_refused_before_git_runs() {
        for url in [
            "https://localhost/x.git",
            "https://127.0.0.1/x.git",
            "https://[::1]/x.git",
            "https://169.254.169.254/latest",
            "https://10.0.0.8/x",
        ] {
            let error = check_host_is_public(url).unwrap_err();
            assert!(error.contains("not a public address"), "{url}: {error}");
        }
    }

    fn listing(records: &[&str]) -> Vec<u8> {
        records.iter().flat_map(|record| record.bytes().chain(std::iter::once(0))).collect()
    }

    const BLOB: &str = "100644 blob 0123456789abcdef0123456789abcdef01234567";

    #[test]
    fn a_tree_of_regular_files_with_plain_paths_is_accepted() {
        let entries = check_tree(
            &listing(&[&format!("{BLOB}      12\tmentor.yml"), &format!("{BLOB}     340\tgit/images/zones.svg")]),
            &Limits::default(),
        )
        .unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!((entries[1].path.as_str(), entries[1].size), ("git/images/zones.svg", 340));
    }

    #[test]
    fn links_submodules_odd_paths_and_oversized_content_are_refused() {
        let limits = Limits::default();
        let refused = |record: String| check_tree(&listing(&[&record]), &limits).unwrap_err();
        assert!(refused("120000 blob 0123456789abcdef0123456789abcdef01234567      11\tgit/secret".into()).contains("symbolic link"));
        assert!(refused("160000 commit 0123456789abcdef0123456789abcdef01234567       -\tvendor".into()).contains("submodule"));
        for path in ["../outside", "a/../../b", "a//b", "a\\b", "c:evil", "a/./b"] {
            assert!(refused(format!("{BLOB}      12\t{path}")).contains("not accepted"), "{path}");
        }
        assert!(refused(format!("{BLOB} {}\tbig.bin", limits.file_bytes + 1)).contains("allowed for one file"));
        let small = Limits { files: 1, ..Limits::default() };
        let two = listing(&[&format!("{BLOB} 1\ta"), &format!("{BLOB} 1\tb")]);
        assert!(check_tree(&two, &small).unwrap_err().contains("more than 1 files"));
        assert!(check_tree(&[], &limits).unwrap_err().contains("no file"));
    }

    /// A repository on disk, built with the system Git as an author would.
    struct Repository {
        folder: tempfile::TempDir,
    }

    impl Repository {
        fn new() -> Self {
            let repository = Self { folder: tempfile::tempdir().unwrap() };
            repository.git(&["init", "--quiet", "--initial-branch=main"]);
            repository
        }

        fn git(&self, args: &[&str]) -> String {
            let output = Command::new("git")
                .current_dir(self.folder.path())
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .args(["-c", "user.name=Author", "-c", "user.email=author@example.org", "-c", "commit.gpgsign=false"])
                .args(args)
                .output()
                .unwrap();
            assert!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }

        fn write(&self, path: &str, content: &str) {
            let file = self.folder.path().join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, content).unwrap();
        }

        fn commit(&self, message: &str) -> String {
            self.git(&["add", "--all"]);
            self.git(&["commit", "--quiet", "--message", message]);
            self.git(&["rev-parse", "HEAD"])
        }

        fn source(&self, reference: &str, path: &str) -> Source {
            Source { url: format!("file://{}", self.folder.path().display()), reference: reference.into(), path: path.into() }
        }
    }

    fn files_of(directory: &Path) -> Vec<String> {
        fn walk(folder: &Path, prefix: &str, names: &mut Vec<String>) {
            for entry in std::fs::read_dir(folder).unwrap().map(|entry| entry.unwrap()) {
                let name = format!("{prefix}{}", entry.file_name().to_string_lossy());
                if entry.path().is_dir() {
                    walk(&entry.path(), &format!("{name}/"), names);
                } else {
                    names.push(name);
                }
            }
        }
        let mut names = Vec::new();
        walk(directory, "", &mut names);
        names.sort();
        names
    }

    fn fetch_local(source: &Source) -> Result<Fetched, String> {
        fetch(source, &Limits::default(), Transport::LocalFiles)
    }

    #[test]
    fn a_commit_is_fetched_by_tag_branch_or_identifier_without_a_checkout() {
        let repository = Repository::new();
        repository.write("mentor.yml", "format: 1\n");
        repository.write("git/course.md", "premier\n");
        let first = repository.commit("v1");
        repository.git(&["tag", "v1.0.0"]);
        repository.git(&["tag", "--annotate", "--message", "annotated", "v1.0.1"]);
        repository.write("git/course.md", "second\n");
        repository.write("git/images/zones.svg", "<svg/>\n");
        let second = repository.commit("v2");

        let by_tag = fetch_local(&repository.source("v1.0.0", "")).unwrap();
        assert_eq!(by_tag.commit, first);
        assert_eq!(files_of(&by_tag.directory), ["git/course.md", "mentor.yml"]);
        assert_eq!(std::fs::read_to_string(by_tag.directory.join("git/course.md")).unwrap(), "premier\n");
        // An annotated tag resolves to the commit it points to, not to the tag object.
        assert_eq!(fetch_local(&repository.source("v1.0.1", "")).unwrap().commit, first);

        for reference in ["main", "HEAD", second.as_str()] {
            let fetched = fetch_local(&repository.source(reference, "")).unwrap();
            assert_eq!(fetched.commit, second, "{reference}");
            assert_eq!(files_of(&fetched.directory), ["git/course.md", "git/images/zones.svg", "mentor.yml"]);
        }
        // Only the package is written: no `.git`, no working tree of Git's making.
        let fetched = fetch_local(&repository.source("main", "")).unwrap();
        assert!(!fetched.directory.join(".git").exists());

        // A package in a sub-folder of the repository.
        let inner = fetch_local(&repository.source("main", "git")).unwrap();
        assert_eq!(files_of(&inner.directory), ["course.md", "images/zones.svg"]);

        for (reference, path, expected) in [
            ("nope", "", "no tag or branch named `nope`"),
            ("main", "missing", "the folder `missing` cannot be read"),
            ("main", "../etc", "not a folder path inside a repository"),
            ("--upload-pack=touch /tmp/x", "", "is not a tag, a branch or a commit identifier"),
        ] {
            let error = fetch_local(&repository.source(reference, path)).unwrap_err();
            assert!(error.contains(expected), "{reference} {path}: {error}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_repository_with_a_link_or_a_submodule_writes_nothing() {
        let repository = Repository::new();
        repository.write("mentor.yml", "format: 1\n");
        std::os::unix::fs::symlink("/etc/passwd", repository.folder.path().join("secret")).unwrap();
        repository.commit("a link out of the package");
        let error = fetch_local(&repository.source("main", "")).unwrap_err();
        assert!(error.contains("`secret` is a symbolic link"), "{error}");

        let repository = Repository::new();
        repository.write("mentor.yml", "format: 1\n");
        let commit = repository.commit("first");
        repository.git(&["update-index", "--add", "--cacheinfo", &format!("160000,{commit},vendor")]);
        repository.git(&["commit", "--quiet", "--message", "a submodule"]);
        let error = fetch_local(&repository.source("main", "")).unwrap_err();
        assert!(error.contains("`vendor` is a submodule"), "{error}");
    }

    #[test]
    fn a_name_that_is_both_a_tag_and_a_branch_is_refused() {
        let repository = Repository::new();
        repository.write("mentor.yml", "format: 1\n");
        repository.commit("first");
        repository.git(&["tag", "stable"]);
        repository.git(&["branch", "stable"]);
        let error = fetch_local(&repository.source("stable", "")).unwrap_err();
        assert!(error.contains("both a tag and a branch"), "{error}");
    }

    #[test]
    fn other_protocols_are_refused_even_when_asked_politely() {
        // Through the public transport, a `file://` address never reaches Git.
        let repository = Repository::new();
        repository.write("mentor.yml", "format: 1\n");
        repository.commit("first");
        let error = fetch(&repository.source("main", ""), &Limits::default(), Transport::PublicHttps).unwrap_err();
        assert!(error.contains("only `https://`"), "{error}");
    }
}
