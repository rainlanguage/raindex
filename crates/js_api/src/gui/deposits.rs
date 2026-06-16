use super::*;
use rain_orderbook_app_settings::gui::GuiDepositCfg;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct TokenDeposit {
    pub token: String,
    pub amount: String,
    #[tsify(type = "string")]
    pub address: Address,
}

impl DotrainOrderGui {
    pub fn check_deposits(&self) -> Result<(), GuiError> {
        let deployment = self.get_current_deployment()?;

        for deposit in deployment.deposits.iter() {
            if deposit.token.is_none() {
                return Err(GuiError::MissingDepositToken(deployment.key.clone()));
            }

            let token = deposit.token.as_ref().unwrap();
            if !self.deposits.contains_key(&token.key) {
                return Err(GuiError::DepositNotSet(
                    token
                        .symbol
                        .clone()
                        .unwrap_or(token.label.clone().unwrap_or(token.key.clone())),
                ));
            }
        }
        Ok(())
    }
}

#[wasm_export]
impl DotrainOrderGui {
    fn get_gui_deposit(&self, key: &str) -> Result<GuiDepositCfg, GuiError> {
        let deployment = self.get_current_deployment()?;
        let gui_deposit = deployment
            .deposits
            .iter()
            .find(|dg| dg.token.as_ref().is_some_and(|t| t.key == *key))
            .ok_or(GuiError::DepositTokenNotFound(key.to_string()))?;
        Ok(gui_deposit.clone())
    }

    /// Gets all configured deposit amounts with token information.
    ///
    /// Returns deposits as token deposits with human-readable amounts and addresses.
    /// This combines the stored deposit amounts with token metadata for display and
    /// transaction preparation.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = gui.getDeposits();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const [deposit1, deposit2, ...] = result.value;
    /// const {
    ///   // token is the token address
    ///   token,
    ///   // amount is the deposit amount
    ///   amount,
    ///   // address is the token address
    /// } = deposit1;
    /// ```
    #[wasm_export(
        js_name = "getDeposits",
        unchecked_return_type = "TokenDeposit[]",
        return_description = "Array of deposits with token details and amounts"
    )]
    pub fn get_deposits(&self) -> Result<Vec<TokenDeposit>, GuiError> {
        self.deposits
            .iter()
            .map(|(key, value)| {
                let gui_deposit = self.get_gui_deposit(key)?;
                let amount: String = if value.is_preset {
                    let index = value
                        .value
                        .parse::<usize>()
                        .map_err(|_| GuiError::InvalidPreset)?;
                    gui_deposit
                        .presets
                        .as_ref()
                        .ok_or(GuiError::PresetsNotSet)?
                        .get(index)
                        .ok_or(GuiError::InvalidPreset)?
                        .clone()
                } else {
                    value.value.clone()
                };

                if gui_deposit.token.is_none() {
                    return Err(GuiError::TokenMustBeSelected(key.clone()));
                }
                let token = gui_deposit.token.as_ref().unwrap();

                Ok(TokenDeposit {
                    token: token.key.clone(),
                    amount,
                    address: token.address,
                })
            })
            .collect::<Result<Vec<TokenDeposit>, GuiError>>()
    }

    /// Sets a deposit amount for a specific token.
    ///
    /// Sets the deposit amount for a token, automatically detecting if the amount
    /// matches a preset value.
    ///
    /// ## Preset Detection
    ///
    /// If the amount matches a preset value from the configuration, it's stored
    /// as a preset reference for efficiency and consistency.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Set a custom deposit amount
    /// const result = await gui.setDeposit("usdc", "1000.50");
    /// if (result.error) {
    ///   console.error("Deposit error:", result.error.readableMsg);
    ///   return;
    /// }
    /// ```
    #[wasm_export(js_name = "setDeposit", unchecked_return_type = "void")]
    pub async fn set_deposit(
        &mut self,
        #[wasm_export(param_description = "Token key from the deposits configuration")]
        token: String,
        #[wasm_export(param_description = "Human-readable amount (e.g., '100.5')")] amount: String,
    ) -> Result<(), GuiError> {
        let gui_deposit = self.get_gui_deposit(&token)?;

        if amount.is_empty() {
            return Err(GuiError::DepositAmountCannotBeEmpty);
        }

        if let Some(validation) = &gui_deposit.validation {
            let token_info = self.get_token_info(token.clone()).await?;
            validation::validate_deposit_amount(&token_info.name, &amount, validation)?;
        }

        let value = match gui_deposit.presets.as_ref() {
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

        self.execute_state_update_callback()?;
        Ok(())
    }

    /// Unsets the deposit amount for a specific token.
    ///
    /// Use this to clear a deposit that's no longer needed.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Unset a specific token deposit
    /// const result = gui.unsetDeposit("usdc");
    /// if (result.error) {
    ///   console.error("Unset failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// ```
    #[wasm_export(js_name = "unsetDeposit", unchecked_return_type = "void")]
    pub fn unset_deposit(
        &mut self,
        #[wasm_export(param_description = "Token key to remove deposit for")] token: String,
    ) -> Result<(), GuiError> {
        self.deposits.remove(&token);
        self.execute_state_update_callback()?;
        Ok(())
    }

    /// Gets preset amounts available for a specific deposit token.
    ///
    /// Returns the preset values configured for this token in the deployment,
    /// useful for building quick-select interfaces.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = gui.getDepositPresets("usdc");
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// // items are preset amounts
    /// const [preset1, preset2, ...] = result.value;
    /// ```
    #[wasm_export(
        js_name = "getDepositPresets",
        unchecked_return_type = "string[]",
        return_description = "Array of preset amounts, or empty if no presets"
    )]
    pub fn get_deposit_presets(
        &self,
        #[wasm_export(param_description = "Token key from deposits configuration")] key: String,
    ) -> Result<Vec<String>, GuiError> {
        let gui_deposit = self.get_gui_deposit(&key)?;
        Ok(gui_deposit.presets.clone().unwrap_or(vec![]))
    }

    /// Lists tokens that require deposits but haven't been configured.
    ///
    /// Returns token keys for deposits that are required by the deployment
    /// but haven't been set yet. Use this for validation and user guidance.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = gui.getMissingDeposits();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// // items are token keys
    /// const [missing1, missing2, ...] = result.value;
    /// if (missing.length > 0) {
    ///   // Do something with the missing tokens
    /// }
    /// ```
    #[wasm_export(
        js_name = "getMissingDeposits",
        unchecked_return_type = "string[]",
        return_description = "Array of token keys needing deposits"
    )]
    pub fn get_missing_deposits(&self) -> Result<Vec<String>, GuiError> {
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

    /// Checks if any deposits have been configured.
    ///
    /// Quick check to determine if the user has started configuring deposits.
    /// Useful for showing different UI states or progress indicators.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = gui.hasAnyDeposit();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const hasDeposits = result.value;
    /// if (!hasDeposits) {
    ///   // Do something
    /// }
    /// ```
    #[wasm_export(
        js_name = "hasAnyDeposit",
        unchecked_return_type = "boolean",
        return_description = "True if at least one deposit exists"
    )]
    pub fn has_any_deposit(&self) -> Result<bool, GuiError> {
        Ok(!self.deposits.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::tests::{
        initialize_gui, initialize_gui_with_select_tokens, initialize_validation_gui,
    };
    use crate::gui::validation;
    use std::str::FromStr;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    async fn test_get_gui_deposit() {
        let gui = initialize_gui(None).await;

        let deposit = gui.get_gui_deposit("token1").unwrap();
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

        let err = gui.get_gui_deposit("token2").unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::DepositTokenNotFound("token2".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The deposit token 'token2' was not found in the YAML configuration."
        );

        let gui = initialize_gui_with_select_tokens().await;
        let err = gui.get_gui_deposit("token3").unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::DepositTokenNotFound("token3".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The deposit token 'token3' was not found in the YAML configuration."
        );
    }

    #[wasm_bindgen_test]
    async fn test_get_deposits() {
        let mut gui = initialize_gui(None).await;

        gui.set_deposit("token1".to_string(), "999".to_string())
            .await
            .unwrap();

        let deposit = gui.get_deposits().unwrap();
        assert_eq!(deposit.len(), 1);
        assert_eq!(deposit[0].token, "token1");
        assert_eq!(deposit[0].amount, "999");
        assert_eq!(
            deposit[0].address,
            Address::from_str("0xc2132d05d31c914a87c6611c10748aeb04b58e8f").unwrap()
        );
    }

    #[wasm_bindgen_test]
    async fn test_set_deposit() {
        let mut gui = initialize_gui(None).await;

        gui.set_deposit("token1".to_string(), "999".to_string())
            .await
            .unwrap();

        let deposit = gui.get_deposits().unwrap();
        assert_eq!(deposit.len(), 1);
        assert_eq!(deposit[0].token, "token1");
        assert_eq!(deposit[0].amount, "999");
        assert_eq!(
            deposit[0].address,
            Address::from_str("0xc2132d05d31c914a87c6611c10748aeb04b58e8f").unwrap()
        );

        let err = gui
            .set_deposit("token1".to_string(), "".to_string())
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::DepositAmountCannotBeEmpty.to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The deposit amount cannot be an empty string. Please set a valid amount."
        );

        let mut gui = initialize_gui_with_select_tokens().await;
        let err = gui
            .set_deposit("token3".to_string(), "999".to_string())
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::DepositTokenNotFound("token3".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The deposit token 'token3' was not found in the YAML configuration."
        );
    }

    #[wasm_bindgen_test]
    async fn test_unset_deposit() {
        let mut gui = initialize_gui(None).await;

        gui.set_deposit("token1".to_string(), "999".to_string())
            .await
            .unwrap();
        let deposit = gui.get_deposits().unwrap();
        assert_eq!(deposit.len(), 1);

        gui.unset_deposit("token1".to_string()).unwrap();
        assert_eq!(gui.get_deposits().unwrap().len(), 0);
    }

    #[wasm_bindgen_test]
    async fn test_get_deposit_presets() {
        let gui = initialize_gui(None).await;

        let presets = gui.get_deposit_presets("token1".to_string()).unwrap();
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
    #[wasm_bindgen_test]
    async fn test_get_missing_deposits() {
        let gui = initialize_gui(None).await;

        let missing_deposits = gui.get_missing_deposits().unwrap();
        assert_eq!(missing_deposits, vec!["token1".to_string()]);
    }

    #[wasm_bindgen_test]
    async fn test_has_any_deposit() {
        let mut gui = initialize_gui(None).await;

        let has_any_deposit = gui.has_any_deposit().unwrap();
        assert!(!has_any_deposit);

        gui.set_deposit("token1".to_string(), "999".to_string())
            .await
            .unwrap();
        let has_any_deposit = gui.has_any_deposit().unwrap();
        assert!(has_any_deposit);
    }

    #[wasm_bindgen_test]
    async fn test_check_deposits() {
        let mut gui = initialize_gui(None).await;

        gui.set_deposit("token1".to_string(), "999".to_string())
            .await
            .unwrap();
        gui.check_deposits().unwrap();
        gui.unset_deposit("token1".to_string()).unwrap();

        let err = gui.check_deposits().unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::DepositNotSet("T1".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "A deposit for token 'T1' is required but has not been set."
        );

        let gui = initialize_gui_with_select_tokens().await;
        let err = gui.check_deposits().unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::MissingDepositToken("select-token-deployment".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "A deposit for token is required but has not been set for deployment 'select-token-deployment'."
        );
    }

    #[wasm_bindgen_test]
    async fn test_save_deposit_minimum_validation() {
        let mut gui = initialize_validation_gui().await;
        let result = gui
            .set_deposit("token1".to_string(), "50".to_string())
            .await;
        match result {
            Err(GuiError::ValidationError(validation::GuiValidationError::BelowMinimum {
                name,
                value,
                minimum,
            })) => {
                assert_eq!(name, "Token 1");
                assert_eq!(value, "50");
                assert_eq!(minimum, "100");
            }
            _ => panic!("Expected BelowMinimum error"),
        }
        let result = gui
            .set_deposit("token1".to_string(), "100".to_string())
            .await;
        assert!(result.is_ok());
        let result = gui
            .set_deposit("token1".to_string(), "500".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_save_deposit_maximum_validation() {
        let mut gui = initialize_validation_gui().await;
        let result = gui
            .set_deposit("token2".to_string(), "5000".to_string())
            .await;
        assert!(result.is_ok());
        let result = gui
            .set_deposit("token2".to_string(), "10000".to_string())
            .await;
        assert!(result.is_ok());
        let result = gui
            .set_deposit("token2".to_string(), "15000".to_string())
            .await;
        match result {
            Err(GuiError::ValidationError(validation::GuiValidationError::AboveMaximum {
                name,
                value,
                maximum,
            })) => {
                assert_eq!(name, "Token 2");
                assert_eq!(value, "15000");
                assert_eq!(maximum, "10000");
            }
            _ => panic!("Expected AboveMaximum error"),
        }
    }

    #[wasm_bindgen_test]
    async fn test_save_deposit_exclusive_bounds() {
        let mut gui = initialize_validation_gui().await;
        let result = gui.set_deposit("token3".to_string(), "0".to_string()).await;
        match result {
            Err(GuiError::ValidationError(
                validation::GuiValidationError::BelowExclusiveMinimum {
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
        let result = gui
            .set_deposit("token3".to_string(), "0.001".to_string())
            .await;
        assert!(result.is_ok());
        let result = gui
            .set_deposit("token3".to_string(), "49999.999".to_string())
            .await;
        assert!(result.is_ok());
        let result = gui
            .set_deposit("token3".to_string(), "50000".to_string())
            .await;
        match result {
            Err(GuiError::ValidationError(
                validation::GuiValidationError::AboveExclusiveMaximum {
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

    #[wasm_bindgen_test]
    async fn test_save_deposit_multiple_constraints() {
        let mut gui = initialize_validation_gui().await;

        let result = gui
            .set_deposit("token4".to_string(), "50".to_string())
            .await;
        assert!(result.is_ok());

        let result = gui.set_deposit("token4".to_string(), "5".to_string()).await;
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::BelowMinimum { .. }
            ))
        ));

        let result = gui
            .set_deposit("token4".to_string(), "1005".to_string())
            .await;
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::AboveMaximum { .. }
            ))
        ));

        let result = gui
            .set_deposit("token4".to_string(), "10".to_string())
            .await;
        assert!(result.is_ok());

        let result = gui
            .set_deposit("token4".to_string(), "1000".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_save_deposit_no_validation() {
        let mut gui = initialize_validation_gui().await;
        let result = gui.set_deposit("token6".to_string(), "0".to_string()).await;
        assert!(result.is_ok());

        let result = gui
            .set_deposit("token6".to_string(), "123456789.123456789".to_string())
            .await;
        assert!(result.is_ok());

        let result = gui
            .set_deposit("token6".to_string(), "0.00000001".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_save_deposit_invalid_formats() {
        let mut gui = initialize_validation_gui().await;

        let result = gui
            .set_deposit("token1".to_string(), "abc".to_string())
            .await;
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::FloatError(..)
            ))
        ));

        let result = gui
            .set_deposit("token1".to_string(), "12.34.56".to_string())
            .await;
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::FloatError(..)
            ))
        ));

        let result = gui
            .set_deposit("token1".to_string(), "12,345".to_string())
            .await;
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::FloatError(..)
            ))
        ));
    }

    #[wasm_bindgen_test]
    async fn test_save_deposit_edge_cases() {
        let mut gui = initialize_validation_gui().await;
        let result = gui
            .set_deposit("token3".to_string(), "0.001".to_string())
            .await;
        assert!(result.is_ok());
        let result = gui
            .set_deposit("token2".to_string(), "9999.999999999".to_string())
            .await;
        assert!(result.is_ok());
        let result = gui
            .set_deposit("token1".to_string(), "100.00000".to_string())
            .await;
        assert!(result.is_ok());
        let result = gui
            .set_deposit("token1".to_string(), "00100".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_save_deposit_with_presets_and_validation() {
        let mut gui = initialize_validation_gui().await;
        let result = gui
            .set_deposit("token1".to_string(), "200".to_string())
            .await;
        assert!(result.is_ok());
        let deposits = gui.get_deposits().unwrap();
        assert_eq!(deposits.len(), 1);
        assert_eq!(deposits[0].token, "token1");
        assert_eq!(deposits[0].amount, "200");
    }
}
