use crate::local_db::OrderbookIdentifier;
use alloy::primitives::Address;
use std::str::FromStr;
use wasm_bindgen_utils::prelude::*;

#[derive(Default, Clone, Debug)]
#[wasm_bindgen]
pub struct PaginationParams {
    pub(crate) page: Option<u16>,
    pub(crate) page_size: Option<u16>,
}

#[wasm_bindgen]
impl PaginationParams {
    #[wasm_bindgen(constructor)]
    pub fn new(page: Option<u16>, page_size: Option<u16>) -> Self {
        Self { page, page_size }
    }
}

#[derive(Default, Clone, Debug)]
#[wasm_bindgen]
pub struct TimeFilter {
    pub(crate) start: Option<u64>,
    pub(crate) end: Option<u64>,
}

#[wasm_bindgen]
impl TimeFilter {
    #[wasm_bindgen(constructor)]
    pub fn new(start: Option<u64>, end: Option<u64>) -> Self {
        Self { start, end }
    }
}

#[derive(Default, Clone, Debug)]
#[wasm_bindgen]
pub struct OrderbookIdentifierParams {
    pub(crate) chain_id: u32,
    pub(crate) orderbook_address: String,
}

#[wasm_bindgen]
impl OrderbookIdentifierParams {
    #[wasm_bindgen(constructor)]
    pub fn new(chain_id: u32, orderbook_address: String) -> Self {
        Self {
            chain_id,
            orderbook_address,
        }
    }
}

impl OrderbookIdentifierParams {
    pub fn try_into_ob_id(
        self,
    ) -> Result<OrderbookIdentifier, alloy::primitives::hex::FromHexError> {
        Ok(OrderbookIdentifier::new(
            self.chain_id,
            Address::from_str(&self.orderbook_address)?,
        ))
    }
}
