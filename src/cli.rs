//! Command-line interface

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Supported blockchain types
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BlockchainArg {
    Ethereum,
    Bitcoin,
    Auto,
}

impl BlockchainArg {
    pub fn to_blockchain_type(&self) -> Option<crate::models::BlockchainType> {
        match self {
            BlockchainArg::Ethereum => Some(crate::models::BlockchainType::Ethereum),
            BlockchainArg::Bitcoin => Some(crate::models::BlockchainType::Bitcoin),
            BlockchainArg::Auto => None, // Will be auto-detected
        }
    }
}

/// Query a single blockchain address
#[derive(Parser, Debug)]
#[command(name = "query")]
#[command(about = "Query a single blockchain address")]
pub struct QueryArgs {
    /// The blockchain address to query
    #[arg(value_name = "ADDRESS")]
    pub address: String,

    /// Specify blockchain type (auto-detected if not specified)
    #[arg(short, long, value_enum, default_value_t = BlockchainArg::Auto)]
    pub blockchain: BlockchainArg,

    /// Output format
    #[arg(short, long, default_value = "table")]
    pub format: OutputFormat,

    /// Include transaction details
    #[arg(short, long, default_value_t = false)]
    pub include_txs: bool,

    /// Output file (optional)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Export format (when --output is specified)
    #[arg(long, value_enum, default_value_t = ExportFormatArg::Json)]
    pub export_format: ExportFormatArg,
}

/// Query multiple addresses from a file
#[derive(Parser, Debug)]
#[command(name = "batch")]
#[command(about = "Query multiple addresses from a file")]
pub struct BatchArgs {
    /// File containing addresses (one per line)
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// Output CSV file
    #[arg(short, long, default_value = "results.csv")]
    pub output: PathBuf,

    /// Parallel queries (default: 5)
    #[arg(short, long, default_value_t = 5)]
    pub parallel: usize,

    /// Specify blockchain type for all addresses
    #[arg(short, long, value_enum)]
    pub blockchain: Option<BlockchainArg>,
}

/// Match BTC/ETH addresses inside a text file
#[derive(Parser, Debug)]
#[command(name = "match")]
#[command(about = "Match BTC/ETH addresses in a text file and write the result to a file")]
pub struct MatchArgs {
    /// Text file to scan (logs, notes, exports, any text)
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// Output file for the match result
    #[arg(short, long, default_value = "matches.txt")]
    pub output: PathBuf,

    /// Output format (defaults to the output file extension, otherwise table)
    #[arg(short, long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Only match addresses of this blockchain (default: both)
    #[arg(short, long, value_enum)]
    pub blockchain: Option<BlockchainArg>,

    /// Match on format only, without verifying Bitcoin checksums
    #[arg(long, default_value_t = false)]
    pub no_checksum: bool,

    /// List every occurrence instead of unique addresses
    #[arg(long, default_value_t = false)]
    pub all_occurrences: bool,

    /// Also query balance and transaction count for every matched address
    #[arg(long, default_value_t = false)]
    pub query: bool,

    /// Parallel queries (only used with --query)
    #[arg(short, long, default_value_t = 5)]
    pub parallel: usize,
}

/// Export existing results to different formats
#[derive(Parser, Debug)]
#[command(name = "export")]
#[command(about = "Export results to CSV format")]
pub struct ExportArgs {
    /// Input file (JSON format)
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,

    /// Output CSV file
    #[arg(short, long, default_value = "export.csv")]
    pub output: PathBuf,
}

/// Compare multiple addresses
#[derive(Parser, Debug)]
#[command(name = "compare")]
#[command(about = "Compare multiple blockchain addresses")]
pub struct CompareArgs {
    /// Addresses to compare
    #[arg(value_name = "ADDRESSES", required = true, num_args = 2..)]
    pub addresses: Vec<String>,

    /// Output file (optional)
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

/// Output format options
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

/// Export format options
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExportFormatArg {
    Json,
    Csv,
}

/// Main CLI arguments
#[derive(Parser, Debug)]
#[command(
    name = "blockchain-explorer",
    about = "Blockchain data explorer - Query ETH, BTC addresses",
    long_about = None,
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true, default_value_t = false)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long, global = true, default_value_t = false)]
    pub no_color: bool,

    /// Etherscan API key (or set ETHERSCAN_API_KEY environment variable)
    /// Get your free API key at: https://etherscan.io/apidashboard
    #[arg(long, global = true, env = "ETHERSCAN_API_KEY")]
    pub etherscan_api_key: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Query a single blockchain address
    Query(QueryArgs),

    /// Batch query multiple addresses from a file
    Batch(BatchArgs),

    /// Match BTC/ETH addresses in a text file and write the result to a file
    Match(MatchArgs),

    /// Export results to CSV
    Export(ExportArgs),

    /// Compare multiple addresses
    Compare(CompareArgs),

    /// Detect blockchain type from address
    Detect {
        /// Address to detect
        #[arg(value_name = "ADDRESS")]
        address: String,
    },
}

impl Cli {
    /// Parse command-line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_command() {
        let args = Cli::try_parse_from([
            "blockchain-explorer",
            "query",
            "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21",
        ])
        .unwrap();

        match args.command {
            Commands::Query(query) => {
                assert_eq!(query.address, "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21");
                assert!(matches!(query.blockchain, BlockchainArg::Auto));
            }
            _ => panic!("Expected Query command"),
        }
    }

    #[test]
    fn test_parse_batch_command() {
        let args = Cli::try_parse_from([
            "blockchain-explorer",
            "batch",
            "addresses.txt",
            "-o",
            "output.csv",
        ])
        .unwrap();

        match args.command {
            Commands::Batch(batch) => {
                assert_eq!(batch.file, PathBuf::from("addresses.txt"));
                assert_eq!(batch.output, PathBuf::from("output.csv"));
            }
            _ => panic!("Expected Batch command"),
        }
    }

    #[test]
    fn test_parse_match_command() {
        let args = Cli::try_parse_from([
            "blockchain-explorer",
            "match",
            "notes.txt",
            "-o",
            "found.csv",
            "--no-checksum",
        ])
        .unwrap();

        match args.command {
            Commands::Match(matched) => {
                assert_eq!(matched.file, PathBuf::from("notes.txt"));
                assert_eq!(matched.output, PathBuf::from("found.csv"));
                assert!(matched.no_checksum);
                assert!(!matched.query);
                assert!(matched.format.is_none());
            }
            _ => panic!("Expected Match command"),
        }
    }

    #[test]
    fn test_match_command_defaults() {
        let args = Cli::try_parse_from(["blockchain-explorer", "match", "notes.txt"]).unwrap();

        match args.command {
            Commands::Match(matched) => {
                assert_eq!(matched.output, PathBuf::from("matches.txt"));
                assert_eq!(matched.parallel, 5);
                assert!(!matched.all_occurrences);
                assert!(matched.blockchain.is_none());
            }
            _ => panic!("Expected Match command"),
        }
    }

    #[test]
    fn test_parse_detect_command() {
        let args = Cli::try_parse_from([
            "blockchain-explorer",
            "detect",
            "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2",
        ])
        .unwrap();

        match args.command {
            Commands::Detect { address } => {
                assert_eq!(address, "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2");
            }
            _ => panic!("Expected Detect command"),
        }
    }
}
