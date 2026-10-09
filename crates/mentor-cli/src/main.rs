//! `mentor`: administration of a Mentor platform.
//!
//! Commands that touch the database connect with the role that owns it (see `mentor-db`), never with the
//! application role. `package-check` needs no database: it is the command an author runs on a course package.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use mentor_db::import_v1::{import, Dump, Names};
use mentor_db::packages::{self, Impact, Outcome, Removals};
use mentor_db::platform;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

#[derive(Parser)]
#[command(name = "mentor", about = "Administration of a Mentor platform")]
struct Cli {
    /// Connection URL of the role that owns the database.
    #[arg(long, env = "MENTOR_DATABASE_URL", global = true, hide_env_values = true)]
    database_url: Option<String>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Applies pending database migrations.
    Migrate,
    /// Creates a tenant.
    TenantCreate {
        /// Short identifier: lower-case letters, digits and dashes.
        slug: String,
        /// Display name.
        name: String,
        /// Host name served for this tenant; repeat for several.
        #[arg(long = "host")]
        hosts: Vec<String>,
    },
    /// Imports the data of a v1 portal into a tenant. Can be replayed.
    ImportV1 {
        /// Output of v1's `manage.py dumpdata` (see the documentation of `mentor_db::import_v1`).
        export: PathBuf,
        /// Slug of the tenant that receives the data.
        #[arg(long)]
        tenant: String,
        /// v1's compiled catalogue (`manage.py export_catalogue`), to translate exam question identifiers.
        #[arg(long, requires = "catalogue")]
        v1_catalogue: Option<PathBuf>,
        /// v2 catalogue directory matching `--v1-catalogue`.
        #[arg(long, requires = "v1_catalogue")]
        catalogue: Option<PathBuf>,
    },
    /// Installs a course package for a tenant from a directory, or replaces the installed package of the same
    /// name. The package is validated and compiled first; the tenant's catalogue changes all at once.
    PackageInstall {
        /// Directory of the package (the one holding `mentor.yml`).
        directory: PathBuf,
        /// Slug of the tenant that gets the package.
        #[arg(long)]
        tenant: String,
        /// Only say what the installation would change for learners; write nothing.
        #[arg(long)]
        dry_run: bool,
        /// Accept that the new version takes away courses or lessons of the installed one. Learners' progress
        /// on them is kept, and shows again if they come back.
        #[arg(long)]
        confirm_removals: bool,
    },
    /// Brings back the version a package had before it was last replaced. Running it again undoes that.
    PackageRollback {
        /// Name of the package (the `name` of its manifest).
        name: String,
        #[arg(long)]
        tenant: String,
    },
    /// Lists the packages installed for a tenant, in the order of its catalogue.
    PackageList {
        #[arg(long)]
        tenant: String,
    },
    /// Removes a package from a tenant. Learners' progress on its courses is kept.
    PackageRemove {
        /// Name of the package (the `name` of its manifest).
        name: String,
        #[arg(long)]
        tenant: String,
    },
    /// Validates a course package: its `mentor.yml` manifest, its files and its courses. Needs no database.
    PackageCheck {
        /// Directory of the package (the one holding `mentor.yml`).
        #[arg(default_value = ".")]
        directory: PathBuf,
    },
}

type Failure = Box<dyn std::error::Error>;

/// Identifiers of the lab steps of every lesson, in order: v1 stored the positions of validated steps.
fn step_ids(catalogue: &Path) -> Result<BTreeMap<String, Vec<String>>, Failure> {
    let catalogue = mentor_content::load_catalogue(catalogue)?;
    let mut ids = BTreeMap::new();
    for course in &catalogue.courses {
        for lesson in &course.lessons {
            if let Some(lab) = &lesson.lab {
                ids.insert(format!("{}/{}", course.slug, lesson.slug), lab.steps.iter().map(|step| step.id.clone()).collect());
            }
        }
    }
    Ok(ids)
}

/// Maps v1 exam question identifiers to v2 ones. Both catalogues hold the same questions in the same order;
/// only their identifiers differ: v1 digested the rendered HTML of a question, v2 its source text.
fn question_ids(v1_catalogue: &Path, catalogue: &Path) -> Result<BTreeMap<String, String>, Failure> {
    let v1: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(v1_catalogue)?)?;
    let v2 = mentor_content::load_catalogue(catalogue)?;
    let mut ids = BTreeMap::new();
    for course in &v2.courses {
        let Some(exam) = &course.exam else { continue };
        let old: Vec<&str> = v1["courses"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|candidate| candidate["slug"] == course.slug.as_str())
            .and_then(|candidate| candidate["exam"]["questions"].as_array())
            .map(|questions| questions.iter().filter_map(|question| question["id"].as_str()).collect())
            .unwrap_or_default();
        if old.len() != exam.questions.len() {
            eprintln!(
                "warning: course {}: {} exam questions in v1, {} in v2; its identifiers are not translated",
                course.slug,
                old.len(),
                exam.questions.len()
            );
            continue;
        }
        ids.extend(old.into_iter().zip(&exam.questions).map(|(old, new)| (old.to_string(), new.id.clone())));
    }
    Ok(ids)
}

fn plural(count: usize, noun: &str) -> String {
    format!("{count} {noun}{}", if count == 1 { "" } else { "s" })
}

/// Prints what replacing a package changes for learners, most serious first.
fn print_impact(impact: &Impact) {
    let diff = &impact.diff;
    if diff.is_empty() {
        println!("  nothing changes for learners: no course, lesson, lab step, quiz or exam question is added or removed");
        return;
    }
    let list = |names: &[String]| names.join(", ");
    if !diff.courses_removed.is_empty() {
        println!("  REMOVED {}: {}", plural(diff.courses_removed.len(), "course"), list(&diff.courses_removed));
    }
    for course in diff.courses_changed.iter().filter(|course| !course.lessons_removed.is_empty()) {
        println!("  REMOVED from {}: {} ({})", course.slug, plural(course.lessons_removed.len(), "lesson"), list(&course.lessons_removed));
    }
    if diff.removes_progress() {
        println!(
            "    {} did something there; their progress is kept but no longer shown",
            plural(impact.learners_on_removed as usize, "learner")
        );
    }
    if !diff.courses_added.is_empty() {
        println!("  added {}: {}", plural(diff.courses_added.len(), "course"), list(&diff.courses_added));
    }
    for course in &diff.courses_changed {
        if !course.lessons_added.is_empty() {
            println!("  added to {}: {} ({})", course.slug, plural(course.lessons_added.len(), "lesson"), list(&course.lessons_added));
        }
        for lesson in &course.lessons_changed {
            let mut changes = Vec::new();
            if lesson.steps_added > 0 || lesson.steps_removed > 0 {
                changes.push(format!("lab steps +{} -{}", lesson.steps_added, lesson.steps_removed));
            }
            if lesson.questions_before != lesson.questions_after {
                changes.push(format!("quiz {} -> {} questions", lesson.questions_before, lesson.questions_after));
            }
            println!("  changed {}/{}: {}", course.slug, lesson.slug, changes.join(", "));
        }
        if course.exam_changed() {
            println!("  exam of {}: +{} -{} questions", course.slug, course.exam_questions_added, course.exam_questions_removed);
        }
        if course.published_before != course.published_after {
            println!("  {} is {}", course.slug, if course.published_after { "now published" } else { "no longer published" });
        }
    }
    if impact.completions_lost > 0 {
        println!(
            "    {} had completed a course that gains lessons: it will count as unfinished",
            plural(impact.completions_lost as usize, "learner")
        );
    }
    if impact.open_attempts > 0 {
        println!(
            "    {} in progress on a changed exam: questions that left the pool will be ignored",
            plural(impact.open_attempts as usize, "attempt")
        );
    }
    if !diff.paths_added.is_empty() || !diff.paths_removed.is_empty() {
        println!("  training paths: added [{}], removed [{}]", list(&diff.paths_added), list(&diff.paths_removed));
    }
}

/// Compiles a package and prints what it holds; the first error stops it and names the file.
fn package_check(directory: &Path) -> Result<(), Failure> {
    let package = mentor_content::load_package(directory)?;
    let manifest = &package.manifest;
    println!("package {} {} (format {}, {}): {}", manifest.name, manifest.version, manifest.format, manifest.license, manifest.title);
    let mut warnings = Vec::new();
    for course in &package.catalogue.courses {
        let labs = course.lessons.iter().filter(|lesson| lesson.lab.is_some()).count();
        let exam = match &course.exam {
            Some(exam) => format!("exam of {}", plural(exam.questions.len(), "question")),
            None => "no exam".to_string(),
        };
        let environments: Vec<&str> = course.environments.keys().map(String::as_str).collect();
        let environments = if environments.is_empty() { "none".to_string() } else { environments.join(", ") };
        let state = if course.published { "" } else { " (not published)" };
        println!(
            "  {}{state}: {}, {}, {exam}, environments: {environments}",
            course.slug,
            plural(course.lessons.len(), "lesson"),
            plural(labs, "lab")
        );
        for (name, environment) in &course.environments {
            let raised = environment["warnings"].as_array().into_iter().flatten().filter_map(|warning| warning.as_str());
            warnings.extend(raised.map(|warning| format!("{}: environment `{name}`: {warning}", course.slug)));
        }
        if let Some(exam) = &course.exam {
            warnings.extend(exam.warnings.iter().map(|warning| format!("{}: exam: {warning}", course.slug)));
        }
    }
    println!(
        "{}, {}, {} bytes: valid",
        plural(package.catalogue.courses.len(), "course"),
        plural(package.tree.files, "file"),
        package.tree.bytes
    );
    for warning in &warnings {
        eprintln!("warning: {warning}");
    }
    Ok(())
}

async fn run(cli: Cli) -> Result<(), Failure> {
    if let Command::PackageCheck { directory } = &cli.command {
        return package_check(directory);
    }
    let url = cli.database_url.ok_or("no database: pass --database-url or set MENTOR_DATABASE_URL")?;
    let owner: PgPool = PgPoolOptions::new().max_connections(2).connect(&url).await?;
    match cli.command {
        Command::Migrate => {
            mentor_db::migrate(&owner).await?;
            println!("migrations applied");
        }
        Command::TenantCreate { slug, name, hosts } => {
            let hosts: Vec<&str> = hosts.iter().map(String::as_str).collect();
            let id = platform::create_tenant(&owner, &slug, &name, &hosts).await?;
            println!("tenant {slug} created ({id})");
        }
        Command::ImportV1 { export, tenant, v1_catalogue, catalogue } => {
            let id = platform::tenant_by_slug(&owner, &tenant).await?.ok_or_else(|| format!("no tenant with slug {tenant}"))?;
            let dump = Dump::parse(&std::fs::read_to_string(&export)?)?;
            let mut names = Names::default();
            if let Some(v2) = &catalogue {
                names.step_ids = step_ids(v2)?;
                if let Some(v1) = &v1_catalogue {
                    names.question_ids = question_ids(v1, v2)?;
                }
            }
            let report = import(&owner, id, &dump, &names).await?;
            println!(
                "imported into {tenant}: {} learners, {} lesson progress rows, {} XP events, {} badges, {} cohorts, {} memberships, {} exam attempts",
                report.learners, report.progress, report.awards, report.badges, report.cohorts, report.memberships, report.attempts
            );
            for warning in &report.warnings {
                eprintln!("warning: {warning}");
            }
        }
        Command::PackageInstall { directory, tenant, dry_run, confirm_removals } => {
            let id = platform::tenant_by_slug(&owner, &tenant).await?.ok_or_else(|| format!("no tenant with slug {tenant}"))?;
            // Validated and compiled before anything is written: a package that does not compile changes nothing.
            let package = mentor_content::load_package(&directory)?;
            let images = mentor_content::package_images(&directory, &package)?;
            let (name, version) = (&package.manifest.name, &package.manifest.version);
            let impact = packages::preview(&owner, id, &package).await?;
            match &impact {
                Some(impact) => {
                    println!("package {name}: {} is installed for {tenant}, {version} would replace it", impact.installed_version);
                    print_impact(impact);
                }
                None => println!("package {name} {version} is not installed for {tenant} yet"),
            }
            if dry_run {
                println!("dry run: nothing was written");
                return Ok(());
            }
            let removals = if confirm_removals { Removals::Confirmed } else { Removals::Refuse };
            let source = directory.canonicalize().unwrap_or(directory).display().to_string();
            let outcome = match packages::install(&owner, id, &package, &images, &source, removals).await {
                Err(mentor_db::Error::RemovalsNotConfirmed(_)) => {
                    return Err("this version removes courses or lessons (see above): nothing was written. \
                                Run again with --confirm-removals to install it; learners keep their progress on what is removed"
                        .into());
                }
                other => other?,
            };
            println!(
                "package {name} {version} {} for {tenant}: {}, {}, {}",
                match outcome {
                    Outcome::Installed => "installed",
                    Outcome::Replaced => "replaced the installed version (undo with package-rollback)",
                },
                plural(package.catalogue.courses.len(), "course"),
                plural(package.paths.len(), "training path"),
                plural(images.len(), "picture")
            );
        }
        Command::PackageRollback { name, tenant } => {
            let id = platform::tenant_by_slug(&owner, &tenant).await?.ok_or_else(|| format!("no tenant with slug {tenant}"))?;
            match packages::rollback(&owner, id, &name).await? {
                Some(version) => println!("package {name}: version {version} is back for {tenant} (run again to undo)"),
                None => return Err(format!("no previous version of {name} is kept for {tenant}").into()),
            }
        }
        Command::PackageList { tenant } => {
            let id = platform::tenant_by_slug(&owner, &tenant).await?.ok_or_else(|| format!("no tenant with slug {tenant}"))?;
            let installed = packages::list(&owner, id).await?;
            if installed.is_empty() {
                println!("no package is installed for {tenant}: its catalogue is empty");
            }
            for package in installed {
                println!(
                    "{} {} ({}): {}, {}, from {}",
                    package.name,
                    package.version,
                    package.title,
                    plural(package.courses, "course"),
                    plural(package.paths, "training path"),
                    package.source
                );
            }
        }
        Command::PackageRemove { name, tenant } => {
            let id = platform::tenant_by_slug(&owner, &tenant).await?.ok_or_else(|| format!("no tenant with slug {tenant}"))?;
            if !packages::remove(&owner, id, &name).await? {
                return Err(format!("no package named {name} is installed for {tenant}").into());
            }
            println!("package {name} removed from {tenant}");
        }
        Command::PackageCheck { .. } => unreachable!("handled before connecting to the database"),
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    match run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("mentor: {err}");
            ExitCode::FAILURE
        }
    }
}
