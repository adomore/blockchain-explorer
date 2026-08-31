//! Match Bitcoin and Ethereum addresses inside free-form text files
//!
//! Unlike [`crate::read_addresses_from_file`], which expects one address per
//! line, this module scans arbitrary text (logs, chat exports, notes, HTML,
//! ...) and extracts every address it can find, together with where it was
//! found. The result is a [`MatchReport`] that can be written to a file as a
//! text table, CSV or JSON.
//!
//! Matching rules:
//! - **Ethereum**: `0x` followed by exactly 40 hex characters. The EIP-55
//!   mixed case checksum is *not* verified.
//! - **Bitcoin**: `1...`/`3...` verified with Base58Check, `bc1...` verified
//!   with Bech32/Bech32m (BIP-173 / BIP-350). With
//!   [`MatchOptions::verify_checksum`] disabled only the prefix, alphabet and
//!   length are checked.
//!
//! A candidate is only considered when it is delimited by non-alphanumeric
//! characters, so addresses glued to surrounding words are not reported.

use crate::checksum;
use crate::csv_export;
use crate::models::{AddressInfo, BatchResult, BlockchainType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use thiserror::Error;

/// Errors that can occur while matching addresses or writing a report
#[derive(Error, Debug)]
pub enum MatchError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("CSV error: {0}")]
    CsvError(#[from] csv_export::CsvExportError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Concrete address format of a match
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressKind {
    /// Ethereum account address (`0x` + 40 hex characters)
    EthereumHex,
    /// Legacy pay-to-public-key-hash (`1...`)
    P2pkh,
    /// Pay-to-script-hash (`3...`)
    P2sh,
    /// SegWit v0 pay-to-witness-public-key-hash (`bc1q...`, 20 byte program)
    P2wpkh,
    /// SegWit v0 pay-to-witness-script-hash (`bc1q...`, 32 byte program)
    P2wsh,
    /// Taproot (`bc1p...`)
    P2tr,
    /// SegWit program with an unassigned witness version
    SegwitOther,
}

impl AddressKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            AddressKind::EthereumHex => "ETH",
            AddressKind::P2pkh => "P2PKH",
            AddressKind::P2sh => "P2SH",
            AddressKind::P2wpkh => "P2WPKH",
            AddressKind::P2wsh => "P2WSH",
            AddressKind::P2tr => "P2TR",
            AddressKind::SegwitOther => "SegWit",
        }
    }
}

/// How much confidence the checksum check gives for a match
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChecksumStatus {
    /// Base58Check or Bech32/Bech32m checksum verified
    Verified,
    /// Checksum verification was turned off, the format alone matched
    Skipped,
    /// The format carries no checksum this tool verifies (Ethereum)
    NotApplicable,
}

impl ChecksumStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChecksumStatus::Verified => "Verified",
            ChecksumStatus::Skipped => "Skipped",
            ChecksumStatus::NotApplicable => "N/A",
        }
    }
}

/// A single address found in the scanned text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressMatch {
    /// The address exactly as it appears in the source text
    pub address: String,
    pub blockchain: BlockchainType,
    pub kind: AddressKind,
    pub checksum: ChecksumStatus,
    /// 1-based line where the address was first seen
    pub line: usize,
    /// 1-based character column where the address was first seen
    pub column: usize,
    /// Number of times the address occurs in the scanned text
    pub occurrences: usize,
    /// Trimmed (and possibly truncated) source line of the first occurrence
    pub context: String,
    /// Balance, filled in when the addresses are queried
    pub balance: Option<String>,
    /// Transaction count, filled in when the addresses are queried
    pub total_transactions: Option<u64>,
    /// Why the query for this address failed, if it did
    pub query_error: Option<String>,
}

impl AddressMatch {
    /// The address in the shape the block explorer APIs expect
    ///
    /// Base58 is case sensitive and must be passed through unchanged, while
    /// hex and Bech32 addresses are normalized to lower case.
    pub fn query_form(&self) -> String {
        match self.kind {
            AddressKind::P2pkh | AddressKind::P2sh => self.address.clone(),
            _ => self.address.to_ascii_lowercase(),
        }
    }
}

/// Options controlling how a text file is matched
#[derive(Debug, Clone)]
pub struct MatchOptions {
    /// Restrict matching to one chain; `None` matches both
    pub blockchain: Option<BlockchainType>,
    /// Verify Bitcoin checksums instead of matching the format only
    pub verify_checksum: bool,
    /// Collapse repeated addresses into one entry with an occurrence count
    pub unique: bool,
    /// Maximum number of characters of the source line kept as context
    pub context_len: usize,
}

impl Default for MatchOptions {
    fn default() -> Self {
        Self {
            blockchain: None,
            verify_checksum: true,
            unique: true,
            context_len: 120,
        }
    }
}

/// The result of matching one text file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReport {
    /// Where the text came from (file path)
    pub source: String,
    pub scanned_lines: usize,
    /// Number of address occurrences found, duplicates included
    pub total_occurrences: usize,
    pub unique_addresses: usize,
    pub ethereum_addresses: usize,
    pub bitcoin_addresses: usize,
    /// Whether Bitcoin checksums were verified during the scan
    pub checksum_verified: bool,
    /// Whether balances were queried for the matched addresses
    pub queried: bool,
    pub matches: Vec<AddressMatch>,
}

impl MatchReport {
    /// Distinct addresses of all matches, ready to be handed to the explorer
    ///
    /// Duplicates are dropped so that listing every occurrence does not query
    /// the same address several times.
    pub fn addresses(&self) -> Vec<String> {
        let mut seen = HashSet::new();
        self.matches
            .iter()
            .map(|entry| entry.query_form())
            .filter(|address| seen.insert(address.clone()))
            .collect()
    }

    /// Merge the outcome of a batch query back into the matches
    ///
    /// The results are expected in the order produced by [`Self::addresses`].
    pub fn apply_query_results(&mut self, batch: BatchResult) {
        let outcomes: HashMap<String, Result<AddressInfo, String>> =
            self.addresses().into_iter().zip(batch.results).collect();

        for entry in self.matches.iter_mut() {
            match outcomes.get(&entry.query_form()) {
                Some(Ok(info)) => {
                    entry.balance = Some(info.balance.clone());
                    entry.total_transactions = Some(info.total_transactions);
                    entry.query_error = None;
                }
                Some(Err(error)) => {
                    entry.balance = None;
                    entry.total_transactions = None;
                    entry.query_error = Some(error.clone());
                }
                None => {}
            }
        }
        self.queried = true;
    }
}

/// Read a text file and match every Bitcoin/Ethereum address in it
///
/// The file is decoded lossily so that logs and other files which are not
/// valid UTF-8 can be scanned as well.
pub fn match_file(path: &Path, options: &MatchOptions) -> Result<MatchReport, MatchError> {
    let bytes = std::fs::read(path)?;
    let text = String::from_utf8_lossy(&bytes);
    Ok(scan_text(&path.display().to_string(), &text, options))
}

/// Match every Bitcoin/Ethereum address in `text`
pub fn scan_text(source: &str, text: &str, options: &MatchOptions) -> MatchReport {
    let mut matches: Vec<AddressMatch> = Vec::new();
    // Maps the deduplication key to the match it belongs to
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut scanned_lines = 0usize;
    let mut total_occurrences = 0usize;

    for (line_index, line) in text.lines().enumerate() {
        scanned_lines += 1;

        for (column, token) in tokenize(line) {
            let candidate = match classify(token, options) {
                Some(candidate) => candidate,
                None => continue,
            };
            total_occurrences += 1;

            let key = dedup_key(token, candidate.kind);
            if options.unique {
                if let Some(position) = seen.get(&key) {
                    matches[*position].occurrences += 1;
                    continue;
                }
            }

            seen.insert(key, matches.len());
            matches.push(AddressMatch {
                address: token.to_string(),
                blockchain: candidate.blockchain,
                kind: candidate.kind,
                checksum: candidate.checksum,
                line: line_index + 1,
                column,
                occurrences: 1,
                context: truncate(line, options.context_len),
                balance: None,
                total_transactions: None,
                query_error: None,
            });
        }
    }

    let ethereum_addresses = seen
        .values()
        .filter(|position| matches[**position].blockchain == BlockchainType::Ethereum)
        .count();

    MatchReport {
        source: source.to_string(),
        scanned_lines,
        total_occurrences,
        unique_addresses: seen.len(),
        ethereum_addresses,
        bitcoin_addresses: seen.len() - ethereum_addresses,
        checksum_verified: options.verify_checksum,
        queried: false,
        matches,
    }
}

/// A token that was recognized as an address
struct Candidate {
    blockchain: BlockchainType,
    kind: AddressKind,
    checksum: ChecksumStatus,
}

/// Split a line into maximal runs of ASCII alphanumeric characters
///
/// Addresses only ever consist of ASCII letters and digits, so anything else
/// (quotes, commas, slashes, CJK text, ...) acts as a delimiter. The returned
/// columns are 1-based character positions.
fn tokenize(line: &str) -> Vec<(usize, &str)> {
    let mut tokens = Vec::new();
    // Start of the current run as (byte offset, 1-based column)
    let mut start: Option<(usize, usize)> = None;

    for (column, (offset, ch)) in line.char_indices().enumerate() {
        if ch.is_ascii_alphanumeric() {
            if start.is_none() {
                start = Some((offset, column + 1));
            }
        } else if let Some((begin, first_column)) = start.take() {
            tokens.push((first_column, &line[begin..offset]));
        }
    }

    if let Some((begin, first_column)) = start {
        tokens.push((first_column, &line[begin..]));
    }

    tokens
}

/// Decide whether a token is an address the caller asked for
fn classify(token: &str, options: &MatchOptions) -> Option<Candidate> {
    let candidate =
        classify_ethereum(token).or_else(|| classify_bitcoin(token, options.verify_checksum))?;

    match options.blockchain {
        Some(wanted) if wanted != candidate.blockchain => None,
        _ => Some(candidate),
    }
}

/// `0x` followed by exactly 40 hex characters
fn classify_ethereum(token: &str) -> Option<Candidate> {
    if token.len() != 42 || !token.starts_with("0x") {
        return None;
    }
    if !token[2..].bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }

    Some(Candidate {
        blockchain: BlockchainType::Ethereum,
        kind: AddressKind::EthereumHex,
        checksum: ChecksumStatus::NotApplicable,
    })
}

/// Legacy Base58Check (`1...`, `3...`) or SegWit Bech32 (`bc1...`) addresses
fn classify_bitcoin(token: &str, verify: bool) -> Option<Candidate> {
    let first = *token.as_bytes().first()?;

    if first == b'1' || first == b'3' {
        // 25 payload bytes encode to 26-35 Base58 characters
        if !(26..=35).contains(&token.len()) || !checksum::is_base58(token) {
            return None;
        }

        let kind = if verify {
            match checksum::base58check_verify(token)? {
                checksum::P2PKH_VERSION => AddressKind::P2pkh,
                checksum::P2SH_VERSION => AddressKind::P2sh,
                _ => return None,
            }
        } else if first == b'1' {
            AddressKind::P2pkh
        } else {
            AddressKind::P2sh
        };

        return Some(Candidate {
            blockchain: BlockchainType::Bitcoin,
            kind,
            checksum: status(verify),
        });
    }

    let lowercase = token.to_ascii_lowercase();
    if !lowercase.starts_with("bc1") {
        return None;
    }

    let kind = if verify {
        let decoded = checksum::bech32_verify(token)?;
        if decoded.hrp != "bc" {
            return None;
        }
        match (decoded.witness_version, decoded.program_len) {
            (0, 20) => AddressKind::P2wpkh,
            (0, 32) => AddressKind::P2wsh,
            (1, 32) => AddressKind::P2tr,
            _ => AddressKind::SegwitOther,
        }
    } else {
        // Shortest useful SegWit address is 14 characters, longest is 74
        if !(14..=74).contains(&token.len()) {
            return None;
        }
        if !lowercase[3..]
            .bytes()
            .all(|b| checksum::BECH32_ALPHABET.contains(&b))
        {
            return None;
        }
        match (lowercase.as_bytes()[3], token.len()) {
            (b'q', 42) => AddressKind::P2wpkh,
            (b'q', 62) => AddressKind::P2wsh,
            (b'p', 62) => AddressKind::P2tr,
            _ => AddressKind::SegwitOther,
        }
    };

    Some(Candidate {
        blockchain: BlockchainType::Bitcoin,
        kind,
        checksum: status(verify),
    })
}

fn status(verified: bool) -> ChecksumStatus {
    if verified {
        ChecksumStatus::Verified
    } else {
        ChecksumStatus::Skipped
    }
}

/// Key used to recognize an address that was already matched
///
/// Base58 is case sensitive, hex and Bech32 are not.
fn dedup_key(address: &str, kind: AddressKind) -> String {
    match kind {
        AddressKind::P2pkh | AddressKind::P2sh => address.to_string(),
        _ => address.to_ascii_lowercase(),
    }
}

/// Trim a source line and cut it to at most `max` characters
fn truncate(text: &str, max: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max {
        return trimmed.to_string();
    }

    let kept: String = trimmed.chars().take(max).collect();
    format!("{}...", kept)
}

/// File format of a written report
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// Human readable table
    Text,
    /// One row per match
    Csv,
    /// The whole [`MatchReport`] as JSON
    Json,
}

impl ReportFormat {
    /// Derive the format from the output file extension, text by default
    pub fn from_path(path: &Path) -> Self {
        match path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .as_deref()
        {
            Some("csv") => ReportFormat::Csv,
            Some("json") => ReportFormat::Json,
            _ => ReportFormat::Text,
        }
    }
}

/// Write a match report to `path` in the requested format
pub fn write_report(
    report: &MatchReport,
    path: &Path,
    format: ReportFormat,
) -> Result<(), MatchError> {
    match format {
        ReportFormat::Text => std::fs::write(path, render_text_report(report))?,
        ReportFormat::Json => std::fs::write(path, serde_json::to_string_pretty(report)?)?,
        ReportFormat::Csv => csv_export::export_matches_to_csv(report, path)?,
    }

    Ok(())
}

/// Render a match report as a human readable table
pub fn render_text_report(report: &MatchReport) -> String {
    let address_width = report
        .matches
        .iter()
        .map(|entry| entry.address.chars().count())
        .max()
        .unwrap_or(7)
        .max(7);

    // Build the table first, its widest line decides the separator width
    let mut table: Vec<String> = Vec::with_capacity(report.matches.len() + 1);
    let mut header = format!(
        "{:<4} {:<8} {:<7} {:<9} {:>6} {:>5} {:>5} {:<width$}",
        "#",
        "Chain",
        "Type",
        "Checksum",
        "Line",
        "Col",
        "Hits",
        "Address",
        width = address_width
    );
    if report.queried {
        header.push_str(&format!(" {:>18} {:>6}", "Balance", "Txs"));
    }
    table.push(header.trim_end().to_string());

    for (index, entry) in report.matches.iter().enumerate() {
        let mut row = format!(
            "{:<4} {:<8} {:<7} {:<9} {:>6} {:>5} {:>5} {:<width$}",
            index + 1,
            entry.blockchain.as_str(),
            entry.kind.as_str(),
            entry.checksum.as_str(),
            entry.line,
            entry.column,
            entry.occurrences,
            entry.address,
            width = address_width
        );
        if report.queried {
            let balance = match (&entry.balance, &entry.query_error) {
                (Some(balance), _) => balance.clone(),
                (None, Some(_)) => "FAILED".to_string(),
                (None, None) => "-".to_string(),
            };
            let transactions = entry
                .total_transactions
                .map(|count| count.to_string())
                .unwrap_or_else(|| "-".to_string());
            row.push_str(&format!(" {:>18} {:>6}", balance, transactions));
        }
        table.push(row.trim_end().to_string());
    }

    let width = table
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(80)
        .max(80);
    let double_rule = "=".repeat(width);
    let single_rule = "-".repeat(width);

    let mut out = String::new();
    out.push_str(&format!("{}\n", double_rule));
    out.push_str("BTC / ETH Address Match Report\n");
    out.push_str(&format!("{}\n", double_rule));
    out.push_str(&format!("{:<22} {}\n", "Source File:", report.source));
    out.push_str(&format!(
        "{:<22} {}\n",
        "Scanned Lines:", report.scanned_lines
    ));
    out.push_str(&format!(
        "{:<22} {}\n",
        "Matched Occurrences:", report.total_occurrences
    ));
    out.push_str(&format!(
        "{:<22} {}\n",
        "Unique Addresses:", report.unique_addresses
    ));
    out.push_str(&format!(
        "{:<22} {}\n",
        "  Ethereum:", report.ethereum_addresses
    ));
    out.push_str(&format!(
        "{:<22} {}\n",
        "  Bitcoin:", report.bitcoin_addresses
    ));
    out.push_str(&format!(
        "{:<22} {}\n",
        "Checksum Verified:",
        if report.checksum_verified {
            "yes"
        } else {
            "no"
        }
    ));
    out.push_str(&format!(
        "{:<22} {}\n",
        "Balance Queried:",
        if report.queried { "yes" } else { "no" }
    ));
    out.push_str(&format!("{}\n", single_rule));

    if report.matches.is_empty() {
        out.push_str("No BTC/ETH addresses matched.\n");
    } else {
        for (index, line) in table.iter().enumerate() {
            out.push_str(line);
            out.push('\n');
            if index == 0 {
                out.push_str(&format!("{}\n", single_rule));
            }
        }
    }

    let failures: Vec<&AddressMatch> = report
        .matches
        .iter()
        .filter(|entry| entry.query_error.is_some())
        .collect();
    if !failures.is_empty() {
        out.push_str(&format!("{}\n", single_rule));
        out.push_str("Query Errors:\n");
        for entry in failures {
            out.push_str(&format!(
                "  {}  {}\n",
                entry.address,
                entry.query_error.as_deref().unwrap_or("")
            ));
        }
    }

    out.push_str(&format!("{}\n", single_rule));
    out.push_str("Notes:\n");
    out.push_str("  - Ethereum addresses match on format (0x + 40 hex characters); the\n");
    out.push_str("    EIP-55 upper/lower case checksum is not verified.\n");
    out.push_str("  - Bitcoin addresses are verified with Base58Check (1.../3...) or\n");
    out.push_str("    Bech32/Bech32m (bc1...) unless checksum verification was disabled.\n");
    out.push_str(&format!("{}\n", double_rule));

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const ETH: &str = "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21";
    const P2PKH: &str = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
    const P2SH: &str = "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy";
    const BECH32: &str = "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq";
    const TAPROOT: &str = "bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqzk5jj0";

    fn scan(text: &str) -> MatchReport {
        scan_text("test", text, &MatchOptions::default())
    }

    #[test]
    fn test_match_addresses_in_prose() {
        let text = format!(
            "Payment sent to {} on Monday.\n\
             Refund address is {} (legacy), see also {}.\n",
            ETH, P2PKH, BECH32
        );

        let report = scan(&text);
        assert_eq!(report.unique_addresses, 3);
        assert_eq!(report.ethereum_addresses, 1);
        assert_eq!(report.bitcoin_addresses, 2);
        assert_eq!(report.scanned_lines, 2);
        assert_eq!(report.matches[0].address, ETH);
        assert_eq!(report.matches[0].line, 1);
        assert_eq!(report.matches[0].column, 17);
    }

    #[test]
    fn test_match_all_address_kinds() {
        let text = format!("{}\n{}\n{}\n{}\n{}\n", ETH, P2PKH, P2SH, BECH32, TAPROOT);
        let report = scan(&text);

        let kinds: Vec<AddressKind> = report.matches.iter().map(|m| m.kind).collect();
        assert_eq!(
            kinds,
            vec![
                AddressKind::EthereumHex,
                AddressKind::P2pkh,
                AddressKind::P2sh,
                AddressKind::P2wpkh,
                AddressKind::P2tr,
            ]
        );
    }

    #[test]
    fn test_addresses_are_extracted_from_delimiters() {
        let text = format!(
            "{{\"from\":\"{}\",\"url\":\"https://blockstream.info/address/{}\"}}",
            ETH, BECH32
        );
        let report = scan(&text);
        assert_eq!(report.unique_addresses, 2);
    }

    #[test]
    fn test_duplicates_are_counted_once() {
        let text = format!("{}\n{}\nand again {}\n", ETH, ETH, ETH);
        let report = scan(&text);

        assert_eq!(report.unique_addresses, 1);
        assert_eq!(report.total_occurrences, 3);
        assert_eq!(report.matches.len(), 1);
        assert_eq!(report.matches[0].occurrences, 3);
        assert_eq!(report.matches[0].line, 1);
    }

    #[test]
    fn test_all_occurrences_mode() {
        let text = format!("{}\n{}\n", ETH, ETH);
        let options = MatchOptions {
            unique: false,
            ..MatchOptions::default()
        };
        let report = scan_text("test", &text, &options);

        assert_eq!(report.matches.len(), 2);
        assert_eq!(report.unique_addresses, 1);
        assert_eq!(report.matches[1].line, 2);
    }

    #[test]
    fn test_checksum_rejects_corrupted_addresses() {
        // Last character changed in each Bitcoin address
        let text = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNb \
                    bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdp";
        assert_eq!(scan(text).unique_addresses, 0);

        let lenient = MatchOptions {
            verify_checksum: false,
            ..MatchOptions::default()
        };
        let report = scan_text("test", text, &lenient);
        assert_eq!(report.unique_addresses, 2);
        assert!(report
            .matches
            .iter()
            .all(|m| m.checksum == ChecksumStatus::Skipped));
    }

    #[test]
    fn test_ignores_non_addresses() {
        let text = "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21ff \
                    0x742d35Cc6634C0532925a3b844Bc9e7595f8fE2 \
                    0xzzzd35Cc6634C0532925a3b844Bc9e7595f8fE21 \
                    1234567890 \
                    transaction 0x88df016429689c079f3b2f6ad39fa052532c56795b733da78a91ebe6a713944b";
        assert_eq!(scan(text).unique_addresses, 0);
    }

    #[test]
    fn test_address_glued_to_text_is_not_matched() {
        let text = format!("prefix{} {}suffix", ETH, P2PKH);
        assert_eq!(scan(&text).unique_addresses, 0);
    }

    #[test]
    fn test_blockchain_filter() {
        let text = format!("{} {} {}", ETH, P2PKH, BECH32);

        let ethereum_only = MatchOptions {
            blockchain: Some(BlockchainType::Ethereum),
            ..MatchOptions::default()
        };
        let report = scan_text("test", &text, &ethereum_only);
        assert_eq!(report.unique_addresses, 1);
        assert_eq!(report.ethereum_addresses, 1);

        let bitcoin_only = MatchOptions {
            blockchain: Some(BlockchainType::Bitcoin),
            ..MatchOptions::default()
        };
        let report = scan_text("test", &text, &bitcoin_only);
        assert_eq!(report.unique_addresses, 2);
        assert_eq!(report.bitcoin_addresses, 2);
    }

    #[test]
    fn test_case_handling() {
        // Hex and Bech32 are case insensitive, Base58 is not
        let text = format!("{} {} {}", ETH, ETH.to_lowercase(), BECH32.to_uppercase());
        let report = scan(&text);

        assert_eq!(report.unique_addresses, 2);
        assert_eq!(report.matches[0].occurrences, 2);
        assert_eq!(report.matches[1].address, BECH32.to_uppercase());
        assert_eq!(report.matches[1].query_form(), BECH32);
    }

    #[test]
    fn test_context_and_position_on_non_ascii_line() {
        let text = format!("收款地址: {} (备注)", ETH);
        let report = scan(&text);

        assert_eq!(report.matches[0].column, 7);
        assert!(report.matches[0].context.contains("收款地址"));
    }

    #[test]
    fn test_context_is_truncated() {
        let options = MatchOptions {
            context_len: 20,
            ..MatchOptions::default()
        };
        let text = format!("{} padded with a very long trailing comment", ETH);
        let report = scan_text("test", &text, &options);

        assert_eq!(report.matches[0].context.chars().count(), 23);
        assert!(report.matches[0].context.ends_with("..."));
    }

    #[test]
    fn test_report_format_from_path() {
        assert_eq!(
            ReportFormat::from_path(Path::new("out.csv")),
            ReportFormat::Csv
        );
        assert_eq!(
            ReportFormat::from_path(Path::new("out.JSON")),
            ReportFormat::Json
        );
        assert_eq!(
            ReportFormat::from_path(Path::new("out.txt")),
            ReportFormat::Text
        );
        assert_eq!(
            ReportFormat::from_path(Path::new("out")),
            ReportFormat::Text
        );
    }

    #[test]
    fn test_render_text_report_contains_matches() {
        let report = scan(&format!("{} {}", ETH, P2PKH));
        let rendered = render_text_report(&report);

        assert!(rendered.contains("BTC / ETH Address Match Report"));
        assert!(rendered.contains(ETH));
        assert!(rendered.contains(P2PKH));
        assert!(rendered.contains("P2PKH"));
    }

    #[test]
    fn test_render_text_report_without_matches() {
        let report = scan("nothing to see here");
        let rendered = render_text_report(&report);

        assert!(rendered.contains("No BTC/ETH addresses matched."));
    }

    #[test]
    fn test_apply_query_results() {
        let mut report = scan(&format!("{} {}", ETH, P2PKH));

        let mut batch = BatchResult::new();
        let mut info = crate::models::AddressInfo::new(ETH.to_string(), BlockchainType::Ethereum);
        info.balance = "1.5".to_string();
        info.total_transactions = 10;
        batch.add_result(Ok(info));
        batch.add_result(Err("Network error".to_string()));

        report.apply_query_results(batch);

        assert!(report.queried);
        assert_eq!(report.matches[0].balance.as_deref(), Some("1.5"));
        assert_eq!(report.matches[0].total_transactions, Some(10));
        assert_eq!(
            report.matches[1].query_error.as_deref(),
            Some("Network error")
        );

        let rendered = render_text_report(&report);
        assert!(rendered.contains("Query Errors:"));
        assert!(rendered.contains("FAILED"));
    }

    #[test]
    fn test_repeated_occurrences_share_one_query() {
        let options = MatchOptions {
            unique: false,
            ..MatchOptions::default()
        };
        let mut report = scan_text("test", &format!("{}\n{}\n", ETH, ETH), &options);

        assert_eq!(report.matches.len(), 2);
        // The same address must not be queried twice
        assert_eq!(report.addresses().len(), 1);

        let mut batch = BatchResult::new();
        let mut info = crate::models::AddressInfo::new(ETH.to_string(), BlockchainType::Ethereum);
        info.balance = "2.25".to_string();
        batch.add_result(Ok(info));

        report.apply_query_results(batch);

        assert_eq!(report.matches[0].balance.as_deref(), Some("2.25"));
        assert_eq!(report.matches[1].balance.as_deref(), Some("2.25"));
    }

    #[test]
    fn test_match_file_reads_invalid_utf8() {
        let mut path = std::env::temp_dir();
        path.push("blockchain_explorer_match_invalid_utf8.log");

        let mut bytes = b"binary \xff\xfe log ".to_vec();
        bytes.extend_from_slice(ETH.as_bytes());
        std::fs::write(&path, bytes).unwrap();

        let report = match_file(&path, &MatchOptions::default()).unwrap();
        assert_eq!(report.unique_addresses, 1);
        assert_eq!(report.matches[0].address, ETH);

        std::fs::remove_file(&path).ok();
    }
}
