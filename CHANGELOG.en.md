# Changelog

[简体中文](CHANGELOG.md) | **English**

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

0.3.0 is dated from this repository's initial commit, whose `Cargo.toml` already
declared 0.3.0. 0.1.0 and 0.2.0 predate any recorded history here: there are no
tags and no earlier commits, so they are left undated rather than given a
made-up date.

## [Unreleased]

### Added
- `match` command: scan any text file for BTC/ETH addresses and write the
  result to a file (table, CSV or JSON, inferred from the output extension)
- Bitcoin checksum verification: Base58Check for `1.../3...` and
  Bech32/Bech32m (BIP-173 / BIP-350) for `bc1...`, with address type detection
  (P2PKH, P2SH, P2WPKH, P2WSH, P2TR)
- `--no-checksum`, `--all-occurrences`, `--blockchain` and `--query` options
  for the `match` command
- Report entries record the line, column and occurrence count of each address
- GitHub Actions CI: tests on Linux, macOS and Windows, plus `cargo fmt`,
  `cargo clippy -D warnings` and a job pinned to the declared MSRV
- `rust-version = "1.85"` in `Cargo.toml`, matching the highest requirement
  among the locked dependencies
- Unit tests for `blockchain::utils` and `models`, which had none
- Every document now has a Chinese original and an English mirror, checked by
  `scripts/check_lockstep.py` in CI

### Changed
- ETH and BTC balances now render identically: integer division, trailing
  zeros trimmed. 1 BTC prints as `1`, not `1.00000000`
- `--no-color` now actually disables ANSI colour in the log output

### Removed
- `--continue-on-error` and `--show-progress` on the `batch` command. Neither
  was ever read: batch queries always continue past failures and there is no
  progress reporting to switch off

### Fixed
- Missing public re-exports (`BlockchainType`, `Transaction`, `CsvRecord`) that
  prevented the integration test suite from compiling
- Tests wrote to a hardcoded `/tmp` path and failed on Windows
- `wei_to_eth` converted through `f64`, silently rounding balances above
  2^53 wei. Conversion now stays in `u128`

## [0.3.0] - 2026-04-12

### Added
- Etherscan API V2 support (deprecated V1)
- API Key configuration via environment variable or CLI argument
- Debug logging for API requests
- Enhanced error handling with better messages

### Fixed
- Ethereum balance display format (wei to ETH conversion)
- Bitcoin balance calculation (now shows spendable balance, not total received)

### Changed
- Updated base URL for Ethereum API
- Improved balance formatting (removes trailing zeros)

## [0.2.0] - date unknown

### Added
- Bitcoin address support via Blockstream API
- CSV export functionality
- Batch query support
- Transaction details display

## [0.1.0] - date unknown

### Added
- Initial release
- Ethereum address query via Etherscan API
- Basic CLI interface
- Table and JSON output formats
