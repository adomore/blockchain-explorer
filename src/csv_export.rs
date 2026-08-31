//! CSV export functionality

use crate::address_match::MatchReport;
use crate::models::{AddressInfo, CsvRecord};
use csv::Writer;
use std::fs::File;
use std::io;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during CSV export
#[derive(Error, Debug)]
pub enum CsvExportError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),

    #[error("No data to export")]
    NoData,
}

/// Export address info to CSV file
pub fn export_to_csv(addresses: &[AddressInfo], output_path: &str) -> Result<(), CsvExportError> {
    if addresses.is_empty() {
        return Err(CsvExportError::NoData);
    }

    let file = File::create(output_path)?;
    let mut writer = Writer::from_writer(file);

    // Write header
    writer.write_record([
        "Address",
        "Blockchain",
        "Balance",
        "Balance_USD",
        "Total_Transactions",
        "First_Tx_Hash",
        "First_Tx_From",
        "First_Tx_To",
        "First_Tx_Value",
        "First_Tx_Timestamp",
        "Last_Tx_Hash",
        "Last_Tx_From",
        "Last_Tx_To",
        "Last_Tx_Value",
        "Last_Tx_Timestamp",
    ])?;

    // Write data rows
    for info in addresses {
        let record = CsvRecord::from(info);
        writer.write_record([
            &record.address,
            &record.blockchain,
            &record.balance,
            record.balance_usd.as_deref().unwrap_or("N/A"),
            &record.total_transactions.to_string(),
            record.first_tx_hash.as_deref().unwrap_or("N/A"),
            record.first_tx_from.as_deref().unwrap_or("N/A"),
            record.first_tx_to.as_deref().unwrap_or("N/A"),
            record.first_tx_value.as_deref().unwrap_or("N/A"),
            record.first_tx_timestamp.as_deref().unwrap_or("N/A"),
            record.last_tx_hash.as_deref().unwrap_or("N/A"),
            record.last_tx_from.as_deref().unwrap_or("N/A"),
            record.last_tx_to.as_deref().unwrap_or("N/A"),
            record.last_tx_value.as_deref().unwrap_or("N/A"),
            record.last_tx_timestamp.as_deref().unwrap_or("N/A"),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

/// Export batch results to CSV
pub fn export_batch_to_csv(
    results: &[(String, Result<AddressInfo, String>)],
    output_path: &str,
) -> Result<(), CsvExportError> {
    if results.is_empty() {
        return Err(CsvExportError::NoData);
    }

    let file = File::create(output_path)?;
    let mut writer = Writer::from_writer(file);

    // Write header
    writer.write_record([
        "Input_Address",
        "Status",
        "Detected_Blockchain",
        "Balance",
        "Balance_USD",
        "Total_Transactions",
        "Error_Message",
    ])?;

    // Write data rows
    for (address, result) in results {
        match result {
            Ok(info) => {
                writer.write_record([
                    address,
                    "Success",
                    &info.blockchain,
                    &info.balance,
                    info.balance_usd.as_deref().unwrap_or("N/A"),
                    &info.total_transactions.to_string(),
                    "N/A",
                ])?;
            }
            Err(error) => {
                // Try to detect blockchain type even for failed queries
                let blockchain = crate::models::BlockchainType::from_address(address)
                    .map(|b| b.as_str().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                writer.write_record([
                    address,
                    "Failed",
                    &blockchain,
                    "N/A",
                    "N/A",
                    "N/A",
                    error,
                ])?;
            }
        }
    }

    writer.flush()?;
    Ok(())
}

/// Export the addresses matched in a text file to CSV
///
/// Unlike the other exports this one always writes the header, so an empty
/// report still produces a valid file.
pub fn export_matches_to_csv(
    report: &MatchReport,
    output_path: &Path,
) -> Result<(), CsvExportError> {
    let file = File::create(output_path)?;
    let mut writer = Writer::from_writer(file);

    // Write header
    writer.write_record([
        "Index",
        "Address",
        "Blockchain",
        "Address_Type",
        "Checksum",
        "Line",
        "Column",
        "Occurrences",
        "Balance",
        "Total_Transactions",
        "Query_Error",
        "Context",
    ])?;

    // Write data rows
    for (index, entry) in report.matches.iter().enumerate() {
        writer.write_record([
            &(index + 1).to_string(),
            &entry.address,
            entry.blockchain.as_str(),
            entry.kind.as_str(),
            entry.checksum.as_str(),
            &entry.line.to_string(),
            &entry.column.to_string(),
            &entry.occurrences.to_string(),
            entry.balance.as_deref().unwrap_or("N/A"),
            &entry
                .total_transactions
                .map(|count| count.to_string())
                .unwrap_or_else(|| "N/A".to_string()),
            entry.query_error.as_deref().unwrap_or("N/A"),
            &entry.context,
        ])?;
    }

    writer.flush()?;
    Ok(())
}

/// Export to CSV from string data (for CLI piping)
pub fn export_from_str(data: &str, output_path: &str) -> Result<(), CsvExportError> {
    let addresses: Vec<AddressInfo> = serde_json::from_str(data).map_err(|e| {
        CsvExportError::CsvError(csv::Error::from(io::Error::new(
            io::ErrorKind::InvalidData,
            e,
        )))
    })?;

    export_to_csv(&addresses, output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AddressInfo, BlockchainType, Transaction};

    fn create_test_address_info() -> AddressInfo {
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
        info
    }

    #[test]
    fn test_export_to_csv() {
        let info = create_test_address_info();
        let temp_path = crate::temp_file_path("test_export.csv");

        let result = export_to_csv(&[info], &temp_path);
        assert!(result.is_ok());

        // Read back and verify
        let content = std::fs::read_to_string(&temp_path).unwrap();
        assert!(content.contains("0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21"));
        assert!(content.contains("Ethereum"));
        assert!(content.contains("1.5"));
    }

    #[test]
    fn test_export_empty_data() {
        let result = export_to_csv(&[], &crate::temp_file_path("empty.csv"));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CsvExportError::NoData));
    }

    #[test]
    fn test_export_batch_results() {
        let info = create_test_address_info();
        let results = vec![
            (
                "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21".to_string(),
                Ok(info),
            ),
            (
                "invalid".to_string(),
                Err("Invalid address format".to_string()),
            ),
        ];

        let temp_path = crate::temp_file_path("test_batch_export.csv");
        let result = export_batch_to_csv(&results, &temp_path);
        assert!(result.is_ok());

        // Read back and verify
        let content = std::fs::read_to_string(&temp_path).unwrap();
        assert!(content.contains("Success"));
        assert!(content.contains("Failed"));
    }
}
