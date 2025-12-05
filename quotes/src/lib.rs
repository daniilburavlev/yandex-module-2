#![deny(unreachable_pub)]
#![warn(missing_docs)]
//! The `quotes` create

mod error;
mod serialization;

pub use error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

/// Main data of the stock
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StockQuote {
    /// The stock name
    pub ticker: String,
    /// Current price
    pub price: u64,
    /// Volume of stocks traded
    pub volume: u32,
    /// Timestamp, then stock change was published
    pub timestamp: u64,
}

impl StockQuote {
    /// Creates new `StockQuote` instance with current timestamp
    pub fn new(ticker: &str, price: u64, volume: u32) -> Self {
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
}
