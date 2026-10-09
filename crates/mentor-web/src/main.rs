//! `mentor-web`: the web server.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use mentor_web::{router, AppState};
use sqlx::postgres::PgPoolOptions;

#[derive(Parser)]
#[command(name = "mentor-web", about = "Web server of Mentor")]
struct Options {
    /// Connection URL of the application role (a member of `mentor_app`, never the owning role).
    #[arg(long, env = "MENTOR_APP_DATABASE_URL", hide_env_values = true)]
    database_url: String,
    /// Directory of style sheets, scripts and default brand images.
    #[arg(long, env = "MENTOR_STATIC", default_value = "static")]
    static_dir: PathBuf,
    #[arg(long, env = "MENTOR_LISTEN", default_value = "127.0.0.1:8300")]
    listen: SocketAddr,
    /// Secret that signs session cookies. Without it a random one is used and sessions end when the server
    /// stops.
    #[arg(long, env = "MENTOR_SESSION_SECRET", hide_env_values = true)]
    session_secret: Option<String>,
    /// Offer a password-less sign-in page. For development only: anyone could sign in as anyone.
    #[arg(long)]
    dev_login: bool,
}

async fn run(options: Options) -> Result<(), Box<dyn std::error::Error>> {
    let db = PgPoolOptions::new().max_connections(10).connect(&options.database_url).await?;
    let secret = match options.session_secret {
        Some(secret) if secret.len() >= 32 => secret.into_bytes(),
        Some(_) => return Err("the session secret must be at least 32 characters long".into()),
        None => {
            eprintln!("mentor-web: no session secret given: using a random one, sessions will not survive a restart");
            [uuid::Uuid::new_v4().into_bytes(), uuid::Uuid::new_v4().into_bytes()].concat()
        }
    };
    if options.dev_login {
        eprintln!("mentor-web: WARNING: development sign-in is enabled, anyone can sign in as anyone");
    }
    let app = router(AppState::new(db, secret, options.dev_login), &options.static_dir);
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
