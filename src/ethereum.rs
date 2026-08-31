//! Ethereum blockchain data provider
//!
//! Uses Etherscan API V2 (https://docs.etherscan.io/v2-migration)
//! - Base URL: https://api.etherscan.io/v2/api
//! - Requires chainid parameter (1 for Ethereum mainnet)
//! - API key can be set via ETHERSCAN_API_KEY environment variable

use crate::blockchain::{utils, BlockchainError, BlockchainProvider};
use crate::models::{AddressInfo, BlockchainType, Transaction};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

/// Ethereum chain ID for Etherscan API V2
const ETH_CHAIN_ID: &str = "1";

/// Wei per ETH, as a power of ten
const ETH_DECIMALS: u8 = 18;

/// Default API key placeholder (rate limited)
const DEFAULT_API_KEY: &str = "YourApiKeyToken";

/// Ethereum provider using Etherscan API V2
pub struct EthereumProvider {
    client: Client,
    base_url: String,
    api_key: String,
}

impl EthereumProvider {
    pub fn new() -> Self {
        // Try to get API key from environment variable
        let api_key =
            std::env::var("ETHERSCAN_API_KEY").unwrap_or_else(|_| DEFAULT_API_KEY.to_string());

        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            // Etherscan API V2 base URL
            base_url: "https://api.etherscan.io/v2/api".to_string(),
            api_key,
        }
    }

    pub fn with_api_key(api_key: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: "https://api.etherscan.io/v2/api".to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub fn with_base_url(base_url: &str) -> Self {
        let api_key =
            std::env::var("ETHERSCAN_API_KEY").unwrap_or_else(|_| DEFAULT_API_KEY.to_string());

        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.to_string(),
            api_key,
        }
    }

    /// Get ETH balance in wei
    async fn fetch_balance(&self, address: &str) -> Result<String, BlockchainError> {
        // Etherscan API V2 format: includes chainid parameter
        let url = format!(
            "{}?chainid={}&module=account&action=balance&address={}&tag=latest&apikey={}",
            self.base_url, ETH_CHAIN_ID, address, self.api_key
        );

        tracing::debug!(
            "Fetching balance from: {}",
            url.replace(&self.api_key, "***")
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| BlockchainError::NetworkError(e.to_string()))?;

        let data: EtherScanResponse = response
            .json()
            .await
            .map_err(|e| BlockchainError::ParseError(e.to_string()))?;

        tracing::debug!(
            "API Response: status={}, message={}, result={}",
            data.status,
            data.message,
            data.result
        );

        if data.status == "1" {
            // Convert wei to ETH for display
            Ok(Self::wei_to_eth(&data.result))
        } else if data.result.contains("rate limit") {
            Err(BlockchainError::RateLimitExceeded)
        } else if data.result.contains("Invalid API Key") || data.result.contains("rate limit") {
            Err(BlockchainError::ApiError(format!(
                "API Error: {} - {}",
                data.message, data.result
            )))
        } else {
            // For addresses without transactions, 0 is returned
            Ok(Self::wei_to_eth(&data.result))
        }
    }

    /// Get transactions for an address
    async fn fetch_transactions(&self, address: &str) -> Result<Vec<Transaction>, BlockchainError> {
        // Etherscan API V2 format: includes chainid parameter
        let url = format!(
            "{}?chainid={}&module=account&action=txlist&address={}&startblock=0&endblock=99999999&page=1&offset=100&sort=desc&apikey={}",
            self.base_url, ETH_CHAIN_ID, address, self.api_key
        );

        tracing::debug!(
            "Fetching transactions from: {}",
            url.replace(&self.api_key, "***")
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| BlockchainError::NetworkError(e.to_string()))?;

        let data: EtherScanTxListResponse = response
            .json()
            .await
            .map_err(|e| BlockchainError::ParseError(e.to_string()))?;

        tracing::debug!(
            "API Response: status={}, message={}, result_count={}",
            data.status,
            data.message,
            data.result.len()
        );

        let mut transactions = Vec::new();

        if data.status == "1" {
            for tx in data.result {
                transactions.push(Transaction {
                    hash: tx.hash,
                    from: tx.from,
                    to: tx.to,
                    value: Self::wei_to_eth(&tx.value),
                    timestamp: tx.time_stamp.parse().ok(),
                    block_number: tx.block_number.parse().ok(),
                    gas_used: Some(tx.gas_used),
                    gas_price: Some(Self::wei_to_eth(&tx.gas_price)),
                    status: if tx.is_error == "0" {
                        "Success".to_string()
                    } else {
                        "Failed".to_string()
                    },
                });
            }
        } else if data.message != "OK" && !data.message.is_empty() {
            tracing::warn!("API returned error: {} - {:?}", data.message, data.result);
        }

        Ok(transactions)
    }

    /// Convert wei to ETH
    ///
    /// Division happens in `u128`, so no wei is lost. Going through `f64` would
    /// silently round: an `f64` mantissa holds 53 bits, and 1 ETH is 10^18 wei.
    fn wei_to_eth(wei: &str) -> String {
        utils::format_balance(wei, ETH_DECIMALS)
    }
}

impl Default for EthereumProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BlockchainProvider for EthereumProvider {
    fn name(&self) -> &'static str {
        "Etherscan"
    }

    fn blockchain_type(&self) -> BlockchainType {
        BlockchainType::Ethereum
    }

    fn supports_address(&self, address: &str) -> bool {
        let addr = address.trim();
        addr.starts_with("0x")
            && addr.len() == 42
            && addr[2..].chars().all(|c| c.is_ascii_hexdigit())
    }

    async fn get_address_info(&self, address: &str) -> Result<AddressInfo, BlockchainError> {
        let balance = self.get_balance(address).await?;
        let transactions = self.get_transactions(address).await?;

        let mut info = AddressInfo::new(address.to_string(), BlockchainType::Ethereum);
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
struct EtherScanResponse {
    status: String,
    message: String,
    result: String,
}

#[derive(Debug, Deserialize)]
struct EtherScanTxListResponse {
    status: String,
    message: String,
    result: Vec<EtherScanTx>,
}

#[derive(Debug, Deserialize)]
struct EtherScanTx {
    #[serde(rename = "hash")]
    hash: String,
    #[serde(rename = "from")]
    from: String,
    #[serde(rename = "to")]
    to: String,
    #[serde(rename = "value")]
    value: String,
    #[serde(rename = "timeStamp")]
    time_stamp: String,
    #[serde(rename = "blockNumber")]
    block_number: String,
    #[serde(rename = "gasUsed")]
    gas_used: String,
    #[serde(rename = "gasPrice")]
    gas_price: String,
    #[serde(rename = "isError")]
    is_error: String,
}

/// Mock Ethereum provider for testing
pub struct MockEthereumProvider {
    pub mock_balance: String,
    pub mock_transactions: Vec<Transaction>,
}

impl MockEthereumProvider {
    pub fn new() -> Self {
        Self {
            mock_balance: "1000000000000000000".to_string(), // 1 ETH in wei
            mock_transactions: vec![Transaction {
                hash: "0xabc123".to_string(),
                from: "0xfrom123".to_string(),
                to: "0xto456".to_string(),
                value: "0.5".to_string(),
                timestamp: Some(1234567890),
                block_number: Some(12345678),
                gas_used: Some("21000".to_string()),
                gas_price: Some("0.000000020".to_string()),
                status: "Success".to_string(),
            }],
        }
    }
}

impl Default for MockEthereumProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BlockchainProvider for MockEthereumProvider {
    fn name(&self) -> &'static str {
        "MockEthereum"
    }

    fn blockchain_type(&self) -> BlockchainType {
        BlockchainType::Ethereum
    }

    fn supports_address(&self, address: &str) -> bool {
        address.starts_with("0x") && address.len() == 42
    }

    async fn get_address_info(&self, address: &str) -> Result<AddressInfo, BlockchainError> {
        let mut info = AddressInfo::new(address.to_string(), BlockchainType::Ethereum);
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
    fn test_wei_to_eth_conversion() {
        // Test basic conversions
        assert_eq!(EthereumProvider::wei_to_eth("1000000000000000000"), "1");
        assert_eq!(EthereumProvider::wei_to_eth("500000000000000000"), "0.5");
        assert_eq!(EthereumProvider::wei_to_eth("0"), "0");

        // Test with trailing zeros
        assert_eq!(EthereumProvider::wei_to_eth("10000000000000000"), "0.01");
        assert_eq!(
            EthereumProvider::wei_to_eth("8627620683100812558"),
            "8.627620683100812558"
        );

        // Test with small amounts (should keep 8 decimal places minimum)
        assert_eq!(EthereumProvider::wei_to_eth("100000000000000"), "0.0001");
    }

    #[test]
    fn test_eth_address_validation() {
        let provider = EthereumProvider::new();

        // Valid ETH addresses
        assert!(provider.supports_address("0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21"));
        assert!(provider.supports_address("0x0000000000000000000000000000000000000000"));

        // Invalid addresses
        assert!(!provider.supports_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2")); // BTC address
        assert!(!provider.supports_address("invalid")); // Not 0x prefixed
        assert!(!provider.supports_address("0x123")); // Too short
    }

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockEthereumProvider::new();
        let info = provider
            .get_address_info("0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21")
            .await
            .unwrap();

        assert_eq!(info.blockchain, "Ethereum");
        assert_eq!(info.total_transactions, 1);
    }
}
