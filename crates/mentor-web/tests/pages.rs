//! Pages served per tenant, against a real PostgreSQL and the repository's catalogue.
//!
//! Needs `MENTOR_TEST_DATABASE_URL` (a superuser URL, see the README); skipped without it. As in `mentor-db`,
//! migrations run as an ordinary owner and the server connects as an unprivileged application role.

use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use mentor_db::platform;
use mentor_web::{router, AppState};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, PgPool};
use tower::ServiceExt;
use uuid::Uuid;

const PASSWORD: &str = "mentor-test";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

async fn ensure_role(admin: &PgPool, definition: &str) {
    if let Err(err) = sqlx::query(&format!("CREATE ROLE {definition}")).execute(admin).await {
        let message = err.to_string();
        assert!(message.contains("already exists") || message.contains("duplicate"), "{message}");
    }
}

/// A server with three tenants: `acme.test` with its own brand, `plain.test` with none, and `hostile.test`
/// whose brand texts contain markup.
async fn app_with(dev_login: bool) -> Option<Router> {
    app_built(dev_login, true).await
}

/// Same, with or without the training paths of the catalogue.
async fn app_built(dev_login: bool, with_paths: bool) -> Option<Router> {
    let Ok(url) = std::env::var("MENTOR_TEST_DATABASE_URL") else {
        eprintln!("skipped: MENTOR_TEST_DATABASE_URL is not set");
        return None;
    };
    let admin_options: PgConnectOptions = url.parse().unwrap();
    let admin = PgPoolOptions::new().max_connections(1).connect_with(admin_options.clone()).await.unwrap();
    ensure_role(&admin, &format!("mentor_test_owner LOGIN CREATEROLE PASSWORD '{PASSWORD}'")).await;
    let name = format!("mentor_web_test_{}", Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE DATABASE {name} OWNER mentor_test_owner")).execute(&admin).await.unwrap();
    let connect = |login: &str| admin_options.clone().database(&name).username(login).password(PASSWORD).disable_statement_logging();
    let owner = PgPoolOptions::new().max_connections(2).connect_with(connect("mentor_test_owner")).await.unwrap();
    mentor_db::migrate(&owner).await.unwrap();
    ensure_role(&admin, &format!("mentor_test_app LOGIN PASSWORD '{PASSWORD}' IN ROLE mentor_app")).await;

    let acme = platform::create_tenant(&owner, "acme", "Acme", &["acme.test"]).await.unwrap();
    let brand =
        serde_json::json!({"name": "Acme Academy", "organisation": "Acme Corp", "contact_email": "help@acme.test", "primary": "#112233"});
    platform::set_branding(&owner, acme, &brand).await.unwrap();
    platform::create_tenant(&owner, "plain", "Plain", &["plain.test"]).await.unwrap();
    // A tenant whose administrator typed markup everywhere a brand accepts text.
    let hostile = platform::create_tenant(&owner, "hostile", "<script>alert(1)</script>", &["hostile.test"]).await.unwrap();
    let markup = serde_json::json!({"tagline": "<img src=x onerror=alert(2)>", "organisation": "\"><script>alert(3)</script>", "contact_email": "x\" onclick=\"alert(4)"});
    platform::set_branding(&owner, hostile, &markup).await.unwrap();

    let db = PgPoolOptions::new().max_connections(4).connect_with(connect("mentor_test_app")).await.unwrap();
    let catalogue_dir = root().join("catalogue");
    let catalogue = mentor_content::load_catalogue(&catalogue_dir).expect("the catalogue compiles");
    let paths =
        if with_paths { mentor_content::load_paths(&catalogue_dir, &catalogue.courses).expect("the paths are valid") } else { Vec::new() };
    let state = AppState::new(db, catalogue, b"a secret that only these tests know".to_vec(), dev_login).with_paths(paths);
    Some(router(state, &root().join("static"), &catalogue_dir))
}

async fn app() -> Option<Router> {
    app_with(true).await
}

/// One request. `cookie` is a session cookie value; `body` makes it a POST of that content type, sent from
/// `origin` (the site itself unless stated otherwise).
struct Call<'a> {
    host: &'a str,
    path: &'a str,
    cookie: Option<&'a str>,
    body: Option<(&'a str, &'a str)>,
    origin: Option<&'a str>,
}

struct Answer {
    status: StatusCode,
    body: String,
    /// Value of the session cookie the response sets, if any.
    session: Option<String>,
}

async fn call(app: &Router, call: Call<'_>) -> Answer {
    let mut request = Request::builder().uri(call.path).header("host", call.host);
    if let Some(cookie) = call.cookie {
        request = request.header("cookie", format!("mentor_session={cookie}"));
    }
    let request = match call.body {
        Some((content_type, body)) => {
            let origin = call.origin.map(str::to_string).unwrap_or_else(|| format!("http://{}", call.host));
            request.method("POST").header("content-type", content_type).header("origin", origin).body(Body::from(body.to_string())).unwrap()
        }
        None => request.body(Body::empty()).unwrap(),
    };
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let session = response
        .headers()
        .get("set-cookie")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("mentor_session="))
        .and_then(|value| value.split(';').next())
        .map(str::to_string);
    let body = axum::body::to_bytes(response.into_body(), 16 * 1024 * 1024).await.unwrap();
    Answer { status, body: String::from_utf8_lossy(&body).into_owned(), session }
}

async fn get(app: &Router, host: &str, path: &str) -> (StatusCode, String) {
    let answer = call(app, Call { host, path, cookie: None, body: None, origin: None }).await;
    (answer.status, answer.body)
}

async fn sign_in(app: &Router, host: &str, username: &str) -> String {
    let form = format!("username={username}");
    let answer =
        call(app, Call { host, path: "/dev/login", cookie: None, body: Some(("application/x-www-form-urlencoded", &form)), origin: None })
            .await;
    assert_eq!(answer.status, StatusCode::SEE_OTHER, "{}", answer.body);
    answer.session.expect("signing in sets the session cookie")
}

async fn progress(app: &Router, host: &str, cookie: Option<&str>, json: &str) -> (StatusCode, serde_json::Value) {
    let answer = call(app, Call { host, path: "/api/progress", cookie, body: Some(("application/json", json)), origin: None }).await;
    (answer.status, serde_json::from_str(&answer.body).unwrap_or(serde_json::Value::Null))
}

/// A quiz score on the first lesson of `git-basics` (3 questions, a real lab that does not gate the quiz here).
const QUIZ: &str = r#"{"course": "git-basics", "lesson": "introduction", "type": "quiz", "score": 3}"#;

#[tokio::test]
async fn each_host_gets_its_own_brand() {
    let Some(app) = app().await else { return };
    let (status, acme) = get(&app, "acme.test", "/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(acme.contains("Acme Academy") && acme.contains("Acme Corp") && acme.contains("help@acme.test"));
    assert!(acme.contains("--primary: #112233"));

    let (status, plain) = get(&app, "PLAIN.test:8300", "/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(plain.contains("<strong>Plain</strong>") && plain.contains("--primary: #4f46e5"));
    assert!(!plain.contains("Acme") && !plain.contains("mailto:"));
}

#[tokio::test]
async fn an_unknown_host_is_served_nothing() {
    let Some(app) = app().await else { return };
    for path in ["/", "/catalogue/", "/courses/git-basics/"] {
        let (status, body) = get(&app, "elsewhere.test", path).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
        assert!(!body.contains("Acme") && !body.contains("Plain") && !body.contains("<html"), "{path}");
    }
    // The health check is the one route that needs no tenant.
    assert_eq!(get(&app, "elsewhere.test", "/healthz").await.0, StatusCode::OK);
}

#[tokio::test]
async fn catalogue_course_and_lesson_pages_render() {
    let Some(app) = app().await else { return };
    let (status, catalogue) = get(&app, "acme.test", "/catalogue/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(catalogue.contains(r#"data-course="git-basics""#) && catalogue.contains(r#"href="/courses/git-basics/""#));
    // Unpublished courses are listed as upcoming, without a link.
    assert!(catalogue.contains("À venir") && !catalogue.contains(r#"href="/courses/django/""#));

    let (status, course) = get(&app, "acme.test", "/courses/git-basics/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(course.contains("<h1>Git basics</h1>") && course.contains(r#"href="/courses/git-basics/introduction/""#));
    assert!(course.contains("/static/catalogue/git-basics/images/banniere.svg"));

    let (status, lesson) = get(&app, "acme.test", "/courses/git-basics/introduction/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(lesson.contains("<h1>Pourquoi Git ?</h1>") && lesson.contains("Leçon 1 / 7"));
    assert!(lesson.contains("Leçon suivante") && !lesson.contains("Leçon précédente"));
    // The lesson body is compiled HTML, inserted as is.
    assert!(lesson.contains(r#"<div class="cards">"#));
    assert!(lesson.contains("Labo à venir"));
    // A visitor is invited to sign in; the answers of the quiz are not sent to them.
    assert!(lesson.contains("Connecte-toi pour répondre au quiz") && !lesson.contains("lesson-data"));
}

#[tokio::test]
async fn missing_pages_answer_404_with_the_brand() {
    let Some(app) = app().await else { return };
    for path in ["/courses/nope/", "/courses/git-basics/nope/", "/courses/django/", "/nowhere"] {
        let (status, body) = get(&app, "acme.test", path).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
        assert!(body.contains("Page introuvable") && body.contains("Acme Academy"), "{path}");
    }
}

#[tokio::test]
async fn static_files_and_catalogue_images_are_served() {
    let Some(app) = app().await else { return };
    assert_eq!(get(&app, "acme.test", "/static/css/portail.css").await.0, StatusCode::OK);
    assert_eq!(get(&app, "acme.test", "/static/brand/logo.svg").await.0, StatusCode::OK);
    let (status, image) = get(&app, "acme.test", "/static/catalogue/git-basics/images/banniere.svg").await;
    assert_eq!(status, StatusCode::OK);
    assert!(image.contains("<svg"));
    assert_eq!(get(&app, "acme.test", "/static/css/../../Cargo.toml").await.0, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn text_from_the_brand_is_escaped() {
    let Some(app) = app().await else { return };
    let (status, page) = get(&app, "hostile.test", "/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        !page.contains("<script>alert") && !page.contains("<img src=x") && !page.contains("onclick=\"alert"),
        "markup from the brand reached the page"
    );
    assert!(page.contains("&lt;script&gt;alert(1)&lt;/script&gt;") || page.contains("&#60;script&#62;alert(1)&#60;/script&#62;"));
}

#[tokio::test]
async fn a_signed_in_learner_gets_the_quiz_and_earns_xp_once() {
    let Some(app) = app().await else { return };
    let cookie = sign_in(&app, "acme.test", "alice").await;

    let page =
        call(&app, Call { host: "acme.test", path: "/courses/git-basics/introduction/", cookie: Some(&cookie), body: None, origin: None })
            .await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.body.contains(r#"<div id="quiz-root""#) && page.body.contains(r#"id="lesson-data""#));
    assert!(page.body.contains(r#""apiUrl":"/api/progress""#) && page.body.contains(r#""labRequired":false"#));
    assert!(page.body.contains("Niv. 1 · 0 XP") && page.body.contains("Déconnexion"));

    let (status, first) = progress(&app, "acme.test", Some(&cookie), QUIZ).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    // Three correct answers (8 XP each) and the lesson (50 XP); its first step into level 1 → still level 1.
    assert_eq!((first["xp_gained"].as_u64(), first["lesson_completed"].as_bool()), (Some(74), Some(true)));
    assert_eq!(first["level"]["xp"], 74);
    assert_eq!(first["new_badges"][0]["slug"], "sans-faute");
    assert_eq!(first["progress"]["quiz_best"], 3);

    let (status, again) = progress(&app, "acme.test", Some(&cookie), QUIZ).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!((again["xp_gained"].as_u64(), again["lesson_completed"].as_bool()), (Some(0), Some(false)));
    assert!(again["new_badges"].as_array().unwrap().is_empty());

    // The next page load shows the stored state.
    let page =
        call(&app, Call { host: "acme.test", path: "/courses/git-basics/introduction/", cookie: Some(&cookie), body: None, origin: None })
            .await;
    assert!(page.body.contains("Niv. 1 · 74 XP") && page.body.contains(r#""quizBest":3"#) && page.body.contains(r#""completed":true"#));
}

#[tokio::test]
async fn progress_is_refused_without_a_session_from_another_site_or_for_another_tenant() {
    let Some(app) = app().await else { return };
    let cookie = sign_in(&app, "acme.test", "alice").await;

    assert_eq!(progress(&app, "acme.test", None, QUIZ).await.0, StatusCode::UNAUTHORIZED);
    // A page of another site making the learner's browser post: the cookie is there, the origin is wrong.
    let forged = call(
        &app,
        Call {
            host: "acme.test",
            path: "/api/progress",
            cookie: Some(&cookie),
            body: Some(("application/json", QUIZ)),
            origin: Some("https://evil.test"),
        },
    )
    .await;
    assert_eq!(forged.status, StatusCode::FORBIDDEN);
    // The same cookie presented to another tenant designates nobody there.
    assert_eq!(progress(&app, "plain.test", Some(&cookie), QUIZ).await.0, StatusCode::UNAUTHORIZED);
    // A tampered cookie is no session at all.
    let flipped = if cookie.ends_with('0') { '1' } else { '0' };
    let tampered = format!("{}{flipped}", &cookie[..cookie.len() - 1]);
    assert_eq!(progress(&app, "acme.test", Some(&tampered), QUIZ).await.0, StatusCode::UNAUTHORIZED);

    // Nothing above was recorded.
    let page = call(&app, Call { host: "acme.test", path: "/", cookie: Some(&cookie), body: None, origin: None }).await;
    assert!(page.body.contains("Niv. 1 · 0 XP"));
}

#[tokio::test]
async fn the_rules_decide_what_the_browser_may_report() {
    let Some(app) = app().await else { return };
    let cookie = sign_in(&app, "acme.test", "alice").await;
    let post = |json: &'static str| progress(&app, "acme.test", Some(&cookie), json);
    // A step of a real lab can only come from the server.
    let (status, body) = post(r#"{"course": "git-basics", "lesson": "introduction", "type": "task", "task": "s0123456789"}"#).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(post(r#"{"course": "git-basics", "lesson": "introduction", "type": "quiz", "score": 99}"#).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(post(r#"{"course": "git-basics", "lesson": "nope", "type": "quiz", "score": 1}"#).await.0, StatusCode::NOT_FOUND);
    assert_eq!(post(r#"{"course": "git-basics", "lesson": "introduction", "type": "dance"}"#).await.0, StatusCode::BAD_REQUEST);
    // `docker-advanced` requires `docker-hello`, which this learner has not completed.
    let (status, body) = post(r#"{"course": "docker-advanced", "lesson": "dockerfile", "type": "quiz", "score": 1}"#).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
}

#[tokio::test]
async fn development_sign_in_exists_only_when_enabled_and_checks_its_input() {
    let Some(disabled) = app_with(false).await else { return };
    assert_eq!(get(&disabled, "acme.test", "/dev/login").await.0, StatusCode::NOT_FOUND);
    let form = Some(("application/x-www-form-urlencoded", "username=alice"));
    let attempt = call(&disabled, Call { host: "acme.test", path: "/dev/login", cookie: None, body: form, origin: None }).await;
    assert!(attempt.status != StatusCode::SEE_OTHER && attempt.session.is_none());
    // Without it, the page offers no way to sign in.
    assert!(!get(&disabled, "acme.test", "/").await.1.contains("/dev/login"));

    let Some(enabled) = app().await else { return };
    assert!(get(&enabled, "acme.test", "/").await.1.contains(r#"href="/dev/login""#));
    let bad = Some(("application/x-www-form-urlencoded", "username=%3Cscript%3E"));
    let refused = call(&enabled, Call { host: "acme.test", path: "/dev/login", cookie: None, body: bad, origin: None }).await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert!(refused.session.is_none() && refused.body.contains("Choisis un identifiant"));
    let cross =
        call(&enabled, Call { host: "acme.test", path: "/dev/login", cookie: None, body: form, origin: Some("https://evil.test") }).await;
    assert_eq!(cross.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn signing_out_closes_the_session() {
    let Some(app) = app().await else { return };
    let cookie = sign_in(&app, "acme.test", "alice").await;
    let out = call(
        &app,
        Call {
            host: "acme.test",
            path: "/logout",
            cookie: Some(&cookie),
            body: Some(("application/x-www-form-urlencoded", "")),
            origin: None,
        },
    )
    .await;
    assert_eq!(out.status, StatusCode::SEE_OTHER);
    assert_eq!(out.session.as_deref(), Some(""));
}

async fn page(app: &Router, cookie: &str, path: &str) -> String {
    let answer = call(app, Call { host: "acme.test", path, cookie: Some(cookie), body: None, origin: None }).await;
    assert_eq!(answer.status, StatusCode::OK, "{path}");
    answer.body
}

#[tokio::test]
async fn the_dashboard_follows_what_the_learner_did() {
    let Some(app) = app().await else { return };
    let cookie = sign_in(&app, "acme.test", "alice").await;

    let dashboard = page(&app, &cookie, "/dashboard/").await;
    assert!(dashboard.contains("Salut alice") && dashboard.contains("Encore 100 XP pour devenir <strong>Apprenti·e</strong>"));
    // Nothing done yet: the first lesson of the first course is proposed, not "resumed".
    assert!(dashboard.contains("Prochaine leçon") && dashboard.contains(r#"href="/courses/git-basics/introduction/">Commencer"#));
    assert!(dashboard.contains("0 / 7 leçons · 0 %"));
    // A course whose prerequisite is not completed is shown locked, with what it takes to open it.
    assert!(dashboard.contains("🔒 Verrouillé") && dashboard.contains("À débloquer en terminant : <strong>Docker"));
    assert!(dashboard.contains(r#"href="/dashboard/" aria-current="page""#));

    assert_eq!(progress(&app, "acme.test", Some(&cookie), QUIZ).await.0, StatusCode::OK);

    let dashboard = page(&app, &cookie, "/dashboard/").await;
    assert!(dashboard.contains("1 / 7 leçons · 14 %") && dashboard.contains("Encore 26 XP"));
    assert!(dashboard.contains(r#"<span class="muted">1 / "#));
    // The next lesson of the same course, which was not touched.
    assert!(dashboard.contains("Prochaine leçon") && !dashboard.contains(r#"href="/courses/git-basics/introduction/">"#));

    let badges = page(&app, &cookie, "/badges/").await;
    assert!(badges.contains("1 badge sur ") && badges.contains("Obtenu le ") && badges.contains("🔒 À débloquer"));
    assert!(badges.contains(r#"class="levels__item is-current""#) && badges.contains("Légende de la maison"));

    let course = page(&app, &cookie, "/courses/git-basics/").await;
    assert!(course.contains("timeline__item--done") && course.contains("Quiz 3 / 3") && course.contains("Terminée"));
    assert!(course.contains("timeline__item--new") && course.contains("À faire"));
    let lesson = page(&app, &cookie, "/courses/git-basics/introduction/").await;
    assert!(lesson.contains("stepper__item--done is-current") && lesson.contains("stepper__item--new"));
    let catalogue = page(&app, &cookie, "/catalogue/").await;
    assert!(catalogue.contains(r#"data-course="git-basics""#) && catalogue.contains(r#"data-state="en-cours""#));
    assert!(catalogue.contains(r#"data-filter="termines""#));
}

#[tokio::test]
async fn learner_pages_need_a_session_and_show_only_that_tenants_progress() {
    let Some(app) = app().await else { return };
    for path in ["/dashboard/", "/badges/"] {
        let answer = call(&app, Call { host: "acme.test", path, cookie: None, body: None, origin: None }).await;
        assert_eq!(answer.status, StatusCode::SEE_OTHER, "{path}");
    }
    // A visitor sees no progress, no state and no learner navigation.
    let course = get(&app, "acme.test", "/courses/git-basics/").await.1;
    assert!(!course.contains("timeline__item--") && !course.contains("À faire") && !course.contains("/dashboard/"));
    assert!(!get(&app, "acme.test", "/catalogue/").await.1.contains(r#"data-filter="termines""#));

    // The same user name on two tenants is two learners.
    let acme = sign_in(&app, "acme.test", "alice").await;
    let plain = sign_in(&app, "plain.test", "alice").await;
    assert_eq!(progress(&app, "acme.test", Some(&acme), QUIZ).await.0, StatusCode::OK);
    let elsewhere = call(&app, Call { host: "plain.test", path: "/dashboard/", cookie: Some(&plain), body: None, origin: None }).await;
    assert!(elsewhere.body.contains("0 / 7 leçons · 0 %") && elsewhere.body.contains("Niv. 1 · 0 XP"));
    // And a session of one tenant opens nothing on the other.
    let crossed = call(&app, Call { host: "plain.test", path: "/dashboard/", cookie: Some(&acme), body: None, origin: None }).await;
    assert_eq!(crossed.status, StatusCode::SEE_OTHER);
}

async fn post_json(app: &Router, cookie: Option<&str>, path: &str, json: &str) -> (StatusCode, serde_json::Value) {
    let answer = call(app, Call { host: "acme.test", path, cookie, body: Some(("application/json", json)), origin: None }).await;
    (answer.status, serde_json::from_str(&answer.body).unwrap_or(serde_json::Value::Null))
}

/// Answers of an attempt: the right option of every question, or a wrong one, found in the catalogue by the
/// text of the options since the page never receives which one is right.
fn exam_answers(attempt: &serde_json::Value, right: bool) -> serde_json::Value {
    let catalogue = mentor_content::load_catalogue(&root().join("catalogue")).unwrap();
    let pool = &catalogue.courses.iter().find(|course| course.slug == "git-basics").unwrap().exam.as_ref().unwrap().questions;
    let mut answers = serde_json::Map::new();
    for question in attempt["questions"].as_array().unwrap() {
        let source = pool.iter().find(|candidate| candidate.id == question["id"].as_str().unwrap()).unwrap();
        let correct = &source.options.iter().find(|option| option.correct).unwrap().html;
        let shown = question["options"].as_array().unwrap();
        let position = shown.iter().position(|option| (option["html"].as_str() == Some(correct.as_str())) == right).unwrap();
        answers.insert(source.id.clone(), position.into());
    }
    serde_json::json!({ "attempt": attempt["attempt"], "answers": answers })
}

#[tokio::test]
async fn an_exam_is_drawn_once_graded_on_the_server_and_validates_the_course() {
    let Some(app) = app().await else { return };
    let cookie = sign_in(&app, "acme.test", "alice").await;

    let course = page(&app, &cookie, "/courses/git-basics/").await;
    assert!(course.contains(r#"href="/courses/git-basics/exam/">Passer l'examen de validation"#));
    let exam_page = page(&app, &cookie, "/courses/git-basics/exam/").await;
    assert!(exam_page.contains(r#"id="exam-root""#) && exam_page.contains(r#""startUrl":"/api/exam/git-basics/start""#));
    assert!(exam_page.contains(r#""open":false"#) && exam_page.contains(r#""retryAfter":0"#));

    let (status, attempt) = post_json(&app, Some(&cookie), "/api/exam/git-basics/start", "{}").await;
    assert_eq!(status, StatusCode::OK, "{attempt}");
    let questions = attempt["questions"].as_array().unwrap();
    assert!(!questions.is_empty() && attempt["total"] == questions.len() && attempt["seconds_left"].as_i64().unwrap() > 60);
    // Nothing tells which option is right, and no explanation is sent before grading.
    for question in questions {
        let keys: Vec<&String> = question.as_object().unwrap().keys().collect();
        assert_eq!(keys, ["id", "options", "question"]);
        assert!(question["options"].as_array().unwrap().iter().all(|option| option.as_object().unwrap().len() == 1));
    }
    // Starting again resumes the same attempt, with the same questions in the same order.
    let (_, resumed) = post_json(&app, Some(&cookie), "/api/exam/git-basics/start", "{}").await;
    assert_eq!((&resumed["attempt"], &resumed["questions"]), (&attempt["attempt"], &attempt["questions"]));
    assert!(page(&app, &cookie, "/courses/git-basics/exam/").await.contains(r#""open":true"#));

    // An attempt cannot be submitted by someone else, nor for another course.
    let answers = exam_answers(&attempt, true).to_string();
    let bob = sign_in(&app, "acme.test", "bob").await;
    assert_eq!(post_json(&app, Some(&bob), "/api/exam/git-basics/submit", &answers).await.0, StatusCode::NOT_FOUND);
    assert_eq!(post_json(&app, Some(&cookie), "/api/exam/docker-hello/submit", &answers).await.0, StatusCode::NOT_FOUND);

    let (status, result) = post_json(&app, Some(&cookie), "/api/exam/git-basics/submit", &answers).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(
        (result["passed"].as_bool(), &result["score"], result["course_validated"].as_bool()),
        (Some(true), &result["total"], Some(true))
    );
    assert_eq!((result["lessons_validated"].as_u64(), result["xp_gained"].as_u64()), (Some(7), Some(100)));
    // The correction comes with the result: options as shown, the right one, the explanation.
    let first = &result["results"][0];
    assert_eq!(first["options"].as_array().unwrap().len(), questions[0]["options"].as_array().unwrap().len());
    assert!(first["correct"] == true && first["chosen"] == first["correct_position"] && first["explanation"].is_string());
    assert!(result["new_badges"].as_array().unwrap().iter().any(|badge| badge["slug"] == "parcours-git-basics"));

    // Closed for good, and the course is validated everywhere.
    assert_eq!(post_json(&app, Some(&cookie), "/api/exam/git-basics/submit", &answers).await.0, StatusCode::CONFLICT);
    assert_eq!(post_json(&app, Some(&cookie), "/api/exam/git-basics/start", "{}").await.0, StatusCode::CONFLICT);
    let exam_page = page(&app, &cookie, "/courses/git-basics/exam/").await;
    assert!(exam_page.contains("Cours déjà validé") && exam_page.contains("par examen") && !exam_page.contains("exam-data"));
    let course = page(&app, &cookie, "/courses/git-basics/").await;
    assert!(
        course.contains("✓ Cours validé par examen") && course.contains("Validée par examen") && course.contains("7 / 7 leçons · 100 %")
    );
    assert!(page(&app, &cookie, "/dashboard/").await.contains("Niv. 2 · 100 XP"));
}

#[tokio::test]
async fn a_failed_exam_validates_nothing_and_must_wait() {
    let Some(app) = app().await else { return };
    let cookie = sign_in(&app, "acme.test", "alice").await;
    let (_, attempt) = post_json(&app, Some(&cookie), "/api/exam/git-basics/start", "{}").await;
    let (status, result) = post_json(&app, Some(&cookie), "/api/exam/git-basics/submit", &exam_answers(&attempt, false).to_string()).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!((result["passed"].as_bool(), result["score"].as_u64(), result["retry_after"].as_i64()), (Some(false), Some(0), Some(600)));
    assert!(result.get("course_validated").is_none() && result["results"][0]["correct"] == false);

    let (status, refused) = post_json(&app, Some(&cookie), "/api/exam/git-basics/start", "{}").await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert!((590..=600).contains(&refused["retry_after"].as_i64().unwrap()));
    let exam_page = page(&app, &cookie, "/courses/git-basics/exam/").await;
    assert!(exam_page.contains(r#""open":false"#) && !exam_page.contains(r#""retryAfter":0"#));
    assert!(page(&app, &cookie, "/dashboard/").await.contains("0 / 7 leçons · 0 %"));
}

#[tokio::test]
async fn exams_are_refused_to_visitors_other_sites_and_locked_courses() {
    let Some(app) = app().await else { return };
    let cookie = sign_in(&app, "acme.test", "alice").await;
    assert_eq!(post_json(&app, None, "/api/exam/git-basics/start", "{}").await.0, StatusCode::UNAUTHORIZED);
    let forged = Call {
        host: "acme.test",
        path: "/api/exam/git-basics/start",
        cookie: Some(&cookie),
        body: Some(("application/json", "{}")),
        origin: Some("https://evil.test"),
    };
    assert_eq!(call(&app, forged).await.status, StatusCode::FORBIDDEN);
    assert_eq!(post_json(&app, Some(&cookie), "/api/exam/nope/start", "{}").await.0, StatusCode::NOT_FOUND);
    // `docker-advanced` requires `docker-hello`.
    assert_eq!(post_json(&app, Some(&cookie), "/api/exam/docker-advanced/start", "{}").await.0, StatusCode::FORBIDDEN);
    let locked = page(&app, &cookie, "/courses/docker-advanced/exam/").await;
    assert!(locked.contains("Cours verrouillé") && !locked.contains("exam-data"));
    assert!(page(&app, &cookie, "/courses/docker-advanced/").await.contains("Il se débloque avec ce cours"));
    assert_eq!(post_json(&app, Some(&cookie), "/api/exam/git-basics/submit", r#"{"attempt": "not-an-id"}"#).await.0, StatusCode::NOT_FOUND);

    let visitor = call(&app, Call { host: "acme.test", path: "/courses/git-basics/exam/", cookie: None, body: None, origin: None }).await;
    assert_eq!(visitor.status, StatusCode::SEE_OTHER);
    assert!(!get(&app, "acme.test", "/courses/git-basics/").await.1.contains("exam-cta"));
}

/// Records a full quiz score on every lesson of `course`, which completes it while labs are not run.
async fn complete(app: &Router, cookie: &str, course: &str) {
    let catalogue = mentor_content::load_catalogue(&root().join("catalogue")).unwrap();
    for lesson in &catalogue.courses.iter().find(|candidate| candidate.slug == course).unwrap().lessons {
        let json = format!(r#"{{"course": "{course}", "lesson": "{}", "type": "quiz", "score": {}}}"#, lesson.slug, lesson.quiz.len());
        assert_eq!(progress(app, "acme.test", Some(cookie), &json).await.0, StatusCode::OK, "{json}");
    }
}

/// The state of a course on the map of a path, as its card declares it.
fn node_state(page: &str, course: &str) -> String {
    let marker = format!(r#"data-course="{course}" data-state=""#);
    let start = page.find(&marker).unwrap_or_else(|| panic!("{course} is not on the map")) + marker.len();
    page[start..].split('"').next().unwrap().to_string()
}

/// Number of prerequisite links drawn on the map in `state` (`done`, `todo`), in both layouts together.
/// The legend above the map shows a sample of each: it is left out.
fn links(page: &str, state: &str) -> usize {
    let map = &page[page.find(r#"<div class="path-map"#).expect("the page has a map")..];
    map.matches(&format!(r#"<path class="path-link path-link--{state}" d="M"#)).count()
}

#[tokio::test]
async fn a_visitor_sees_the_paths_and_their_map_without_progress() {
    let Some(app) = app().await else { return };
    let (status, list) = get(&app, "acme.test", "/paths/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(list.contains("<h1>Les parcours de formation</h1>") && list.contains(r#"href="/paths/" aria-current="page""#));
    for path in ["socle-commun", "developpement-frontend", "backend-python", "devops-infrastructure"] {
        assert!(list.contains(&format!(r#"data-path="{path}""#)) && list.contains(&format!(r#"href="/paths/{path}/""#)), "{path}");
    }
    assert!(list.contains("8 cours") && list.contains("3 étapes") && list.contains("7 cours (dont 1 en option)"));
    // The strip of a card is decoration; the same courses are named in text.
    assert!(list.contains("ordre : Linux et shell, Git basics, Docker hello world, SQL et PostgreSQL.</p>"));
    assert!(!list.contains("progressbar") && list.contains("Acme Academy") && list.contains("--primary: #112233"));
    // Every page links to the paths.
    assert!(get(&app, "acme.test", "/catalogue/").await.1.contains(r#"<a href="/paths/">Parcours de formation</a>"#));

    let (status, map) = get(&app, "acme.test", "/paths/devops-infrastructure/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(map.contains("<h1>DevOps et infrastructure</h1>") && map.contains("Acme Academy") && map.contains("--accent: #326CE5"));
    // Three stages, eight courses in order, each a link to its course.
    assert!(map.contains("Étape 3</span> Orchestrer et exploiter") && !map.contains("Étape 4"));
    let order: Vec<usize> =
        ["linux-shell", "git-basics", "docker-hello", "ci-gitlab", "docker-advanced", "kubernetes-helm", "terraform", "sauvegardes-s3"]
            .iter()
            .map(|course| map.find(&format!(r#"id="course-{course}""#)).unwrap_or_else(|| panic!("{course} is missing")))
            .collect();
    assert!(order.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(map.contains(r#"<a class="stretched" href="/courses/docker-advanced/">Docker advanced</a>"#));
    // Five prerequisite links, drawn once for each layout, and said in words on the course they lead to.
    assert_eq!(map.matches(r#"<path class="path-link" d="M"#).count(), 10);
    assert_eq!(map.matches(r#"<polygon class="path-arrow" points=""#).count(), 5);
    assert!(map.contains(r#"viewBox="0 0 3000 "#) && map.contains(r#"preserveAspectRatio="none""#) && map.contains("--pm-columns: 3;"));
    assert!(map.contains("Prérequis dans ce parcours de formation : Docker advanced."));
    // No state, no progress, nothing to resume: only where to start.
    assert_eq!(node_state(&map, "docker-advanced"), "plain");
    assert!(!map.contains("progressbar") && !map.contains("Verrouillé") && !map.contains("path-link--") && !map.contains("À suivre"));
    assert!(map.contains("Par où commencer") && map.contains(r#"href="/courses/linux-shell/">Voir ce cours"#));

    // A course that is not published yet is announced, without a link.
    let backend = get(&app, "acme.test", "/paths/backend-python/").await.1;
    assert_eq!(node_state(&backend, "django"), "soon");
    assert!(backend.contains("Bientôt disponible") && !backend.contains(r#"href="/courses/django/""#) && backend.contains("En option"));
    // A path written as a plain list has no stage.
    let flat = get(&app, "acme.test", "/paths/socle-commun/").await.1;
    assert!(flat.contains("path-map--flat") && flat.contains("--pm-columns: 4;") && !flat.contains("path-stage__title"));

    // A course page names the paths it belongs to.
    let course = get(&app, "acme.test", "/courses/docker-advanced/").await.1;
    assert!(
        course.contains("Fait partie du parcours de formation")
            && course.contains(r#"href="/paths/devops-infrastructure/#course-docker-advanced""#)
    );
    assert!(get(&app, "acme.test", "/courses/git-basics/").await.1.contains("Fait partie des parcours de formation"));
    assert!(!get(&app, "acme.test", "/courses/go/").await.1.contains("Fait partie d"));

    let (status, body) = get(&app, "acme.test", "/paths/nope/").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.contains("Page introuvable") && body.contains("Acme Academy"));
    assert_eq!(get(&app, "elsewhere.test", "/paths/").await.0, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_map_of_a_path_follows_what_the_learner_did() {
    let Some(app) = app().await else { return };
    let cookie = sign_in(&app, "acme.test", "alice").await;
    let path = "/paths/devops-infrastructure/";

    let map = page(&app, &cookie, path).await;
    assert!(map.contains("0 / 8 cours · 0 %") && map.contains(r#"aria-label="Progression dans le parcours de formation""#));
    assert_eq!((node_state(&map, "linux-shell"), node_state(&map, "git-basics")), ("available".into(), "available".into()));
    // A course whose prerequisite is not completed is locked, and says what to complete.
    assert_eq!((node_state(&map, "docker-advanced"), node_state(&map, "terraform")), ("locked".into(), "locked".into()));
    assert!(map.contains("🔒 Verrouillé") && map.contains("abord : Docker hello world</p>"));
    assert_eq!((links(&map, "done"), links(&map, "todo")), (0, 10));
    // The first course of the path is the next thing to do, and is marked on the map.
    assert!(
        map.contains("Prochaine étape du parcours de formation")
            && map.contains(r#"href="/courses/linux-shell/naviguer-fichiers/">Commencer"#),
        "next step"
    );
    assert_eq!(map.matches("À suivre").count(), 1);
    let marked = map.find("is-next").unwrap();
    assert!(map[..marked].ends_with(r#"data-course="linux-shell" data-state="available" class="path-node path-node--available "#));

    // One lesson of seven in one course of eight.
    assert_eq!(progress(&app, "acme.test", Some(&cookie), QUIZ).await.0, StatusCode::OK);
    let map = page(&app, &cookie, path).await;
    assert_eq!(node_state(&map, "git-basics"), "started");
    assert!(map.contains("En cours · 14 %") && map.contains("0 / 8 cours · 2 %"));
    assert_eq!(node_state(&map, "ci-gitlab"), "locked");

    // A completed course opens what it leads to; the link between them is drawn as done.
    complete(&app, &cookie, "docker-hello").await;
    let map = page(&app, &cookie, path).await;
    assert_eq!((node_state(&map, "docker-hello"), node_state(&map, "docker-advanced")), ("done".into(), "available".into()));
    assert_eq!(node_state(&map, "kubernetes-helm"), "locked");
    assert!(map.contains("✓ Terminé") && map.contains("1 / 8 cours · 14 %"));
    assert_eq!((links(&map, "done"), links(&map, "todo")), (2, 8));

    // The dashboard and the list show the same advancement; paths the learner is in come first.
    let dashboard = page(&app, &cookie, "/dashboard/").await;
    assert!(dashboard.contains("Voir tous les parcours de formation") && dashboard.contains("1 / 8 cours · 14 %"));
    assert!(dashboard.contains(r#"aria-label="Progression dans le parcours de formation DevOps et infrastructure""#));
    assert!(dashboard.find(r#"data-path="socle-commun""#).unwrap() < dashboard.find(r#"data-path="devops-infrastructure""#).unwrap());
    let list = page(&app, &cookie, "/paths/").await;
    assert!(list.contains("path-strip__dot--done") && list.contains("path-strip__dot--locked") && list.contains("1 / 4 cours"));

    // A whole path: its three other courses have no prerequisite.
    for course in ["linux-shell", "git-basics", "sql-postgresql"] {
        complete(&app, &cookie, course).await;
    }
    let done = page(&app, &cookie, "/paths/socle-commun/").await;
    assert!(done.contains("4 / 4 cours · 100 % · ✓ Parcours de formation terminé") && done.contains("Bravo, tu es allé·e au bout"));
    assert!(!done.contains("À suivre"));
    assert!(page(&app, &cookie, "/paths/").await.contains(r#"class="path-card is-done""#));
    // Completing a path stores nothing: no badge is named after it.
    assert!(!page(&app, &cookie, "/badges/").await.contains("Socle commun"));
}

#[tokio::test]
async fn a_path_shows_only_the_progress_made_on_its_own_tenant() {
    let Some(app) = app().await else { return };
    let acme = sign_in(&app, "acme.test", "alice").await;
    let plain = sign_in(&app, "plain.test", "alice").await;
    complete(&app, &acme, "docker-hello").await;
    let path = "/paths/devops-infrastructure/";
    assert_eq!(node_state(&page(&app, &acme, path).await, "docker-hello"), "done");

    // The same user name on the other tenant has done nothing, and sees that tenant's brand.
    let elsewhere = call(&app, Call { host: "plain.test", path, cookie: Some(&plain), body: None, origin: None }).await;
    assert_eq!(elsewhere.status, StatusCode::OK);
    assert_eq!(
        (node_state(&elsewhere.body, "docker-hello"), node_state(&elsewhere.body, "docker-advanced")),
        ("available".into(), "locked".into())
    );
    assert!(elsewhere.body.contains("0 / 8 cours · 0 %"));
    assert_eq!((links(&elsewhere.body, "done"), links(&elsewhere.body, "todo")), (0, 10));
    assert!(elsewhere.body.contains("<strong>Plain</strong>") && !elsewhere.body.contains("Acme"));
    let list = call(&app, Call { host: "plain.test", path: "/paths/", cookie: Some(&plain), body: None, origin: None }).await;
    assert!(list.body.contains("0 / 8 cours · 0 %") && !list.body.contains("path-strip__dot--done"));

    // A session of one tenant is nobody on the other: the path is shown as to a visitor.
    let crossed = call(&app, Call { host: "plain.test", path, cookie: Some(&acme), body: None, origin: None }).await;
    assert_eq!(crossed.status, StatusCode::OK);
    assert_eq!(node_state(&crossed.body, "docker-hello"), "plain");
    assert!(!crossed.body.contains("progressbar") && !crossed.body.contains("Déconnexion"));
}

#[tokio::test]
async fn a_catalogue_without_paths_shows_none() {
    let Some(app) = app_built(true, false).await else { return };
    let cookie = sign_in(&app, "acme.test", "alice").await;
    for path in ["/", "/catalogue/", "/courses/git-basics/"] {
        let body = get(&app, "acme.test", path).await.1;
        assert!(!body.contains("/paths/") && !body.contains("Parcours de formation") && !body.contains("parcours de formation"), "{path}");
    }
    assert!(!page(&app, &cookie, "/dashboard/").await.contains("ursus"));
    let (status, list) = get(&app, "acme.test", "/paths/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(list.contains("Aucun parcours de formation pour le moment"));
    assert_eq!(get(&app, "acme.test", "/paths/devops-infrastructure/").await.0, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn only_the_pictures_of_the_catalogue_can_be_downloaded() {
    let Some(app) = app().await else { return };
    let picture = call(
        &app,
        Call { host: "acme.test", path: "/static/catalogue/git-basics/images/banniere.svg", cookie: None, body: None, origin: None },
    )
    .await;
    assert_eq!(picture.status, StatusCode::OK);
    assert!(picture.body.contains("<svg"));
    // Lesson sources carry the solutions of their labs, exam pools their answers, environments their tools.
    for path in [
        "/static/catalogue/git-basics/exam.md",
        "/static/catalogue/git-basics/01-introduction.md",
        "/static/catalogue/git-basics/course.md",
        "/static/catalogue/git-basics/environnement/Dockerfile",
        "/static/catalogue/mentor.yml",
        "/static/catalogue/paths.yml",
        "/static/catalogue/_checks.yml",
        "/static/catalogue/git-basics/images/../exam.md",
        "/static/catalogue/git-basics/images/%2e%2e/exam.md",
        "/static/catalogue/git-basics/images/..%2fexam.md",
        "/static/catalogue/nope/images/banniere.svg",
        "/static/catalogue/git-basics/images/",
    ] {
        assert_eq!(get(&app, "acme.test", path).await.0, StatusCode::NOT_FOUND, "{path}");
    }
    // The style sheets and scripts of the site are still served.
    assert_eq!(get(&app, "acme.test", "/static/css/portail.css").await.0, StatusCode::OK);
}
