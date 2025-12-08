use crate::udp::ClientCommand;
use crossbeam::channel::Sender;
use quotes::StockQuote;
use rand::rngs::ThreadRng;
use rand::{Rng, rng};
use std::thread;
use std::time::Duration;

/// Maximum growth of 0.02% to emulate growth
const MAX_CHANGE: u64 = 10002;
/// Stock price decreases by 0.01%
const MIN_CHANGE: u64 = 9999;
/// 100%
const DIVIDER: u64 = 10000;

pub(crate) fn run(stocks: Vec<String>, stock_tx: Sender<ClientCommand>) {
    thread::spawn(move || {
        let mut generator = QuoteGenerator::new(stocks);
        loop {
            thread::sleep(Duration::from_millis(10));
            if let Some(random) = generator.random() {
                let _ = stock_tx.send(ClientCommand::Send(random));
            }
        }
    });
}

struct QuoteGenerator {
    stocks_rng: ThreadRng,
    prices_rng: ThreadRng,
    volumes_rng: ThreadRng,
    stocks: Vec<StockQuote>,
}

impl QuoteGenerator {
    fn new(stocks: Vec<String>) -> Self {
        Self {
            stocks_rng: rand::rng(),
            prices_rng: rand::rng(),
            volumes_rng: rand::rng(),
            stocks: Self::build_stocks(stocks),
        }
    }

    fn build_stocks(tickers: Vec<String>) -> Vec<StockQuote> {
        tickers
            .iter()
            .enumerate()
            .map(|(i, ticker)| Self::random_stock(ticker, i as u64))
            .collect()
    }

    fn random(&mut self) -> Option<StockQuote> {
        if self.stocks.is_empty() {
            return None;
        }
        let idx = self.random_idx();
        let mut stock = self.stocks.get(idx).cloned()?;
        self.correct_stock(&mut stock);
        self.stocks[idx] = stock.clone();
        Some(stock)
    }

    fn random_idx(&mut self) -> usize {
        let len = self.stocks.len();
        let total_weight: usize = len * (len + 1) / 2;
        let mut random_weight = self.stocks_rng.random_range(0..total_weight);
        for i in 0..len {
            let weight = len - i;
            if random_weight < weight {
                return i;
            }
            random_weight -= weight;
        }
        0
    }

    fn random_stock(ticker: &str, weight: u64) -> StockQuote {
        StockQuote::new(ticker, Self::random_price(), Self::random_volume(weight))
    }

    fn random_price() -> u64 {
        rng().random_range(100..50000)
    }

    fn random_volume(mut weight: u64) -> u64 {
        if weight == 0 {
            weight = 1;
        }
        let max = u32::MAX as u64 / weight;
        let min = max / 2;
        rng().random_range(min..max)
    }

    fn correct_stock(&mut self, stock_quote: &mut StockQuote) {
        stock_quote.update(
            self.correct_price(stock_quote.price),
            self.correct_volume(stock_quote.volume),
        );
    }

    fn correct_price(&mut self, price: u64) -> u64 {
        let correct: u64 = self.prices_rng.random_range(MIN_CHANGE..MAX_CHANGE);
        (price * correct).div_ceil(DIVIDER)
    }

    fn correct_volume(&mut self, volume: u64) -> u64 {
        let correct: u64 = self.volumes_rng.random_range(MIN_CHANGE..MAX_CHANGE);
        (volume * correct).div_ceil(DIVIDER)
    }
}

impl Default for QuoteGenerator {
    fn default() -> Self {
        Self {
            stocks_rng: rand::rng(),
            prices_rng: rand::rng(),
            volumes_rng: rand::rng(),
            stocks: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_stock() {
        let mut generator = QuoteGenerator::new(vec!["APPL".to_string()]);
        let mut prev = generator.random().unwrap();
        for _ in 0..1000 {
            let curr = generator.random().unwrap();
            if prev.ticker == curr.ticker {
                if prev.price > curr.price {
                    let difference = prev.price - curr.price;
                    assert!(difference <= prev.price.div_ceil(100));
                } else {
                    let difference = curr.price - prev.price;
                    assert!(difference <= (prev.price * 2).div_ceil(100));
                }
            }
            prev = curr;
        }
    }
}
