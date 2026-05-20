use alloy::primitives::{keccak256, Address, B256, U256};

/// Builds the subgraph represented order ID, given a raindex address and an order hash
/// An order ID on subgraph is keccak256 of concated raindex address + order hash
pub fn make_order_id(raindex: Address, order_hash: U256) -> B256 {
    let mut id_bytes = vec![];
    id_bytes.extend_from_slice(raindex.as_ref());
    id_bytes.extend_from_slice(&B256::from(order_hash).0);
    keccak256(id_bytes)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_make_order_id() {
        let bytes = [4u8; 52];
        let result = make_order_id(
            Address::from_slice(&bytes[..20]),
            U256::from_be_slice(&bytes[20..]),
        );
        let expected = keccak256(bytes);

        assert_eq!(result, expected)
    }
}
