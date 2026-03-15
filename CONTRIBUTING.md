# Contributing to aitop

Thanks for your interest in contributing to aitop!

## Development Setup

```bash
git clone https://github.com/bugkill3r/aitop.git
cd aitop
cargo build
cargo run
```

## Running Tests

```bash
cargo test                    # all tests
cargo clippy -- -D warnings   # lint (zero warnings policy)
```

## Code Quality

- All PRs must pass `cargo clippy -- -D warnings` with zero warnings
- All PRs must pass `cargo test`
- No `unsafe` code unless absolutely necessary (and justified in PR description)

## Project Structure

- `src/data/` — Data layer: JSONL parsing, SQLite storage, aggregation queries
- `src/ui/` — TUI rendering: Ratatui-based views, themes, shared formatting
- `src/config.rs` — Configuration file parsing
- `src/app.rs` — Application state
- `src/main.rs` — Entry point and event loop
- `tests/` — Integration tests

## Adding Model Pricing

Model pricing is managed by `PricingRegistry` in `src/data/pricing.rs`. Built-in pricing covers Claude models. Users can override or add new model pricing via `config.toml`:

```toml
[model_pricing."my-model"]
input = 5.0
output = 25.0
cache_read = 0.50
cache_creation = 6.25
```

## Pull Request Process

1. Fork the repo and create a feature branch
2. Make your changes
3. Ensure `cargo clippy -- -D warnings` and `cargo test` pass
4. Open a PR with a clear description of the change

## Reporting Issues

Use the [GitHub issue tracker](https://github.com/bugkill3r/aitop/issues) with the appropriate template (bug report or feature request).

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
