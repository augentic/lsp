//! Handler and provider trait names, read from the linted workspace's
//! `dylint.toml` under the `[augentic_omnia]` table.

#[derive(Clone, serde::Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Conf {
    /// Name of the handler trait whose impls are analysed.
    pub handler: String,
    /// Names of the Omnia provider traits.
    pub providers: Vec<String>,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            handler: "Handler".to_owned(),
            providers: [
                "Config",
                "HttpRequest",
                "Publisher",
                "StateStore",
                "Identity",
                "TableStore",
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
