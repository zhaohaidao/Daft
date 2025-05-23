use common_error::DaftResult;
use daft_core::{
    prelude::{Field, Schema},
    series::Series,
};
use daft_dsl::{
    functions::{ScalarFunction, ScalarUDF},
    ExprRef,
};
use serde::{Deserialize, Serialize};

use super::{evaluate_single_numeric, to_field_single_numeric};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Floor {}

#[typetag::serde]
impl ScalarUDF for Floor {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn name(&self) -> &'static str {
        "floor"
    }

    fn to_field(&self, inputs: &[ExprRef], schema: &Schema) -> DaftResult<Field> {
        to_field_single_numeric(self, inputs, schema)
    }

    fn evaluate(&self, inputs: &[Series]) -> DaftResult<Series> {
        // todo: can't move this one because of floor_div
        evaluate_single_numeric(inputs, Series::floor)
    }
}

#[must_use]
pub fn floor(input: ExprRef) -> ExprRef {
    ScalarFunction::new(Floor {}, vec![input]).into()
}
