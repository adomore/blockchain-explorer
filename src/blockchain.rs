//! Blockchain trait and common utilities

use crate::models::{AddressInfo, BlockchainType, Transaction};
use async_trait::async_trait;
use thiserror::Error;

/// Errors that can occur during blockchain operations
#[derive(Error, Debug)]
pub enum BlockchainError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Unsupported blockchain")]
    UnsupportedBlockchain,
}

/// Trait for blockchain data providers
#[async_trait]
pub trait BlockchainProvider: Send + Sync {
    /// Get provider name
    fn name(&self) -> &'static str;

    /// Get supported blockchain type
    fn blockchain_type(&self) -> BlockchainType;

    /// Check if this provider supports the given address
    fn supports_address(&self, address: &str) -> bool;

    /// Fetch address information including balance and transactions
    async fn get_address_info(&self, address: &str) -> Result<AddressInfo, BlockchainError>;

    /// Fetch transactions for an address
    async fn get_transactions(&self, address: &str) -> Result<Vec<Transaction>, BlockchainError>;

    /// Fetch balance for an address
    async fn get_balance(&self, address: &str) -> Result<String, BlockchainError>;
}

/// Utility functions
pub mod utils {
    use super::*;

    /// Detect blockchain type from address
    pub fn detect_blockchain(address: &str) -> Option<BlockchainType> {
        BlockchainType::from_address(address)
    }

    /// Format address for display (truncate if too long)
    pub fn format_address(address: &str) -> String {
        if address.len() > 16 {
            format!("{}...{}", &address[..8], &address[address.len() - 6..])
        } else {
            address.to_string()
        }
    }

    /// Format balance with appropriate precision
    pub fn format_balance(balance: &str, decimals: u8) -> String {
        // Try to parse as u128 and format properly
        if let Ok(value) = balance.parse::<u128>() {
            let divisor = 10u128.pow(decimals as u32);
            let whole = value / divisor;
            let fraction = value % divisor;

            if fraction == 0 {
                format!("{}", whole)
            } else {
                let fraction_str = format!("{:0>width$}", fraction, width = decimals as usize);
                format!("{}.{}", whole, fraction_str.trim_end_matches('0'))
            }
        } else {
            balance.to_string()
        }
    }
}
