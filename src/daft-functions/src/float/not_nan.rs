use common_error::{DaftError, DaftResult};
use daft_core::{
    array::ops::DaftNotNan,
    prelude::{DataType, Field, Schema},
    series::{IntoSeries, Series},
    with_match_float_and_null_daft_types,
};
use daft_dsl::{
    functions::{ScalarFunction, ScalarUDF},
    ExprRef,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NotNan {}

#[typetag::serde]
impl ScalarUDF for NotNan {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn name(&self) -> &'static str {
        "not_nan"
    }

    fn to_field(&self, inputs: &[ExprRef], schema: &Schema) -> DaftResult<Field> {
        match inputs {
            [data] => match data.to_field(schema) {
                Ok(data_field) => match &data_field.dtype {
                    // DataType::Float16 |
                    DataType::Null | DataType::Float32 | DataType::Float64 => {
                        Ok(Field::new(data_field.name, DataType::Boolean))
                    }
                    _ => Err(DaftError::TypeError(format!(
                        "Expects input to not_nan to be float, but received {data_field}",
                    ))),
                },
                Err(e) => Err(e),
            },
            _ => Err(DaftError::SchemaMismatch(format!(
                "Expected 1 input args, got {}",
                inputs.len()
            ))),
        }
    }

    fn evaluate(&self, inputs: &[Series]) -> DaftResult<Series> {
        match inputs {
            [data] => {
                with_match_float_and_null_daft_types!(data.data_type(), |$T| {
                    Ok(DaftNotNan::not_nan(data.downcast::<<$T as DaftDataType>::ArrayType>()?)?.into_series())
                })
            }
            _ => Err(DaftError::ValueError(format!(
                "Expected 1 input args, got {}",
                inputs.len()
            ))),
        }
    }
}

#[must_use]
pub fn not_nan(input: ExprRef) -> ExprRef {
    ScalarFunction::new(NotNan {}, vec![input]).into()
}
