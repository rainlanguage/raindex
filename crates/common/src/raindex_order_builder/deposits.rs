use super::*;
use rain_orderbook_app_settings::order_builder::OrderBuilderDepositCfg;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TokenDeposit {
    pub token: String,
    pub amount: String,
    pub address: Address,
}

impl RaindexOrderBuilder {
    pub fn check_deposits(&self) -> Result<(), RaindexOrderBuilderError> {
        let deployment = self.get_current_deployment()?;

        for deposit in deployment.deposits.iter() {
            if deposit.token.is_none() {
                return Err(RaindexOrderBuilderError::MissingDepositToken(
                    deployment.key.clone(),
                ));
            }

            let token = deposit.token.as_ref().unwrap();
            if !self.deposits.contains_key(&token.key) {
                return Err(RaindexOrderBuilderError::DepositNotSet(
                    token
                        .symbol
                        .clone()
                        .unwrap_or(token.label.clone().unwrap_or(token.key.clone())),
                ));
            }
        }
        Ok(())
    }

    pub fn get_deposit_config(
        &self,
        key: &str,
    ) -> Result<OrderBuilderDepositCfg, RaindexOrderBuilderError> {
        let deployment = self.get_current_deployment()?;
        let deposit_config = deployment
            .deposits
            .iter()
            .find(|dg| dg.token.as_ref().is_some_and(|t| t.key == *key))
            .ok_or(RaindexOrderBuilderError::DepositTokenNotFound(
                key.to_string(),
            ))?;
        Ok(deposit_config.clone())
    }

    pub fn get_deposits(&self) -> Result<Vec<TokenDeposit>, RaindexOrderBuilderError> {
        self.deposits
            .iter()
            .map(|(key, value)| {
                let deposit_config = self.get_deposit_config(key)?;
                let amount: String = if value.is_preset {
                    let index = value
                        .value
                        .parse::<usize>()
                        .map_err(|_| RaindexOrderBuilderError::InvalidPreset)?;
                    deposit_config
                        .presets
                        .as_ref()
                        .ok_or(RaindexOrderBuilderError::PresetsNotSet)?
                        .get(index)
                        .ok_or(RaindexOrderBuilderError::InvalidPreset)?
                        .clone()
                } else {
                    value.value.clone()
                };

                if deposit_config.token.is_none() {
                    return Err(RaindexOrderBuilderError::TokenMustBeSelected(key.clone()));
                }
                let token = deposit_config.token.as_ref().unwrap();

                Ok(TokenDeposit {
                    token: token.key.clone(),
                    amount,
                    address: token.address,
                })
            })
            .collect::<Result<Vec<TokenDeposit>, RaindexOrderBuilderError>>()
    }

    pub async fn set_deposit(
        &mut self,
        token: String,
        amount: String,
    ) -> Result<(), RaindexOrderBuilderError> {
        let deposit_config = self.get_deposit_config(&token)?;

        if amount.is_empty() {
            return Err(RaindexOrderBuilderError::DepositAmountCannotBeEmpty);
        }

        if let Some(validation) = &deposit_config.validation {
            let token_info = self.get_token_info(token.clone()).await?;
            validation::validate_deposit_amount(&token_info.name, &amount, validation)?;
        }

        let value = match deposit_config.presets.as_ref() {
            Some(presets) => match presets.iter().position(|p| **p == amount) {
                Some(index) => field_values::PairValue {
                    is_preset: true,
                    value: index.to_string(),
                },
                None => field_values::PairValue {
                    is_preset: false,
                    value: amount,
                },
            },
            None => field_values::PairValue {
                is_preset: false,
                value: amount,
            },
        };

        self.deposits.insert(token, value);

        Ok(())
    }

    pub fn unset_deposit(&mut self, token: String) -> Result<(), RaindexOrderBuilderError> {
        self.deposits.remove(&token);
        Ok(())
    }

    pub fn get_deposit_presets(
        &self,
        key: String,
    ) -> Result<Vec<String>, RaindexOrderBuilderError> {
        let deposit_config = self.get_deposit_config(&key)?;
        Ok(deposit_config.presets.clone().unwrap_or(vec![]))
    }

    pub fn get_missing_deposits(&self) -> Result<Vec<String>, RaindexOrderBuilderError> {
        let deployment = self.get_current_deployment()?;
        let mut missing_deposits = Vec::new();

        for deposit in deployment.deposits.iter() {
            if let Some(token) = &deposit.token {
                if !self.deposits.contains_key(&token.key) {
                    missing_deposits.push(token.key.clone());
                }
            }
        }
        Ok(missing_deposits)
    }

    pub fn has_any_deposit(&self) -> Result<bool, RaindexOrderBuilderError> {
        Ok(!self.deposits.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raindex_order_builder::tests::{
        initialize_builder, initialize_builder_with_select_tokens, initialize_validation_builder,
    };
    use crate::raindex_order_builder::validation;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_get_deposit_config() {
        let builder = initialize_builder(None).await;

        let deposit = builder.get_deposit_config("token1").unwrap();
        assert_eq!(deposit.token.unwrap().key, "token1");
        assert_eq!(
            deposit.presets,
            Some(vec![
                "0".to_string(),
                "10".to_string(),
                "100".to_string(),
                "1000".to_string(),
                "10000".to_string()
            ])
        );

        let err = builder.get_deposit_config("token2").unwrap_err();
        assert_eq!(
            err.to_string(),
            RaindexOrderBuilderError::DepositTokenNotFound("token2".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The deposit token 'token2' was not found in the YAML configuration."
        );

        let builder = initialize_builder_with_select_tokens().await;
        let err = builder.get_deposit_config("token3").unwrap_err();
        assert_eq!(
            err.to_string(),
            RaindexOrderBuilderError::DepositTokenNotFound("token3".to_string()).to_string()
        );
    }

    #[tokio::test]
    async fn test_get_deposits() {
        let mut builder = initialize_builder(None).await;

        builder
            .set_deposit("token1".to_string(), "999".to_string())
            .await
            .unwrap();

        let deposit = builder.get_deposits().unwrap();
        assert_eq!(deposit.len(), 1);
        assert_eq!(deposit[0].token, "token1");
        assert_eq!(deposit[0].amount, "999");
        assert_eq!(
            deposit[0].address,
            Address::from_str("0xc2132d05d31c914a87c6611c10748aeb04b58e8f").unwrap()
        );
    }

    #[tokio::test]
    async fn test_set_deposit() {
        let mut builder = initialize_builder(None).await;

        builder
            .set_deposit("token1".to_string(), "999".to_string())
            .await
            .unwrap();

        let deposit = builder.get_deposits().unwrap();
        assert_eq!(deposit.len(), 1);
        assert_eq!(deposit[0].token, "token1");
        assert_eq!(deposit[0].amount, "999");
        assert_eq!(
            deposit[0].address,
            Address::from_str("0xc2132d05d31c914a87c6611c10748aeb04b58e8f").unwrap()
        );

        let err = builder
            .set_deposit("token1".to_string(), "".to_string())
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            RaindexOrderBuilderError::DepositAmountCannotBeEmpty.to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The deposit amount cannot be an empty string. Please set a valid amount."
        );

        let mut builder = initialize_builder_with_select_tokens().await;
        let err = builder
            .set_deposit("token3".to_string(), "999".to_string())
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            RaindexOrderBuilderError::DepositTokenNotFound("token3".to_string()).to_string()
        );
    }

    #[tokio::test]
    async fn test_unset_deposit() {
        let mut builder = initialize_builder(None).await;

        builder
            .set_deposit("token1".to_string(), "999".to_string())
            .await
            .unwrap();
        let deposit = builder.get_deposits().unwrap();
        assert_eq!(deposit.len(), 1);

        builder.unset_deposit("token1".to_string()).unwrap();
        assert_eq!(builder.get_deposits().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_get_deposit_presets() {
        let builder = initialize_builder(None).await;

        let presets = builder.get_deposit_presets("token1".to_string()).unwrap();
        assert_eq!(
            presets,
            vec![
                "0".to_string(),
                "10".to_string(),
                "100".to_string(),
                "1000".to_string(),
                "10000".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn test_get_missing_deposits() {
        let builder = initialize_builder(None).await;

        let missing_deposits = builder.get_missing_deposits().unwrap();
        assert_eq!(missing_deposits, vec!["token1".to_string()]);
    }

    #[tokio::test]
    async fn test_has_any_deposit() {
        let mut builder = initialize_builder(None).await;

        let has_any_deposit = builder.has_any_deposit().unwrap();
        assert!(!has_any_deposit);

        builder
            .set_deposit("token1".to_string(), "999".to_string())
            .await
            .unwrap();
        let has_any_deposit = builder.has_any_deposit().unwrap();
        assert!(has_any_deposit);
    }

    #[tokio::test]
    async fn test_check_deposits() {
        let mut builder = initialize_builder(None).await;

        builder
            .set_deposit("token1".to_string(), "999".to_string())
            .await
            .unwrap();
        builder.check_deposits().unwrap();
        builder.unset_deposit("token1".to_string()).unwrap();

        let err = builder.check_deposits().unwrap_err();
        assert_eq!(
            err.to_string(),
            RaindexOrderBuilderError::DepositNotSet("T1".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "A deposit for token 'T1' is required but has not been set."
        );

        let builder = initialize_builder_with_select_tokens().await;
        let err = builder.check_deposits().unwrap_err();
        assert_eq!(
            err.to_string(),
            RaindexOrderBuilderError::MissingDepositToken("select-token-deployment".to_string())
                .to_string()
        );
    }

    #[tokio::test]
    async fn test_save_deposit_minimum_validation() {
        let mut builder = initialize_validation_builder().await;
        let result = builder
            .set_deposit("token1".to_string(), "50".to_string())
            .await;
        match result {
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::BelowMinimum {
                    name,
                    value,
                    minimum,
                },
            )) => {
                assert_eq!(name, "Token 1");
                assert_eq!(value, "50");
                assert_eq!(minimum, "100");
            }
            _ => panic!("Expected BelowMinimum error"),
        }
        let result = builder
            .set_deposit("token1".to_string(), "100".to_string())
            .await;
        assert!(result.is_ok());
        let result = builder
            .set_deposit("token1".to_string(), "500".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_save_deposit_maximum_validation() {
        let mut builder = initialize_validation_builder().await;
        let result = builder
            .set_deposit("token2".to_string(), "5000".to_string())
            .await;
        assert!(result.is_ok());
        let result = builder
            .set_deposit("token2".to_string(), "10000".to_string())
            .await;
        assert!(result.is_ok());
        let result = builder
            .set_deposit("token2".to_string(), "15000".to_string())
            .await;
        match result {
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::AboveMaximum {
                    name,
                    value,
                    maximum,
                },
            )) => {
                assert_eq!(name, "Token 2");
                assert_eq!(value, "15000");
                assert_eq!(maximum, "10000");
            }
            _ => panic!("Expected AboveMaximum error"),
        }
    }

    #[tokio::test]
    async fn test_save_deposit_no_validation() {
        let mut builder = initialize_validation_builder().await;
        let result = builder
            .set_deposit("token6".to_string(), "0".to_string())
            .await;
        assert!(result.is_ok());

        let result = builder
            .set_deposit("token6".to_string(), "123456789.123456789".to_string())
            .await;
        assert!(result.is_ok());

        let result = builder
            .set_deposit("token6".to_string(), "0.00000001".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_save_deposit_exclusive_bounds() {
        let mut builder = initialize_validation_builder().await;
        let result = builder
            .set_deposit("token3".to_string(), "0".to_string())
            .await;
        match result {
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::BelowExclusiveMinimum {
                    name,
                    value,
                    exclusive_minimum,
                },
            )) => {
                assert_eq!(name, "Token 3");
                assert_eq!(value, "0");
                assert_eq!(exclusive_minimum, "0");
            }
            _ => panic!("Expected BelowExclusiveMinimum error"),
        }
        let result = builder
            .set_deposit("token3".to_string(), "0.001".to_string())
            .await;
        assert!(result.is_ok());
        let result = builder
            .set_deposit("token3".to_string(), "49999.999".to_string())
            .await;
        assert!(result.is_ok());
        let result = builder
            .set_deposit("token3".to_string(), "50000".to_string())
            .await;
        match result {
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::AboveExclusiveMaximum {
                    name,
                    value,
                    exclusive_maximum,
                },
            )) => {
                assert_eq!(name, "Token 3");
                assert_eq!(value, "50000");
                assert_eq!(exclusive_maximum, "50000");
            }
            _ => panic!("Expected AboveExclusiveMaximum error"),
        }
    }

    #[tokio::test]
    async fn test_save_deposit_multiple_constraints() {
        let mut builder = initialize_validation_builder().await;

        let result = builder
            .set_deposit("token4".to_string(), "50".to_string())
            .await;
        assert!(result.is_ok());

        let result = builder
            .set_deposit("token4".to_string(), "5".to_string())
            .await;
        assert!(matches!(
            result,
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::BelowMinimum { .. }
            ))
        ));

        let result = builder
            .set_deposit("token4".to_string(), "1005".to_string())
            .await;
        assert!(matches!(
            result,
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::AboveMaximum { .. }
            ))
        ));

        let result = builder
            .set_deposit("token4".to_string(), "10".to_string())
            .await;
        assert!(result.is_ok());

        let result = builder
            .set_deposit("token4".to_string(), "1000".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_save_deposit_invalid_formats() {
        let mut builder = initialize_validation_builder().await;

        let result = builder
            .set_deposit("token1".to_string(), "abc".to_string())
            .await;
        assert!(matches!(
            result,
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::FloatError(..)
            ))
        ));

        let result = builder
            .set_deposit("token1".to_string(), "12.34.56".to_string())
            .await;
        assert!(matches!(
            result,
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::FloatError(..)
            ))
        ));

        let result = builder
            .set_deposit("token1".to_string(), "12,345".to_string())
            .await;
        assert!(matches!(
            result,
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::FloatError(..)
            ))
        ));
    }

    #[tokio::test]
    async fn test_save_deposit_edge_cases() {
        let mut builder = initialize_validation_builder().await;
        let result = builder
            .set_deposit("token3".to_string(), "0.001".to_string())
            .await;
        assert!(result.is_ok());
        let result = builder
            .set_deposit("token2".to_string(), "9999.999999999".to_string())
            .await;
        assert!(result.is_ok());
        let result = builder
            .set_deposit("token1".to_string(), "100.00000".to_string())
            .await;
        assert!(result.is_ok());
        let result = builder
            .set_deposit("token1".to_string(), "00100".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_save_deposit_with_presets_and_validation() {
        let mut builder = initialize_validation_builder().await;
        let result = builder
            .set_deposit("token1".to_string(), "200".to_string())
            .await;
        assert!(result.is_ok());
        let deposits = builder.get_deposits().unwrap();
        assert_eq!(deposits.len(), 1);
        assert_eq!(deposits[0].token, "token1");
        assert_eq!(deposits[0].amount, "200");
    }
}
