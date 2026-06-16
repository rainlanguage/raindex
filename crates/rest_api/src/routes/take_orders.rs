use crate::error::{ApiError, ApiErrorResponse};
use rain_orderbook_common::raindex_client::take_orders::TakeOrdersRequest;
use rain_orderbook_common::raindex_client::RaindexClient;
use rain_orderbook_common::take_orders::TakeOrdersMode;
use rocket::serde::json::Json;
use rocket::{post, Route};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuyRequest {
    /// YAML configuration containing network RPC endpoints, subgraph URLs, and orderbook addresses
    #[schema(
        example = "networks:\n  base:\n    rpc: https://mainnet.base.org\n    chain-id: 8453\nsubgraphs:\n  base: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-base/0.9/gn\norderbooks:\n  base:\n    address: 0xd2938e7c9fe3597f78832ce780feb61945c377d7\n    network: base\n    subgraph: base"
    )]
    pub yaml_content: String,
    /// Address that will execute the transaction
    #[schema(example = "0x1111111111111111111111111111111111111111")]
    pub taker: String,
    /// Chain ID where the trade will be executed
    #[schema(example = 8453)]
    pub chain_id: u32,
    /// Token address you are giving (spending)
    #[schema(example = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913")]
    pub token_in: String,
    /// Token address you are receiving (buying)
    #[schema(example = "0x4200000000000000000000000000000000000006")]
    pub token_out: String,
    /// Amount of tokenOut to receive (human-readable decimal string)
    #[schema(example = "1000")]
    pub amount: String,
    /// Maximum price ratio (tokenIn per 1 tokenOut). Trade fails if actual ratio exceeds this.
    #[schema(example = "0.0005")]
    pub max_ratio: String,
    /// If true, transaction reverts unless exactly the specified amount is received. If false (default), receives up to the specified amount.
    #[serde(default)]
    #[schema(example = false)]
    pub exact: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SellRequest {
    /// YAML configuration containing network RPC endpoints, subgraph URLs, and orderbook addresses
    #[schema(
        example = "networks:\n  base:\n    rpc: https://mainnet.base.org\n    chain-id: 8453\nsubgraphs:\n  base: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-base/0.9/gn\norderbooks:\n  base:\n    address: 0xd2938e7c9fe3597f78832ce780feb61945c377d7\n    network: base\n    subgraph: base"
    )]
    pub yaml_content: String,
    /// Address that will execute the transaction
    #[schema(example = "0x1111111111111111111111111111111111111111")]
    pub taker: String,
    /// Chain ID where the trade will be executed
    #[schema(example = 8453)]
    pub chain_id: u32,
    /// Token address you are giving (selling)
    #[schema(example = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913")]
    pub token_in: String,
    /// Token address you are receiving
    #[schema(example = "0x4200000000000000000000000000000000000006")]
    pub token_out: String,
    /// Amount of tokenIn to spend (human-readable decimal string)
    #[schema(example = "500")]
    pub amount: String,
    /// Maximum price ratio (tokenIn per 1 tokenOut). Trade fails if actual ratio exceeds this.
    #[schema(example = "0.0005")]
    pub max_ratio: String,
    /// If true, transaction reverts unless exactly the specified amount is spent. If false (default), spends up to the specified amount.
    #[serde(default)]
    #[schema(example = false)]
    pub exact: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "token": "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
    "spender": "0xd2938e7c9fe3597f78832ce780feb61945c377d7",
    "amount": "1000",
    "formattedAmount": "1000",
    "calldata": "0x095ea7b3..."
}))]
pub struct ApprovalApiResponse {
    /// Token address that needs approval
    #[schema(example = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913")]
    pub token: String,
    /// Spender address (the orderbook contract)
    #[schema(example = "0xd2938e7c9fe3597f78832ce780feb61945c377d7")]
    pub spender: String,
    /// Amount to approve (raw value)
    #[schema(example = "1000")]
    pub amount: String,
    /// Human-readable formatted amount
    #[schema(example = "1000")]
    pub formatted_amount: String,
    /// ABI-encoded approval calldata
    #[schema(example = "0x095ea7b3...")]
    pub calldata: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "orderbook": "0xd2938e7c9fe3597f78832ce780feb61945c377d7",
    "calldata": "0x...",
    "effectivePrice": "0.00045",
    "prices": ["0.00044", "0.00046"],
    "expectedSell": "450",
    "maxSellCap": "500"
}))]
pub struct TakeOrdersReadyResponse {
    /// Address of the orderbook contract to call
    #[schema(example = "0xd2938e7c9fe3597f78832ce780feb61945c377d7")]
    pub orderbook: String,
    /// ABI-encoded calldata for the takeOrders4 function
    #[schema(example = "0x...")]
    pub calldata: String,
    /// Blended effective price across all selected orders (tokenIn per 1 tokenOut)
    #[schema(example = "0.00045")]
    pub effective_price: String,
    /// Individual prices for each order leg, sorted from best to worst
    #[schema(example = json!(["0.00044", "0.00046"]))]
    pub prices: Vec<String>,
    /// Expected amount of tokenIn to spend based on current quotes
    #[schema(example = "450")]
    pub expected_sell: String,
    /// Maximum tokenIn that could be spent (worst-case based on maxRatio)
    #[schema(example = "500")]
    pub max_sell_cap: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", tag = "status", content = "data")]
pub enum TakeOrdersApiResponse {
    #[schema(title = "NeedsApproval")]
    NeedsApproval(ApprovalApiResponse),
    #[schema(title = "Ready")]
    Ready(TakeOrdersReadyResponse),
}

async fn execute_take_orders(
    yaml_content: String,
    request: TakeOrdersRequest,
) -> Result<TakeOrdersApiResponse, ApiError> {
    let client = RaindexClient::new(vec![yaml_content], None, None).await?;

    let result = client.get_take_orders_calldata(request).await?;

    if let Some(approval_info) = result.approval_info() {
        let amount = approval_info.amount().format().map_err(|e| {
            ApiError::Raindex(rain_orderbook_common::raindex_client::RaindexError::Float(
                e,
            ))
        })?;

        Ok(TakeOrdersApiResponse::NeedsApproval(ApprovalApiResponse {
            token: approval_info.token().to_string(),
            spender: approval_info.spender().to_string(),
            amount,
            formatted_amount: approval_info.formatted_amount().to_string(),
            calldata: approval_info.calldata().to_string(),
        }))
    } else if let Some(take_orders_info) = result.take_orders_info() {
        let effective_price = take_orders_info.effective_price().format().map_err(|e| {
            ApiError::Raindex(rain_orderbook_common::raindex_client::RaindexError::Float(
                e,
            ))
        })?;

        let prices: Result<Vec<String>, _> = take_orders_info
            .prices()
            .iter()
            .map(|p| {
                p.format().map_err(|e| {
                    ApiError::Raindex(rain_orderbook_common::raindex_client::RaindexError::Float(
                        e,
                    ))
                })
            })
            .collect();

        let expected_sell = take_orders_info.expected_sell().format().map_err(|e| {
            ApiError::Raindex(rain_orderbook_common::raindex_client::RaindexError::Float(
                e,
            ))
        })?;

        let max_sell_cap = take_orders_info.max_sell_cap().format().map_err(|e| {
            ApiError::Raindex(rain_orderbook_common::raindex_client::RaindexError::Float(
                e,
            ))
        })?;

        Ok(TakeOrdersApiResponse::Ready(TakeOrdersReadyResponse {
            orderbook: take_orders_info.orderbook().to_string(),
            calldata: take_orders_info.calldata().to_string(),
            effective_price,
            prices: prices?,
            expected_sell,
            max_sell_cap,
        }))
    } else {
        unreachable!("TakeOrdersCalldataResult must be either NeedsApproval or Ready")
    }
}

#[utoipa::path(
    post,
    path = "/take-orders/buy",
    tag = "Take Orders",
    request_body = BuyRequest,
    responses(
        (status = 200, description = "Successfully generated buy calldata. Returns either approval info if token approval is needed, or ready calldata if approval is sufficient.", body = TakeOrdersApiResponse,
            examples(
                ("Ready" = (
                    summary = "Calldata ready to execute",
                    description = "Returned when the taker has sufficient token approval. The calldata can be submitted directly to the orderbook.",
                    value = json!({
                        "status": "ready",
                        "data": {
                            "orderbook": "0xd2938e7c9fe3597f78832ce780feb61945c377d7",
                            "calldata": "0x...",
                            "effectivePrice": "0.00045",
                            "prices": ["0.00044", "0.00046"],
                            "expectedSell": "450",
                            "maxSellCap": "500"
                        }
                    })
                )),
                ("NeedsApproval" = (
                    summary = "Token approval required",
                    description = "Returned when the taker needs to approve token spending before executing. Submit the approval calldata first, then retry the request.",
                    value = json!({
                        "status": "needsApproval",
                        "data": {
                            "token": "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
                            "spender": "0xd2938e7c9fe3597f78832ce780feb61945c377d7",
                            "amount": "1000",
                            "formattedAmount": "1000",
                            "calldata": "0x095ea7b3..."
                        }
                    })
                ))
            )
        ),
        (status = 400, description = "Invalid request parameters", body = ApiErrorResponse),
        (status = 404, description = "No liquidity found or configuration not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    )
)]
#[post("/take-orders/buy", data = "<request>")]
pub async fn buy(request: Json<BuyRequest>) -> Result<Json<TakeOrdersApiResponse>, ApiError> {
    let mode = if request.exact {
        TakeOrdersMode::BuyExact
    } else {
        TakeOrdersMode::BuyUpTo
    };

    let yaml_content = request.yaml_content.clone();
    let take_request = TakeOrdersRequest {
        taker: request.taker.clone(),
        chain_id: request.chain_id,
        sell_token: request.token_in.clone(),
        buy_token: request.token_out.clone(),
        mode,
        amount: request.amount.clone(),
        price_cap: request.max_ratio.clone(),
    };

    // RaindexClient contains Rc<RefCell<...>> which is not Send, but Rocket requires
    // Send futures. We use spawn_blocking with a dedicated runtime to run everything
    // on a single thread where Rc<RefCell> is safe.
    let response = tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| ApiError::Internal(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(execute_take_orders(yaml_content, take_request))
    })
    .await
    .map_err(|e| ApiError::Internal(format!("Task execution failed: {}", e)))??;

    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/take-orders/sell",
    tag = "Take Orders",
    request_body = SellRequest,
    responses(
        (status = 200, description = "Successfully generated sell calldata. Returns either approval info if token approval is needed, or ready calldata if approval is sufficient.", body = TakeOrdersApiResponse,
            examples(
                ("Ready" = (
                    summary = "Calldata ready to execute",
                    description = "Returned when the taker has sufficient token approval. The calldata can be submitted directly to the orderbook.",
                    value = json!({
                        "status": "ready",
                        "data": {
                            "orderbook": "0xd2938e7c9fe3597f78832ce780feb61945c377d7",
                            "calldata": "0x...",
                            "effectivePrice": "0.00045",
                            "prices": ["0.00044", "0.00046"],
                            "expectedSell": "450",
                            "maxSellCap": "500"
                        }
                    })
                )),
                ("NeedsApproval" = (
                    summary = "Token approval required",
                    description = "Returned when the taker needs to approve token spending before executing. Submit the approval calldata first, then retry the request.",
                    value = json!({
                        "status": "needsApproval",
                        "data": {
                            "token": "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
                            "spender": "0xd2938e7c9fe3597f78832ce780feb61945c377d7",
                            "amount": "1000",
                            "formattedAmount": "1000",
                            "calldata": "0x095ea7b3..."
                        }
                    })
                ))
            )
        ),
        (status = 400, description = "Invalid request parameters", body = ApiErrorResponse),
        (status = 404, description = "No liquidity found or configuration not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    )
)]
#[post("/take-orders/sell", data = "<request>")]
pub async fn sell(request: Json<SellRequest>) -> Result<Json<TakeOrdersApiResponse>, ApiError> {
    let mode = if request.exact {
        TakeOrdersMode::SpendExact
    } else {
        TakeOrdersMode::SpendUpTo
    };

    let yaml_content = request.yaml_content.clone();
    let take_request = TakeOrdersRequest {
        taker: request.taker.clone(),
        chain_id: request.chain_id,
        sell_token: request.token_in.clone(),
        buy_token: request.token_out.clone(),
        mode,
        amount: request.amount.clone(),
        price_cap: request.max_ratio.clone(),
    };

    // RaindexClient contains Rc<RefCell<...>> which is not Send, but Rocket requires
    // Send futures. We use spawn_blocking with a dedicated runtime to run everything
    // on a single thread where Rc<RefCell> is safe.
    let response = tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| ApiError::Internal(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(execute_take_orders(yaml_content, take_request))
    })
    .await
    .map_err(|e| ApiError::Internal(format!("Task execution failed: {}", e)))??;

    Ok(Json(response))
}

pub fn routes() -> Vec<Route> {
    rocket::routes![buy, sell]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buy_request_deserialization() {
        let json = r#"{
            "yamlContent": "version: 1",
            "taker": "0x1111111111111111111111111111111111111111",
            "chainId": 1,
            "tokenIn": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "tokenOut": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "amount": "100",
            "maxRatio": "2.5"
        }"#;

        let request: BuyRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.yaml_content, "version: 1");
        assert_eq!(request.taker, "0x1111111111111111111111111111111111111111");
        assert_eq!(request.chain_id, 1);
        assert_eq!(
            request.token_in,
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(
            request.token_out,
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        );
        assert_eq!(request.amount, "100");
        assert_eq!(request.max_ratio, "2.5");
        assert!(!request.exact);
    }

    #[test]
    fn test_buy_request_deserialization_exact() {
        let json = r#"{
            "yamlContent": "version: 1",
            "taker": "0x1111111111111111111111111111111111111111",
            "chainId": 137,
            "tokenIn": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "tokenOut": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "amount": "50.5",
            "maxRatio": "1.0",
            "exact": true
        }"#;

        let request: BuyRequest = serde_json::from_str(json).unwrap();

        assert!(request.exact);
    }

    #[test]
    fn test_sell_request_deserialization() {
        let json = r#"{
            "yamlContent": "version: 1",
            "taker": "0x1111111111111111111111111111111111111111",
            "chainId": 1,
            "tokenIn": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "tokenOut": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "amount": "100",
            "maxRatio": "2.5"
        }"#;

        let request: SellRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.yaml_content, "version: 1");
        assert_eq!(request.taker, "0x1111111111111111111111111111111111111111");
        assert_eq!(request.chain_id, 1);
        assert_eq!(
            request.token_in,
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(
            request.token_out,
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        );
        assert_eq!(request.amount, "100");
        assert_eq!(request.max_ratio, "2.5");
        assert!(!request.exact);
    }

    #[test]
    fn test_sell_request_deserialization_exact() {
        let json = r#"{
            "yamlContent": "version: 1",
            "taker": "0x1111111111111111111111111111111111111111",
            "chainId": 1,
            "tokenIn": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "tokenOut": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "amount": "75",
            "maxRatio": "3.0",
            "exact": true
        }"#;

        let request: SellRequest = serde_json::from_str(json).unwrap();

        assert!(request.exact);
    }

    #[test]
    fn test_ready_response_serialization() {
        let response = TakeOrdersApiResponse::Ready(TakeOrdersReadyResponse {
            orderbook: "0x1234567890123456789012345678901234567890".to_string(),
            calldata: "0xabcdef".to_string(),
            effective_price: "1.5".to_string(),
            prices: vec!["1.4".to_string(), "1.6".to_string()],
            expected_sell: "150".to_string(),
            max_sell_cap: "200".to_string(),
        });

        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"status\":\"ready\""));
        assert!(json.contains("\"data\":"));
        assert!(json.contains("\"orderbook\":"));
        assert!(json.contains("\"calldata\":"));
        assert!(json.contains("\"effectivePrice\":"));
        assert!(json.contains("\"prices\":"));
        assert!(json.contains("\"expectedSell\":"));
        assert!(json.contains("\"maxSellCap\":"));
    }

    #[test]
    fn test_needs_approval_response_serialization() {
        let response = TakeOrdersApiResponse::NeedsApproval(ApprovalApiResponse {
            token: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            spender: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            amount: "1000".to_string(),
            formatted_amount: "1000".to_string(),
            calldata: "0xabcdef".to_string(),
        });

        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"status\":\"needsApproval\""));
        assert!(json.contains("\"data\":"));
        assert!(json.contains("\"token\":"));
        assert!(json.contains("\"spender\":"));
        assert!(json.contains("\"amount\":"));
        assert!(json.contains("\"formattedAmount\":"));
        assert!(json.contains("\"calldata\":"));
    }
}
