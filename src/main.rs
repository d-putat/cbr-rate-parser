use chrono::NaiveDate;
use scraper::{Html, Selector};
use std::error::Error;
use std::fs::File;
use thiserror::Error;

#[derive(Debug, Clone)]
struct ExchangeRate {
    date: NaiveDate,
    units: u32,
    rate: f64,
}

#[derive(Error, Debug)]
enum ParserError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Failed to parse HTML")]
    ParseError(String),
    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Date parse error: {0}")]
    DateError(#[from] chrono::ParseError),
}

/// Fetches USD exchange rates from CBR website for the given date range
fn fetch_cbr_rates(from_date: &str, to_date: &str) -> Result<String, ParserError> {
    let url = format!(
        "https://www.cbr.ru/currency_base/dynamics/?\
        UniDbQuery.Posted=True&\
        UniDbQuery.so=1&\
        UniDbQuery.mode=1&\
        UniDbQuery.date_req1=&\
        UniDbQuery.date_req2=&\
        UniDbQuery.VAL_NM_RQ=R01235&\
        UniDbQuery.From={}&\
        UniDbQuery.To={}",
        from_date, to_date
    );

    println!("Fetching data from CBR...");
    println!("URL: {}", url);

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    let response = client.get(&url).send()?;
    let html = response.text()?;

    Ok(html)
}

/// Parses HTML and extracts exchange rate data
fn parse_rates(html: &str) -> Result<Vec<ExchangeRate>, ParserError> {
    let document = Html::parse_document(html);

    // The table with rates has class "data"
    let table_selector = Selector::parse("table.data tbody tr")
        .map_err(|e| ParserError::ParseError(format!("Invalid selector: {:?}", e)))?;

    let td_selector = Selector::parse("td")
        .map_err(|e| ParserError::ParseError(format!("Invalid selector: {:?}", e)))?;

    let mut rates = Vec::new();

    for row in document.select(&table_selector) {
        let cells: Vec<String> = row
            .select(&td_selector)
            .map(|cell| cell.text().collect::<String>().trim().to_string())
            .collect();

        if cells.len() >= 3 {
            // Parse date (format: DD.MM.YYYY)
            let date = match NaiveDate::parse_from_str(&cells[0], "%d.%m.%Y") {
                Ok(d) => d,
                Err(_) => continue, // Skip rows with invalid date (e.g., header)
            };

            // Parse units (usually 1)
            let units: u32 = match cells[1].parse() {
                Ok(u) => u,
                Err(_) => continue,
            };

            // Parse rate (format: XX,XXXX with comma as decimal separator)
            let rate_str = cells[2].replace(',', ".").replace('\u{a0}', "");
            let rate: f64 = match rate_str.parse() {
                Ok(r) => r,
                Err(_) => continue,
            };

            rates.push(ExchangeRate { date, units, rate });
        }
    }

    if rates.is_empty() {
        return Err(ParserError::ParseError(
            "No exchange rates found in the response".to_string(),
        ));
    }

    // Sort by date ascending
    rates.sort_by(|a, b| a.date.cmp(&b.date));

    Ok(rates)
}

/// Writes exchange rates to a CSV file
fn write_csv(rates: &[ExchangeRate], filename: &str) -> Result<(), ParserError> {
    let file = File::create(filename)?;
    let mut writer = csv::Writer::from_writer(file);

    // Write header
    writer.write_record(["date", "units", "rate"])?;

    // Write data
    for rate in rates {
        writer.write_record([
            rate.date.format("%Y-%m-%d").to_string(),
            rate.units.to_string(),
            format!("{:.4}", rate.rate),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Date range for parsing (format: DD.MM.YYYY)
    let from_date = "01.01.2022";
    let to_date = "30.01.2024";

    println!("CBR USD Exchange Rate Parser");
    println!("============================");
    println!("Date range: {} - {}", from_date, to_date);
    println!();

    // Fetch HTML from CBR website
    let html = fetch_cbr_rates(from_date, to_date)?;

    // Parse exchange rates
    let rates = parse_rates(&html)?;

    println!("Parsed {} exchange rate records", rates.len());

    // Show first and last few records
    if !rates.is_empty() {
        println!();
        println!("First 3 records:");
        for rate in rates.iter().take(3) {
            println!(
                "  {} | {} unit(s) | {:.4} RUB",
                rate.date, rate.units, rate.rate
            );
        }

        println!();
        println!("Last 3 records:");
        for rate in rates.iter().rev().take(3).collect::<Vec<_>>().into_iter().rev() {
            println!(
                "  {} | {} unit(s) | {:.4} RUB",
                rate.date, rate.units, rate.rate
            );
        }
    }

    // Write to CSV
    let output_file = "usd_rates.csv";
    write_csv(&rates, output_file)?;
    println!();
    println!("Results saved to: {}", output_file);

    Ok(())
}
