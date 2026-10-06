//! Course packages: the manifest, the checks on the files of a package, and the built-in catalogue loaded
//! as a package.

use std::fs;
use std::path::{Path, PathBuf};

use mentor_content::{load_catalogue, load_manifest, load_package, load_package_with, Feature, Limits, Manifest};

const MANIFEST: &str = "format: 1\nname: demo-pack\nversion: 1.2.0\ntitle: \"Démo\"\nlicense: CC-BY-SA-4.0\ncourses: [demo]\n";
const COURSE: &str = "---\ntitle: \"Démo\"\nicon: \"🧪\"\nsummary: \"Un parcours.\"\nenvironment: env\n---\nPrésentation.\n";
const LESSON: &str = "---\nid: intro\ntitle: \"Intro\"\nsummary: \"Une leçon.\"\nminutes: 5\n---\nTexte.\n\n:::lab\nsteps:\n  - text: \"Crée `a.txt`\"\n    checks:\n      - env-file-exists: a.txt\n    solution:\n      - touch a.txt\n:::\n";

/// A fresh package directory holding one course with one lab, under Cargo's directory for test files.
fn package(test: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("packages").join(test);
    let _ = fs::remove_dir_all(&root);
    write(&root, "mentor.yml", MANIFEST);
    write(&root, "demo/course.md", COURSE);
    write(&root, "demo/01-intro.md", LESSON);
    write(&root, "demo/env/devcontainer.json", "{}\n");
    root
}

fn write(root: &Path, name: &str, content: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// The message of the error `load_package` returns, without the path it starts with.
fn refused(root: &Path) -> String {
    load_package(root).unwrap_err().message
}

fn manifest_error(source: &str) -> String {
    Manifest::parse(source, Path::new("mentor.yml")).unwrap_err().to_string()
}

#[test]
fn a_package_compiles_with_its_manifest() {
    let root = package("compiles");
    let package = load_package(&root).unwrap();
    assert_eq!((package.manifest.name.as_str(), package.manifest.version.as_str()), ("demo-pack", "1.2.0"));
    assert_eq!(package.tree.files, 4);
    let course = &package.catalogue.courses[0];
    assert_eq!((course.slug.as_str(), course.lessons.len()), ("demo", 1));
    assert_eq!(course.lessons[0].environment, "demo--env");
    assert_eq!(course.lessons[0].lab.as_ref().unwrap().steps[0].checks[0].name, "env-file-exists");
}

#[test]
fn the_manifest_is_strict() {
    assert_eq!(
        manifest_error(&MANIFEST.replace("title:", "titel:")),
        "mentor.yml: invalid manifest: unknown field `titel`, expected one of `format`, `name`, `version`, `title`, `summary`, `license`, `authors`, `courses`, `features`, `build_egress` at line 4 column 1"
    );
    assert!(manifest_error(&MANIFEST.replace("license: CC-BY-SA-4.0\n", "")).contains("missing field `license`"));
    assert!(manifest_error(&MANIFEST.replace("format: 1\n", "")).contains("`format` is required"));
    assert_eq!(
        manifest_error(&MANIFEST.replace("format: 1", "format: 2\nfuture_key: x")),
        "mentor.yml: unsupported package format `2`: this version of Mentor reads format 1"
    );
    assert!(manifest_error("- a\n- b\n").contains("must be a YAML mapping"));
    assert!(manifest_error(&MANIFEST.replace("demo-pack", "Demo Pack")).contains("`name` must be lower-case"));
    assert!(manifest_error(&MANIFEST.replace("1.2.0", "v1.2")).contains("`version` must be written `MAJOR.MINOR.PATCH`"));
    assert!(manifest_error(&MANIFEST.replace("CC-BY-SA-4.0", "\"free; see <LICENSE>\"")).contains("`license` must be an SPDX expression"));
    assert!(manifest_error(&MANIFEST.replace("[demo]", "[]")).contains("`courses` must list 1 to 200"));
    assert!(manifest_error(&MANIFEST.replace("[demo]", "[demo, demo]")).contains("`demo` is listed twice"));
    assert!(manifest_error(&format!("{MANIFEST}features: [gpu]\n")).contains("unknown variant `gpu`, expected `docker`"));
}

#[test]
fn a_course_entry_cannot_leave_the_package() {
    for entry in ["../other", "/etc", "demo/sub", "_template", ".", "Demo"] {
        let error = manifest_error(&MANIFEST.replace("[demo]", &format!("[\"{entry}\"]")));
        assert!(error.contains("is not a course folder name"), "{entry}: {error}");
    }
}

#[test]
fn build_egress_takes_host_names_only() {
    let with = |hosts: &str| Manifest::parse(&format!("{MANIFEST}features: [docker]\nbuild_egress: [{hosts}]\n"), Path::new("mentor.yml"));
    let manifest = with("registry-1.docker.io, \"*.debian.org\"").unwrap();
    assert_eq!(manifest.features, vec![Feature::Docker]);
    assert_eq!(manifest.build_egress, vec!["registry-1.docker.io", "*.debian.org"]);
    for host in ["\"*\"", "https://pypi.org", "pypi.org:443", "10.0.0.1", "localhost", "pypi.org/simple", "user@pypi.org"] {
        let error = with(host).unwrap_err().message;
        assert!(error.contains("is not a host name"), "{host}: {error}");
    }
}

#[test]
fn a_directory_without_manifest_is_not_a_package() {
    let root = package("no-manifest");
    fs::remove_file(root.join("mentor.yml")).unwrap();
    assert_eq!(refused(&root), "not a package: `mentor.yml` is missing at the root of the directory");
}

#[cfg(unix)]
#[test]
fn symbolic_links_are_refused_wherever_they_are() {
    use std::os::unix::fs::symlink;

    // A link the compiler would have followed: the environment of the course.
    let root = package("symlink-environment");
    fs::remove_dir_all(root.join("demo/env")).unwrap();
    symlink("/etc", root.join("demo/env")).unwrap();
    assert_eq!(
        load_package(&root).unwrap_err().to_string(),
        format!("{}: symbolic links are not allowed in a package", root.join("demo/env").display())
    );

    // A link nothing refers to is refused as well: the whole directory is what gets distributed.
    let root = package("symlink-unused");
    symlink("/etc/passwd", root.join("notes")).unwrap();
    assert!(refused(&root).contains("symbolic links are not allowed"));

    let root = package("symlink-manifest");
    fs::rename(root.join("mentor.yml"), root.join("real.yml")).unwrap();
    symlink("real.yml", root.join("mentor.yml")).unwrap();
    assert!(refused(&root).contains("the manifest must be an ordinary file"));
}

#[test]
fn submodules_git_filters_and_odd_names_are_refused() {
    let root = package("submodule");
    write(&root, ".gitmodules", "[submodule \"x\"]\n\tpath = x\n\turl = https://example.org/x.git\n");
    assert!(refused(&root).contains("Git submodules are not allowed"));

    let root = package("lfs");
    write(&root, "demo/.gitattributes", "*.bin filter=lfs diff=lfs merge=lfs -text\n");
    assert!(refused(&root).contains("Git filters"));

    let root = package("odd-name");
    write(&root, "demo/images/mon image.svg", "<svg/>");
    assert!(refused(&root).contains("file and folder names hold only letters"));
}

#[test]
fn limits_are_enforced() {
    let root = package("limits");
    write(&root, "demo/env/big.bin", &"x".repeat(2000));
    let refused = |limits: Limits| load_package_with(&root, &limits).unwrap_err().message;
    assert_eq!(refused(Limits { file_bytes: 1000, ..Limits::default() }), "the file is 2000 bytes, the limit is 1000");
    assert_eq!(refused(Limits { total_bytes: 1500, ..Limits::default() }), "the package is larger than 1500 bytes");
    assert_eq!(refused(Limits { files: 3, ..Limits::default() }), "the package holds more than 3 files");
    assert_eq!(refused(Limits { depth: 1, ..Limits::default() }), "folders are nested more than 1 levels deep");
    // Files the compiler parses have a limit of their own.
    assert!(refused(Limits { source_bytes: 50, ..Limits::default() }).starts_with("the file is "));
    assert!(load_package(&root).is_ok());
}

#[test]
fn the_git_directory_of_a_working_copy_is_ignored() {
    let root = package("git-directory");
    write(&root, ".git/hooks/pre-commit name with spaces", "#!/bin/sh\n");
    assert_eq!(load_package(&root).unwrap().tree.files, 4);
}

#[test]
fn an_environment_is_a_folder_of_its_course_not_a_path() {
    let root = package("environment-path");
    write(&root, "other/env/devcontainer.json", "{}\n");
    write(&root, "demo/course.md", &COURSE.replace("environment: env", "environment: ../other/env"));
    assert_eq!(refused(&root), "`environment` must be the name of a folder of the course, not a path (got `../other/env`)");

    let root = package("environment-path-lab");
    write(&root, "demo/01-intro.md", &LESSON.replace(":::lab\n", ":::lab\nenvironment: /etc\n"));
    assert!(refused(&root).contains("not a path (got `/etc`)"));
}

#[test]
fn prerequisites_stay_inside_the_package() {
    let root = package("prerequisite");
    write(&root, "demo/course.md", &COURSE.replace("environment: env", "environment: env\nrequires: [git-basics]"));
    assert_eq!(refused(&root), "unknown prerequisite `git-basics`: a course can only require courses of its own package");
}

#[test]
fn a_package_cannot_bring_its_own_checks() {
    let root = package("own-checks");
    write(&root, "_checks.yml", "checks:\n  run-as-root: {arguments: [command]}\n");
    write(&root, "demo/01-intro.md", &LESSON.replace("env-file-exists: a.txt", "run-as-root: id"));
    assert!(refused(&root).contains("unknown check `run-as-root`. Available: command-fails, command-succeeds"));
}

#[test]
fn a_yaml_alias_bomb_is_refused_not_expanded() {
    let root = package("alias-bomb");
    let mut lab = String::from(":::lab\na0: &a0 [x, x, x, x, x, x, x, x, x]\n");
    for level in 1..=9 {
        let previous = format!("*a{}", level - 1);
        lab.push_str(&format!("a{level}: &a{level} [{}]\n", [previous.as_str(); 9].join(", ")));
    }
    lab.push_str("steps: *a9\n:::\n");
    write(&root, "demo/01-intro.md", &LESSON.replace(&LESSON[LESSON.find(":::lab").unwrap()..], &lab));
    assert!(refused(&root).contains("invalid YAML: repetition limit exceeded"), "{}", refused(&root));
}

#[test]
fn the_built_in_catalogue_is_a_package() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalogue");
    let package = load_package(&directory).unwrap();
    assert_eq!(load_manifest(&directory).unwrap(), package.manifest);
    // Same courses, same order, same compiled output as the catalogue the web server loads today.
    assert_eq!(package.catalogue, load_catalogue(&directory).unwrap());
    // 19 courses taken over from v1, and the three AWS courses written for v2.
    assert_eq!(package.catalogue.courses.len(), 22);
}

#[test]
fn the_environments_of_the_built_in_catalogue_raise_no_warning() {
    let package =
        load_package(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalogue")).expect("the built-in catalogue is a valid package");
    for course in &package.catalogue.courses {
        for (name, environment) in &course.environments {
            assert_eq!(environment["warnings"], serde_json::json!([]), "{}: environment `{name}`", course.slug);
        }
    }
}
