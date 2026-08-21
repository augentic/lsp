//! Typed `Handler`/provider-bound lints for Omnia applications as a
//! Dylint library. Trait names are configured in the linted workspace's
//! `dylint.toml` under `[augentic_omnia]`.

#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod config;
mod provider;

dylint_linting::dylint_library!();

#[unsafe(no_mangle)]
pub fn register_lints(sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    dylint_linting::init_config(sess);
    let conf = config::Conf::load();
    lint_store.register_lints(&[provider::UNUSED_PROVIDER_BOUND, provider::MISSING_PROVIDER_BOUND]);
    lint_store.register_late_pass(move |_| Box::new(provider::ProviderBounds::new(&conf)));
}
