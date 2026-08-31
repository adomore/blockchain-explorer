# Contributing to Blockchain Explorer

[简体中文](CONTRIBUTING.md) | **English**

Thank you for your interest in contributing!

## How to Contribute

### Reporting Bugs

1. Check existing [issues](https://github.com/adomore/blockchain-explorer/issues) first
2. Create a new issue with:
   - Clear title and description
   - Steps to reproduce
   - Expected vs actual behavior
   - Rust version and OS information

### Suggesting Features

1. Create a new issue labeled as "enhancement"
2. Describe the feature and its use case
3. Explain why it would be beneficial

### Pull Requests

1. Fork the repository
2. Create a new branch: `git checkout -b feature/your-feature-name`
3. Make your changes
4. Run tests: `cargo test`
5. Commit your changes: `git commit -m "Add feature: description"`
6. Push to the branch: `git push origin feature/your-feature-name`
7. Open a Pull Request

### Code Style

- Follow Rust idioms and conventions
- Run `cargo fmt` before committing
- Run `cargo clippy` to check for common mistakes -- CI treats warnings as errors
- Add tests for new functionality

### Documentation

Every document exists twice: a Chinese original and an English mirror, named
`README.md` and `README.en.md`, and likewise for the changelog and this guide.
The two sides must stay in lockstep -- same headings in the same order, same
code samples, same tables, same links, same numbers. Only the prose is
translated; a code sample is translated in its comments and nowhere else.

Update both sides in the same commit, and check them before pushing:

```bash
python3 scripts/check_lockstep.py
```

## Development Setup

```bash
# Clone your fork
git clone https://github.com/adomore/blockchain-explorer.git
cd blockchain-explorer

# Install dependencies
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- query <ADDRESS>
```

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
