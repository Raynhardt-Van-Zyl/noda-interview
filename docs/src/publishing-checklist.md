# Appendix: Future Publishing

Before publishing to crates.io, decide and add:

- License.
- Repository, homepage, and documentation URLs in `Cargo.toml`.
- Crate description, keywords, and categories.
- Decide whether to split the binary into a reusable library API.
- Public API stability expectations if a library API is added.
- README content that works well on crates.io.
- Versioning and release notes process.

The current crate is a binary, so publishing work should include an explicit API
decision rather than accidentally exposing internals.
