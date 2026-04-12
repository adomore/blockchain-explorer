---
AIGC:
    ContentProducer: Minimax Agent AI
    ContentPropagator: Minimax Agent AI
    Label: AIGC
    ProduceID: "00000000000000000000000000000000"
    PropagateID: "00000000000000000000000000000000"
    ReservedCode1: 3044022077be2ae431d114b03fcaa4a6bb9d13c7359621cc9ae6d0dd19d7a44e45151234022056cc87f847861acfe5818f47d4971d56894d60d7bd22a58ce94635c7de8c72a0
    ReservedCode2: 30440220191a32a0668d3667aa0440b12ebdee8829670c3b01586c70c45d491909ea2fa102203459121942c2ecbe98d23ab339e67156373bd1a9a57a223bf0959fbded39c653
---

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2024-XX-XX

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

## [0.2.0] - 2024-XX-XX

### Added
- Bitcoin address support via Blockstream API
- CSV export functionality
- Batch query support
- Transaction details display

## [0.1.0] - 2024-XX-XX

### Added
- Initial release
- Ethereum address query via Etherscan API
- Basic CLI interface
- Table and JSON output formats
