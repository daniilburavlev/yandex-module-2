#![deny(unreachable_pub)]
#![warn(missing_docs)]
//! The `quotes` create

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Main data of the stock
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct StockQuote {
    /// The stock name
    pub ticker: String,
    /// Current price
    pub price: u64,
    /// Volume of stocks traded
    pub volume: u64,
    /// Timestamp, then stock change was published
    pub timestamp: u64,
}

impl StockQuote {
    /// Creates new `StockQuote` instance with current timestamp
    /// # Example:
    /// ```rust
    /// use quotes::StockQuote;
    /// let stock = StockQuote::new("AAPL", 180, 3000000);
    /// ```
    pub fn new(ticker: &str, price: u64, volume: u64) -> Self {
        Self {
            ticker: ticker.to_string(),
            price,
            volume,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    /// Update price, volume and current timestamp
    /// # Example
    /// ```rust
    /// use quotes::StockQuote;
    /// let mut stock = StockQuote::new("AAPL", 180, 3000000);
    /// stock.update(200, 3500000);
    /// ```
    pub fn update(&mut self, price: u64, volume: u64) {
        self.price = price;
        self.volume = volume;
        self.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
    }
}

/// Read TICKERS from file
///
/// # Example
/// ```rust
/// use quotes::parse_tickers;
/// let tickers = parse_tickers("AAPL\r\nNFLX\r\n");
/// ```
pub fn parse_tickers(data: &str) -> Vec<String> {
    let data = data.trim();
    data.lines()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}
