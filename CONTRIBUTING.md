---
AIGC:
    ContentProducer: Minimax Agent AI
    ContentPropagator: Minimax Agent AI
    Label: AIGC
    ProduceID: "00000000000000000000000000000000"
    PropagateID: "00000000000000000000000000000000"
    ReservedCode1: 30450221008686f01d613a60436dd2bf46e1eebfc71fd9040daab3756194060213ded238bf02205ba7492ca921dbc82a0f47d54e70601074ea2490118a61c486eb532e8314a2fe
    ReservedCode2: 3045022100a47cfefb60be1cf6995ec19702e0a701a5ff48061b1e70c5b840565713f2d08502207f2ab52e00853c94205a151b6149eebefcc794bb199e2e1803d88b8cb0a5a385
---

# Contributing to Blockchain Explorer

Thank you for your interest in contributing!

## How to Contribute

### Reporting Bugs

1. Check existing [issues](https://github.com/YOUR_USERNAME/blockchain-explorer/issues) first
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
- Run `cargo clippy` to check for common mistakes
- Add tests for new functionality

## Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/blockchain-explorer.git
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
