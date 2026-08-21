//! Augentic house prose budgets as a Dylint library. Caps and the
//! phrase list are configured in the linted workspace's `dylint.toml`
//! under `[style]`.

#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_lexer;
extern crate rustc_lint;
extern crate rustc_session;
extern crate rustc_span;

mod comments;
mod config;
mod docs;
mod idents;

dylint_linting::dylint_library!();

#[unsafe(no_mangle)]
pub fn register_lints(sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    dylint_linting::init_config(sess);
    let conf = config::Conf::load();
    lint_store.register_lints(&[
        idents::IDENT_LENGTH,
        docs::MODULE_DOC,
        docs::ITEM_DOC_OVERVIEW,
        comments::LINE_COMMENT_RUN,
        comments::HISTORICAL_COMMENT,
    ]);
    let for_idents = conf.clone();
    lint_store.register_early_pass(move || Box::new(idents::IdentLength::new(&for_idents)));
    let for_docs = conf.clone();
    lint_store.register_early_pass(move || Box::new(docs::Docs::new(&for_docs)));
    lint_store.register_early_pass(move || Box::new(comments::Comments::new(&conf)));
}
