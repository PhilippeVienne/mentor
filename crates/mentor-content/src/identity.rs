//! Identifiers derived from what an author wrote.
//!
//! Stored data refers to exam questions and lab steps: an attempt keeps the questions it drew, progress keeps
//! the steps a learner validated. Those references must survive what does not change the meaning (another
//! Markdown renderer, a reordering, a question added before) and stop matching when the thing itself is
//! rewritten. So an identifier is a digest of the **source text**, never of the rendered HTML nor of a position.

use sha1::{Digest, Sha1};

/// Ten hexadecimal digits of the SHA-1 of `text`, where runs of white space count as one space: re-wrapping a
/// paragraph or re-indenting a block does not make it another text.
pub fn fingerprint(text: &str) -> String {
    let normalised = text.split_whitespace().collect::<Vec<_>>().join(" ");
    Sha1::digest(normalised.as_bytes()).iter().take(5).map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_space_does_not_make_another_text() {
        assert_eq!(fingerprint("Que fait `git init` ?"), fingerprint("  Que fait\n   `git init` ?\n"));
        assert_ne!(fingerprint("Que fait `git init` ?"), fingerprint("Que fait `git add` ?"));
        assert_eq!(fingerprint("Question ?").len(), 10);
    }
}
