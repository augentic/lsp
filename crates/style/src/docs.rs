//! `module_doc` and `item_doc_overview`: prose budgets on `//!` blocks
//! and `///` overviews. Fenced code is exempt; item-doc lines from the
//! first `#` heading on are exempt.

use clippy_utils::diagnostics::span_lint_and_help;
use rustc_ast as ast;
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_span::Span;

rustc_session::declare_lint! {
    /// ### What it does
    ///
    /// Caps the number of non-blank prose lines in a module (`//!`) doc.
    ///
    /// ### Why restrict this?
    ///
    /// Module docs are an overview, not a design document; depth belongs
    /// in the Developer Guide.
    pub MODULE_DOC,
    Deny,
    "module doc longer than the configured cap"
}

rustc_session::declare_lint! {
    /// ### What it does
    ///
    /// Caps the `///` overview: non-blank prose lines before the first
    /// `#` heading. `# Errors` / `# Panics` bodies do not count.
    ///
    /// ### Why restrict this?
    ///
    /// Item docs lead with a short overview; structured depth goes under
    /// headings, and essays go in guides.
    pub ITEM_DOC_OVERVIEW,
    Deny,
    "item doc overview longer than the configured cap"
}

pub struct Docs {
    module_cap: usize,
    item_cap: usize,
}

rustc_session::impl_lint_pass!(Docs => [MODULE_DOC, ITEM_DOC_OVERVIEW]);

/// One doc line with the span of the attribute it came from.
struct Line {
    text: String,
    span: Span,
}

fn doc_lines(attrs: &[ast::Attribute], style: ast::AttrStyle) -> Vec<Line> {
    let mut lines = Vec::new();
    for attr in attrs {
        if attr.style != style {
            continue;
        }
        let Some(text) = attr.doc_str() else { continue };
        for line in text.as_str().lines() {
            lines.push(Line {
                text: line.trim().to_owned(),
                span: attr.span,
            });
        }
    }
    lines
}

/// Count non-blank prose lines, skipping fenced code; `stop_at_heading`
/// ends the count at the first `#` heading (item-doc overview rule).
fn prose(lines: &[Line], stop_at_heading: bool) -> usize {
    let mut count = 0;
    let mut in_fence = false;
    for line in lines {
        if line.text.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if stop_at_heading && line.text.starts_with('#') {
            break;
        }
        if !line.text.is_empty() {
            count += 1;
        }
    }
    count
}

fn block_span(lines: &[Line]) -> Option<Span> {
    let first = lines.first()?;
    let last = lines.last()?;
    Some(first.span.to(last.span))
}

impl Docs {
    pub fn new(conf: &crate::config::Conf) -> Self {
        Self {
            module_cap: conf.module_doc,
            item_cap: conf.item_doc_overview,
        }
    }

    fn check_module(&self, cx: &EarlyContext<'_>, attrs: &[ast::Attribute]) {
        let lines = doc_lines(attrs, ast::AttrStyle::Inner);
        let count = prose(&lines, false);
        if count > self.module_cap
            && let Some(span) = block_span(&lines)
            && !span.from_expansion()
        {
            span_lint_and_help(
                cx,
                MODULE_DOC,
                span,
                format!("module doc (`//!`) runs {count} prose lines (cap {})", self.module_cap),
                None,
                "keep the overview short; move depth into guides",
            );
        }
    }

    fn check_overview(&self, cx: &EarlyContext<'_>, attrs: &[ast::Attribute]) {
        let lines = doc_lines(attrs, ast::AttrStyle::Outer);
        let count = prose(&lines, true);
        if count > self.item_cap
            && let Some(span) = block_span(&lines)
            && !span.from_expansion()
        {
            span_lint_and_help(
                cx,
                ITEM_DOC_OVERVIEW,
                span,
                format!(
                    "item doc (`///`) overview runs {count} prose lines (cap {})",
                    self.item_cap
                ),
                None,
                "lead with a short overview; move depth under `#` headings",
            );
        }
    }
}

impl EarlyLintPass for Docs {
    fn check_crate(&mut self, cx: &EarlyContext<'_>, krate: &ast::Crate) {
        self.check_module(cx, &krate.attrs);
    }

    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &ast::Item) {
        if matches!(item.kind, ast::ItemKind::Mod(..)) {
            self.check_module(cx, &item.attrs);
        }
        self.check_overview(cx, &item.attrs);
        match &item.kind {
            ast::ItemKind::Struct(_, _, data) | ast::ItemKind::Union(_, _, data) => {
                for field in data.fields() {
                    self.check_overview(cx, &field.attrs);
                }
            }
            _ => {}
        }
    }

    fn check_variant(&mut self, cx: &EarlyContext<'_>, variant: &ast::Variant) {
        self.check_overview(cx, &variant.attrs);
        for field in variant.data.fields() {
            self.check_overview(cx, &field.attrs);
        }
    }

    fn check_trait_item(&mut self, cx: &EarlyContext<'_>, item: &ast::AssocItem) {
        self.check_overview(cx, &item.attrs);
    }

    fn check_impl_item(&mut self, cx: &EarlyContext<'_>, item: &ast::AssocItem) {
        self.check_overview(cx, &item.attrs);
    }
}
