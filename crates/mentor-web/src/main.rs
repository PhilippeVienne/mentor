//! `mentor-web`: the web server.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use clap::Parser;
use mentor_web::{router, AppState};
use sqlx::postgres::PgPoolOptions;

#[derive(Parser)]
#[command(name = "mentor-web", about = "Web server of Mentor")]
struct Options {
    /// Connection URL of the application role (a member of `mentor_app`, never the owning role).
    #[arg(long, env = "MENTOR_APP_DATABASE_URL", hide_env_values = true)]
    database_url: String,
    /// Catalogue directory, compiled at start-up.
    #[arg(long, env = "MENTOR_CATALOGUE", default_value = "catalogue")]
    catalogue: PathBuf,
    /// Directory of style sheets, scripts and default brand images.
    #[arg(long, env = "MENTOR_STATIC", default_value = "static")]
    static_dir: PathBuf,
    #[arg(long, env = "MENTOR_LISTEN", default_value = "127.0.0.1:8300")]
    listen: SocketAddr,
}

async fn run(options: Options) -> Result<(), Box<dyn std::error::Error>> {
    let catalogue = mentor_content::load_catalogue(&options.catalogue)?;
    eprintln!("mentor-web: {} courses compiled from {}", catalogue.courses.len(), options.catalogue.display());
    let db = PgPoolOptions::new().max_connections(10).connect(&options.database_url).await?;
    let app = router(AppState { db, catalogue: Arc::new(catalogue) }, &options.static_dir, &options.catalogue);
    let listener = tokio::net::TcpListener::bind(options.listen).await?;
    eprintln!("mentor-web: listening on http://{}", options.listen);
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    match run(Options::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("mentor-web: {err}");
            ExitCode::FAILURE
        }
    }
}
