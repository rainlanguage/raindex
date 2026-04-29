use rain_math_float::{Float, FloatError};
use rain_orderbook_app_settings::order_builder::{DepositValidationCfg, FieldValueValidationCfg};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuilderValidationError {
    #[error("The {name} field contains an invalid number: '{value}'. Please enter a valid numeric value.")]
    InvalidNumber { name: String, value: String },

    #[error(
        "The {name} field value '{value}' is too small. The minimum allowed value is {minimum}."
    )]
    BelowMinimum {
        name: String,
        value: String,
        minimum: String,
    },

    #[error("The {name} field value '{value}' must be greater than {exclusive_minimum}. Please enter a larger value.")]
    BelowExclusiveMinimum {
        name: String,
        value: String,
        exclusive_minimum: String,
    },

    #[error(
        "The {name} field value '{value}' is too large. The maximum allowed value is {maximum}."
    )]
    AboveMaximum {
        name: String,
        value: String,
        maximum: String,
    },

    #[error("The {name} field value '{value}' must be less than {exclusive_maximum}. Please enter a smaller value.")]
    AboveExclusiveMaximum {
        name: String,
        value: String,
        exclusive_maximum: String,
    },

    #[error("The {name} field text is too short ({length} characters). It must be at least {minimum} characters long.")]
    StringTooShort {
        name: String,
        length: u32,
        minimum: u32,
    },

    #[error("The {name} field text is too long ({length} characters). It cannot exceed {maximum} characters.")]
    StringTooLong {
        name: String,
        length: u32,
        maximum: u32,
    },

    #[error("The {name} field contains an invalid boolean value: '{value}'. Please enter either 'true' or 'false'.")]
    InvalidBoolean { name: String, value: String },

    #[error(transparent)]
    FloatError(#[from] FloatError),
}

pub fn validate_field_value(
    field_name: &str,
    value: &str,
    validation: &FieldValueValidationCfg,
) -> Result<(), BuilderValidationError> {
    match validation {
        FieldValueValidationCfg::Number {
            minimum,
            exclusive_minimum,
            maximum,
            exclusive_maximum,
        } => validate_number(
            field_name,
            value,
            minimum,
            exclusive_minimum,
            maximum,
            exclusive_maximum,
        ),
        FieldValueValidationCfg::String {
            min_length,
            max_length,
        } => validate_string(field_name, value, min_length, max_length),
        FieldValueValidationCfg::Boolean => validate_boolean(field_name, value),
    }
}

pub fn validate_deposit_amount(
    token_name: &str,
    amount: &str,
    validation: &DepositValidationCfg,
) -> Result<(), BuilderValidationError> {
    validate_number(
        token_name,
        amount,
        &validation.minimum,
        &validation.exclusive_minimum,
        &validation.maximum,
        &validation.exclusive_maximum,
    )
}

fn validate_number(
    name: &str,
    value: &str,
    minimum: &Option<String>,
    exclusive_minimum: &Option<String>,
    maximum: &Option<String>,
    exclusive_maximum: &Option<String>,
) -> Result<(), BuilderValidationError> {
    if value.is_empty() {
        return Err(BuilderValidationError::InvalidNumber {
            name: name.to_string(),
            value: value.to_string(),
        });
    }

    let float_value = Float::parse(value.to_string())?;
    let zero = Float::parse("0".to_string())?;

    if float_value.lt(zero)? {
        return Err(BuilderValidationError::InvalidNumber {
            name: name.to_string(),
            value: value.to_string(),
        });
    }

    if let Some(min) = minimum {
        let float_min = Float::parse(min.clone())?;
        if float_value.lt(float_min)? {
            return Err(BuilderValidationError::BelowMinimum {
                name: name.to_string(),
                value: value.to_string(),
                minimum: min.clone(),
            });
        }
    }

    if let Some(exclusive_min) = exclusive_minimum {
        let exclusive_min_float = Float::parse(exclusive_min.clone())?;
        if float_value.lte(exclusive_min_float)? {
            return Err(BuilderValidationError::BelowExclusiveMinimum {
                name: name.to_string(),
                value: value.to_string(),
                exclusive_minimum: exclusive_min.clone(),
            });
        }
    }

    if let Some(max) = maximum {
        let max_float = Float::parse(max.clone())?;
        if float_value.gt(max_float)? {
            return Err(BuilderValidationError::AboveMaximum {
                name: name.to_string(),
                value: value.to_string(),
                maximum: max.clone(),
            });
        }
    }

    if let Some(exclusive_max) = exclusive_maximum {
        let exclusive_max_float = Float::parse(exclusive_max.clone())?;
        if float_value.gte(exclusive_max_float)? {
            return Err(BuilderValidationError::AboveExclusiveMaximum {
                name: name.to_string(),
                value: value.to_string(),
                exclusive_maximum: exclusive_max.clone(),
            });
        }
    }

    Ok(())
}

fn validate_string(
    name: &str,
    value: &str,
    min_length: &Option<u32>,
    max_length: &Option<u32>,
) -> Result<(), BuilderValidationError> {
    let trimmed_value = value.trim();
    let length = trimmed_value.len() as u32;

    if let Some(min) = min_length {
        if length < *min {
            return Err(BuilderValidationError::StringTooShort {
                name: name.to_string(),
                length,
                minimum: *min,
            });
        }
    }

    if let Some(max) = max_length {
        if length > *max {
            return Err(BuilderValidationError::StringTooLong {
                name: name.to_string(),
                length,
                maximum: *max,
            });
        }
    }

    Ok(())
}

fn validate_boolean(name: &str, value: &str) -> Result<(), BuilderValidationError> {
    match value {
        "true" | "false" => Ok(()),
        _ => Err(BuilderValidationError::InvalidBoolean {
            name: name.to_string(),
            value: value.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_app_settings::order_builder::FieldValueValidationCfg;

    #[test]
    fn test_validate_number_minimum() {
        let result = validate_number(
            "Test Field",
            "5",
            &Some("10".to_string()),
            &None,
            &None,
            &None,
        );
        match &result {
            Err(BuilderValidationError::BelowMinimum {
                name,
                value,
                minimum,
            }) => {
                assert_eq!(name, "Test Field");
                assert_eq!(value, "5");
                assert_eq!(minimum, "10");
            }
            _ => panic!("Expected BelowMinimum error"),
        }

        let result = validate_number(
            "Test Field",
            "10",
            &Some("10".to_string()),
            &None,
            &None,
            &None,
        );
        assert!(result.is_ok());

        let result = validate_number(
            "Test Field",
            "15",
            &Some("10".to_string()),
            &None,
            &None,
            &None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_number_exclusive_minimum() {
        let result = validate_number("Price", "10", &None, &Some("10".to_string()), &None, &None);
        match &result {
            Err(BuilderValidationError::BelowExclusiveMinimum {
                name,
                value,
                exclusive_minimum,
            }) => {
                assert_eq!(name, "Price");
                assert_eq!(value, "10");
                assert_eq!(exclusive_minimum, "10");
            }
            _ => panic!("Expected BelowExclusiveMinimum error"),
        }

        let result = validate_number(
            "Price",
            "10.1",
            &None,
            &Some("10".to_string()),
            &None,
            &None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_number_maximum() {
        let result = validate_number(
            "Amount",
            "101",
            &None,
            &None,
            &Some("100".to_string()),
            &None,
        );
        match &result {
            Err(BuilderValidationError::AboveMaximum {
                name,
                value,
                maximum,
            }) => {
                assert_eq!(name, "Amount");
                assert_eq!(value, "101");
                assert_eq!(maximum, "100");
            }
            _ => panic!("Expected AboveMaximum error"),
        }

        let result = validate_number(
            "Amount",
            "100",
            &None,
            &None,
            &Some("100".to_string()),
            &None,
        );
        assert!(result.is_ok());

        let result = validate_number(
            "Amount",
            "99.9",
            &None,
            &None,
            &Some("100".to_string()),
            &None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_number_exclusive_maximum() {
        let result = validate_number(
            "Token Amount",
            "100",
            &None,
            &None,
            &None,
            &Some("100".to_string()),
        );
        match &result {
            Err(BuilderValidationError::AboveExclusiveMaximum {
                name,
                value,
                exclusive_maximum,
            }) => {
                assert_eq!(name, "Token Amount");
                assert_eq!(value, "100");
                assert_eq!(exclusive_maximum, "100");
            }
            _ => panic!("Expected AboveExclusiveMaximum error"),
        }

        let result = validate_number(
            "Token Amount",
            "99.999",
            &None,
            &None,
            &None,
            &Some("100".to_string()),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_number_combined_constraints() {
        let result = validate_number(
            "Complex Field",
            "50",
            &Some("10".to_string()),
            &None,
            &Some("100".to_string()),
            &None,
        );
        assert!(result.is_ok());

        let result = validate_number(
            "Complex Field",
            "5",
            &Some("10".to_string()),
            &None,
            &Some("100".to_string()),
            &None,
        );
        assert!(matches!(
            result,
            Err(BuilderValidationError::BelowMinimum { .. })
        ));

        let result = validate_number(
            "Complex Field",
            "105",
            &Some("10".to_string()),
            &None,
            &Some("100".to_string()),
            &None,
        );
        assert!(matches!(
            result,
            Err(BuilderValidationError::AboveMaximum { .. })
        ));
    }

    #[test]
    fn test_validate_number_parsing() {
        let result = validate_number("Test Field", "100.5", &None, &None, &None, &None);
        assert!(result.is_ok());

        let result = validate_number("Test Field", "0.000001", &None, &None, &None, &None);
        assert!(result.is_ok());

        let result = validate_number(
            "Test Field",
            "123456789.123456789",
            &None,
            &None,
            &None,
            &None,
        );
        assert!(result.is_ok());

        let result = validate_number("Test Field", "not a number", &None, &None, &None, &None);
        assert!(matches!(
            result,
            Err(BuilderValidationError::FloatError(..))
        ));

        let result = validate_number("Test Field", "12.34.56", &None, &None, &None, &None);
        assert!(matches!(
            result,
            Err(BuilderValidationError::FloatError(..))
        ));

        let result = validate_number("Test Field", "", &None, &None, &None, &None);
        assert!(matches!(
            result,
            Err(BuilderValidationError::InvalidNumber { .. })
        ));
    }

    #[test]
    fn test_validate_number_decimals() {
        let result = validate_number("USDC Amount", "100.123456", &None, &None, &None, &None);
        assert!(result.is_ok());

        let result = validate_number(
            "ETH Amount",
            "1.123456789012345678",
            &None,
            &None,
            &None,
            &None,
        );
        assert!(result.is_ok());

        let result = validate_number("BTC Amount", "0.12345678", &None, &None, &None, &None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_number_edge_cases() {
        let result = validate_number("Amount", "0", &None, &None, &None, &None);
        assert!(result.is_ok());

        let result = validate_number("Amount", "0.000000000000000001", &None, &None, &None, &None);
        assert!(result.is_ok());

        let result = validate_number(
            "Amount",
            "999999999999999999999999999",
            &None,
            &None,
            &None,
            &None,
        );
        assert!(result.is_ok());

        let result = validate_number("Amount", "100", &None, &None, &None, &None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_number_rejects_negative() {
        let result = validate_number("Amount", "-1", &None, &None, &None, &None);
        assert!(matches!(
            result,
            Err(BuilderValidationError::InvalidNumber { .. })
        ));

        let result = validate_number("Amount", "-0.01", &None, &None, &None, &None);
        assert!(matches!(
            result,
            Err(BuilderValidationError::InvalidNumber { .. })
        ));

        let result = validate_number("Amount", "-100.5", &None, &None, &None, &None);
        assert!(matches!(
            result,
            Err(BuilderValidationError::InvalidNumber { .. })
        ));
    }

    #[test]
    fn test_validate_string_length() {
        let result = validate_string("Username", "hello", &Some(10), &None);
        match &result {
            Err(BuilderValidationError::StringTooShort {
                name,
                length,
                minimum,
            }) => {
                assert_eq!(name, "Username");
                assert_eq!(*length, 5);
                assert_eq!(*minimum, 10);
            }
            _ => panic!("Expected StringTooShort error"),
        }

        let result = validate_string("Username", "hello world", &Some(5), &Some(20));
        assert!(result.is_ok());

        let result = validate_string("Username", "hello world", &Some(11), &None);
        assert!(result.is_ok());

        let result = validate_string("Description", &"a".repeat(100), &None, &Some(50));
        match &result {
            Err(BuilderValidationError::StringTooLong {
                name,
                length,
                maximum,
            }) => {
                assert_eq!(name, "Description");
                assert_eq!(*length, 100);
                assert_eq!(*maximum, 50);
            }
            _ => panic!("Expected StringTooLong error"),
        }
    }

    #[test]
    fn test_validate_string_edge_cases() {
        let result = validate_string("Field", "", &None, &None);
        assert!(result.is_ok());

        let result = validate_string("Field", "", &Some(1), &None);
        assert!(matches!(
            result,
            Err(BuilderValidationError::StringTooShort { .. })
        ));

        let result = validate_string("Field", "12345", &Some(5), &Some(5));
        assert!(result.is_ok());

        let result = validate_string("Field", "🦀", &Some(4), &None);
        assert!(result.is_ok());

        let result = validate_string("Field", "🦀", &Some(5), &None);
        assert!(matches!(
            result,
            Err(BuilderValidationError::StringTooShort { .. })
        ));
    }

    #[test]
    fn test_validate_string_trimming() {
        let result = validate_string("Username", "  hello  ", &Some(3), &Some(10));
        assert!(result.is_ok());

        let result = validate_string("Username", "  hi  ", &Some(5), &None);
        assert!(matches!(
            result,
            Err(BuilderValidationError::StringTooShort { .. })
        ));

        let result = validate_string("Username", "\t\nhello world\t\n", &Some(5), &Some(15));
        assert!(result.is_ok());

        let result = validate_string("Description", "   ", &Some(1), &None);
        assert!(matches!(
            result,
            Err(BuilderValidationError::StringTooShort { .. })
        ));

        let result = validate_string("Field", "  toolong  ", &None, &Some(6));
        assert!(matches!(
            result,
            Err(BuilderValidationError::StringTooLong { .. })
        ));
    }

    #[test]
    fn test_validate_boolean() {
        let result = validate_boolean("Enable Feature", "true");
        assert!(result.is_ok());

        let result = validate_boolean("Enable Feature", "false");
        assert!(result.is_ok());

        let result = validate_boolean("Enable Feature", "yes");
        match &result {
            Err(BuilderValidationError::InvalidBoolean { name, value }) => {
                assert_eq!(name, "Enable Feature");
                assert_eq!(value, "yes");
            }
            _ => panic!("Expected InvalidBoolean error"),
        }

        let result = validate_boolean("Enable Feature", "True");
        assert!(matches!(
            result,
            Err(BuilderValidationError::InvalidBoolean { .. })
        ));

        let result = validate_boolean("Enable Feature", "FALSE");
        assert!(matches!(
            result,
            Err(BuilderValidationError::InvalidBoolean { .. })
        ));

        let result = validate_boolean("Enable Feature", "False");
        assert!(matches!(
            result,
            Err(BuilderValidationError::InvalidBoolean { .. })
        ));

        let result = validate_boolean("Enable Feature", "");
        assert!(matches!(
            result,
            Err(BuilderValidationError::InvalidBoolean { .. })
        ));
    }

    #[test]
    fn test_validate_field_value_number() {
        let validation = FieldValueValidationCfg::Number {
            minimum: Some("10".to_string()),
            exclusive_minimum: None,
            maximum: Some("100".to_string()),
            exclusive_maximum: None,
        };

        let result = validate_field_value("Price Field", "50", &validation);
        assert!(result.is_ok());

        let result = validate_field_value("Price Field", "5", &validation);
        assert!(matches!(
            result,
            Err(BuilderValidationError::BelowMinimum { .. })
        ));

        let result = validate_field_value("Price Field", "102", &validation);
        assert!(matches!(
            result,
            Err(BuilderValidationError::AboveMaximum { .. })
        ));
    }

    #[test]
    fn test_validate_field_value_string() {
        let validation = FieldValueValidationCfg::String {
            min_length: Some(3),
            max_length: Some(10),
        };

        let result = validate_field_value("Name Field", "hello", &validation);
        assert!(result.is_ok());

        let result = validate_field_value("Name Field", "hi", &validation);
        assert!(matches!(
            result,
            Err(BuilderValidationError::StringTooShort { .. })
        ));

        let result = validate_field_value("Name Field", "hello world!", &validation);
        assert!(matches!(
            result,
            Err(BuilderValidationError::StringTooLong { .. })
        ));
    }

    #[test]
    fn test_validate_field_value_boolean() {
        let validation = FieldValueValidationCfg::Boolean;

        let result = validate_field_value("Toggle Field", "true", &validation);
        assert!(result.is_ok());

        let result = validate_field_value("Toggle Field", "false", &validation);
        assert!(result.is_ok());

        let result = validate_field_value("Toggle Field", "maybe", &validation);
        assert!(matches!(
            result,
            Err(BuilderValidationError::InvalidBoolean { .. })
        ));
    }
}
