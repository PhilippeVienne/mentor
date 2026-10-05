//! Splits a document into Markdown and directives (`:::info`, `:::cartes`, `:::quiz`, `:::labo`).

use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;

use crate::error::Result;
use crate::lab::{parse_lab, Lab, LabContext};
use crate::markdown::{escape, md_inline, render_md, Render, FENCE_RE};

static DIRECTIVE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^:::(\w+)[ \t]*(.*?)[ \t]*$").unwrap());
static OPTION_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[-*] \[( |x|X)\] (.+)$").unwrap());
static CARD_TITLE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^### +(.+)$").unwrap());

/// Default title of a callout, `None` when the name is not a callout.
fn callout_title(kind: &str) -> Option<&'static str> {
    match kind {
        "info" => Some("À savoir"),
        "tip" => Some("Astuce"),
        "warning" => Some("Attention"),
        "danger" => Some("Danger"),
        _ => None,
    }
}

#[derive(Debug, PartialEq)]
pub enum Segment {
    Markdown(String),
    Directive { name: String, title: String, body: String, line: usize },
}

/// Single-choice question of a lesson quiz or an exam pool.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Quiz {
    pub question: String,
    pub options: Vec<QuizOption>,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuizOption {
    pub html: String,
    pub correct: bool,
}

/// What a document produced besides its HTML.
#[derive(Debug, Default)]
pub struct Collected {
    pub quizzes: Vec<Quiz>,
    pub labs: Vec<Lab>,
}

/// Splits into Markdown and directive segments, without looking inside code blocks.
pub fn split_segments(text: &str, ctx: &Render) -> Result<Vec<Segment>> {
    let mut segments = Vec::new();
    let mut buffer: Vec<&str> = Vec::new();
    let mut fence: Option<String> = None;
    let mut directive: Option<(String, String, Vec<&str>, usize)> = None;
    for (index, line) in text.split('\n').enumerate() {
        let fence_match = FENCE_RE.captures(line);
        if let Some(caps) = &fence_match {
            let marker = &caps[1];
            match &fence {
                None => fence = Some(marker.to_string()),
                // CommonMark closing fence: same character, at least as long as the opening one.
                Some(open) if caps[2].is_empty() && marker.as_bytes()[0] == open.as_bytes()[0] && marker.len() >= open.len() => {
                    fence = None
                }
                Some(_) => {}
            }
        }
        if fence.is_none() && fence_match.is_none() {
            if directive.is_none() {
                if let Some(caps) = DIRECTIVE_RE.captures(line) {
                    if !buffer.is_empty() {
                        segments.push(Segment::Markdown(buffer.join("\n")));
                        buffer.clear();
                    }
                    directive = Some((caps[1].to_string(), caps[2].to_string(), Vec::new(), index + 1));
                    continue;
                }
            }
            if line.trim() == ":::" {
                if let Some((name, title, body, line)) = directive.take() {
                    segments.push(Segment::Directive { name, title, body: body.join("\n"), line });
                    continue;
                }
            }
        }
        match &mut directive {
            Some((_, _, body, _)) => body.push(line),
            None => buffer.push(line),
        }
    }
    if let Some((name, _, _, line)) = directive {
        return ctx.fail(format!("line {line}: the `:::{name}` block is never closed by `:::`"));
    }
    if fence.is_some() {
        return ctx.fail("a ``` code block is never closed");
    }
    if !buffer.is_empty() {
        segments.push(Segment::Markdown(buffer.join("\n")));
    }
    Ok(segments)
}

fn render_callout(kind: &str, title: &str, body: &str, ctx: &mut Render) -> Result<String> {
    let title = if title.is_empty() { callout_title(kind).unwrap_or_default() } else { title };
    Ok(format!(
        r#"<aside class="callout callout--{kind}"><p class="callout__title">{}</p>{}</aside>"#,
        escape(title),
        render_md(body, ctx)?
    ))
}

fn render_cards(body: &str, ctx: &mut Render) -> Result<String> {
    let titles: Vec<_> = CARD_TITLE_RE.captures_iter(body).collect();
    if titles.is_empty() {
        return ctx.fail("a `:::cards` block must contain `### Title` sub-headings");
    }
    let mut cards = String::new();
    for (i, caps) in titles.iter().enumerate() {
        let start = caps.get(0).unwrap().end();
        let end = titles.get(i + 1).map_or(body.len(), |next| next.get(0).unwrap().start());
        cards.push_str(&format!(r#"<div class="card"><h3>{}</h3>{}</div>"#, escape(caps[1].trim()), render_md(&body[start..end], ctx)?));
    }
    Ok(format!(r#"<div class="cards">{cards}</div>"#))
}

fn parse_quiz(body: &str, start_line: usize, ctx: &Render) -> Result<Quiz> {
    let mut question: Vec<&str> = Vec::new();
    let mut options: Vec<(bool, String)> = Vec::new();
    let mut explanation: Vec<&str> = Vec::new();
    let mut in_explanation = false;
    for line in body.split('\n') {
        if let Some(caps) = OPTION_RE.captures(line.trim()) {
            options.push((caps[1].eq_ignore_ascii_case("x"), caps[2].to_string()));
            in_explanation = false;
        } else if line.starts_with('>') {
            explanation.push(line.trim_start_matches('>').trim());
            in_explanation = true;
        } else if options.is_empty() {
            question.push(line);
        } else if in_explanation && !line.trim().is_empty() {
            explanation.push(line.trim());
        }
    }
    let place = format!("line {start_line}: `:::quiz`");
    if question.concat().trim().is_empty() {
        return ctx.fail(format!("{place} has no question (write the question before the list of answers)"));
    }
    if options.len() < 2 {
        return ctx.fail(format!("{place}: at least 2 answers are required (`- [ ] …` / `- [x] …`)"));
    }
    if options.iter().filter(|(correct, _)| *correct).count() != 1 {
        return ctx.fail(format!("{place}: exactly one answer must be ticked `[x]`"));
    }
    Ok(Quiz {
        question: md_inline(&question.join("\n")),
        options: options.into_iter().map(|(correct, text)| QuizOption { html: md_inline(&text), correct }).collect(),
        explanation: if explanation.is_empty() { String::new() } else { md_inline(&explanation.join("\n")) },
    })
}

/// Renders a Markdown body; labs and quizzes are collected separately. `labs` is `None` for an exam.
pub fn render_document(body: &str, ctx: &mut Render, mut labs: Option<&mut LabContext>) -> Result<(String, Collected)> {
    let mut parts = Vec::new();
    let mut collected = Collected::default();
    for segment in split_segments(body, ctx)? {
        match segment {
            Segment::Markdown(text) => parts.push(render_md(&text, ctx)?),
            Segment::Directive { name, title, body, line } => match name.as_str() {
                kind if callout_title(kind).is_some() => parts.push(render_callout(kind, &title, &body, ctx)?),
                "cards" => parts.push(render_cards(&body, ctx)?),
                "quiz" => collected.quizzes.push(parse_quiz(&body, line, ctx)?),
                "lab" => match labs.as_deref_mut() {
                    Some(lab_ctx) => collected.labs.push(parse_lab(&body, line, lab_ctx, ctx)?),
                    None => return ctx.fail(format!("line {line}: an exam cannot contain a `:::lab` block")),
                },
                other => {
                    return ctx.fail(format!("line {line}: unknown directive `:::{other}` (info, tip, warning, danger, cards, lab, quiz)"))
                }
            },
        }
    }
    Ok((parts.join("\n"), collected))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    fn ctx() -> Render<'static> {
        Render::new(Path::new("lecon.md"), "demo")
    }

    #[test]
    fn directive_inside_a_code_block_is_ignored() {
        let segments = split_segments("avant\n```\n:::info\n```\n:::tip Titre\ncorps\n:::\naprès", &ctx()).unwrap();
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[1], Segment::Directive { name: "tip".into(), title: "Titre".into(), body: "corps".into(), line: 5 });
    }

    #[test]
    fn longer_fence_can_contain_a_shorter_one() {
        let segments = split_segments("````md\n```\n:::quiz\n```\n````\nfin", &ctx()).unwrap();
        assert_eq!(segments.len(), 1);
    }

    #[test]
    fn unclosed_directive_is_an_error() {
        let err = split_segments("texte\n:::info\ncorps", &ctx()).unwrap_err();
        assert_eq!(err.to_string(), "lecon.md: line 2: the `:::info` block is never closed by `:::`");
    }

    #[test]
    fn valid_quiz() {
        let quiz = parse_quiz("Quelle commande ?\n\n- [ ] `git add`\n- [x] `git init`\n> Parce que.\nsuite", 3, &ctx()).unwrap();
        assert_eq!(quiz.question, "Quelle commande ?");
        assert_eq!(quiz.options[1], QuizOption { html: "<code>git init</code>".into(), correct: true });
        assert_eq!(quiz.explanation, "Parce que.\nsuite");
    }

    #[test]
    fn quiz_with_two_correct_answers_is_rejected() {
        let err = parse_quiz("Q ?\n- [x] a\n- [x] b", 7, &ctx()).unwrap_err();
        assert!(err.message.starts_with("line 7: `:::quiz`: exactly one answer"));
    }

    #[test]
    fn callout_and_cards() {
        let (html, _) = render_document(":::warning\nDoucement.\n:::\n:::cards\n### Un\nA\n### Deux\nB\n:::", &mut ctx(), None).unwrap();
        assert!(
            html.contains(r#"<aside class="callout callout--warning"><p class="callout__title">Attention</p><p>Doucement.</p></aside>"#)
        );
        assert!(html.contains(r#"<div class="card"><h3>Deux</h3><p>B</p></div>"#));
    }

    #[test]
    fn unknown_directive_is_an_error() {
        let err = render_document(":::truc\nx\n:::", &mut ctx(), None).unwrap_err();
        assert!(err.message.contains("unknown directive `:::truc`"));
    }
}
