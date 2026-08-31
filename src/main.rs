//! Blockchain Explorer - Command-line tool for querying blockchain data
//!
//! Usage:
//!   blockchain-explorer query <ADDRESS> [--api-key YOUR_KEY]
//!   ETHERSCAN_API_KEY=xxx blockchain-explorer query <ADDRESS>
//!
//! Get your free API key at: https://etherscan.io/apidashboard

use blockchain_explorer::{
    address_match, cli::Cli, csv_export, read_addresses_from_file, AddressInfo, Explorer,
    MatchOptions, ReportFormat,
};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

/// Print address info in table format
fn print_address_table(info: &AddressInfo, include_txs: bool) {
    println!("\n{}", "=".repeat(80));
    println!("Address Information");
    println!("{}", "=".repeat(80));
    println!("{:<20} {}", "Address:", info.address);
    println!("{:<20} {}", "Blockchain:", info.blockchain);
    println!("{:<20} {}", "Balance:", info.balance);
    if let Some(usd) = &info.balance_usd {
        println!("{:<20} {}", "Balance (USD):", usd);
    }
    println!("{:<20} {}", "Total Transactions:", info.total_transactions);

    if include_txs && !info.transactions.is_empty() {
        println!("\n{}", "-".repeat(80));
        println!("Recent Transactions:");
        println!("{}", "-".repeat(80));

        for (i, tx) in info.transactions.iter().enumerate() {
            println!("\nTransaction #{}", i + 1);
            println!("  Hash:        {}", tx.hash);
            println!("  From:        {}", tx.from);
            println!("  To:          {}", tx.to);
            println!("  Value:       {}", tx.value);
            if let Some(timestamp) = tx.timestamp {
                println!("  Timestamp:   {}", timestamp);
            }
            if let Some(block) = tx.block_number {
                println!("  Block:       {}", block);
            }
            println!("  Status:      {}", tx.status);
        }
    }
    println!("{}", "=".repeat(80));
}

/// Print address info as JSON
fn print_address_json(info: &AddressInfo) {
    let json = serde_json::to_string_pretty(info).unwrap();
    println!("{}", json);
}

/// Print address info as CSV row
fn print_address_csv(info: &AddressInfo) {
    println!(
        "{},{},{},{},{}",
        info.address,
        info.blockchain,
        info.balance,
        info.balance_usd.as_deref().unwrap_or("N/A"),
        info.total_transactions
    );
}

/// Create explorer with optional API key
fn create_explorer(api_key: Option<&str>) -> Explorer {
    match api_key {
        Some(key) => {
            // Mask API key for logging (show first 4 and last 4 characters)
            let masked_key = if key.len() > 8 {
                format!("{}...{}", &key[..4], &key[key.len()-4..])
            } else {
                "***".to_string()
            };
            info!("Using Etherscan API Key: {}", masked_key);
            Explorer::with_etherscan_api_key(key)
        }
        None => {
            info!("No API key provided, using default (rate limited)");
            Explorer::new()
        }
    }
}

/// Handle query command
async fn handle_query(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let blockchain_explorer::cli::Commands::Query(args) = &cli.command else {
        unreachable!();
    };

    let explorer = create_explorer(cli.etherscan_api_key.as_deref());

    info!("Querying address: {}", args.address);

    let blockchain = args.blockchain.to_blockchain_type();

    let result = explorer.query(&args.address, blockchain).await;

    match result {
        Ok(info) => {
            match args.format {
                blockchain_explorer::cli::OutputFormat::Json => print_address_json(&info),
                blockchain_explorer::cli::OutputFormat::Csv => print_address_csv(&info),
                blockchain_explorer::cli::OutputFormat::Table => {
                    print_address_table(&info, args.include_txs)
                }
            }

            // Export to file if specified
            if let Some(output_path) = &args.output {
                match args.export_format {
                    blockchain_explorer::cli::ExportFormatArg::Json => {
                        let json = serde_json::to_string_pretty(&info)?;
                        std::fs::write(output_path, json)?;
                    }
                    blockchain_explorer::cli::ExportFormatArg::Csv => {
                        csv_export::export_to_csv(&[info], output_path.to_str().unwrap())?;
                    }
                }
                info!("Results exported to: {}", output_path.display());
            }
        }
        Err(e) => {
            error!("Failed to query address: {}", e);
            return Err(Box::new(e));
        }
    }

    Ok(())
}

/// Handle batch command
async fn handle_batch(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let blockchain_explorer::cli::Commands::Batch(args) = &cli.command else {
        unreachable!();
    };

    let explorer = create_explorer(cli.etherscan_api_key.as_deref());

    info!("Reading addresses from: {}", args.file.display());
    let addresses = read_addresses_from_file(args.file.to_str().unwrap())?;

    if addresses.is_empty() {
        error!("No addresses found in file");
        return Err("No addresses found".into());
    }

    println!("Found {} addresses to query", addresses.len());
    println!("Parallel queries: {}", args.parallel);

    let blockchain = args.blockchain.as_ref().and_then(|b| b.to_blockchain_type());

    let result = explorer.batch_query(addresses.clone(), blockchain, args.parallel).await;

    println!("\nBatch Query Results:");
    println!("  Total:     {}", result.total);
    println!("  Success:   {}", result.successful);
    println!("  Failed:    {}", result.failed);

    // Export to CSV
    let results: Vec<(String, Result<AddressInfo, String>)> = result
        .results
        .into_iter()
        .zip(addresses.iter())
        .map(|(r, addr)| (addr.clone(), r))
        .collect();

    csv_export::export_batch_to_csv(&results, args.output.to_str().unwrap())?;
    info!("Results exported to: {}", args.output.display());

    Ok(())
}

/// Handle match command
async fn handle_match(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let blockchain_explorer::cli::Commands::Match(args) = &cli.command else {
        unreachable!();
    };

    info!("Scanning file: {}", args.file.display());

    let options = MatchOptions {
        blockchain: args
            .blockchain
            .as_ref()
            .and_then(|b| b.to_blockchain_type()),
        verify_checksum: !args.no_checksum,
        unique: !args.all_occurrences,
        ..MatchOptions::default()
    };

    let mut report = address_match::match_file(&args.file, &options)?;

    println!("\nAddress Match Results:");
    println!("  {:<20} {}", "Scanned lines:", report.scanned_lines);
    println!("  {:<20} {}", "Occurrences:", report.total_occurrences);
    println!("  {:<20} {}", "Unique addresses:", report.unique_addresses);
    println!("  {:<20} {}", "  Ethereum:", report.ethereum_addresses);
    println!("  {:<20} {}", "  Bitcoin:", report.bitcoin_addresses);

    if report.matches.is_empty() {
        println!("\nNo BTC/ETH addresses matched.");
    } else if args.query {
        let explorer = create_explorer(cli.etherscan_api_key.as_deref());
        let parallel = args.parallel.max(1);
        let addresses = report.addresses();

        println!(
            "\nQuerying {} addresses (parallel: {})...",
            addresses.len(),
            parallel
        );

        let batch = explorer.batch_query(addresses, None, parallel).await;

        println!("  Success:   {}", batch.successful);
        println!("  Failed:    {}", batch.failed);

        report.apply_query_results(batch);
    }

    // An explicit --format wins, otherwise the output extension decides
    let format = match args.format {
        Some(blockchain_explorer::cli::OutputFormat::Csv) => ReportFormat::Csv,
        Some(blockchain_explorer::cli::OutputFormat::Json) => ReportFormat::Json,
        Some(blockchain_explorer::cli::OutputFormat::Table) => ReportFormat::Text,
        None => ReportFormat::from_path(&args.output),
    };

    address_match::write_report(&report, &args.output, format)?;
    info!("Results written to: {}", args.output.display());

    Ok(())
}

/// Handle export command
fn handle_export(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let blockchain_explorer::cli::Commands::Export(args) = &cli.command else {
        unreachable!();
    };

    info!("Reading from: {}", args.input.display());

    let content = std::fs::read_to_string(&args.input)?;
    let addresses: Vec<AddressInfo> = serde_json::from_str(&content)?;

    csv_export::export_to_csv(&addresses, args.output.to_str().unwrap())?;
    info!("Exported {} addresses to: {}", addresses.len(), args.output.display());

    Ok(())
}

/// Handle compare command
async fn handle_compare(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let blockchain_explorer::cli::Commands::Compare(args) = &cli.command else {
        unreachable!();
    };

    let explorer = create_explorer(cli.etherscan_api_key.as_deref());

    println!("\n{}", "=".repeat(80));
    println!("Address Comparison");
    println!("{}", "=".repeat(80));

    let mut results: Vec<AddressInfo> = Vec::new();

    for address in &args.addresses {
        match explorer.query(address, None).await {
            Ok(info) => {
                results.push(info);
            }
            Err(e) => {
                error!("Failed to query {}: {}", address, e);
            }
        }
    }

    // Print comparison table
    println!("\n{:<42} {:<12} {:<15} {:<10}",
        "Address", "Blockchain", "Balance", "Tx Count");
    println!("{}", "-".repeat(80));

    for info in &results {
        println!(
            "{:<42} {:<12} {:<15} {:<10}",
            &info.address[..std::cmp::min(42, info.address.len())],
            info.blockchain,
            info.balance,
            info.total_transactions
        );
    }

    // Export if specified
    if let Some(output_path) = &args.output {
        csv_export::export_to_csv(&results, output_path.to_str().unwrap())?;
        info!("Results exported to: {}", output_path.display());
    }

    Ok(())
}

/// Handle detect command
fn handle_detect(address: &str) {
    let explorer = Explorer::new();

    match explorer.detect_blockchain(address) {
        Some(blockchain) => {
            println!("Address: {}", address);
            println!("Blockchain: {}", blockchain.as_str());
        }
        None => {
            println!("Address: {}", address);
            println!("Blockchain: Unknown (not a valid ETH or BTC address)");
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse_args();

    // Initialize logging
    let log_level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let _subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    // Execute command
    let result = match &cli.command {
        blockchain_explorer::cli::Commands::Query(_) => handle_query(&cli).await,
        blockchain_explorer::cli::Commands::Batch(_) => handle_batch(&cli).await,
        blockchain_explorer::cli::Commands::Match(_) => handle_match(&cli).await,
        blockchain_explorer::cli::Commands::Export(_) => handle_export(&cli),
        blockchain_explorer::cli::Commands::Compare(_) => handle_compare(&cli).await,
        blockchain_explorer::cli::Commands::Detect { address } => {
            handle_detect(address);
            Ok(())
        }
    };

    if let Err(e) = result {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}
