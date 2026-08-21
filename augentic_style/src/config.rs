//! Caps and phrase list, read from the linted workspace's `dylint.toml`
//! under the `[augentic_style]` table.

#[derive(Clone, serde::Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Conf {
    /// Maximum Unicode-scalar length of a declared identifier.
    pub ident_length: usize,
    /// Non-blank prose lines allowed per module (`//!`) doc block.
    pub module_doc: usize,
    /// Non-blank `///` overview lines allowed before the first `#` heading.
    pub item_doc_overview: usize,
    /// Consecutive non-blank `//` lines allowed per run.
    pub line_comment_run: usize,
    /// Phrases that mark a comment as archaeology.
    pub historical_phrases: Vec<String>,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            ident_length: 25,
            module_doc: 3,
            item_doc_overview: 8,
            line_comment_run: 3,
            historical_phrases: [
                "Phase ",
                "formerly",
                "previously lived",
                "old contract",
                "former tests",
                "to avoid the",
            ]
            .map(String::from)
            .to_vec(),
        }
    }
}

impl Conf {
    pub fn load() -> Self {
        dylint_linting::config_or_default(env!("CARGO_PKG_NAME"))
    }
}
