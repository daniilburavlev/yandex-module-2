use crate::StockQuote;
use std::io::{BufReader, ErrorKind, Read};
use std::{io, mem};

const PRICE_SIZE: usize = 8;
const VOLUME_SIZE: usize = 8;
const TIMESTAMP_SIZE: usize = 8;

const INVALID_FORMAT_MSG: &str = "Неверный формат файла!";

const MAX_TICKER_LEN: usize = 4;

impl TryInto<Vec<u8>> for StockQuote {
    type Error = io::Error;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        let ticker = self.ticker.as_bytes();
        let ticker_len = ticker.len();
        if ticker_len > MAX_TICKER_LEN {
            return Err(max_ticker_len_error(ticker_len));
        }
        let len = get_size(ticker_len);

        let mut bytes = Vec::with_capacity(len);

        bytes.extend_from_slice((len as u32).to_be_bytes().as_ref());
        bytes.extend_from_slice(self.price.to_be_bytes().as_ref());
        bytes.extend_from_slice(self.volume.to_be_bytes().as_ref());
        bytes.extend_from_slice(self.timestamp.to_be_bytes().as_ref());
        bytes.extend_from_slice(ticker);

        Ok(bytes)
    }
}

macro_rules! read_num {
    ($reader:expr, $ty:ty) => {{
        const SIZE: usize = mem::size_of::<$ty>();
        let mut buffer = [0u8; SIZE];
        $reader
            .read_exact(&mut buffer)
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, INVALID_FORMAT_MSG.to_string()))?;
        <$ty>::from_be_bytes(buffer)
    }};
}

impl TryFrom<Vec<u8>> for StockQuote {
    type Error = io::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut reader = BufReader::new(value.as_slice());

        let len = read_num!(reader, u32);
        let price = read_num!(reader, u64);
        let volume = read_num!(reader, u64);
        let timestamp = read_num!(reader, u64);
        let ticker_len = get_ticker_len(len as usize);

        if ticker_len > MAX_TICKER_LEN {
            return Err(max_ticker_len_error(ticker_len));
        }

        let mut ticker = vec![0u8; ticker_len];
        reader.read_exact(&mut ticker)?;
        let ticker = String::from_utf8(ticker).map_err(|_| {
            io::Error::new(
                ErrorKind::InvalidData,
                "Описание должно быть в формате UTF-8".to_string(),
            )
        })?;

        Ok(Self {
            ticker,
            price,
            volume,
            timestamp,
        })
    }
}

fn get_size(ticker_len: usize) -> usize {
    ticker_len + PRICE_SIZE + VOLUME_SIZE + TIMESTAMP_SIZE
}

fn get_ticker_len(len: usize) -> usize {
    len - (PRICE_SIZE + VOLUME_SIZE + TIMESTAMP_SIZE)
}

fn max_ticker_len_error(ticker_len: usize) -> io::Error {
    io::Error::new(
        ErrorKind::InvalidData,
        format!(
            "Максимальный размер размер тикера: {}, получено: {}",
            MAX_TICKER_LEN, ticker_len
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize() {
        let stock = StockQuote::new("AAPL", 100, 3000000000);
        let bytes: Vec<u8> = stock.clone().try_into().unwrap();
        let restored = StockQuote::try_from(bytes).unwrap();
        assert_eq!(restored, stock);
    }

    #[test]
    #[should_panic(expected = "Максимальный размер размер тикера: 4, получено: 5")]
    fn test_max_ticker_size() {
        let stock = StockQuote::new("APPLE", 100, 3000000000);
        let _: Vec<u8> = stock.clone().try_into().unwrap();
    }
}
