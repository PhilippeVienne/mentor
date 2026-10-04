//! Compiles a catalogue directory and prints it as JSON (same shape as v1 `export_catalogue`).
use std::path::Path;

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| "catalogue".to_string());
    match mentor_content::load_catalogue(Path::new(&dir)) {
        Ok(catalogue) => println!("{}", serde_json::to_string_pretty(&catalogue).unwrap()),
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}
