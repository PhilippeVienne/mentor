//! `mentor`: administration of a Mentor platform.
//!
//! Every command connects with the role that owns the database (see `mentor-db`), never with the
//! application role.

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

async fn run(cli: Cli) -> Result<(), Failure> {
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
