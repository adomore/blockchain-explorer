//! Bitcoin blockchain data provider

use crate::blockchain::{utils, BlockchainError, BlockchainProvider};
use crate::models::{AddressInfo, BlockchainType, Transaction};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

/// Satoshis per BTC, as a power of ten
const BTC_DECIMALS: u8 = 8;

/// Bitcoin provider using Blockstream API
pub struct BitcoinProvider {
    client: Client,
    base_url: String,
}

impl BitcoinProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: "https://blockstream.info/api".to_string(),
        }
    }

    pub fn with_base_url(base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.to_string(),
        }
    }

    /// Fetch balance for a Bitcoin address
    pub async fn fetch_balance(&self, address: &str) -> Result<String, BlockchainError> {
        let url = format!("{}/address/{}", self.base_url, address);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| BlockchainError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(BlockchainError::ApiError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let data: BlockstreamAddress = response
            .json()
            .await
            .map_err(|e| BlockchainError::ParseError(e.to_string()))?;

        // Calculate actual balance: Total Received (funded) - Total Sent (spent)
        // This gives the spendable balance (unspent outputs)
        let balance_sats = data.chain_stats.funded_txo_sum.saturating_sub(data.chain_stats.spent_txo_sum);
        Ok(Self::sats_to_btc(balance_sats))
    }

    /// Fetch transactions for a Bitcoin address
    pub async fn fetch_transactions(&self, address: &str) -> Result<Vec<Transaction>, BlockchainError> {
        let url = format!("{}/address/{}/txs", self.base_url, address);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| BlockchainError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(BlockchainError::ApiError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let txs: Vec<BlockstreamTx> = response
            .json()
            .await
            .map_err(|e| BlockchainError::ParseError(e.to_string()))?;

        let mut transactions = Vec::new();

        for tx in txs {
            // Determine if address is sender or receiver
            let is_sent = tx.vin.iter().any(|input| input.prevout.scriptpubkey_address == Some(address.to_string()));
            let is_received = tx.vout.iter().any(|output| output.scriptpubkey_address == Some(address.to_string()));

            if is_sent {
                for output in &tx.vout {
                    if output.scriptpubkey_address.as_deref() != Some(address) {
                        transactions.push(Transaction {
                            hash: tx.txid.clone(),
                            from: address.to_string(),
                            to: output.scriptpubkey_address.clone().unwrap_or_default(),
                            value: Self::sats_to_btc(output.value),
                            timestamp: Some(tx.status.block_time as i64),
                            block_number: Some(tx.status.block_height),
                            gas_used: None,
                            gas_price: None,
                            status: "Confirmed".to_string(),
                        });
                    }
                }
            }

            if is_received {
                for output in &tx.vout {
                    if output.scriptpubkey_address.as_deref() == Some(address) {
                        transactions.push(Transaction {
                            hash: tx.txid.clone(),
                            from: tx.vin.first().map(|i| i.prevout.scriptpubkey_address.clone().unwrap_or_default()).unwrap_or_default(),
                            to: address.to_string(),
                            value: Self::sats_to_btc(output.value),
                            timestamp: Some(tx.status.block_time as i64),
                            block_number: Some(tx.status.block_height),
                            gas_used: None,
                            gas_price: None,
                            status: "Confirmed".to_string(),
                        });
                    }
                }
            }
        }

        Ok(transactions)
    }

    /// Convert satoshis to BTC
    ///
    /// Shares [`utils::format_balance`] with the Ethereum provider so both
    /// chains render balances the same way.
    fn sats_to_btc(sats: u64) -> String {
        utils::format_balance(&sats.to_string(), BTC_DECIMALS)
    }

    /// Check if address is valid Bitcoin address format
    pub fn is_valid_address(address: &str) -> bool {
        let addr = address.trim();

        // P2PKH: starts with 1, 25-34 characters
        if addr.starts_with('1') && (25..=34).contains(&addr.len()) {
            return addr.chars().all(|c| c.is_ascii_alphanumeric());
        }

        // P2SH: starts with 3, 25-34 characters
        if addr.starts_with('3') && (25..=34).contains(&addr.len()) {
            return addr.chars().all(|c| c.is_ascii_alphanumeric());
        }

        // Bech32: starts with bc1, 42-62 characters
        if addr.starts_with("bc1") && (42..=62).contains(&addr.len()) {
            return addr.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
        }

        false
    }
}

impl Default for BitcoinProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BlockchainProvider for BitcoinProvider {
    fn name(&self) -> &'static str {
        "Blockstream"
    }

    fn blockchain_type(&self) -> BlockchainType {
        BlockchainType::Bitcoin
    }

    fn supports_address(&self, address: &str) -> bool {
        Self::is_valid_address(address)
    }

    async fn get_address_info(&self, address: &str) -> Result<AddressInfo, BlockchainError> {
        let balance = self.get_balance(address).await?;
        let transactions = self.get_transactions(address).await?;

        let mut info = AddressInfo::new(address.to_string(), BlockchainType::Bitcoin);
        info.balance = balance;
        info.total_transactions = transactions.len() as u64;
        info.transactions = transactions;

        Ok(info)
    }

    async fn get_transactions(&self, address: &str) -> Result<Vec<Transaction>, BlockchainError> {
        self.fetch_transactions(address).await
    }

    async fn get_balance(&self, address: &str) -> Result<String, BlockchainError> {
        self.fetch_balance(address).await
    }
}

// API Response types
#[derive(Debug, Deserialize)]
struct BlockstreamAddress {
    chain_stats: ChainStats,
}

#[derive(Debug, Deserialize)]
struct ChainStats {
    funded_txo_sum: u64,
    spent_txo_sum: u64,
}

#[derive(Debug, Deserialize)]
struct BlockstreamTx {
    txid: String,
    vin: Vec<Vin>,
    vout: Vec<Vout>,
    status: TxStatus,
}

#[derive(Debug, Deserialize)]
struct Vin {
    #[serde(rename = "prevout")]
    prevout: Prevout,
}

#[derive(Debug, Deserialize)]
struct Prevout {
    #[serde(rename = "scriptpubkey_address")]
    scriptpubkey_address: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Vout {
    #[serde(rename = "scriptpubkey_address")]
    scriptpubkey_address: Option<String>,
    value: u64,
}

#[derive(Debug, Deserialize)]
struct TxStatus {
    #[serde(rename = "block_height")]
    block_height: u64,
    #[serde(rename = "block_time")]
    block_time: u64,
}

/// Mock Bitcoin provider for testing
pub struct MockBitcoinProvider {
    pub mock_balance: String,
    pub mock_transactions: Vec<Transaction>,
}

impl MockBitcoinProvider {
    pub fn new() -> Self {
        Self {
            mock_balance: "1.5".to_string(),
            mock_transactions: vec![
                Transaction {
                    hash: "abc123def456".to_string(),
                    from: "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2".to_string(),
                    to: "1CounterpartyXXXXXXXXXXXXXXXUWLpVr".to_string(),
                    value: "0.5".to_string(),
                    timestamp: Some(1234567890),
                    block_number: Some(123456),
                    gas_used: None,
                    gas_price: None,
                    status: "Confirmed".to_string(),
                },
            ],
        }
    }
}

impl Default for MockBitcoinProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BlockchainProvider for MockBitcoinProvider {
    fn name(&self) -> &'static str {
        "MockBitcoin"
    }

    fn blockchain_type(&self) -> BlockchainType {
        BlockchainType::Bitcoin
    }

    fn supports_address(&self, address: &str) -> bool {
        BitcoinProvider::is_valid_address(address)
    }

    async fn get_address_info(&self, address: &str) -> Result<AddressInfo, BlockchainError> {
        let mut info = AddressInfo::new(address.to_string(), BlockchainType::Bitcoin);
        info.balance = self.mock_balance.clone();
        info.total_transactions = self.mock_transactions.len() as u64;
        info.transactions = self.mock_transactions.clone();
        Ok(info)
    }

    async fn get_transactions(&self, _address: &str) -> Result<Vec<Transaction>, BlockchainError> {
        Ok(self.mock_transactions.clone())
    }

    async fn get_balance(&self, _address: &str) -> Result<String, BlockchainError> {
        Ok(self.mock_balance.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_btc_address_validation() {
        // Valid P2PKH addresses
        assert!(BitcoinProvider::is_valid_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2"));
        assert!(BitcoinProvider::is_valid_address("1CounterpartyXXXXXXXXXXXXXXXUWLpVr"));

        // Valid P2SH addresses
        assert!(BitcoinProvider::is_valid_address("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy"));

        // Valid Bech32 addresses
        assert!(BitcoinProvider::is_valid_address("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq"));

        // Invalid addresses
        assert!(!BitcoinProvider::is_valid_address("0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21")); // ETH format
        assert!(!BitcoinProvider::is_valid_address("short")); // Too short
    }

    #[test]
    fn test_sats_to_btc() {
        // Trailing zeros are trimmed, matching the Ethereum side
        assert_eq!(BitcoinProvider::sats_to_btc(100_000_000), "1");
        assert_eq!(BitcoinProvider::sats_to_btc(50_000_000), "0.5");
        assert_eq!(BitcoinProvider::sats_to_btc(150_000_000), "1.5");
        assert_eq!(BitcoinProvider::sats_to_btc(1), "0.00000001");
        assert_eq!(BitcoinProvider::sats_to_btc(0), "0");
    }

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockBitcoinProvider::new();
        let info = provider
            .get_address_info("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2")
            .await
            .unwrap();

        assert_eq!(info.blockchain, "Bitcoin");
        assert_eq!(info.total_transactions, 1);
    }
}
