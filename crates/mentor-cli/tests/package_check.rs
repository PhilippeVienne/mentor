//! `mentor package-check`: runs without a database, prints what a package holds, fails on a broken one.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn check(directory: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mentor")).arg("package-check").arg(directory).env_remove("MENTOR_DATABASE_URL").output().unwrap()
}

fn built_in() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalogue")
}

#[test]
fn the_built_in_catalogue_is_a_valid_package() {
    let output = check(&built_in());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(stdout.starts_with("package mentor-courses 1.0.0 (format 1, AGPL-3.0-or-later): "), "{stdout}");
    assert!(stdout.contains("\n  git-basics: 7 lessons, 7 labs, exam of "), "{stdout}");
    assert!(stdout.contains("\n  django (not published): 0 lessons, 0 labs, no exam, environments: none\n"), "{stdout}");
    assert!(stdout.contains("\n19 courses, "), "{stdout}");
    assert!(stdout.trim_end().ends_with("bytes: valid"), "{stdout}");
}

#[test]
fn a_broken_package_fails_and_names_the_file() {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("package-check-broken");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("mentor.yml"), "format: 1\nname: broken\nversion: 0.1.0\ntitle: Broken\nlicense: MIT\ncourses: [absent]\n")
        .unwrap();
    let output = check(&root);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(String::from_utf8(output.stderr).unwrap(), format!("mentor: {}: `course.md` is missing\n", root.join("absent").display()));
}
