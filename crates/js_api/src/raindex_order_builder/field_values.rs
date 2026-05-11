use super::*;
use rain_orderbook_common::raindex_order_builder::field_values as inner_fv;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct FieldValuePair {
    field: String,
    value: String,
}
impl_wasm_traits!(FieldValuePair);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct PairValue {
    pub is_preset: bool,
    pub value: String,
}
impl_wasm_traits!(PairValue);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct FieldValue {
    pub field: String,
    pub value: String,
    pub is_preset: bool,
}
impl_wasm_traits!(FieldValue);

impl RaindexOrderBuilder {
    pub fn check_field_values(&mut self) -> Result<(), RaindexOrderBuilderWasmError> {
        self.inner.check_field_values()?;
        Ok(())
    }
}

#[wasm_export]
impl RaindexOrderBuilder {
    #[wasm_export(js_name = "setFieldValue", unchecked_return_type = "void")]
    pub fn set_field_value(
        &mut self,
        #[wasm_export(param_description = "Field identifier from the YAML configuration")]
        field: String,
        #[wasm_export(param_description = "Value to save (can be a preset value or custom input)")]
        value: String,
    ) -> Result<(), RaindexOrderBuilderWasmError> {
        self.inner.set_field_value(field, value)?;
        self.execute_state_update_callback()?;
        Ok(())
    }

    #[wasm_export(js_name = "setFieldValues", unchecked_return_type = "void")]
    pub fn set_field_values(
        &mut self,
        #[wasm_export(param_description = "Array of field-value pairs to save")] field_values: Vec<
            FieldValuePair,
        >,
    ) -> Result<(), RaindexOrderBuilderWasmError> {
        for fv in field_values {
            self.inner.set_field_value(fv.field, fv.value)?;
            self.execute_state_update_callback()?;
        }
        Ok(())
    }

    #[wasm_export(js_name = "unsetFieldValue", unchecked_return_type = "void")]
    pub fn unset_field_value(
        &mut self,
        #[wasm_export(param_description = "Field identifier from the YAML configuration")]
        field: String,
    ) -> Result<(), RaindexOrderBuilderWasmError> {
        self.inner.unset_field_value(field)?;
        self.execute_state_update_callback()?;
        Ok(())
    }

    #[wasm_export(
        js_name = "getFieldValue",
        unchecked_return_type = "FieldValue",
        return_description = "Field value with the identifier, value, and preset flag"
    )]
    pub fn get_field_value(
        &self,
        #[wasm_export(param_description = "Field identifier from the YAML configuration")]
        field: String,
    ) -> Result<inner_fv::FieldValue, RaindexOrderBuilderWasmError> {
        Ok(self.inner.get_field_value(field)?)
    }

    #[wasm_export(
        js_name = "getAllFieldValues",
        unchecked_return_type = "FieldValue[]",
        return_description = "Array of all configured field values"
    )]
    pub fn get_all_field_values(
        &self,
    ) -> Result<Vec<inner_fv::FieldValue>, RaindexOrderBuilderWasmError> {
        Ok(self.inner.get_all_field_values()?)
    }

    #[wasm_export(
        js_name = "getFieldDefinition",
        unchecked_return_type = "OrderBuilderFieldDefinitionCfg",
        return_description = "Complete field configuration"
    )]
    pub fn get_field_definition(
        &self,
        #[wasm_export(param_description = "Field binding identifier to look up")] field: &str,
    ) -> Result<OrderBuilderFieldDefinitionCfg, RaindexOrderBuilderWasmError> {
        Ok(self.inner.get_field_definition(field)?)
    }

    #[wasm_export(
        js_name = "getAllFieldDefinitions",
        unchecked_return_type = "OrderBuilderFieldDefinitionCfg[]",
        return_description = "Filtered field definitions"
    )]
    pub fn get_all_field_definitions(
        &self,
        #[wasm_export(
            param_description = "Optional filter: **true** for fields with defaults, **false** for fields without defaults, **undefined** for all"
        )]
        filter_defaults: Option<bool>,
    ) -> Result<Vec<OrderBuilderFieldDefinitionCfg>, RaindexOrderBuilderWasmError> {
        Ok(self.inner.get_all_field_definitions(filter_defaults)?)
    }

    #[wasm_export(
        js_name = "getMissingFieldValues",
        unchecked_return_type = "OrderBuilderFieldDefinitionCfg[]",
        return_description = "Array of field definitions that need to be set"
    )]
    pub fn get_missing_field_values(
        &self,
    ) -> Result<Vec<OrderBuilderFieldDefinitionCfg>, RaindexOrderBuilderWasmError> {
        Ok(self.inner.get_missing_field_values()?)
    }
}
