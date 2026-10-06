//! `mentor`: administration of a Mentor platform.
//!
//! Commands that touch the database connect with the role that owns it (see `mentor-db`), never with the
//! application role. `package-check` needs no database: it is the command an author runs on a course package.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use mentor_db::import_v1::{import, Dump};
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
    /// Validates a course package: its `mentor.yml` manifest, its files and its courses. Needs no database.
    PackageCheck {
        /// Directory of the package (the one holding `mentor.yml`).
        #[arg(default_value = ".")]
        directory: PathBuf,
    },
}

type Failure = Box<dyn std::error::Error>;

/// Maps v1 exam question identifiers to v2 ones. Both catalogues hold the same questions in the same order;
/// only their identifiers differ, because they are digests of differently rendered HTML.
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
            let ids = match (v1_catalogue, catalogue) {
                (Some(v1), Some(v2)) => question_ids(&v1, &v2)?,
                _ => BTreeMap::new(),
            };
            let report = import(&owner, id, &dump, &ids).await?;
            println!(
                "imported into {tenant}: {} learners, {} lesson progress rows, {} XP events, {} badges, {} cohorts, {} memberships, {} exam attempts",
                report.learners, report.progress, report.awards, report.badges, report.cohorts, report.memberships, report.attempts
            );
            for warning in &report.warnings {
                eprintln!("warning: {warning}");
            }
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
