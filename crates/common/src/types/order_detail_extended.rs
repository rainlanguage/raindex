use crate::meta::{TryDecodeRainlangSource, TryDecodeRainlangSourceError};
use raindex_subgraph_client::types::common::SgOrder;
use serde::{Deserialize, Serialize};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct OrderDetailExtended {
    pub order: SgOrder,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub rainlang: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(OrderDetailExtended);

impl TryFrom<SgOrder> for OrderDetailExtended {
    type Error = TryDecodeRainlangSourceError;

    fn try_from(val: SgOrder) -> Result<Self, TryDecodeRainlangSourceError> {
        let rainlang = val
            .clone()
            .meta
            .map(|meta| meta.0.try_decode_rainlangsource())
            .transpose()?;

        Ok(Self {
            order: val,
            rainlang,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta::TryDecodeRainlangSourceError;
    use raindex_subgraph_client::types::common::{
        SgAddOrder, SgBigInt, SgBytes, SgErc20, SgOrderStructPartialTrade, SgRaindex,
        SgRemoveOrder, SgTransaction, SgVault,
    };
    use std::convert::TryInto;

    fn mock_sg_order(id_str: &str, meta_option: Option<SgBytes>) -> SgOrder {
        let default_sg_bytes = SgBytes(format!("default_{}", id_str));
        let default_big_int = SgBigInt("0".to_string());

        let raindex = SgRaindex {
            id: default_sg_bytes.clone(),
        };

        let token = SgErc20 {
            id: default_sg_bytes.clone(),
            address: default_sg_bytes.clone(),
            name: Some("TestToken".to_string()),
            symbol: Some("TT".to_string()),
            decimals: Some(SgBigInt("18".to_string())),
        };

        let vault = SgVault {
            id: default_sg_bytes.clone(),
            owner: default_sg_bytes.clone(),
            vault_id: default_sg_bytes.clone(),
            balance: default_sg_bytes.clone(),
            token: token.clone(),
            raindex: raindex.clone(),
            orders_as_output: vec![],
            orders_as_input: vec![],
            balance_changes: vec![],
        };

        let transaction = SgTransaction {
            id: default_sg_bytes.clone(),
            from: default_sg_bytes.clone(),
            block_number: default_big_int.clone(),
            timestamp: default_big_int.clone(),
        };

        SgOrder {
            id: SgBytes(id_str.to_string()),
            order_bytes: default_sg_bytes.clone(),
            order_hash: default_sg_bytes.clone(),
            owner: default_sg_bytes.clone(),
            outputs: vec![vault.clone()],
            inputs: vec![vault.clone()],
            raindex,
            active: true,
            timestamp_added: default_big_int.clone(),
            meta: meta_option,
            add_events: vec![SgAddOrder {
                transaction: transaction.clone(),
            }],
            trades: vec![SgOrderStructPartialTrade {
                id: default_sg_bytes.clone(),
            }],
            remove_events: vec![SgRemoveOrder {
                transaction: transaction.clone(),
            }],
        }
    }

    const VALID_META_BYTES_CONTENT: &str = "0xff0a89c674ee7874a30058ef2f2a20302e2063616c63756c6174652d696f202a2f200a7573696e672d776f7264732d66726f6d203078466532343131434461313933443945346538334135633233344337466433323031303138383361430a6d61782d6f75747075743a206d61782d76616c756528292c0a696f3a206966280a2020657175616c2d746f280a202020206f75747075742d746f6b656e28290a202020203078316438306334396262626364316330393131333436363536623532396466396535633266373833640a2020290a202031320a2020696e76283132290a293b0a0a2f2a20312e2068616e646c652d696f202a2f200a3a3b011bff13109e41336ff20278186170706c69636174696f6e2f6f637465742d73747265616d";
    const EMPTY_STRING_META_BYTES_CONTENT: &str = "0x";
    const ERROR_META_BYTES_CONTENT: &str = "0x0f0a89c674ee7874a30058ef2f2a20302e2063616c63756c6174652d696f202a2f200a7573696e672d776f7264732d66726f6d203078466532343131434461313933443945346538334135633233344337466433323031303138383361430a6d61782d6f75747075743a206d61782d76616c756528292c0a696f3a206966280a2020657175616c2d746f280a202020206f75747075742d746f6b656e28290a202020203078316438306334396262626364316330393131333436363536623532396466396535633266373833640a2020290a202031320a2020696e76283132290a293b0a0a2f2a20312e2068616e646c652d696f202a2f200a3a3b011bff13109e41336ff20278186170706c69636174696f6e2f6f637465742d73747265616d";

    #[test]
    fn test_try_from_sg_order_meta_none() {
        let order_id = "order_meta_none";
        let sg_order = mock_sg_order(order_id, None);
        let sg_order_id_clone = sg_order.id.clone();

        let result: Result<OrderDetailExtended, _> = sg_order.try_into();

        assert!(result.is_ok());
        let extended_order = result.unwrap();
        assert_eq!(extended_order.order.id, sg_order_id_clone);
        assert_eq!(extended_order.rainlang, None);
    }

    #[test]
    fn test_try_from_sg_order_meta_some_success() {
        let order_id = "order_meta_success";
        let meta_bytes = SgBytes(VALID_META_BYTES_CONTENT.to_string());
        let sg_order = mock_sg_order(order_id, Some(meta_bytes));
        let sg_order_id_clone = sg_order.id.clone();

        let expected_rainlang = "/* 0. calculate-io */ \nusing-words-from 0xFe2411CDa193D9E4e83A5c234C7Fd320101883aC\nmax-output: max-value(),\nio: if(\n  equal-to(\n    output-token()\n    0x1d80c49bbbcd1c0911346656b529df9e5c2f783d\n  )\n  12\n  inv(12)\n);\n\n/* 1. handle-io */ \n:;".to_string();
        let result: Result<OrderDetailExtended, TryDecodeRainlangSourceError> = sg_order.try_into();

        match result {
            Ok(extended_order) => {
                assert_eq!(extended_order.order.id, sg_order_id_clone);
                assert_eq!(extended_order.rainlang, Some(expected_rainlang));
            }
            Err(_) => {
                panic!("Decoding was expected to succeed, but it failed.");
            }
        }
    }

    #[test]
    fn test_try_from_sg_order_meta_some_success_empty_string() {
        let order_id = "order_meta_empty_success";
        let meta_bytes = SgBytes(EMPTY_STRING_META_BYTES_CONTENT.to_string());
        let sg_order = mock_sg_order(order_id, Some(meta_bytes));

        let result: Result<OrderDetailExtended, TryDecodeRainlangSourceError> = sg_order.try_into();
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            TryDecodeRainlangSourceError::MissingRainlangSourceV1
        ));
    }

    #[test]
    fn test_try_from_sg_order_meta_some_failure() {
        let order_id = "order_meta_failure";
        let meta_bytes = SgBytes(ERROR_META_BYTES_CONTENT.to_string());
        let sg_order = mock_sg_order(order_id, Some(meta_bytes));

        let result: Result<OrderDetailExtended, TryDecodeRainlangSourceError> = sg_order.try_into();
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            TryDecodeRainlangSourceError::MissingRainlangSourceV1
        ));
    }
}
