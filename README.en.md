# Blockchain Explorer

[简体中文](README.md) | **English**

[![CI](https://github.com/adomore/blockchain-explorer/actions/workflows/ci.yml/badge.svg)](https://github.com/adomore/blockchain-explorer/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> A command-line tool written in Rust for querying blockchain (ETH, BTC) address data.

**Features:**
- 🔗 Query balance and transaction details for ETH/BTC addresses
- 📦 Batch queries from an address list
- 🔍 Match BTC/ETH addresses in any text file (Base58Check / Bech32 checksum verification)
- 📊 CSV export
- ⚡ Asynchronous parallel queries
- 🔒 Open source under the MIT licence

## Installation

### Prerequisites

- **Rust toolchain** 1.85 or newer (stable) - supports Windows 11, macOS, Linux
- **Git** - to clone the repository

### Building from source

```bash
# Clone the project
git clone https://github.com/adomore/blockchain-explorer.git
cd blockchain-explorer

# Build (debug profile)
cargo build

# Build (release profile, recommended for production use)
cargo build --release

# Run
./target/release/blockchain-explorer --help
```

---

## Build steps per operating system

### Windows 11

#### Option 1: PowerShell

```powershell
# 1. Install Rust (if you have not already)
# Download and run rustup-init.exe: https://rustup.rs

# 2. Open PowerShell and clone the project
git clone https://github.com/adomore/blockchain-explorer.git
cd blockchain-explorer

# 3. Build the project
cargo build --release

# 4. Run the program
.\target\release\blockchain-explorer.exe --help

# 5. Set the Etherscan API key (optional)
$env:ETHERSCAN_API_KEY = "YOUR_API_KEY"
.\target\release\blockchain-explorer.exe query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21
```

#### Option 2: Command Prompt (CMD)

```cmd
:: Clone the project
git clone https://github.com/adomore/blockchain-explorer.git
cd blockchain-explorer

:: Build the project
cargo build --release

:: Run the program
target\release\blockchain-explorer.exe --help
```

---

### macOS 15.7.5

#### Terminal

```bash
# 1. Install Homebrew (if you have not already)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. Activate the Rust environment
source ~/.cargo/env

# 4. Clone the project
git clone https://github.com/adomore/blockchain-explorer.git
cd blockchain-explorer

# 5. Build the project
cargo build --release

# 6. Run the program
./target/release/blockchain-explorer --help

# 7. Set the Etherscan API key (optional)
export ETHERSCAN_API_KEY="YOUR_API_KEY"
./target/release/blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21
```

---

### Ubuntu 24.04 LTS

#### Terminal

```bash
# 1. Install build dependencies
sudo apt update
sudo apt install -y build-essential curl git

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. Activate the Rust environment
source ~/.cargo/env

# 4. Clone the project
git clone https://github.com/adomore/blockchain-explorer.git
cd blockchain-explorer

# 5. Build the project
cargo build --release

# 6. Run the program
./target/release/blockchain-explorer --help

# 7. Set the Etherscan API key (optional)
export ETHERSCAN_API_KEY="YOUR_API_KEY"
./target/release/blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21
```

---

## Getting an API key

### Etherscan API key (for Ethereum)

1. Go to https://etherscan.io/apidashboard
2. Register or sign in
3. Create a new API key (free)

### Using the API key

```bash
# Option 1: environment variable
export ETHERSCAN_API_KEY="YOUR_API_KEY"

# Option 2: command-line argument
blockchain-explorer query <ADDRESS> --etherscan-api-key YOUR_API_KEY
```

> **Note**: without an API key the program falls back to a placeholder key and may be rate limited.

## Usage

### Global options

These options apply to every subcommand:

| Option | Description | Default |
|--------|-------------|---------|
| `-v, --verbose` | Emit DEBUG-level logs (the key in request URLs is masked) | off |
| `--no-color` | Turn off ANSI colour in log output, for redirecting to a file | off |
| `--etherscan-api-key <KEY>` | Etherscan API key, equivalent to the `ETHERSCAN_API_KEY` variable | none |

### Querying a single address

```bash
# Detect the blockchain automatically
blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21

# Force Ethereum
blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21 --blockchain ethereum

# Force Bitcoin
blockchain-explorer query 1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2 --blockchain bitcoin

# Include transaction details
blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21 --include-txs

# Print as JSON
blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21 --format json

# Print as a single CSV row (convenient for pipelines)
blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21 --format csv

# Write to a file (JSON by default)
blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21 -o result.json

# Write to a file in a chosen format (json / csv)
blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21 -o result.csv --export-format csv
```

`--format` controls what is printed to the terminal and `--export-format` controls what is written to the `-o` file; the two are independent.

### Batch queries

```bash
# Create an address list, one address per line
cat > addresses.txt << EOF
0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21
0xabcdefabcdefabcdefabcdefabcdefabcdefabcd
1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2
bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq
EOF

# Run the batch query
blockchain-explorer batch addresses.txt -o results.csv

# Set the degree of parallelism
blockchain-explorer batch addresses.txt -o results.csv --parallel 10
```

### Matching addresses in a text file

Unlike `batch`, which expects one address per line, the `match` subcommand scans **any text file**
- logs, chat transcripts, notes, JSON, HTML - for BTC/ETH addresses and writes the result to a file.

```bash
# Scan a text file, writing to the default output file matches.txt
blockchain-explorer match notes.txt

# Choose the output file; the format follows its extension (.csv / .json, otherwise table)
blockchain-explorer match notes.txt -o result.csv
blockchain-explorer match notes.txt -o result.json

# Set the output format explicitly (table / csv / json)
blockchain-explorer match notes.txt -o result.out --format json

# Restrict to a single chain
blockchain-explorer match notes.txt -o btc.csv --blockchain bitcoin

# Match on format alone, skipping Bitcoin checksums (example and forged addresses match too)
blockchain-explorer match notes.txt -o all.csv --no-checksum

# List every occurrence instead of unique addresses
blockchain-explorer match notes.txt -o all.csv --all-occurrences

# Also query balance and transaction count for each match (needs network access)
blockchain-explorer match notes.txt -o result.csv --query --parallel 10
```

**Matching rules:**

| Address type | Format | Verification |
|--------------|--------|--------------|
| Ethereum | `0x` + 40 hex characters | format only (the EIP-55 case checksum is not verified) |
| Bitcoin P2PKH / P2SH | `1...` / `3...`, 26-35 Base58 characters | Base58Check checksum |
| Bitcoin SegWit | `bc1...` (P2WPKH / P2WSH / P2TR) | Bech32 / Bech32m checksum (BIP-173 / BIP-350) |

- An address must be delimited by non-alphanumeric characters; one glued into surrounding text is not matched (for example `abc0x742d...`).
- Transaction hashes (`0x` + 64 hex characters) are not mistaken for addresses.
- Addresses are deduplicated by default, recording the line, column and occurrence count of the first sighting.
- Files are decoded leniently as UTF-8, so logs that are not valid UTF-8 can still be scanned.
- With `--no-checksum` only the format is matched, and false positives increase noticeably.

**Example output (default table format):**

```
=========================================================================================
BTC / ETH Address Match Report
=========================================================================================
Source File:           notes.txt
Scanned Lines:         9
Matched Occurrences:   6
Unique Addresses:      5
  Ethereum:            1
  Bitcoin:             4
Checksum Verified:     yes
Balance Queried:       no
-----------------------------------------------------------------------------------------
#    Chain    Type    Checksum    Line   Col  Hits Address
-----------------------------------------------------------------------------------------
1    Ethereum ETH     N/A            2    14     2 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21
2    Bitcoin  P2PKH   Verified       3    12     1 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
3    Bitcoin  P2SH    Verified       3    58     1 3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy
4    Bitcoin  P2WPKH  Verified       4    36     1 bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq
```

CSV output carries `Index,Address,Blockchain,Address_Type,Checksum,Line,Column,Occurrences,Balance,Total_Transactions,Query_Error,Context`;
JSON output is the complete match report, ready to be read by another program.

### Detecting the address type

```bash
# Detect the address type
blockchain-explorer detect 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21
blockchain-explorer detect 1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2
```

### Comparing several addresses

```bash
# Compare several addresses
blockchain-explorer compare \
    0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21 \
    0xabcdefabcdefabcdefabcdefabcdefabcdefabcd \
    -o comparison.csv
```

### Exporting existing results

```bash
# Convert a JSON result file to CSV
blockchain-explorer export data.json -o output.csv
```

## Output format examples

### Table output

```
================================================================================
Address Information
================================================================================
Address:             0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21
Blockchain:          Ethereum
Balance:             1.5
Total Transactions:  10

--------------------------------------------------------------------------------
Recent Transactions:
--------------------------------------------------------------------------------
Transaction #1
  Hash:        0xabc123...
  From:        0xfrom...
  To:          0xto...
  Value:       0.5
  Timestamp:   1234567890
  Status:      Success
================================================================================
```

### CSV output

```csv
Address,Blockchain,Balance,Total_Transactions,First_Tx_Hash,...
0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21,Ethereum,1.5,10,0xabc123,...
```

### Balance format

ETH and BTC balances are rendered by one rule: integer division (10^18 wei for ETH,
10^8 satoshi for BTC), then trailing zeros are trimmed. 1 ETH prints as `1`, 0.5 BTC
prints as `0.5`, and 1 satoshi prints as `0.00000001`. The conversion stays in `u128`
and never touches floating point, so large balances lose no precision.

> The `Balance_USD` column is always `N/A` - this tool is not wired to any price feed.

## Project layout

```
blockchain-explorer/
├── Cargo.toml            # package manifest (rust-version = 1.85)
├── README.md             # Chinese original
├── README.en.md          # English mirror
├── CHANGELOG.md          # release notes (English mirror in .en.md)
├── CONTRIBUTING.md       # contribution guide (English mirror in .en.md)
├── LICENSE               # MIT
├── .github/
│   └── workflows/
│       └── ci.yml        # tests on three platforms + fmt/clippy + MSRV + lockstep
├── scripts/
│   └── check_lockstep.py # EN/ZH documentation structure gate
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # library root
│   ├── cli.rs           # CLI argument parsing
│   ├── models.rs        # data models
│   ├── blockchain.rs    # blockchain trait
│   ├── ethereum.rs      # Ethereum provider
│   ├── bitcoin.rs       # Bitcoin provider
│   ├── address_match.rs # text address matching and report output
│   ├── checksum.rs      # Base58Check / Bech32 checksums
│   └── csv_export.rs    # CSV export
└── tests/
    └── integration_tests.rs  # integration tests
```

## Testing

```bash
# Run every test
cargo test

# Run specific tests
cargo test test_ethereum
cargo test test_bitcoin

# Run the tests with logging
RUST_LOG=debug cargo test

# The full set of checks CI runs
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

Every test uses mock providers and makes no network calls, so the suite runs offline.

## Technology stack

- **Rust** - systems programming language
- **Tokio** - asynchronous runtime
- **Reqwest** - HTTP client
- **Clap** - CLI argument parsing
- **Serde** - serialization and deserialization
- **CSV** - CSV file handling
- **Tracing** - logging

## About the APIs

This tool uses the following public APIs:

- **Ethereum**: Etherscan API V2 (https://api.etherscan.io/v2/api)
- **Bitcoin**: Blockstream API (https://blockstream.info/api)

### Important notes

- **Etherscan API V2**: the original V1 API was deprecated on 2025-08-15. This tool has been updated to the V2 endpoint.
- **API key**: applying for an Etherscan API key is strongly recommended to avoid rate limits
- **Bitcoin**: uses the public Blockstream API, no API key needed

## License

MIT License
