use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FieldValuePair {
    pub field: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PairValue {
    pub is_preset: bool,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FieldValue {
    pub field: String,
    pub value: String,
    pub is_preset: bool,
}

impl RaindexOrderBuilder {
    pub fn check_field_values(&mut self) -> Result<(), RaindexOrderBuilderError> {
        let deployment = self.get_current_deployment()?;

        for field in deployment.fields.iter() {
            if self.field_values.contains_key(&field.binding) {
                continue;
            }

            match &field.default {
                Some(default_value) => {
                    self.set_field_value(field.binding.clone(), default_value.clone())?;
                }
                None => {
                    return Err(RaindexOrderBuilderError::FieldValueNotSet(
                        field.name.clone(),
                    ))
                }
            }
        }
        Ok(())
    }

    pub fn set_field_value(
        &mut self,
        field: String,
        value: String,
    ) -> Result<(), RaindexOrderBuilderError> {
        let field_definition = self.get_field_definition(&field)?;

        if let Some(validation) = &field_definition.validation {
            validation::validate_field_value(&field_definition.name, &value, validation)?;
        }

        let value = match field_definition.presets.as_ref() {
            Some(presets) => match presets.iter().position(|p| p.value == value) {
                Some(index) => field_values::PairValue {
                    is_preset: true,
                    value: index.to_string(),
                },
                None => field_values::PairValue {
                    is_preset: false,
                    value,
                },
            },
            None => field_values::PairValue {
                is_preset: false,
                value,
            },
        };

        self.field_values.insert(field, value);

        Ok(())
    }

    pub fn set_field_values(
        &mut self,
        field_values: Vec<FieldValuePair>,
    ) -> Result<(), RaindexOrderBuilderError> {
        for field_value in field_values {
            self.set_field_value(field_value.field, field_value.value)?;
        }
        Ok(())
    }

    pub fn unset_field_value(&mut self, field: String) -> Result<(), RaindexOrderBuilderError> {
        self.field_values.remove(&field);
        Ok(())
    }

    pub fn get_field_value(&self, field: String) -> Result<FieldValue, RaindexOrderBuilderError> {
        let field_value =
            self.field_values
                .get(&field)
                .ok_or(RaindexOrderBuilderError::FieldBindingNotFound(
                    field.clone(),
                ))?;
        let preset = match field_value.is_preset {
            true => {
                let field_definition = self.get_field_definition(&field)?;
                let presets = field_definition
                    .presets
                    .ok_or(RaindexOrderBuilderError::BindingHasNoPresets(field.clone()))?;
                presets
                    .iter()
                    .find(|preset| preset.id == field_value.value)
                    .ok_or(RaindexOrderBuilderError::InvalidPreset)?
                    .clone()
            }
            false => OrderBuilderPresetCfg {
                id: "".to_string(),
                name: None,
                value: field_value.value.clone(),
            },
        };
        Ok(FieldValue {
            field: field.clone(),
            value: preset.value.clone(),
            is_preset: field_value.is_preset,
        })
    }

    pub fn get_all_field_values(&self) -> Result<Vec<FieldValue>, RaindexOrderBuilderError> {
        let mut result = Vec::new();
        for (binding, _) in self.field_values.iter() {
            let field_value = self.get_field_value(binding.clone())?;
            result.push(field_value);
        }
        Ok(result)
    }

    pub fn get_field_definition(
        &self,
        field: &str,
    ) -> Result<OrderBuilderFieldDefinitionCfg, RaindexOrderBuilderError> {
        let deployment = self.get_current_deployment()?;
        let field_definition = deployment
            .fields
            .iter()
            .find(|field_definition| field_definition.binding == field)
            .ok_or(RaindexOrderBuilderError::FieldBindingNotFound(
                field.to_string(),
            ))?;
        Ok(field_definition.clone())
    }

    pub fn get_all_field_definitions(
        &self,
        filter_defaults: Option<bool>,
    ) -> Result<Vec<OrderBuilderFieldDefinitionCfg>, RaindexOrderBuilderError> {
        let deployment = self.get_current_deployment()?;
        let mut field_definitions = deployment.fields.clone();

        match filter_defaults {
            Some(true) => field_definitions.retain(|field| field.default.is_some()),
            Some(false) => field_definitions.retain(|field| field.default.is_none()),
            None => (),
        }

        field_definitions.sort_by(|a, b| a.binding.cmp(&b.binding));

        Ok(field_definitions)
    }

    pub fn get_missing_field_values(
        &self,
    ) -> Result<Vec<OrderBuilderFieldDefinitionCfg>, RaindexOrderBuilderError> {
        let deployment = self.get_current_deployment()?;
        let mut missing_field_values = Vec::new();

        for field in deployment.fields.iter() {
            if !self.field_values.contains_key(&field.binding) {
                missing_field_values.push(field.clone());
            }
        }
        Ok(missing_field_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raindex_order_builder::tests::{initialize_builder, initialize_validation_builder};
    use crate::raindex_order_builder::validation;

    #[tokio::test]
    async fn test_set_get_field_value() {
        let mut builder = initialize_builder(None).await;

        builder
            .set_field_value("binding-1".to_string(), "some-default-value".to_string())
            .unwrap();
        builder
            .set_field_value("binding-2".to_string(), "99.2".to_string())
            .unwrap();

        let field_value = builder.get_field_value("binding-1".to_string()).unwrap();
        assert_eq!(field_value.field, "binding-1");
        assert_eq!(field_value.value, "some-default-value");
        assert!(!field_value.is_preset);

        let field_value = builder.get_field_value("binding-2".to_string()).unwrap();
        assert_eq!(field_value.field, "binding-2");
        assert_eq!(field_value.value, "99.2");
        assert!(field_value.is_preset);
    }

    #[tokio::test]
    async fn test_set_get_all_field_values() {
        let mut builder = initialize_builder(None).await;

        builder
            .set_field_values(vec![
                FieldValuePair {
                    field: "binding-1".to_string(),
                    value: "some-default-value".to_string(),
                },
                FieldValuePair {
                    field: "binding-2".to_string(),
                    value: "99.2".to_string(),
                },
            ])
            .unwrap();

        let field_values = builder.get_all_field_values().unwrap();
        assert_eq!(field_values.len(), 2);
        assert_eq!(field_values[0].field, "binding-1");
        assert_eq!(field_values[0].value, "some-default-value");
        assert!(!field_values[0].is_preset);
        assert_eq!(field_values[1].field, "binding-2");
        assert_eq!(field_values[1].value, "99.2");
        assert!(field_values[1].is_preset);
    }

    fn get_binding_1() -> OrderBuilderFieldDefinitionCfg {
        OrderBuilderFieldDefinitionCfg {
            binding: String::from("binding-1"),
            name: String::from("Field 1 name"),
            description: Some(String::from("Field 1 description")),
            presets: Some(Vec::from([
                OrderBuilderPresetCfg {
                    id: String::from("0"),
                    name: Some(String::from("Preset 1")),
                    value: String::from("0x1234567890abcdef1234567890abcdef12345678"),
                },
                OrderBuilderPresetCfg {
                    id: String::from("1"),
                    name: Some(String::from("Preset 2")),
                    value: String::from("false"),
                },
                OrderBuilderPresetCfg {
                    id: String::from("2"),
                    name: Some(String::from("Preset 3")),
                    value: String::from("some-string"),
                },
            ])),
            default: Some(String::from("some-default-value")),
            show_custom_field: None,
            validation: None,
        }
    }

    fn get_binding_2() -> OrderBuilderFieldDefinitionCfg {
        OrderBuilderFieldDefinitionCfg {
            binding: String::from("binding-2"),
            name: String::from("Field 2 name"),
            description: Some(String::from("Field 2 description")),
            presets: Some(Vec::from([
                OrderBuilderPresetCfg {
                    id: String::from("0"),
                    name: None,
                    value: String::from("99.2"),
                },
                OrderBuilderPresetCfg {
                    id: String::from("1"),
                    name: None,
                    value: String::from("582.1"),
                },
                OrderBuilderPresetCfg {
                    id: String::from("2"),
                    name: None,
                    value: String::from("648.239"),
                },
            ])),
            default: None,
            show_custom_field: Some(true),
            validation: None,
        }
    }

    #[tokio::test]
    async fn test_unset_field_value() {
        let mut builder = initialize_builder(None).await;

        builder
            .set_field_value("binding-1".to_string(), "some-default-value".to_string())
            .unwrap();
        builder.get_field_value("binding-1".to_string()).unwrap();

        builder.unset_field_value("binding-1".to_string()).unwrap();
        let err = builder
            .get_field_value("binding-1".to_string())
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            RaindexOrderBuilderError::FieldBindingNotFound("binding-1".to_string()).to_string()
        );
    }

    #[tokio::test]
    async fn test_get_field_definition() {
        let builder = initialize_builder(None).await;

        let field_definition = builder.get_field_definition("binding-1").unwrap();
        assert_eq!(field_definition, get_binding_1());

        let field_definition = builder.get_field_definition("binding-2").unwrap();
        assert_eq!(field_definition, get_binding_2());

        let err = builder.get_field_definition("invalid-binding").unwrap_err();
        assert_eq!(
            err.to_string(),
            RaindexOrderBuilderError::FieldBindingNotFound("invalid-binding".to_string())
                .to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The field binding 'invalid-binding' could not be found in the YAML configuration."
        );
    }

    #[tokio::test]
    async fn test_get_all_field_definitions() {
        let builder = initialize_builder(None).await;

        let field_definitions = builder.get_all_field_definitions(None).unwrap();
        assert_eq!(field_definitions.len(), 2);
        assert_eq!(field_definitions[0], get_binding_1());
        assert_eq!(field_definitions[1], get_binding_2());

        let field_definitions = builder.get_all_field_definitions(Some(true)).unwrap();
        assert_eq!(field_definitions.len(), 1);
        assert_eq!(field_definitions[0], get_binding_1());

        let field_definitions = builder.get_all_field_definitions(Some(false)).unwrap();
        assert_eq!(field_definitions.len(), 1);
        assert_eq!(field_definitions[0], get_binding_2());
    }

    #[tokio::test]
    async fn test_get_missing_field_values() {
        let mut builder = initialize_builder(None).await;

        let field_values = builder.get_missing_field_values().unwrap();
        assert_eq!(field_values.len(), 2);
        assert_eq!(field_values[0], get_binding_1());
        assert_eq!(field_values[1], get_binding_2());

        builder
            .set_field_value("binding-1".to_string(), "some-default-value".to_string())
            .unwrap();

        let field_values = builder.get_missing_field_values().unwrap();
        assert_eq!(field_values.len(), 1);
        assert_eq!(field_values[0], get_binding_2());
    }

    #[tokio::test]
    async fn test_check_field_values() {
        let mut builder = initialize_builder(None).await;

        let err = builder.check_field_values().unwrap_err();
        assert_eq!(
            err.to_string(),
            RaindexOrderBuilderError::FieldValueNotSet("Field 2 name".to_string()).to_string()
        );

        builder
            .set_field_value("binding-2".to_string(), "99.2".to_string())
            .unwrap();
        let res = builder.check_field_values();
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_set_field_value_number_minimum_maximum() {
        let mut builder = initialize_validation_builder().await;

        let result = builder.set_field_value("price-field".to_string(), "50.00".to_string());
        assert!(result.is_ok());

        let result = builder.set_field_value("price-field".to_string(), "5.00".to_string());
        match result {
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::BelowMinimum {
                    name,
                    value,
                    minimum,
                },
            )) => {
                assert_eq!(name, "Price Field");
                assert_eq!(value, "5.00");
                assert_eq!(minimum, "10");
            }
            _ => panic!("Expected BelowMinimum error"),
        }

        let result = builder.set_field_value("price-field".to_string(), "1500.00".to_string());
        match result {
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::AboveMaximum {
                    name,
                    value,
                    maximum,
                },
            )) => {
                assert_eq!(name, "Price Field");
                assert_eq!(value, "1500.00");
                assert_eq!(maximum, "1000");
            }
            _ => panic!("Expected AboveMaximum error"),
        }

        let result = builder.set_field_value("price-field".to_string(), "10".to_string());
        assert!(result.is_ok());

        let result = builder.set_field_value("price-field".to_string(), "1000".to_string());
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_field_value_number_exclusive_bounds() {
        let mut builder = initialize_validation_builder().await;

        let result = builder.set_field_value("quantity-field".to_string(), "0".to_string());
        match result {
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::BelowExclusiveMinimum {
                    name,
                    value,
                    exclusive_minimum,
                },
            )) => {
                assert_eq!(name, "Quantity Field");
                assert_eq!(value, "0");
                assert_eq!(exclusive_minimum, "0");
            }
            _ => panic!("Expected BelowExclusiveMinimum error"),
        }

        let result = builder.set_field_value("quantity-field".to_string(), "0.001".to_string());
        assert!(result.is_ok());

        let result = builder.set_field_value("quantity-field".to_string(), "100000".to_string());
        match result {
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::AboveExclusiveMaximum {
                    name,
                    value,
                    exclusive_maximum,
                },
            )) => {
                assert_eq!(name, "Quantity Field");
                assert_eq!(value, "100000");
                assert_eq!(exclusive_maximum, "100000");
            }
            _ => panic!("Expected AboveExclusiveMaximum error"),
        }

        let result = builder.set_field_value("quantity-field".to_string(), "99999.999".to_string());
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_field_value_boolean() {
        let mut builder = initialize_validation_builder().await;

        let result = builder.set_field_value("enabled-field".to_string(), "true".to_string());
        assert!(result.is_ok());

        let result = builder.set_field_value("enabled-field".to_string(), "false".to_string());
        assert!(result.is_ok());

        let test_cases = vec![
            "True", "FALSE", "yes", "no", "on", "off", "", " true", "true ", "1", "0",
        ];

        for test_value in test_cases {
            let result =
                builder.set_field_value("enabled-field".to_string(), test_value.to_string());
            match result {
                Err(RaindexOrderBuilderError::ValidationError(
                    validation::BuilderValidationError::InvalidBoolean { name, value },
                )) => {
                    assert_eq!(name, "Enabled");
                    assert_eq!(value, test_value);
                }
                _ => panic!("Expected InvalidBoolean error for value: {}", test_value),
            }
        }
    }

    #[tokio::test]
    async fn test_set_field_value_preset_with_validation() {
        let mut builder = initialize_validation_builder().await;

        let result = builder.set_field_value("preset-number-field".to_string(), "100".to_string());
        assert!(result.is_ok());
        let field_value = builder
            .get_field_value("preset-number-field".to_string())
            .unwrap();
        assert!(field_value.is_preset);
        assert_eq!(field_value.value, "100");

        let result = builder.set_field_value("preset-number-field".to_string(), "120".to_string());
        assert!(result.is_ok());
        let field_value = builder
            .get_field_value("preset-number-field".to_string())
            .unwrap();
        assert!(!field_value.is_preset);
        assert_eq!(field_value.value, "120");

        let result = builder.set_field_value("preset-number-field".to_string(), "5".to_string());
        assert!(matches!(
            result,
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::BelowMinimum { .. }
            ))
        ));
    }

    #[tokio::test]
    async fn test_set_field_value_no_validation() {
        let mut builder = initialize_validation_builder().await;

        let test_values = vec![
            "123",
            "abc",
            "true",
            "false",
            "",
            "!@#$%^&*()",
            "very long string with many characters",
            "0",
            "🦀 Rust 🦀",
        ];

        for value in test_values {
            let result =
                builder.set_field_value("no-validation-field".to_string(), value.to_string());
            assert!(result.is_ok(), "Failed to save value: {}", value);

            let field_value = builder
                .get_field_value("no-validation-field".to_string())
                .unwrap();
            assert_eq!(field_value.value, value);
        }
    }

    #[tokio::test]
    async fn test_set_field_value_number_complex_constraints() {
        let mut builder = initialize_validation_builder().await;

        let result = builder.set_field_value("percentage-field".to_string(), "50.5".to_string());
        assert!(result.is_ok());

        let result = builder.set_field_value("percentage-field".to_string(), "0".to_string());
        assert!(result.is_ok());

        let result = builder.set_field_value("percentage-field".to_string(), "100".to_string());
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_field_value_number_invalid_formats() {
        let mut builder = initialize_validation_builder().await;

        let result = builder.set_field_value("simple-number".to_string(), "".to_string());
        match result {
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::InvalidNumber { name, value },
            )) => {
                assert_eq!(name, "Simple Number");
                assert_eq!(value, "");
            }
            _ => panic!("Expected InvalidNumber error"),
        }

        let result = builder.set_field_value("simple-number".to_string(), "abc".to_string());
        assert!(matches!(
            result,
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::FloatError(..)
            ))
        ));

        let result = builder.set_field_value("simple-number".to_string(), "12.34.56".to_string());
        assert!(matches!(
            result,
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::FloatError(..)
            ))
        ));

        let result = builder.set_field_value("simple-number".to_string(), "١٢٣".to_string());
        assert!(matches!(
            result,
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::FloatError(..)
            ))
        ));
    }

    #[tokio::test]
    async fn test_set_field_value_string_length_constraints() {
        let mut builder = initialize_validation_builder().await;

        let result = builder.set_field_value("username-field".to_string(), "john_doe".to_string());
        assert!(result.is_ok());

        let result = builder.set_field_value("username-field".to_string(), "jo".to_string());
        match result {
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::StringTooShort {
                    name,
                    length,
                    minimum,
                },
            )) => {
                assert_eq!(name, "Username");
                assert_eq!(length, 2);
                assert_eq!(minimum, 3);
            }
            _ => panic!("Expected StringTooShort error"),
        }

        let result = builder.set_field_value(
            "username-field".to_string(),
            "this_username_is_way_too_long".to_string(),
        );
        match result {
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::StringTooLong {
                    name,
                    length,
                    maximum,
                },
            )) => {
                assert_eq!(name, "Username");
                assert_eq!(length, 29);
                assert_eq!(maximum, 20);
            }
            _ => panic!("Expected StringTooLong error"),
        }

        let result = builder.set_field_value("username-field".to_string(), "abc".to_string());
        assert!(result.is_ok());

        let result = builder.set_field_value("username-field".to_string(), "a".repeat(20));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_field_value_string_edge_cases() {
        let mut builder = initialize_validation_builder().await;

        let result = builder.set_field_value("description-field".to_string(), "".to_string());
        assert!(result.is_ok());

        let result = builder.set_field_value("code-field".to_string(), "".to_string());
        assert!(matches!(
            result,
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::StringTooShort { .. }
            ))
        ));

        let result = builder.set_field_value("description-field".to_string(), "a".repeat(500));
        assert!(result.is_ok());

        let result = builder.set_field_value("description-field".to_string(), "a".repeat(501));
        assert!(matches!(
            result,
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::StringTooLong { .. }
            ))
        ));

        let result =
            builder.set_field_value("username-field".to_string(), "🦀🦀🦀rust🦀🦀🦀".to_string());
        assert!(matches!(
            result,
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::StringTooLong { .. }
            ))
        ));

        let result = builder.set_field_value(
            "any-string".to_string(),
            "Any value at all!@#$%^&*()".to_string(),
        );
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_field_value_string_preset_with_validation() {
        let mut builder = initialize_validation_builder().await;

        let result =
            builder.set_field_value("preset-string-field".to_string(), "alpha".to_string());
        assert!(result.is_ok());
        let field_value = builder
            .get_field_value("preset-string-field".to_string())
            .unwrap();
        assert!(field_value.is_preset);

        let result =
            builder.set_field_value("preset-string-field".to_string(), "custom".to_string());
        assert!(result.is_ok());
        let field_value = builder
            .get_field_value("preset-string-field".to_string())
            .unwrap();
        assert!(!field_value.is_preset);

        let result = builder.set_field_value("preset-string-field".to_string(), "xyz".to_string());
        assert!(matches!(
            result,
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::StringTooShort { .. }
            ))
        ));

        let result = builder.set_field_value(
            "preset-string-field".to_string(),
            "verylongvalue".to_string(),
        );
        assert!(matches!(
            result,
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::StringTooLong { .. }
            ))
        ));
    }

    #[tokio::test]
    async fn test_set_field_values_batch_with_validation() {
        let mut builder = initialize_validation_builder().await;

        let valid_batch = vec![
            FieldValuePair {
                field: "price-field".to_string(),
                value: "100.00".to_string(),
            },
            FieldValuePair {
                field: "username-field".to_string(),
                value: "valid_user".to_string(),
            },
            FieldValuePair {
                field: "enabled-field".to_string(),
                value: "true".to_string(),
            },
        ];

        let result = builder.set_field_values(valid_batch);
        assert!(result.is_ok());

        let invalid_batch = vec![
            FieldValuePair {
                field: "price-field".to_string(),
                value: "100.00".to_string(),
            },
            FieldValuePair {
                field: "username-field".to_string(),
                value: "ab".to_string(),
            },
            FieldValuePair {
                field: "enabled-field".to_string(),
                value: "true".to_string(),
            },
        ];

        let result = builder.set_field_values(invalid_batch);
        assert!(matches!(
            result,
            Err(RaindexOrderBuilderError::ValidationError(
                validation::BuilderValidationError::StringTooShort { .. }
            ))
        ));

        let field_result = builder.get_field_value("price-field".to_string());
        assert!(field_result.is_ok());
    }

    #[tokio::test]
    async fn test_very_precise_decimal_validation() {
        let mut builder = initialize_validation_builder().await;

        let result = builder.set_field_value(
            "simple-number".to_string(),
            "0.123456789012345678".to_string(),
        );
        assert!(result.is_ok());

        let result = builder.set_field_value("price-field".to_string(), "999.99".to_string());
        assert!(result.is_ok());
    }
}
