use alloy::sol;

sol!(
    #![sol(all_derives = true, rpc)]
    #![sol(extra_derives(serde::Serialize, serde::Deserialize))]
    IOrderBookV6, "../../out/IOrderBookV6.sol/IOrderBookV6.json"
);

sol!(
    #![sol(all_derives = true)]
    #![sol(extra_derives(serde::Serialize, serde::Deserialize))]
    OrderBook, "../../out/OrderBookV6.sol/OrderBookV6.json"
);

sol!(
    #![sol(all_derives = true, rpc)]
    IERC20, "../../out/IERC20.sol/IERC20.json"
);

sol!(
    #![sol(all_derives = true, rpc)]
    ERC20, "../../out/ERC20.sol/ERC20.json"
);

sol!(
    #![sol(all_derives = true)]
    #![sol(extra_derives(serde::Serialize, serde::Deserialize))]
    IInterpreterStoreV3, "../../out/IInterpreterStoreV3.sol/IInterpreterStoreV3.json"
);

pub mod provider;

#[cfg(target_family = "wasm")]
pub mod js_api;

#[cfg(target_family = "wasm")]
pub mod wasm_traits;

pub mod topics {
    use crate::{
        IInterpreterStoreV3::Set,
        IOrderBookV6::{
            AddOrderV3, AfterClearV2, ClearV3, DepositV2, RemoveOrderV3, TakeOrderV3, WithdrawV2,
        },
        OrderBook::MetaV1_2,
    };
    use alloy::{primitives::B256, sol_types::SolEvent};

    pub const ORDERBOOK_EVENT_TOPICS: [B256; 8] = [
        AddOrderV3::SIGNATURE_HASH,
        TakeOrderV3::SIGNATURE_HASH,
        WithdrawV2::SIGNATURE_HASH,
        DepositV2::SIGNATURE_HASH,
        RemoveOrderV3::SIGNATURE_HASH,
        ClearV3::SIGNATURE_HASH,
        AfterClearV2::SIGNATURE_HASH,
        MetaV1_2::SIGNATURE_HASH,
    ];

    pub const STORE_SET_TOPICS: [B256; 1] = [Set::SIGNATURE_HASH];
}
