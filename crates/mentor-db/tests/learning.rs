//! Cohorts, badges and exams, against a real PostgreSQL (see `common`).

mod common;

use std::collections::BTreeMap;

use common::{database, TestDb};
use mentor_core::badges::default_badges;
use mentor_core::exam::{PoolQuestion, Randomness};
use mentor_core::gamification::{XP_EXAM, XP_LESSON, XP_TASK};
use mentor_core::progress::{CourseShape, Event, LessonRules, Source};
use mentor_db::badges::CatalogueView;
use mentor_db::exam::{Attempt, ExamSettings, ExamStatus, Started, Submitted};
use mentor_db::{platform, TenantTx};
use uuid::Uuid;

/// Deterministic randomness: always the first choice.
struct First;

impl Randomness for First {
    fn below(&mut self, _upper: usize) -> usize {
        0
    }
}

const SETTINGS: ExamSettings = ExamSettings { draw: 2, shuffle: true, minutes: 20, pass_mark: 50, cooldown_seconds: 600 };
const NOW: i64 = 1_800_000_000;

fn lesson(slug: &str, tasks: u32) -> LessonRules {
    LessonRules { course: "git".into(), slug: slug.into(), tasks, questions: 0, server_verified: false, lab_available: true }
}

fn pool() -> Vec<PoolQuestion> {
    (0..3).map(|i| PoolQuestion { id: format!("q{i}"), options: 3, correct: 0 }).collect()
}

async fn tenant_with_learner(db: &TestDb, slug: &str) -> (Uuid, Uuid) {
    let tenant = platform::create_tenant(&db.owner, slug, slug, &[]).await.unwrap();
    let mut tx = TenantTx::begin(&db.app, tenant).await.unwrap();
    let learner = tx.upsert_learner("sub-1", "alice", false).await.unwrap().id;
    tx.commit().await.unwrap();
    (tenant, learner)
}

/// Answers every drawn question with its correct option (or a wrong one), wherever it is shown.
fn answers(attempt: &Attempt, correct: bool) -> BTreeMap<String, usize> {
    attempt
        .questions
        .iter()
        .map(|question| {
            let right = question.option_order.iter().position(|&original| original == 0).unwrap();
            (question.id.clone(), if correct { right } else { (right + 1) % 3 })
        })
        .collect()
}

async fn start(tx: &mut TenantTx, learner: Uuid, now: i64) -> Started {
    tx.start_exam(learner, "git", &pool(), SETTINGS, now, &mut First).await.unwrap()
}

fn attempt_of(started: Started) -> Attempt {
    match started {
        Started::Attempt(attempt) => attempt,
        other => panic!("expected an attempt, got {other:?}"),
    }
}

#[tokio::test]
async fn idp_cohorts_follow_the_token_and_leave_manual_ones_alone() {
    let Some(db) = database().await else { return };
    let (tenant, alice) = tenant_with_learner(&db, "acme").await;
    let (other_tenant, bob) = tenant_with_learner(&db, "globex").await;

    let mut tx = TenantTx::begin(&db.app, tenant).await.unwrap();
    tx.add_to_cohort(alice, "tutors", "Tutors").await.unwrap();
    assert_eq!(tx.sync_idp_cohorts(alice, &["/promo-2027/info", "/asso-demo"]).await.unwrap(), ["asso-demo", "promo-2027-info"]);
    assert_eq!(tx.cohorts_of(alice).await.unwrap(), ["asso-demo", "promo-2027-info", "tutors"]);
    // The learner left one group: that membership goes, the manual one stays.
    tx.sync_idp_cohorts(alice, &["/asso-demo"]).await.unwrap();
    assert_eq!(tx.cohorts_of(alice).await.unwrap(), ["asso-demo", "tutors"]);
    tx.commit().await.unwrap();

    let mut tx = TenantTx::begin(&db.app, other_tenant).await.unwrap();
    assert!(tx.cohorts_of(bob).await.unwrap().is_empty());
    // The same group name in another tenant is another cohort.
    tx.sync_idp_cohorts(bob, &["/asso-demo"]).await.unwrap();
    assert_eq!(tx.cohorts_of(bob).await.unwrap(), ["asso-demo"]);
}

#[tokio::test]
async fn badges_are_granted_once_from_stored_progress() {
    let Some(db) = database().await else { return };
    let (tenant, alice) = tenant_with_learner(&db, "acme").await;
    let catalogue = CatalogueView {
        courses: vec![CourseShape { slug: "git".into(), published: true, requires: vec![], lessons: vec!["intro".into()] }],
        questions: BTreeMap::new(),
    };
    let badges = default_badges();

    let mut tx = TenantTx::begin(&db.app, tenant).await.unwrap();
    assert!(tx.award_badges(alice, &badges, &catalogue, 120).await.unwrap().is_empty());
    let done = tx.record_event(alice, &lesson("intro", 1), Event::Task(0), Source::Browser).await.unwrap();
    assert!(done.lesson_completed);
    let stats = tx.learner_stats(alice, &catalogue, 120).await.unwrap();
    assert_eq!((stats.tasks, stats.streak, stats.level), (1, 1, 1));
    assert!(stats.courses_done.contains("git"));
    // First lab step, and the course badge since its only lesson is complete.
    assert_eq!(tx.award_badges(alice, &badges, &catalogue, 120).await.unwrap(), ["premier-pas", "parcours-git"]);
    assert!(tx.award_badges(alice, &badges, &catalogue, 120).await.unwrap().is_empty());
    assert_eq!(tx.badges_of(alice).await.unwrap().len(), 2);
    let dates = tx.badge_dates(alice).await.unwrap();
    assert!(dates.len() == 2 && dates["premier-pas"] > 1_700_000_000);
    let states = tx.lesson_states(alice).await.unwrap();
    assert!(states["git/intro"].completed && states["git/intro"].started() && states["git/intro"].tasks_done == 1);
}

#[tokio::test]
async fn starting_twice_returns_the_same_attempt() {
    let Some(db) = database().await else { return };
    let (tenant, alice) = tenant_with_learner(&db, "acme").await;
    let mut tx = TenantTx::begin(&db.app, tenant).await.unwrap();
    let first = attempt_of(start(&mut tx, alice, NOW).await);
    assert_eq!((first.questions.len(), first.deadline), (2, NOW + 20 * 60));
    let again = attempt_of(start(&mut tx, alice, NOW + 30).await);
    assert_eq!(again, first);
}

#[tokio::test]
async fn a_failed_exam_validates_nothing_and_imposes_a_delay() {
    let Some(db) = database().await else { return };
    let (tenant, alice) = tenant_with_learner(&db, "acme").await;
    let lessons = [lesson("intro", 2)];
    let mut tx = TenantTx::begin(&db.app, tenant).await.unwrap();
    assert_eq!(tx.exam_status(alice, "git", 600, NOW).await.unwrap(), ExamStatus { open: false, retry_after: 0 });
    let attempt = attempt_of(start(&mut tx, alice, NOW).await);
    assert_eq!(tx.exam_status(alice, "git", 600, NOW + 30).await.unwrap(), ExamStatus { open: true, retry_after: 0 });
    let outcome = tx.submit_exam(alice, "git", attempt.id, &answers(&attempt, false), &pool(), SETTINGS, &lessons, NOW + 60).await.unwrap();
    match outcome {
        Submitted::Graded { grade, xp_gained, lessons_validated, .. } => {
            assert_eq!((grade.score, grade.passed, xp_gained, lessons_validated), (0, false, 0, 0));
        }
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(start(&mut tx, alice, NOW + 160).await, Started::RetryAfter(500));
    assert_eq!(tx.exam_status(alice, "git", 600, NOW + 160).await.unwrap(), ExamStatus { open: false, retry_after: 500 });
    // Another course is another exam.
    assert_eq!(tx.exam_status(alice, "docker", 600, NOW + 160).await.unwrap(), ExamStatus { open: false, retry_after: 0 });
    assert!(matches!(start(&mut tx, alice, NOW + 60 + 600).await, Started::Attempt(_)));
}

#[tokio::test]
async fn a_passed_exam_validates_the_course_once() {
    let Some(db) = database().await else { return };
    let (tenant, alice) = tenant_with_learner(&db, "acme").await;
    let lessons = [lesson("intro", 1), lesson("branches", 2)];
    let mut tx = TenantTx::begin(&db.app, tenant).await.unwrap();
    // One lesson was already completed by practising: the exam must not overwrite it.
    tx.record_event(alice, &lessons[0], Event::Task(0), Source::Browser).await.unwrap();
    let attempt = attempt_of(start(&mut tx, alice, NOW).await);
    let outcome = tx.submit_exam(alice, "git", attempt.id, &answers(&attempt, true), &pool(), SETTINGS, &lessons, NOW + 60).await.unwrap();
    match outcome {
        Submitted::Graded { grade, xp_gained, lessons_validated, .. } => {
            assert!(grade.passed);
            assert_eq!((grade.score, xp_gained, lessons_validated), (2, XP_EXAM, 1));
        }
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(tx.total_xp(alice).await.unwrap(), XP_TASK + XP_LESSON + XP_EXAM);
    let practised = tx.lesson_progress(alice, &lessons[0]).await.unwrap();
    let by_exam = tx.lesson_progress(alice, &lessons[1]).await.unwrap();
    assert!(practised.completed && !practised.validated_by_exam);
    assert!(by_exam.completed && by_exam.validated_by_exam && by_exam.tasks_done.len() == 2);
    // Submitting again changes nothing.
    let again = tx.submit_exam(alice, "git", attempt.id, &answers(&attempt, true), &pool(), SETTINGS, &lessons, NOW + 90).await.unwrap();
    assert_eq!(again, Submitted::AlreadyFinished);
    // Lessons validated by the exam do not count as practice.
    assert_eq!(tx.learner_stats(alice, &CatalogueView::default(), 0).await.unwrap().tasks, 1);
}

#[tokio::test]
async fn a_late_submission_expires_the_attempt() {
    let Some(db) = database().await else { return };
    let (tenant, alice) = tenant_with_learner(&db, "acme").await;
    let mut tx = TenantTx::begin(&db.app, tenant).await.unwrap();
    let attempt = attempt_of(start(&mut tx, alice, NOW).await);
    let late = attempt.deadline + 31;
    let outcome =
        tx.submit_exam(alice, "git", attempt.id, &answers(&attempt, true), &pool(), SETTINGS, &[lesson("intro", 1)], late).await.unwrap();
    assert_eq!(outcome, Submitted::Expired { retry_after: 600 });
    assert!(matches!(start(&mut tx, alice, late + 1).await, Started::RetryAfter(599)));
    assert_eq!(tx.total_xp(alice).await.unwrap(), 0);
}

#[tokio::test]
async fn an_attempt_left_open_past_its_deadline_is_closed_on_the_next_start() {
    let Some(db) = database().await else { return };
    let (tenant, alice) = tenant_with_learner(&db, "acme").await;
    let mut tx = TenantTx::begin(&db.app, tenant).await.unwrap();
    let attempt = attempt_of(start(&mut tx, alice, NOW).await);
    assert_eq!(start(&mut tx, alice, attempt.deadline + 100).await, Started::RetryAfter(600));
}

#[tokio::test]
async fn an_attempt_belongs_to_its_learner_and_tenant() {
    let Some(db) = database().await else { return };
    let (tenant, alice) = tenant_with_learner(&db, "acme").await;
    let (other_tenant, bob) = tenant_with_learner(&db, "globex").await;
    let mut tx = TenantTx::begin(&db.app, tenant).await.unwrap();
    let attempt = attempt_of(start(&mut tx, alice, NOW).await);
    tx.commit().await.unwrap();

    let mut tx = TenantTx::begin(&db.app, other_tenant).await.unwrap();
    let outcome = tx.submit_exam(bob, "git", attempt.id, &answers(&attempt, true), &pool(), SETTINGS, &[], NOW + 10).await.unwrap();
    assert_eq!(outcome, Submitted::Unknown);
}
