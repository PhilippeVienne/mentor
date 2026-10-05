//! Migrations are embedded at compile time by `sqlx::migrate!`. Cargo does not watch that directory on its
//! own: without this, adding a migration file would not rebuild the crate and the migration would silently
//! not run.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
