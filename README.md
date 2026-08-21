# Augentic lints

House lints for Augentic repositories, shipped as [Dylint](https://github.com/trailofbits/dylint) libraries. Two libraries live here:

- **`style`** — the house prose budgets: identifier length, module/item doc caps, `//` run caps, and the historical-phrase ban.
- **`omnia`** — typed `Handler`/provider-bound analysis for [Omnia](https://github.com/augentic/omnia) applications. Silent in crates with no `Handler` impls and no provider-bounded helpers.

All lints land `Deny` by default. New noisy lints start `Allow` and promote.

## Adoption

Install the tools once:

```bash
cargo install cargo-dylint dylint-link
```

Add to the consuming workspace's root `Cargo.toml`:

```toml
[workspace.metadata.dylint]
libraries = [{ git = "https://github.com/augentic/lints", branch = "main", pattern = "crates/*" }]

[workspace.lints.rust.unexpected_cfgs]
level = "warn"
check-cfg = ["cfg(dylint_lib, values(any()))"]
```

Then run:

```bash
cargo dylint --all --workspace
```

Dylint builds the libraries with this repository's pinned nightly and runs them through its own driver; the consuming workspace stays on its own (stable) toolchain for everything else. That one metadata entry is the whole adoption for an Omnia application: crates with `Handler` impls get the `omnia` analysis for free, and everything gets the style budgets.

## Configuration

Caps and trait names are read from the consuming workspace's root `dylint.toml`. The defaults:

```toml
[style]
ident-length = 25
module-doc = 3
item-doc-overview = 8
line-comment-run = 3
historical-phrases = [
  "Phase ",
  "formerly",
  "previously lived",
  "old contract",
  "former tests",
  "to avoid the",
]

[omnia]
handler = "Handler"
providers = ["Config", "HttpRequest", "Publisher", "StateStore", "Identity", "TableStore"]
```

## The lints

### `style`

| Lint | Pass | What it does |
| --- | --- | --- |
| `ident_length` | Early | Declared item, field, and variant names longer than `ident-length` Unicode scalars (bare identifier, not the path). |
| `module_doc` | Early | Module `//!` prose longer than `module-doc` non-blank lines. Fenced code is exempt. |
| `item_doc_overview` | Early | Item `///` overview longer than `item-doc-overview` lines before the first `#` heading. Fences exempt; `# Errors` / `# Panics` bodies do not count. |
| `line_comment_run` | SourceMap | More than `line-comment-run` consecutive non-blank `//` lines. |
| `historical_comment` | SourceMap + docs | Comment or doc text matching a configured historical phrase — archaeology belongs in git. |

### `omnia`

| Lint | Pass | What it does |
| --- | --- | --- |
| `unused_provider_bound` | Late | Provider trait bound declared on a `Handler` impl or helper and never used by any call path. Typed: `Config::get` is not `StateStore::get`. |
| `missing_provider_bound` | Late | Provider method used (for example through a concrete provider type) without the matching bound; walks local helper functions. |

## Suppression

Lint names are bare (no tool namespace). Under stock builds the `dylint_lib` cfg is unset, so gate suppressions:

```rust
#[cfg_attr(dylint_lib = "style", allow(ident_length))]
fn a_name_the_cap_would_reject_but_the_wire_format_requires() {}
```

Suppressions should stay near zero; fix the finding instead.

## The wasm deny-list (stock Clippy, not a library)

Guest-only API bans need no custom lints: they are path deny-lists, which stock Clippy's configurable `disallowed_methods` / `disallowed_types` enforce on stable. Consumers apply the canonical config below via `CLIPPY_CONF_DIR` on a `--target wasm32-wasip2` clippy invocation only. `std::fs` / `std::thread` / `std::net` / `std::process` stay the compiler's problem; crate dependencies stay `cargo deny`'s.

```toml
# clippy-wasm.toml — wasm32 guest deny-list. Apply with:
#   CLIPPY_CONF_DIR=<dir> cargo clippy --target wasm32-wasip2 -- -D warnings
disallowed-methods = [
  { path = "std::env::var", reason = "guests receive configuration through the provider seam" },
  { path = "std::env::vars", reason = "guests receive configuration through the provider seam" },
  { path = "std::env::var_os", reason = "guests receive configuration through the provider seam" },
  { path = "std::time::SystemTime::now", reason = "wall-clock time is a host capability" },
  { path = "std::time::Instant::now", reason = "monotonic time is a host capability" },
]
disallowed-types = [
  { path = "std::sync::OnceLock", reason = "global state does not survive the instance-per-request model" },
  { path = "std::sync::LazyLock", reason = "global state does not survive the instance-per-request model" },
  { path = "tokio::runtime::Runtime", reason = "host async runtimes cannot run in the sandbox" },
  { path = "reqwest::Client", reason = "HTTP egress goes through the provider seam" },
  { path = "reqwest::blocking::Client", reason = "HTTP egress goes through the provider seam" },
]
```

## IDE integration (opt-in)

rust-analyzer can run the house lints on save; it is slower than stock clippy-on-save, and CI remains the gate:

```json
{ "rust-analyzer.check.overrideCommand": ["cargo", "dylint", "--all", "--workspace", "--", "--message-format=json"] }
```

## Toolchain

`rust-toolchain.toml` pins the one nightly this repository builds with; it moves together with the `clippy_utils` rev in `Cargo.toml`, both taken from the Dylint release we track. Consumers do not install it by hand — Dylint resolves it from this repository when building the libraries.

## Development

```bash
cargo make check   # fmt, clippy, tests (including UI tests)
cargo make ci      # the full gate: fmt --check, clippy, tests, cargo deny
cargo test -p style   # one library's UI suite
cargo dylint list --path . --pattern 'crates/*'   # the libraries and lints Dylint can see
```

UI fixtures live in each library's `ui/` directory with committed `.stderr` files.
