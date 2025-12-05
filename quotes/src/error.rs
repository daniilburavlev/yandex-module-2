/// The error type for `quote` crate
#[derive(Debug)]
pub enum Error {
    /// Invalid input data i.e serialization/deserialization
    InvalidData(String),
}
