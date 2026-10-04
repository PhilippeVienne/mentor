//! Markdown → HTML, with the catalogue's code blocks (commands to run, files to create, diagrams).

use std::path::Path;
use std::sync::LazyLock;

use pulldown_cmark::{html, Options, Parser};
use regex::{Captures, Regex};

use crate::error::{ContentError, Result};

pub static FENCE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(```+|~~~+)[ \t]*(.*?)[ \t]*$").unwrap());
static PLACEHOLDER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<p>@@BLOCK(\d+)@@</p>").unwrap());
static FIGURE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<p><img src="(?P<src>[^"]+)" alt="(?P<alt>[^"]*)"\s*/?></p>"#).unwrap());
static IMAGE_LINE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^!\[(?P<alt>[^\]]*)\]\((?P<src>[^)\s]+)\)$").unwrap());
static ABSOLUTE_URL_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(https?:)?//|^/").unwrap());

/// Rendering context of a file: where it is, and already rendered blocks waiting to be put back.
pub struct Render<'a> {
    pub path: &'a Path,
    pub course: &'a str,
    pub blocks: Vec<String>,
}

impl<'a> Render<'a> {
    pub fn new(path: &'a Path, course: &'a str) -> Self {
        Self { path, course, blocks: Vec::new() }
    }

    pub fn fail<T>(&self, message: impl Into<String>) -> Result<T> {
        Err(ContentError::new(self.path, message))
    }

    /// Public URL of a course image.
    fn asset_url(&self, relative: &str) -> String {
        format!("/static/catalogue/{}/{}", self.course, relative)
    }
}

/// Escapes `& < > " '` like Python's `html.escape`.
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            other => out.push(other),
        }
    }
    out
}

fn unescape(text: &str) -> String {
    text.replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

/// Plain Markdown (tables included), without the catalogue extensions.
pub fn md(text: &str) -> String {
    let mut out = String::new();
    html::push_html(&mut out, Parser::new_ext(text.trim(), Options::ENABLE_TABLES));
    out.trim_end().to_string()
}

/// One-line Markdown (question, step, objective): the wrapping paragraph is removed.
pub fn md_inline(text: &str) -> String {
    let out = md(text);
    match out.strip_prefix("<p>").and_then(|rest| rest.strip_suffix("</p>")) {
        Some(inner) => inner.to_string(),
        None => out,
    }
}

/// `shell run` or `dockerfile file=Dockerfile` → (language, options).
fn parse_info(info: &str) -> (String, Vec<(String, Option<String>)>) {
    let mut parts = info.split_whitespace();
    let lang = parts.next().unwrap_or("text").to_string();
    let options = parts
        .map(|part| match part.split_once('=') {
            Some((key, value)) if !value.is_empty() => (key.to_string(), Some(value.to_string())),
            Some((key, _)) => (key.to_string(), None),
            None => (part.to_string(), None),
        })
        .collect();
    (lang, options)
}

fn render_code(info: &str, code: &str, ctx: &Render) -> Result<String> {
    let (lang, options) = parse_info(info);
    let code = code.trim_end_matches('\n');
    let option = |name: &str| options.iter().find(|(key, _)| key == name).map(|(_, value)| value);
    if lang == "mermaid" {
        return Ok(format!(r#"<figure class="diagram"><pre class="mermaid">{}</pre></figure>"#, escape(code)));
    }
    if option("run").is_some() {
        let mut rows = String::new();
        for line in code.split('\n').filter(|line| !line.trim().is_empty()) {
            if line.trim_start().starts_with('#') {
                rows.push_str(&format!(r#"<div class="cmd__comment">{}</div>"#, escape(line)));
            } else {
                let escaped = escape(line);
                rows.push_str(&format!(
                    r#"<div class="cmd__line"><code><span class="cmd__prompt" aria-hidden="true">$</span> {escaped}</code><button type="button" class="cmd__run" data-cmd="{escaped}" title="Lancer dans le terminal du labo">▶ Lancer</button></div>"#
                ));
            }
        }
        return Ok(format!(r#"<div class="cmd">{rows}</div>"#));
    }
    if let Some(name) = option("file") {
        let Some(name) = name else {
            return ctx.fail("l'option `file=` d'un bloc de code doit préciser un nom (ex. `file=Dockerfile`)");
        };
        return Ok(format!(
            r#"<div class="filebox" data-file="{name}" data-content="{content}"><div class="filebox__head"><span class="filebox__name">{name}</span><button type="button" class="filebox__create">Créer ce fichier dans le labo</button></div><pre class="code"><code>{code}</code></pre></div>"#,
            name = escape(name),
            content = escape(&format!("{code}\n")),
            code = escape(code),
        ));
    }
    // No syntax highlighting here (v1 used Pygments): the code text is identical.
    Ok(format!(r#"<pre class="code" data-lang="{}"><code>{}</code></pre>"#, escape(&lang), escape(code)))
}

/// Replaces each code block with a marker so that Markdown does not reinterpret it.
fn extract_fences(text: &str, ctx: &mut Render) -> Result<String> {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let Some(caps) = FENCE_RE.captures(lines[i]) else {
            out.push(lines[i].to_string());
            i += 1;
            continue;
        };
        let (marker, info) = (caps[1].to_string(), caps[2].to_string());
        let mut body: Vec<&str> = Vec::new();
        i += 1;
        while i < lines.len() && lines[i].trim() != marker {
            body.push(lines[i]);
            i += 1;
        }
        i += 1;
        let block = render_code(&info, &body.join("\n"), ctx)?;
        ctx.blocks.push(block);
        out.push(format!("\n\n@@BLOCK{}@@\n\n", ctx.blocks.len() - 1));
    }
    Ok(out.join("\n"))
}

/// Builds the figure of an image, captioned with its alt text.
fn figure(alt: &str, src: &str, ctx: &Render) -> String {
    let src = if ABSOLUTE_URL_RE.is_match(src) { src.to_string() } else { ctx.asset_url(src) };
    let caption = if alt.is_empty() { String::new() } else { format!("<figcaption>{}</figcaption>", escape(alt)) };
    format!(r#"<figure class="figure"><img src="{src}" alt="{}" loading="lazy">{caption}</figure>"#, escape(alt))
}

/// An image alone on its line becomes a figure whose caption is the alt text: emphasis markers such
/// as `__name__` are not interpreted inside a caption.
fn extract_figures(text: &str, ctx: &mut Render) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut out = Vec::with_capacity(lines.len());
    for (i, line) in lines.iter().enumerate() {
        let alone = (i == 0 || lines[i - 1].trim().is_empty()) && lines.get(i + 1).is_none_or(|next| next.trim().is_empty());
        match IMAGE_LINE_RE.captures(line.trim()).filter(|_| alone && !line.starts_with("    ")) {
            Some(caps) => {
                // Code spans lose their backticks in a caption, like in v1; other markers stay verbatim.
                let block = figure(&caps["alt"].replace('`', ""), &caps["src"], ctx);
                ctx.blocks.push(block);
                out.push(format!("\n\n@@BLOCK{}@@\n\n", ctx.blocks.len() - 1));
            }
            None => out.push(line.to_string()),
        }
    }
    out.join("\n")
}

/// Fallback for images the line pass did not catch (for instance with a title attribute).
fn figures(rendered: &str, ctx: &Render) -> String {
    FIGURE_RE.replace_all(rendered, |caps: &Captures| figure(&unescape(&caps["alt"]), &caps["src"], ctx)).into_owned()
}

/// Markdown → HTML, with code blocks and figures.
pub fn render_md(text: &str, ctx: &mut Render) -> Result<String> {
    let text = extract_fences(text, ctx)?;
    let text = extract_figures(&text, ctx);
    let out = figures(&md(&text), ctx);
    Ok(PLACEHOLDER_RE.replace_all(&out, |caps: &Captures| ctx.blocks[caps[1].parse::<usize>().unwrap()].clone()).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(text: &str) -> String {
        render_md(text, &mut Render::new(Path::new("lecon.md"), "git-basics")).unwrap()
    }

    #[test]
    fn inline_strips_the_paragraph() {
        assert_eq!(md_inline("Lance `git init`"), "Lance <code>git init</code>");
    }

    #[test]
    fn run_block_yields_runnable_commands() {
        let out = render("```shell run\n# commentaire\ngit status\n\n```");
        assert!(out.contains(r#"<div class="cmd__comment"># commentaire</div>"#));
        assert!(out.contains(r#"data-cmd="git status""#) && !out.contains("<p>@@BLOCK"));
    }

    #[test]
    fn file_block_escapes_its_content() {
        let out = render("```html file=index.html\n<h1>Salut</h1>\n```");
        assert!(out.contains(r#"data-file="index.html""#));
        assert!(out.contains("data-content=\"&lt;h1&gt;Salut&lt;/h1&gt;\n\""));
    }

    #[test]
    fn file_without_a_name_is_an_error() {
        let err = render_md("```html file=\nx\n```", &mut Render::new(Path::new("l.md"), "c")).unwrap_err();
        assert!(err.message.contains("doit préciser un nom"));
    }

    #[test]
    fn relative_image_becomes_a_figure() {
        let out = render("![Trois zones](images/zones.svg)");
        assert_eq!(
            out,
            r#"<figure class="figure"><img src="/static/catalogue/git-basics/images/zones.svg" alt="Trois zones" loading="lazy"><figcaption>Trois zones</figcaption></figure>"#
        );
    }

    #[test]
    fn figure_caption_keeps_markdown_markers_verbatim() {
        assert!(render("avant\n\n![`__name__` vaut __main__](images/x.svg)\n\naprès")
            .contains("<figcaption>__name__ vaut __main__</figcaption>"));
    }

    #[test]
    fn mermaid_stays_escaped_text() {
        assert!(render("```mermaid\nA --> B\n```").contains(r#"<pre class="mermaid">A --&gt; B</pre>"#));
    }
}
