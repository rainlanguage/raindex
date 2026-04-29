// Re-export oracle types and functions from the quote crate.
// This maintains backward compatibility for code in common that uses oracle functionality.
pub use rain_orderbook_quote::oracle::*;
