use raindex_common::raindex_client::RaindexError;
use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::serde::json::Json;
use rocket::Request;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "error": "No liquidity available for the given token pair",
    "readableMessage": "No liquidity available for the given token pair on the specified chain"
}))]
pub struct ApiErrorResponse {
    #[schema(example = "No liquidity available for the given token pair")]
    pub error: String,
    #[schema(example = "No liquidity available for the given token pair on the specified chain")]
    pub readable_message: String,
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    Raindex(#[from] RaindexError),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl ApiError {
    fn status_code(&self) -> Status {
        match self {
            ApiError::Raindex(e) => match e {
                RaindexError::InvalidYamlConfig
                | RaindexError::YamlError(_)
                | RaindexError::FromHexError(_)
                | RaindexError::U256ParseError(_)
                | RaindexError::I256ParseError(_)
                | RaindexError::ZeroAmount
                | RaindexError::NegativeAmount
                | RaindexError::NonPositiveAmount
                | RaindexError::NegativePriceCap
                | RaindexError::SameTokenPair
                | RaindexError::Float(_)
                | RaindexError::ParseInt(_) => Status::BadRequest,

                RaindexError::NoLiquidity | RaindexError::InsufficientLiquidity { .. } => {
                    Status::NotFound
                }

                RaindexError::ChainIdNotFound(_)
                | RaindexError::RaindexNotFound(_, _)
                | RaindexError::OrderNotFound(_, _, _)
                | RaindexError::VaultNotFound(_, _, _)
                | RaindexError::SubgraphNotFound(_, _)
                | RaindexError::SubgraphNotConfigured(_)
                | RaindexError::NoNetworksConfigured => Status::NotFound,

                _ => Status::InternalServerError,
            },
            ApiError::Internal(_) => Status::InternalServerError,
        }
    }

    fn to_response(&self) -> ApiErrorResponse {
        let readable_message = match self {
            ApiError::Raindex(e) => e.to_readable_msg(),
            ApiError::Internal(msg) => msg.clone(),
        };

        ApiErrorResponse {
            error: self.to_string(),
            readable_message,
        }
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
        let status = self.status_code();
        let body = self.to_response();

        response::Response::build_from(Json(body).respond_to(request)?)
            .status(status)
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::B256;

    #[test]
    fn test_status_code_bad_request_errors() {
        let bad_request_errors = vec![
            ApiError::Raindex(RaindexError::InvalidYamlConfig),
            ApiError::Raindex(RaindexError::ZeroAmount),
            ApiError::Raindex(RaindexError::NegativeAmount),
            ApiError::Raindex(RaindexError::NonPositiveAmount),
            ApiError::Raindex(RaindexError::NegativePriceCap),
            ApiError::Raindex(RaindexError::SameTokenPair),
        ];

        for error in bad_request_errors {
            assert_eq!(
                error.status_code(),
                Status::BadRequest,
                "Expected BadRequest for {:?}",
                error
            );
        }
    }

    #[test]
    fn test_status_code_not_found_liquidity_errors() {
        let not_found_errors = vec![
            ApiError::Raindex(RaindexError::NoLiquidity),
            ApiError::Raindex(RaindexError::InsufficientLiquidity {
                requested: "100".to_string(),
                available: "50".to_string(),
            }),
        ];

        for error in not_found_errors {
            assert_eq!(
                error.status_code(),
                Status::NotFound,
                "Expected NotFound for {:?}",
                error
            );
        }
    }

    #[test]
    fn test_status_code_not_found_config_errors() {
        let not_found_errors = vec![
            ApiError::Raindex(RaindexError::ChainIdNotFound(1)),
            ApiError::Raindex(RaindexError::RaindexNotFound("0x123".to_string(), 1)),
            ApiError::Raindex(RaindexError::OrderNotFound(
                "0x123".to_string(),
                1,
                B256::ZERO,
            )),
            ApiError::Raindex(RaindexError::VaultNotFound(
                "0x123".to_string(),
                1,
                "1".to_string(),
            )),
            ApiError::Raindex(RaindexError::SubgraphNotFound(
                "test".to_string(),
                "order".to_string(),
            )),
            ApiError::Raindex(RaindexError::SubgraphNotConfigured("1".to_string())),
            ApiError::Raindex(RaindexError::NoNetworksConfigured),
        ];

        for error in not_found_errors {
            assert_eq!(
                error.status_code(),
                Status::NotFound,
                "Expected NotFound for {:?}",
                error
            );
        }
    }

    #[test]
    fn test_status_code_internal_server_error() {
        let internal_error = ApiError::Internal("Something went wrong".to_string());
        assert_eq!(internal_error.status_code(), Status::InternalServerError);
    }

    #[test]
    fn test_status_code_preflight_error_is_internal() {
        let preflight_error = ApiError::Raindex(RaindexError::PreflightError(
            "Simulation failed".to_string(),
        ));
        assert_eq!(preflight_error.status_code(), Status::InternalServerError);
    }

    #[test]
    fn test_to_response_raindex_error() {
        let error = ApiError::Raindex(RaindexError::NoLiquidity);
        let response = error.to_response();

        assert!(response.error.contains("No liquidity"));
        assert!(response.readable_message.contains("No liquidity available"));
    }

    #[test]
    fn test_to_response_internal_error() {
        let error = ApiError::Internal("Custom error message".to_string());
        let response = error.to_response();

        assert!(response.error.contains("Internal server error"));
        assert_eq!(response.readable_message, "Custom error message");
    }

    #[test]
    fn test_api_error_response_serialization() {
        let response = ApiErrorResponse {
            error: "Test error".to_string(),
            readable_message: "A readable message".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"error\":\"Test error\""));
        assert!(json.contains("\"readableMessage\":\"A readable message\""));
    }

    #[test]
    fn test_api_error_from_raindex_error() {
        let raindex_error = RaindexError::NoLiquidity;
        let api_error: ApiError = raindex_error.into();

        assert!(matches!(
            api_error,
            ApiError::Raindex(RaindexError::NoLiquidity)
        ));
    }
}
