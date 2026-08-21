//! `line_comment_run` and `historical_comment`: `SourceMap` lints —
//! plain `//` lines are not in the AST. Findings are collected once per
//! crate and emitted at the innermost enclosing item so `#[allow]`
//! scoping works.

use clippy_utils::diagnostics::span_lint_and_help;
use rustc_ast as ast;
use rustc_lexer::{FrontmatterAllowed, TokenKind, tokenize};
use rustc_lint::{EarlyContext, EarlyLintPass, Lint, LintContext};
use rustc_span::{BytePos, FileName, Span};

rustc_session::declare_lint! {
    /// ### What it does
    ///
    /// Caps runs of consecutive non-blank `//` line comments.
    ///
    /// ### Why restrict this?
    ///
    /// A long comment run is prose the code should carry itself, or an
    /// overview that belongs in a doc comment.
    pub LINE_COMMENT_RUN,
    Deny,
    "run of `//` line comments longer than the configured cap"
}

rustc_session::declare_lint! {
    /// ### What it does
    ///
    /// Flags comment or doc text matching a configured historical phrase.
    ///
    /// ### Why restrict this?
    ///
    /// Archaeology belongs in git history, not in the source.
    pub HISTORICAL_COMMENT,
    Deny,
    "comment matching a configured historical phrase"
}

struct Finding {
    lint: &'static Lint,
    span: Span,
    message: String,
    help: &'static str,
}

pub struct Comments {
    run_cap: usize,
    phrases: Vec<String>,
    findings: Vec<Finding>,
}

rustc_session::impl_lint_pass!(Comments => [LINE_COMMENT_RUN, HISTORICAL_COMMENT]);

/// One `//` comment: byte range, line number, whether it starts its
/// line, whether it has non-blank content, and whether it is a doc.
struct LineComment {
    lo: u32,
    hi: u32,
    line: usize,
    leading: bool,
    blank: bool,
    doc: bool,
}

impl Comments {
    pub fn new(conf: &crate::config::Conf) -> Self {
        Self {
            run_cap: conf.line_comment_run,
            phrases: conf.historical_phrases.clone(),
            findings: Vec::new(),
        }
    }

    fn scan_file(&mut self, src: &str, start: BytePos) {
        let span_at =
            |lo: u32, hi: u32| Span::with_root_ctxt(start + BytePos(lo), start + BytePos(hi));
        let mut comments = Vec::new();
        let mut offset: u32 = 0;
        let mut line = 0usize;
        let mut line_start: u32 = 0;
        for token in tokenize(src, FrontmatterAllowed::No) {
            let lo = offset;
            let hi = offset + token.len;
            let text = &src[lo as usize..hi as usize];
            match token.kind {
                TokenKind::LineComment { doc_style } => {
                    let before = &src[line_start as usize..lo as usize];
                    let content = text.trim_start_matches('/').trim_start_matches('!');
                    comments.push(LineComment {
                        lo,
                        hi,
                        line,
                        leading: before.trim().is_empty(),
                        blank: content.trim().is_empty(),
                        doc: doc_style.is_some(),
                    });
                    self.check_phrases(text, span_at(lo, hi));
                }
                TokenKind::BlockComment { .. } => {
                    self.check_phrases(text, span_at(lo, hi));
                }
                _ => {}
            }
            for (index, _) in text.match_indices('\n') {
                line += 1;
                line_start = lo + u32::try_from(index).expect("token fits in u32") + 1;
            }
            offset = hi;
        }
        self.check_runs(&comments, span_at);
    }

    fn check_phrases(&mut self, text: &str, span: Span) {
        if let Some(phrase) = self.phrases.iter().find(|phrase| text.contains(phrase.as_str())) {
            self.findings.push(Finding {
                lint: HISTORICAL_COMMENT,
                span,
                message: format!("comment contains historical phrase `{phrase}`"),
                help: "archaeology belongs in git history; delete the comment",
            });
        }
    }

    fn check_runs(&mut self, comments: &[LineComment], span_at: impl Fn(u32, u32) -> Span) {
        let mut run: Vec<&LineComment> = Vec::new();
        let flush = |run: &mut Vec<&LineComment>, findings: &mut Vec<Finding>| {
            let count = run.iter().filter(|comment| !comment.blank).count();
            if count > self.run_cap
                && let (Some(first), Some(last)) = (run.first(), run.last())
            {
                findings.push(Finding {
                    lint: LINE_COMMENT_RUN,
                    span: span_at(first.lo, last.hi),
                    message: format!(
                        "line comment (`//`) run of {count} non-blank lines (cap {})",
                        self.run_cap
                    ),
                    help: "tighten the comment, or promote it to a doc comment",
                });
            }
            run.clear();
        };
        for comment in comments {
            if comment.doc || !comment.leading {
                flush(&mut run, &mut self.findings);
                continue;
            }
            if let Some(previous) = run.last()
                && comment.line != previous.line + 1
            {
                flush(&mut run, &mut self.findings);
            }
            run.push(comment);
        }
        flush(&mut run, &mut self.findings);
    }

    fn emit_within(&mut self, cx: &EarlyContext<'_>, enclosing: Option<Span>) {
        let mut kept = Vec::new();
        for finding in self.findings.drain(..) {
            if enclosing.is_none_or(|span| span.contains(finding.span)) {
                span_lint_and_help(
                    cx,
                    finding.lint,
                    finding.span,
                    finding.message,
                    None,
                    finding.help,
                );
            } else {
                kept.push(finding);
            }
        }
        self.findings = kept;
    }
}

impl EarlyLintPass for Comments {
    fn check_crate(&mut self, cx: &EarlyContext<'_>, _: &ast::Crate) {
        let files = cx.sess().source_map().files();
        for file in files.iter() {
            if !matches!(file.name, FileName::Real(_)) {
                continue;
            }
            let Some(src) = file.src.as_deref() else { continue };
            self.scan_file(src, file.start_pos);
        }
    }

    // Innermost items claim their findings first (post-order), so
    // `#[allow]` on the tightest enclosing item suppresses them.
    fn check_item_post(&mut self, cx: &EarlyContext<'_>, item: &ast::Item) {
        self.emit_within(cx, Some(item.span));
    }

    fn check_crate_post(&mut self, cx: &EarlyContext<'_>, _: &ast::Crate) {
        self.emit_within(cx, None);
    }
}
