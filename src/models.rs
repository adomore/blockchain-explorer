//! Data models for blockchain address information

use serde::{Deserialize, Serialize};

/// Represents a blockchain type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockchainType {
    Ethereum,
    Bitcoin,
}

impl BlockchainType {
    /// Detect blockchain type from address format
    pub fn from_address(address: &str) -> Option<Self> {
        let addr = address.trim();

        // Ethereum addresses: start with 0x, 40 hex characters
        if addr.starts_with("0x")
            && addr.len() == 42
            && addr[2..].chars().all(|c| c.is_ascii_hexdigit())
        {
            return Some(BlockchainType::Ethereum);
        }

        // Bitcoin addresses: various formats
        // P2PKH: starts with 1, 25-34 characters
        // P2SH: starts with 3, 25-34 characters
        // Bech32: starts with bc1, 42-62 characters
        if addr.starts_with('1') && (25..=34).contains(&addr.len()) {
            return Some(BlockchainType::Bitcoin);
        }
        if addr.starts_with('3') && (25..=34).contains(&addr.len()) {
            return Some(BlockchainType::Bitcoin);
        }
        if addr.starts_with("bc1") && (42..=62).contains(&addr.len()) {
            return Some(BlockchainType::Bitcoin);
        }

        None
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            BlockchainType::Ethereum => "Ethereum",
            BlockchainType::Bitcoin => "Bitcoin",
        }
    }
}

/// Transaction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub timestamp: Option<i64>,
    pub block_number: Option<u64>,
    pub gas_used: Option<String>,
    pub gas_price: Option<String>,
    pub status: String,
}

/// Address information containing balance and transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressInfo {
    pub address: String,
    pub blockchain: String,
    pub balance: String,
    pub balance_usd: Option<String>,
    pub total_transactions: u64,
    pub transactions: Vec<Transaction>,
}

impl AddressInfo {
    /// Create a new AddressInfo with basic information
    pub fn new(address: String, blockchain: BlockchainType) -> Self {
        Self {
            address,
            blockchain: blockchain.as_str().to_string(),
            balance: "0".to_string(),
            balance_usd: None,
            total_transactions: 0,
            transactions: Vec::new(),
        }
    }
}

/// CSV record for export
#[derive(Debug, Clone, serde::Serialize)]
pub struct CsvRecord {
    pub address: String,
    pub blockchain: String,
    pub balance: String,
    pub balance_usd: Option<String>,
    pub total_transactions: u64,
    pub first_tx_hash: Option<String>,
    pub first_tx_from: Option<String>,
    pub first_tx_to: Option<String>,
    pub first_tx_value: Option<String>,
    pub first_tx_timestamp: Option<String>,
    pub last_tx_hash: Option<String>,
    pub last_tx_from: Option<String>,
    pub last_tx_to: Option<String>,
    pub last_tx_value: Option<String>,
    pub last_tx_timestamp: Option<String>,
}

impl From<&AddressInfo> for CsvRecord {
    fn from(info: &AddressInfo) -> Self {
        let first = info.transactions.first();
        let last = info.transactions.last();

        Self {
            address: info.address.clone(),
            blockchain: info.blockchain.clone(),
            balance: info.balance.clone(),
            balance_usd: info.balance_usd.clone(),
            total_transactions: info.total_transactions,
            first_tx_hash: first.map(|t| t.hash.clone()),
            first_tx_from: first.map(|t| t.from.clone()),
            first_tx_to: first.map(|t| t.to.clone()),
            first_tx_value: first.map(|t| t.value.clone()),
            first_tx_timestamp: first.and_then(|t| t.timestamp.map(|ts| ts.to_string())),
            last_tx_hash: last.map(|t| t.hash.clone()),
            last_tx_from: last.map(|t| t.from.clone()),
            last_tx_to: last.map(|t| t.to.clone()),
            last_tx_value: last.map(|t| t.value.clone()),
            last_tx_timestamp: last.and_then(|t| t.timestamp.map(|ts| ts.to_string())),
        }
    }
}

/// Batch query result
#[derive(Debug, Clone)]
pub struct BatchResult {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<Result<AddressInfo, String>>,
}

impl BatchResult {
    pub fn new() -> Self {
        Self {
            total: 0,
            successful: 0,
            failed: 0,
            results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: Result<AddressInfo, String>) {
        self.total += 1;
        match &result {
            Ok(_) => self.successful += 1,
            Err(_) => self.failed += 1,
        }
        self.results.push(result);
    }
}

impl Default for BatchResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_address_detects_ethereum() {
        assert_eq!(
            BlockchainType::from_address("0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21"),
            Some(BlockchainType::Ethereum)
        );

        // Surrounding whitespace is trimmed
        assert_eq!(
            BlockchainType::from_address("  0x0000000000000000000000000000000000000000  "),
            Some(BlockchainType::Ethereum)
        );
    }

    #[test]
    fn test_from_address_detects_bitcoin() {
        // P2PKH, P2SH, Bech32
        assert_eq!(
            BlockchainType::from_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2"),
            Some(BlockchainType::Bitcoin)
        );
        assert_eq!(
            BlockchainType::from_address("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy"),
            Some(BlockchainType::Bitcoin)
        );
        assert_eq!(
            BlockchainType::from_address("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq"),
            Some(BlockchainType::Bitcoin)
        );
    }

    #[test]
    fn test_from_address_rejects_non_addresses() {
        assert_eq!(BlockchainType::from_address(""), None);
        assert_eq!(BlockchainType::from_address("invalid"), None);

        // Right prefix and length, but not hex
        assert_eq!(
            BlockchainType::from_address("0xZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ"),
            None
        );

        // An Ethereum transaction hash is 0x + 64 hex, not an address
        assert_eq!(
            BlockchainType::from_address(&format!("0x{}", "a".repeat(64))),
            None
        );

        // Too short for any Bitcoin format
        assert_eq!(BlockchainType::from_address("1abc"), None);
    }

    #[test]
    fn test_address_info_starts_empty() {
        let info = AddressInfo::new("0xabc".to_string(), BlockchainType::Ethereum);

        assert_eq!(info.blockchain, "Ethereum");
        assert_eq!(info.balance, "0");
        assert_eq!(info.total_transactions, 0);
        assert!(info.balance_usd.is_none());
        assert!(info.transactions.is_empty());
    }

    #[test]
    fn test_csv_record_takes_first_and_last_transaction() {
        let mut info = AddressInfo::new("0xabc".to_string(), BlockchainType::Ethereum);
        info.transactions = vec![
            Transaction {
                hash: "first".to_string(),
                from: "a".to_string(),
                to: "b".to_string(),
                value: "1".to_string(),
                timestamp: Some(100),
                block_number: Some(1),
                gas_used: None,
                gas_price: None,
                status: "Success".to_string(),
            },
            Transaction {
                hash: "last".to_string(),
                from: "c".to_string(),
                to: "d".to_string(),
                value: "2".to_string(),
                timestamp: None,
                block_number: Some(2),
                gas_used: None,
                gas_price: None,
                status: "Success".to_string(),
            },
        ];

        let record = CsvRecord::from(&info);
        assert_eq!(record.first_tx_hash.as_deref(), Some("first"));
        assert_eq!(record.last_tx_hash.as_deref(), Some("last"));
        assert_eq!(record.first_tx_timestamp.as_deref(), Some("100"));

        // A missing timestamp stays missing rather than becoming "0"
        assert_eq!(record.last_tx_timestamp, None);
    }

    #[test]
    fn test_csv_record_from_address_without_transactions() {
        let info = AddressInfo::new("0xabc".to_string(), BlockchainType::Ethereum);
        let record = CsvRecord::from(&info);

        assert_eq!(record.first_tx_hash, None);
        assert_eq!(record.last_tx_hash, None);
    }

    #[test]
    fn test_batch_result_counts_both_outcomes() {
        let mut result = BatchResult::new();
        result.add_result(Ok(AddressInfo::new(
            "0xabc".to_string(),
            BlockchainType::Ethereum,
        )));
        result.add_result(Err("boom".to_string()));

        assert_eq!(result.total, 2);
        assert_eq!(result.successful, 1);
        assert_eq!(result.failed, 1);
        assert_eq!(result.results.len(), 2);
    }
}
