//! Integration tests for the blockchain explorer

use blockchain_explorer::{
    cli::{BlockchainArg, Cli, Commands},
    csv_export, read_addresses_from_file, AddressInfo, BlockchainType, Explorer, Transaction,
};

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
    let temp_file = "/tmp/test_addresses.txt";
    std::fs::write(
        temp_file,
        "# Test addresses\n0x1234567890123456789012345678901234567890\n0xabcdefabcdefabcdefabcdefabcdefabcdefabcd\n",
    )
    .unwrap();

    let addresses = read_addresses_from_file(temp_file).unwrap();
    assert_eq!(addresses.len(), 2);
    assert_eq!(
        addresses[0],
        "0x1234567890123456789012345678901234567890"
    );
    assert_eq!(
        addresses[1],
        "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd"
    );
}

/// Test reading empty file
#[test]
fn test_read_addresses_from_empty_file() {
    let temp_file = "/tmp/empty_addresses.txt";
    std::fs::write(temp_file, "").unwrap();

    let addresses = read_addresses_from_file(temp_file).unwrap();
    assert!(addresses.is_empty());
}

/// Test reading file with only comments
#[test]
fn test_read_addresses_with_only_comments() {
    let temp_file = "/tmp/comment_only.txt";
    std::fs::write(temp_file, "# Comment 1\n# Comment 2\n").unwrap();

    let addresses = read_addresses_from_file(temp_file).unwrap();
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

    let temp_file = "/tmp/test_export_single.csv";
    let result = csv_export::export_to_csv(&[info], temp_file);

    assert!(result.is_ok());

    // Verify content
    let content = std::fs::read_to_string(temp_file).unwrap();
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

    let temp_file = "/tmp/test_export_multiple.csv";
    let result = csv_export::export_to_csv(&infos, temp_file);

    assert!(result.is_ok());

    // Verify content
    let content = std::fs::read_to_string(temp_file).unwrap();
    assert!(content.contains("0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21"));
    assert!(content.contains("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2"));
}

/// Test CSV export with empty data
#[test]
fn test_csv_export_empty_data() {
    let result = csv_export::export_to_csv(&[], "/tmp/empty_export.csv");
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
    assert_eq!(explorer.detect_blockchain("742d35Cc6634C0532925a3b844Bc9e7595f8fE21"), None);

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
        (
            "invalid".to_string(),
            Err("Invalid address".to_string()),
        ),
    ];

    let temp_file = "/tmp/test_batch_export.csv";
    let result = csv_export::export_batch_to_csv(&results, temp_file);

    assert!(result.is_ok());

    let content = std::fs::read_to_string(temp_file).unwrap();
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
