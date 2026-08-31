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

#[cfg(test)]
mod tests {
    use super::utils::{format_address, format_balance};

    #[test]
    fn test_format_balance_trims_trailing_zeros() {
        assert_eq!(format_balance("1000000000000000000", 18), "1");
        assert_eq!(format_balance("500000000000000000", 18), "0.5");
        assert_eq!(format_balance("10000000000000000", 18), "0.01");
        assert_eq!(format_balance("0", 18), "0");

        // Same rule at Bitcoin's 8 decimals
        assert_eq!(format_balance("100000000", 8), "1");
        assert_eq!(format_balance("50000000", 8), "0.5");
        assert_eq!(format_balance("1", 8), "0.00000001");
    }

    #[test]
    fn test_format_balance_keeps_every_digit() {
        // This is the case an f64 conversion silently rounds: the value needs
        // 63 bits of mantissa and an f64 only has 53.
        assert_eq!(
            format_balance("8627620683100812558", 18),
            "8.627620683100812558"
        );

        // u128::MAX, to show the divisor never overflows
        assert_eq!(
            format_balance("340282366920938463463374607431768211455", 18),
            "340282366920938463463.374607431768211455"
        );
    }

    #[test]
    fn test_format_balance_passes_through_unparseable_input() {
        assert_eq!(format_balance("N/A", 18), "N/A");
        assert_eq!(format_balance("-1", 8), "-1");
        assert_eq!(format_balance("", 8), "");
    }

    #[test]
    fn test_format_address_truncates_only_long_addresses() {
        assert_eq!(
            format_address("0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21"),
            "0x742d35...f8fE21"
        );

        // 16 characters or fewer are left alone
        assert_eq!(format_address("1BvBMSEYstWetqT"), "1BvBMSEYstWetqT");
    }
}
