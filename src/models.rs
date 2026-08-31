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
