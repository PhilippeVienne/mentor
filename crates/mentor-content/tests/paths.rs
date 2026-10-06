//! The training paths shipped with the repository's catalogue are valid.

use std::path::Path;

use mentor_content::{load_catalogue, load_paths};

#[test]
fn the_paths_of_the_catalogue_are_valid_and_self_contained() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalogue");
    let catalogue = load_catalogue(&directory).expect("the catalogue compiles");
    let paths = load_paths(&directory, &catalogue.courses).expect("paths.yml is valid");
    let ids: Vec<&str> = paths.iter().map(|path| path.id.as_str()).collect();
    assert_eq!(ids, ["socle-commun", "developpement-frontend", "backend-python", "devops-infrastructure"]);
    for path in &paths {
        // What the compiler guarantees, checked here on real data: every prerequisite comes earlier.
        let order: Vec<&str> = path.courses().map(|entry| entry.course.as_str()).collect();
        for (position, slug) in order.iter().enumerate() {
            let course = catalogue.courses.iter().find(|course| course.slug == *slug).unwrap();
            assert!(course.requires.iter().all(|required| order[..position].contains(&required.as_str())), "{}: {slug}", path.id);
        }
    }
    // A flat path is one stage without a title.
    assert_eq!((paths[0].stages.len(), paths[0].stages[0].title.as_str()), (1, ""));
    assert_eq!(paths[3].stages.iter().map(|stage| stage.courses.len()).collect::<Vec<_>>(), [3, 2, 3]);
}

#[test]
fn a_catalogue_without_a_paths_file_has_no_path() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(load_paths(&directory, &[]).unwrap(), []);
}
