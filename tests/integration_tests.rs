//! Integration tests for the blockchain explorer

use blockchain_explorer::{
    address_match::write_report,
    cli::{BlockchainArg, Cli, Commands},
    csv_export, match_file, read_addresses_from_file, AddressInfo, BlockchainType, Explorer,
    MatchOptions, MatchReport, ReportFormat, Transaction,
};
use std::path::Path;

const ETH_ADDRESS: &str = "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21";
const BTC_P2PKH: &str = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
const BTC_P2SH: &str = "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy";
const BTC_BECH32: &str = "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq";

/// Path of a scratch file inside the platform temporary directory
fn temp_path(name: &str) -> String {
    let mut path = std::env::temp_dir();
    path.push(name);
    path.to_string_lossy().into_owned()
}

/// A text file mixing prose, JSON and log lines with BTC/ETH addresses
fn sample_text() -> String {
    format!(
        "# Wallet notes\n\
         Sent 1.5 ETH to {eth} yesterday.\n\
         {{\"refund\":\"{p2pkh}\",\"escrow\":\"{p2sh}\"}}\n\
         2024-01-02 12:00:01 INFO payout to {bech32} confirmed\n\
         See https://etherscan.io/address/{eth} for details\n\
         Not an address: 0x1234 deadbeef 1234567890\n",
        eth = ETH_ADDRESS,
        p2pkh = BTC_P2PKH,
        p2sh = BTC_P2SH,
        bech32 = BTC_BECH32,
    )
}

/// Write `sample_text()` to a scratch file and match it
fn match_sample(name: &str, options: &MatchOptions) -> MatchReport {
    let file = temp_path(name);
    std::fs::write(&file, sample_text()).unwrap();
    match_file(Path::new(&file), options).unwrap()
}

/// Create a test address info
fn create_test_address_info(
    address: &str,
    blockchain: BlockchainType,
    balance: &str,
    tx_count: u64,
) -> AddressInfo {
    let mut info = AddressInfo::new(address.to_string(), blockchain);
    info.balance = balance.to_string();
    info.total_transactions = tx_count;
    info
}

/// Test reading addresses from file
#[test]
fn test_read_addresses_from_file_success() {
    let temp_file = temp_path("test_addresses.txt");
    std::fs::write(
        &temp_file,
        "# Test addresses\n0x1234567890123456789012345678901234567890\n0xabcdefabcdefabcdefabcdefabcdefabcdefabcd\n",
    )
    .unwrap();

    let addresses = read_addresses_from_file(&temp_file).unwrap();
    assert_eq!(addresses.len(), 2);
    assert_eq!(addresses[0], "0x1234567890123456789012345678901234567890");
    assert_eq!(addresses[1], "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd");
}

/// Test reading empty file
#[test]
fn test_read_addresses_from_empty_file() {
    let temp_file = temp_path("empty_addresses.txt");
    std::fs::write(&temp_file, "").unwrap();

    let addresses = read_addresses_from_file(&temp_file).unwrap();
    assert!(addresses.is_empty());
}

/// Test reading file with only comments
#[test]
fn test_read_addresses_with_only_comments() {
    let temp_file = temp_path("comment_only.txt");
    std::fs::write(&temp_file, "# Comment 1\n# Comment 2\n").unwrap();

    let addresses = read_addresses_from_file(&temp_file).unwrap();
    assert!(addresses.is_empty());
}

/// Test CSV export
#[test]
fn test_csv_export_single_address() {
    let info = create_test_address_info(
        "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21",
        BlockchainType::Ethereum,
        "1.5",
        10,
    );

    let temp_file = temp_path("test_export_single.csv");
    let result = csv_export::export_to_csv(&[info], &temp_file);

    assert!(result.is_ok());

    // Verify content
    let content = std::fs::read_to_string(&temp_file).unwrap();
    assert!(content.contains("0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21"));
    assert!(content.contains("Ethereum"));
    assert!(content.contains("1.5"));
    assert!(content.contains("10"));
}

/// Test CSV export with multiple addresses
#[test]
fn test_csv_export_multiple_addresses() {
    let infos = vec![
        create_test_address_info(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21",
            BlockchainType::Ethereum,
            "1.0",
            5,
        ),
        create_test_address_info(
            "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2",
            BlockchainType::Bitcoin,
            "0.5",
            3,
        ),
    ];

    let temp_file = temp_path("test_export_multiple.csv");
    let result = csv_export::export_to_csv(&infos, &temp_file);

    assert!(result.is_ok());

    // Verify content
    let content = std::fs::read_to_string(&temp_file).unwrap();
    assert!(content.contains("0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21"));
    assert!(content.contains("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2"));
}

/// Test CSV export with empty data
#[test]
fn test_csv_export_empty_data() {
    let result = csv_export::export_to_csv(&[], &temp_path("empty_export.csv"));
    assert!(result.is_err());
}

/// Test blockchain type detection for Ethereum
#[test]
fn test_blockchain_detection_ethereum() {
    let explorer = Explorer::new();

    // Standard Ethereum address
    assert_eq!(
        explorer.detect_blockchain("0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21"),
        Some(BlockchainType::Ethereum)
    );

    // Ethereum address with lowercase
    assert_eq!(
        explorer.detect_blockchain("0x0000000000000000000000000000000000000000"),
        Some(BlockchainType::Ethereum)
    );

    // Invalid: not starting with 0x
    assert_eq!(
        explorer.detect_blockchain("742d35Cc6634C0532925a3b844Bc9e7595f8fE21"),
        None
    );

    // Invalid: wrong length
    assert_eq!(
        explorer.detect_blockchain("0x742d35Cc6634C0532925a3b844Bc9e7595f8"),
        None
    );
}

/// Test blockchain type detection for Bitcoin
#[test]
fn test_blockchain_detection_bitcoin() {
    let explorer = Explorer::new();

    // P2PKH address (starts with 1)
    assert_eq!(
        explorer.detect_blockchain("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2"),
        Some(BlockchainType::Bitcoin)
    );

    // P2SH address (starts with 3)
    assert_eq!(
        explorer.detect_blockchain("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy"),
        Some(BlockchainType::Bitcoin)
    );

    // Bech32 address (starts with bc1)
    assert_eq!(
        explorer.detect_blockchain("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq"),
        Some(BlockchainType::Bitcoin)
    );
}

/// Test blockchain type detection for invalid addresses
#[test]
fn test_blockchain_detection_invalid() {
    let explorer = Explorer::new();

    assert_eq!(explorer.detect_blockchain("invalid"), None);
    assert_eq!(explorer.detect_blockchain(""), None);
    assert_eq!(explorer.detect_blockchain("0x"), None);
}

/// Test Explorer with mock providers
#[tokio::test]
async fn test_explorer_mock_eth() {
    let explorer = Explorer::with_mocks();

    let result = explorer
        .query(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21",
            Some(BlockchainType::Ethereum),
        )
        .await;

    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.blockchain, "Ethereum");
}

/// Test Explorer with mock providers for Bitcoin
#[tokio::test]
async fn test_explorer_mock_btc() {
    let explorer = Explorer::with_mocks();

    let result = explorer
        .query(
            "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2",
            Some(BlockchainType::Bitcoin),
        )
        .await;

    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.blockchain, "Bitcoin");
}

/// Test batch query
#[tokio::test]
async fn test_batch_query() {
    let explorer = Explorer::with_mocks();

    let addresses = vec![
        "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21".to_string(),
        "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2".to_string(),
    ];

    let result = explorer.batch_query(addresses, None, 5).await;

    assert_eq!(result.total, 2);
    assert_eq!(result.successful, 2);
    assert_eq!(result.failed, 0);
}

/// Test batch query with invalid address
#[tokio::test]
async fn test_batch_query_with_invalid() {
    let explorer = Explorer::with_mocks();

    let addresses = vec![
        "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21".to_string(),
        "invalid_address".to_string(),
    ];

    let result = explorer.batch_query(addresses, None, 5).await;

    assert_eq!(result.total, 2);
    assert!(result.failed > 0);
}

/// Test batch export
#[test]
fn test_batch_export() {
    let results = vec![
        (
            "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21".to_string(),
            Ok(create_test_address_info(
                "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21",
                BlockchainType::Ethereum,
                "1.0",
                5,
            )),
        ),
        ("invalid".to_string(), Err("Invalid address".to_string())),
    ];

    let temp_file = temp_path("test_batch_export.csv");
    let result = csv_export::export_batch_to_csv(&results, &temp_file);

    assert!(result.is_ok());

    let content = std::fs::read_to_string(&temp_file).unwrap();
    assert!(content.contains("Success"));
    assert!(content.contains("Failed"));
}

/// Test auto-detection in query
#[tokio::test]
async fn test_auto_detection_query() {
    let explorer = Explorer::with_mocks();

    // Query without specifying blockchain (auto-detect)
    let eth_result = explorer
        .query("0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21", None)
        .await;
    assert!(eth_result.is_ok());
    assert_eq!(eth_result.unwrap().blockchain, "Ethereum");

    let btc_result = explorer
        .query("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2", None)
        .await;
    assert!(btc_result.is_ok());
    assert_eq!(btc_result.unwrap().blockchain, "Bitcoin");
}

/// Test AddressInfo to CsvRecord conversion
#[test]
fn test_address_info_to_csv_record() {
    let mut info = AddressInfo::new(
        "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21".to_string(),
        BlockchainType::Ethereum,
    );
    info.balance = "1.5".to_string();
    info.total_transactions = 2;
    info.transactions = vec![
        Transaction {
            hash: "0xabc123".to_string(),
            from: "0xfrom123".to_string(),
            to: "0xto456".to_string(),
            value: "0.5".to_string(),
            timestamp: Some(1234567890),
            block_number: Some(12345678),
            gas_used: Some("21000".to_string()),
            gas_price: Some("0.000000020".to_string()),
            status: "Success".to_string(),
        },
        Transaction {
            hash: "0xdef456".to_string(),
            from: "0xfrom789".to_string(),
            to: "0xto012".to_string(),
            value: "1.0".to_string(),
            timestamp: Some(1234567900),
            block_number: Some(12345679),
            gas_used: Some("21000".to_string()),
            gas_price: Some("0.000000021".to_string()),
            status: "Success".to_string(),
        },
    ];

    let record = blockchain_explorer::CsvRecord::from(&info);

    assert_eq!(record.address, info.address);
    assert_eq!(record.blockchain, "Ethereum");
    assert_eq!(record.balance, "1.5");
    assert_eq!(record.total_transactions, 2);
    assert_eq!(record.first_tx_hash, Some("0xabc123".to_string()));
    assert_eq!(record.last_tx_hash, Some("0xdef456".to_string()));
}

/// Test matching BTC/ETH addresses in a free-form text file
#[test]
fn test_match_addresses_from_text_file() {
    let report = match_sample("match_source.txt", &MatchOptions::default());

    assert_eq!(report.unique_addresses, 4);
    assert_eq!(report.ethereum_addresses, 1);
    assert_eq!(report.bitcoin_addresses, 3);
    // The Ethereum address appears twice, once inside a URL
    assert_eq!(report.total_occurrences, 5);
    assert_eq!(report.scanned_lines, 6);

    assert_eq!(report.matches[0].address, ETH_ADDRESS);
    assert_eq!(report.matches[0].occurrences, 2);
    assert_eq!(report.matches[0].line, 2);

    let addresses: Vec<&str> = report.matches.iter().map(|m| m.address.as_str()).collect();
    assert_eq!(
        addresses,
        vec![ETH_ADDRESS, BTC_P2PKH, BTC_P2SH, BTC_BECH32]
    );
}

/// Test matching only one chain, without checksum verification
#[test]
fn test_match_bitcoin_only_without_checksum() {
    let options = MatchOptions {
        blockchain: Some(BlockchainType::Bitcoin),
        verify_checksum: false,
        ..MatchOptions::default()
    };
    let report = match_sample("match_bitcoin_only.txt", &options);

    assert_eq!(report.ethereum_addresses, 0);
    assert_eq!(report.bitcoin_addresses, 3);
    assert!(!report.checksum_verified);
}

/// Test writing the match result as a text report
#[test]
fn test_match_report_written_as_text() {
    let report = match_sample("match_text_source.txt", &MatchOptions::default());
    let output = temp_path("match_report.txt");

    write_report(&report, Path::new(&output), ReportFormat::Text).unwrap();

    let content = std::fs::read_to_string(&output).unwrap();
    assert!(content.contains("BTC / ETH Address Match Report"));
    assert!(content.contains("Unique Addresses:"));
    assert!(content.contains(ETH_ADDRESS));
    assert!(content.contains(BTC_BECH32));
    assert!(content.contains("P2WPKH"));
}

/// Test writing the match result as CSV
#[test]
fn test_match_report_written_as_csv() {
    let report = match_sample("match_csv_source.txt", &MatchOptions::default());
    let output = temp_path("match_report.csv");

    write_report(&report, Path::new(&output), ReportFormat::Csv).unwrap();

    let content = std::fs::read_to_string(&output).unwrap();
    let mut lines = content.lines();
    assert!(lines
        .next()
        .unwrap()
        .starts_with("Index,Address,Blockchain,Address_Type,Checksum"));
    assert_eq!(lines.count(), 4);
    assert!(content.contains(BTC_P2SH));
    assert!(content.contains("Verified"));
}

/// Test writing the match result as JSON and reading it back
#[test]
fn test_match_report_written_as_json() {
    let report = match_sample("match_json_source.txt", &MatchOptions::default());
    let output = temp_path("match_report.json");

    write_report(&report, Path::new(&output), ReportFormat::Json).unwrap();

    let content = std::fs::read_to_string(&output).unwrap();
    let parsed: MatchReport = serde_json::from_str(&content).unwrap();

    assert_eq!(parsed.unique_addresses, report.unique_addresses);
    assert_eq!(parsed.matches.len(), report.matches.len());
    assert_eq!(parsed.matches[0].address, ETH_ADDRESS);
    assert_eq!(parsed.matches[3].address, BTC_BECH32);
}

/// Test that an empty result still produces a usable report file
#[test]
fn test_match_report_without_matches() {
    let file = temp_path("match_empty_source.txt");
    std::fs::write(&file, "no addresses in this file\n").unwrap();

    let report = match_file(Path::new(&file), &MatchOptions::default()).unwrap();
    assert_eq!(report.unique_addresses, 0);

    let output = temp_path("match_empty_report.csv");
    write_report(&report, Path::new(&output), ReportFormat::Csv).unwrap();

    let content = std::fs::read_to_string(&output).unwrap();
    assert_eq!(content.lines().count(), 1);
}

/// Test parsing the match command and inferring the report format
#[test]
fn test_match_command_arguments() {
    use clap::Parser;

    let args = Cli::try_parse_from([
        "blockchain-explorer",
        "match",
        "notes.txt",
        "--output",
        "found.json",
        "--blockchain",
        "bitcoin",
        "--all-occurrences",
    ])
    .unwrap();

    match args.command {
        Commands::Match(matched) => {
            assert!(matches!(matched.blockchain, Some(BlockchainArg::Bitcoin)));
            assert!(matched.all_occurrences);
            assert!(!matched.no_checksum);
            // Without --format the output extension decides
            assert_eq!(ReportFormat::from_path(&matched.output), ReportFormat::Json);
        }
        _ => panic!("Expected Match command"),
    }
}
