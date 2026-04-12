//! Blockchain Explorer Library
//!
//! A Rust library for querying blockchain data from Ethereum and Bitcoin networks.

pub mod blockchain;
pub mod bitcoin;
pub mod cli;
pub mod csv_export;
pub mod ethereum;
pub mod models;

use blockchain::{BlockchainError, BlockchainProvider};
use bitcoin::BitcoinProvider;
use ethereum::EthereumProvider;
use models::BlockchainType;
use std::sync::Arc;

// Re-export types for public API
pub use models::{AddressInfo, BatchResult};

/// Explorer client for querying blockchain data
pub struct Explorer {
    eth_provider: Arc<dyn BlockchainProvider>,
    btc_provider: Arc<dyn BlockchainProvider>,
}

impl Default for Explorer {
    fn default() -> Self {
        Self::new()
    }
}

impl Explorer {
    /// Create a new explorer instance
    pub fn new() -> Self {
        Self {
            eth_provider: Arc::new(EthereumProvider::new()),
            btc_provider: Arc::new(BitcoinProvider::new()),
        }
    }

    /// Create explorer with custom Etherscan API key
    /// Get your free API key at: https://etherscan.io/apidashboard
    pub fn with_etherscan_api_key(api_key: &str) -> Self {
        Self {
            eth_provider: Arc::new(EthereumProvider::with_api_key(api_key)),
            btc_provider: Arc::new(BitcoinProvider::new()),
        }
    }

    /// Create explorer with mock providers (for testing)
    pub fn with_mocks() -> Self {
        Self {
            eth_provider: Arc::new(ethereum::MockEthereumProvider::new()),
            btc_provider: Arc::new(bitcoin::MockBitcoinProvider::new()),
        }
    }

    /// Get the appropriate provider for an address
    fn get_provider(&self, blockchain: Option<BlockchainType>) -> Option<Arc<dyn BlockchainProvider>> {
        match blockchain {
            Some(BlockchainType::Ethereum) => Some(self.eth_provider.clone()),
            Some(BlockchainType::Bitcoin) => Some(self.btc_provider.clone()),
            None => None, // Will be auto-detected
        }
    }

    /// Auto-detect blockchain and get provider
    fn detect_and_get_provider(
        &self,
        address: &str,
    ) -> Result<Arc<dyn BlockchainProvider>, BlockchainError> {
        // Try Ethereum first
        if self.eth_provider.supports_address(address) {
            return Ok(self.eth_provider.clone());
        }

        // Try Bitcoin
        if self.btc_provider.supports_address(address) {
            return Ok(self.btc_provider.clone());
        }

        Err(BlockchainError::InvalidAddress(format!(
            "Address '{}' is not recognized as a valid Ethereum or Bitcoin address",
            address
        )))
    }

    /// Query address information
    pub async fn query(&self, address: &str, blockchain: Option<BlockchainType>) -> Result<models::AddressInfo, BlockchainError> {
        let provider = match self.get_provider(blockchain) {
            Some(p) => p,
            None => self.detect_and_get_provider(address)?,
        };

        provider.get_address_info(address).await
    }

    /// Query multiple addresses
    pub async fn batch_query(
        &self,
        addresses: Vec<String>,
        blockchain: Option<BlockchainType>,
        parallel: usize,
    ) -> models::BatchResult {
        let mut result = models::BatchResult::new();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(parallel));

        let mut handles = Vec::new();

        for address in addresses {
            let eth_provider = self.eth_provider.clone();
            let btc_provider = self.btc_provider.clone();
            let semaphore = semaphore.clone();
            let blockchain = blockchain;

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.ok();

                let provider = match blockchain {
                    Some(BlockchainType::Ethereum) => eth_provider.clone(),
                    Some(BlockchainType::Bitcoin) => btc_provider.clone(),
                    None => {
                        // Auto-detect
                        if eth_provider.supports_address(&address) {
                            eth_provider.clone()
                        } else if btc_provider.supports_address(&address) {
                            btc_provider.clone()
                        } else {
                            return Err(format!("Invalid address: {}", address));
                        }
                    }
                };

                provider.get_address_info(&address).await.map_err(|e| e.to_string())
            });

            handles.push(handle);
        }

        for handle in handles {
            if let Ok(result_item) = handle.await {
                result.add_result(result_item);
            } else {
                result.add_result(Err("Task cancelled".to_string()));
            }
        }

        result
    }

    /// Detect blockchain type from address
    pub fn detect_blockchain(&self, address: &str) -> Option<BlockchainType> {
        BlockchainType::from_address(address)
    }
}

/// Read addresses from a file (one per line)
pub fn read_addresses_from_file(path: &str) -> Result<Vec<String>, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    let addresses: Vec<String> = content
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    Ok(addresses)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_explorer_query() {
        let explorer = Explorer::with_mocks();

        // Test Ethereum address
        let result = explorer
            .query("0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21", None)
            .await;
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.blockchain, "Ethereum");

        // Test Bitcoin address
        let result = explorer
            .query("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2", None)
            .await;
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.blockchain, "Bitcoin");
    }

    #[test]
    fn test_detect_blockchain() {
        let explorer = Explorer::new();

        // Ethereum
        assert_eq!(
            explorer.detect_blockchain("0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21"),
            Some(BlockchainType::Ethereum)
        );

        // Bitcoin P2PKH
        assert_eq!(
            explorer.detect_blockchain("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2"),
            Some(BlockchainType::Bitcoin)
        );

        // Bitcoin Bech32
        assert_eq!(
            explorer.detect_blockchain("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq"),
            Some(BlockchainType::Bitcoin)
        );

        // Invalid
        assert_eq!(explorer.detect_blockchain("invalid"), None);
    }

    #[test]
    fn test_read_addresses_from_file() {
        let content = "# Comments\n0x123\n\n0x456\n";
        let path = "/tmp/test_addresses.txt";
        std::fs::write(path, content).unwrap();

        let addresses = read_addresses_from_file(path).unwrap();
        assert_eq!(addresses.len(), 2);
        assert_eq!(addresses[0], "0x123");
        assert_eq!(addresses[1], "0x456");
    }

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
}
