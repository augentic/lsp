//! `ident_length`: declared item, field, and variant names longer than
//! the configured cap (bare identifier, Unicode scalars, not the path).

use clippy_utils::diagnostics::span_lint_and_help;
use rustc_ast as ast;
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_span::Ident;

rustc_session::declare_lint! {
    /// ### What it does
    ///
    /// Caps the length of declared item, field, and variant names.
    ///
    /// ### Why restrict this?
    ///
    /// Long identifiers usually smuggle a sentence into a name; the house
    /// style keeps names short and moves the sentence into docs.
    ///
    /// ### Example
    ///
    /// ```rust,ignore
    /// fn resolve_component_metadata_from_cache() {}
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust,ignore
    /// fn cached_metadata() {}
    /// ```
    pub IDENT_LENGTH,
    Deny,
    "declared identifier longer than the configured cap"
}

pub struct IdentLength {
    cap: usize,
}

rustc_session::impl_lint_pass!(IdentLength => [IDENT_LENGTH]);

impl IdentLength {
    pub fn new(conf: &crate::config::Conf) -> Self {
        Self {
            cap: conf.ident_length,
        }
    }

    fn check(&self, cx: &EarlyContext<'_>, ident: Ident) {
        if ident.span.from_expansion() {
            return;
        }
        let name = ident.as_str();
        let length = name.chars().count();
        if length > self.cap {
            span_lint_and_help(
                cx,
                IDENT_LENGTH,
                ident.span,
                format!("`{name}` is {length} chars (cap {})", self.cap),
                None,
                "shorten the declared name",
            );
        }
    }

    fn check_fields(&self, cx: &EarlyContext<'_>, data: &ast::VariantData) {
        for field in data.fields() {
            if let Some(ident) = field.ident {
                self.check(cx, ident);
            }
        }
    }
}

impl EarlyLintPass for IdentLength {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &ast::Item) {
        if let Some(ident) = item.kind.ident() {
            self.check(cx, ident);
        }
        match &item.kind {
            ast::ItemKind::Struct(_, _, data) | ast::ItemKind::Union(_, _, data) => {
                self.check_fields(cx, data);
            }
            _ => {}
        }
    }

    fn check_variant(&mut self, cx: &EarlyContext<'_>, variant: &ast::Variant) {
        self.check(cx, variant.ident);
        self.check_fields(cx, &variant.data);
    }

    fn check_trait_item(&mut self, cx: &EarlyContext<'_>, item: &ast::AssocItem) {
        if let Some(ident) = item.kind.ident() {
            self.check(cx, ident);
        }
    }

    fn check_impl_item(&mut self, cx: &EarlyContext<'_>, item: &ast::AssocItem) {
        if let Some(ident) = item.kind.ident() {
            self.check(cx, ident);
        }
    }
}
