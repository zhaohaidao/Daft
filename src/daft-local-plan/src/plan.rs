use std::{cmp::max, sync::Arc};

use common_resource_request::ResourceRequest;
use common_scan_info::{Pushdowns, ScanTaskLikeRef};
use daft_core::prelude::*;
use daft_dsl::{AggExpr, ExprRef, WindowExpr, WindowFrame};
use daft_logical_plan::{
    stats::{PlanStats, StatsState},
    InMemoryInfo, OutputFileInfo,
};
use serde::{Deserialize, Serialize};

pub type LocalPhysicalPlanRef = Arc<LocalPhysicalPlan>;
#[derive(Debug, strum::IntoStaticStr, Serialize, Deserialize)]
pub enum LocalPhysicalPlan {
    InMemoryScan(InMemoryScan),
    PhysicalScan(PhysicalScan),
    EmptyScan(EmptyScan),
    Project(Project),
    ActorPoolProject(ActorPoolProject),
    Filter(Filter),
    Limit(Limit),
    Explode(Explode),
    Unpivot(Unpivot),
    Sort(Sort),
    // Split(Split),
    Sample(Sample),
    MonotonicallyIncreasingId(MonotonicallyIncreasingId),
    // Coalesce(Coalesce),
    // Flatten(Flatten),
    // FanoutRandom(FanoutRandom),
    // FanoutByHash(FanoutByHash),
    // FanoutByRange(FanoutByRange),
    // ReduceMerge(ReduceMerge),
    UnGroupedAggregate(UnGroupedAggregate),
    HashAggregate(HashAggregate),
    Pivot(Pivot),
    Concat(Concat),
    HashJoin(HashJoin),
    CrossJoin(CrossJoin),
    // SortMergeJoin(SortMergeJoin),
    // BroadcastJoin(BroadcastJoin),
    PhysicalWrite(PhysicalWrite),
    // TabularWriteJson(TabularWriteJson),
    // TabularWriteCsv(TabularWriteCsv),
    #[cfg(feature = "python")]
    CatalogWrite(CatalogWrite),
    #[cfg(feature = "python")]
    LanceWrite(LanceWrite),
    WindowPartitionOnly(WindowPartitionOnly),
    WindowPartitionAndOrderBy(WindowPartitionAndOrderBy),
    WindowPartitionAndDynamicFrame(WindowPartitionAndDynamicFrame),
}

impl LocalPhysicalPlan {
    #[must_use]
    pub fn name(&self) -> &'static str {
        // uses strum::IntoStaticStr
        self.into()
    }

    #[must_use]
    pub fn arced(self) -> LocalPhysicalPlanRef {
        self.into()
    }

    pub fn get_stats_state(&self) -> &StatsState {
        match self {
            Self::InMemoryScan(InMemoryScan { stats_state, .. })
            | Self::PhysicalScan(PhysicalScan { stats_state, .. })
            | Self::EmptyScan(EmptyScan { stats_state, .. })
            | Self::Project(Project { stats_state, .. })
            | Self::ActorPoolProject(ActorPoolProject { stats_state, .. })
            | Self::Filter(Filter { stats_state, .. })
            | Self::Limit(Limit { stats_state, .. })
            | Self::Explode(Explode { stats_state, .. })
            | Self::Unpivot(Unpivot { stats_state, .. })
            | Self::Sort(Sort { stats_state, .. })
            | Self::Sample(Sample { stats_state, .. })
            | Self::MonotonicallyIncreasingId(MonotonicallyIncreasingId { stats_state, .. })
            | Self::UnGroupedAggregate(UnGroupedAggregate { stats_state, .. })
            | Self::HashAggregate(HashAggregate { stats_state, .. })
            | Self::Pivot(Pivot { stats_state, .. })
            | Self::Concat(Concat { stats_state, .. })
            | Self::HashJoin(HashJoin { stats_state, .. })
            | Self::CrossJoin(CrossJoin { stats_state, .. })
            | Self::PhysicalWrite(PhysicalWrite { stats_state, .. })
            | Self::WindowPartitionOnly(WindowPartitionOnly { stats_state, .. })
            | Self::WindowPartitionAndOrderBy(WindowPartitionAndOrderBy { stats_state, .. })
            | Self::WindowPartitionAndDynamicFrame(WindowPartitionAndDynamicFrame {
                stats_state,
                ..
            }) => stats_state,
            #[cfg(feature = "python")]
            Self::CatalogWrite(CatalogWrite { stats_state, .. })
            | Self::LanceWrite(LanceWrite { stats_state, .. }) => stats_state,
        }
    }

    pub(crate) fn in_memory_scan(
        in_memory_info: InMemoryInfo,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        Self::InMemoryScan(InMemoryScan {
            info: in_memory_info,
            stats_state,
        })
        .arced()
    }

    pub(crate) fn physical_scan(
        scan_tasks: Arc<Vec<ScanTaskLikeRef>>,
        pushdowns: Pushdowns,
        schema: SchemaRef,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        Self::PhysicalScan(PhysicalScan {
            scan_tasks,
            pushdowns,
            schema,
            stats_state,
        })
        .arced()
    }

    pub(crate) fn empty_scan(schema: SchemaRef) -> LocalPhysicalPlanRef {
        Self::EmptyScan(EmptyScan {
            schema,
            stats_state: StatsState::Materialized(PlanStats::empty().into()),
        })
        .arced()
    }

    pub(crate) fn filter(
        input: LocalPhysicalPlanRef,
        predicate: ExprRef,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        let schema = input.schema().clone();
        Self::Filter(Filter {
            input,
            predicate,
            schema,
            stats_state,
        })
        .arced()
    }

    pub(crate) fn limit(
        input: LocalPhysicalPlanRef,
        num_rows: i64,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        let schema = input.schema().clone();
        Self::Limit(Limit {
            input,
            num_rows,
            schema,
            stats_state,
        })
        .arced()
    }

    pub(crate) fn explode(
        input: LocalPhysicalPlanRef,
        to_explode: Vec<ExprRef>,
        schema: SchemaRef,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        Self::Explode(Explode {
            input,
            to_explode,
            schema,
            stats_state,
        })
        .arced()
    }

    pub(crate) fn project(
        input: LocalPhysicalPlanRef,
        projection: Vec<ExprRef>,
        schema: SchemaRef,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        Self::Project(Project {
            input,
            projection,
            schema,
            stats_state,
        })
        .arced()
    }

    pub(crate) fn actor_pool_project(
        input: LocalPhysicalPlanRef,
        projection: Vec<ExprRef>,
        schema: SchemaRef,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        Self::ActorPoolProject(ActorPoolProject {
            input,
            projection,
            schema,
            stats_state,
        })
        .arced()
    }

    pub(crate) fn ungrouped_aggregate(
        input: LocalPhysicalPlanRef,
        aggregations: Vec<ExprRef>,
        schema: SchemaRef,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        Self::UnGroupedAggregate(UnGroupedAggregate {
            input,
            aggregations,
            schema,
            stats_state,
        })
        .arced()
    }

    pub(crate) fn hash_aggregate(
        input: LocalPhysicalPlanRef,
        aggregations: Vec<ExprRef>,
        group_by: Vec<ExprRef>,
        schema: SchemaRef,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        Self::HashAggregate(HashAggregate {
            input,
            aggregations,
            group_by,
            schema,
            stats_state,
        })
        .arced()
    }

    pub(crate) fn window_partition_only(
        input: LocalPhysicalPlanRef,
        partition_by: Vec<ExprRef>,
        schema: SchemaRef,
        stats_state: StatsState,
        aggregations: Vec<AggExpr>,
        aliases: Vec<String>,
    ) -> LocalPhysicalPlanRef {
        Self::WindowPartitionOnly(WindowPartitionOnly {
            input,
            partition_by,
            schema,
            stats_state,
            aggregations,
            aliases,
        })
        .arced()
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn window_partition_and_order_by(
        input: LocalPhysicalPlanRef,
        partition_by: Vec<ExprRef>,
        order_by: Vec<ExprRef>,
        descending: Vec<bool>,
        schema: SchemaRef,
        stats_state: StatsState,
        functions: Vec<WindowExpr>,
        aliases: Vec<String>,
    ) -> LocalPhysicalPlanRef {
        Self::WindowPartitionAndOrderBy(WindowPartitionAndOrderBy {
            input,
            partition_by,
            order_by,
            descending,
            schema,
            stats_state,
            functions,
            aliases,
        })
        .arced()
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn window_partition_and_dynamic_frame(
        input: LocalPhysicalPlanRef,
        partition_by: Vec<ExprRef>,
        order_by: Vec<ExprRef>,
        descending: Vec<bool>,
        frame: WindowFrame,
        min_periods: usize,
        schema: SchemaRef,
        stats_state: StatsState,
        aggregations: Vec<AggExpr>,
        aliases: Vec<String>,
    ) -> LocalPhysicalPlanRef {
        Self::WindowPartitionAndDynamicFrame(WindowPartitionAndDynamicFrame {
            input,
            partition_by,
            order_by,
            descending,
            frame,
            min_periods,
            schema,
            stats_state,
            aggregations,
            aliases,
        })
        .arced()
    }

    pub(crate) fn unpivot(
        input: LocalPhysicalPlanRef,
        ids: Vec<ExprRef>,
        values: Vec<ExprRef>,
        variable_name: String,
        value_name: String,
        schema: SchemaRef,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        Self::Unpivot(Unpivot {
            input,
            ids,
            values,
            variable_name,
            value_name,
            schema,
            stats_state,
        })
        .arced()
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn pivot(
        input: LocalPhysicalPlanRef,
        group_by: Vec<ExprRef>,
        pivot_column: ExprRef,
        value_column: ExprRef,
        aggregation: AggExpr,
        names: Vec<String>,
        schema: SchemaRef,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        Self::Pivot(Pivot {
            input,
            group_by,
            pivot_column,
            value_column,
            aggregation,
            names,
            schema,
            stats_state,
        })
        .arced()
    }

    pub(crate) fn sort(
        input: LocalPhysicalPlanRef,
        sort_by: Vec<ExprRef>,
        descending: Vec<bool>,
        nulls_first: Vec<bool>,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        let schema = input.schema().clone();
        Self::Sort(Sort {
            input,
            sort_by,
            descending,
            nulls_first,
            schema,
            stats_state,
        })
        .arced()
    }

    pub(crate) fn sample(
        input: LocalPhysicalPlanRef,
        fraction: f64,
        with_replacement: bool,
        seed: Option<u64>,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        let schema = input.schema().clone();
        Self::Sample(Sample {
            input,
            fraction,
            with_replacement,
            seed,
            schema,
            stats_state,
        })
        .arced()
    }

    pub(crate) fn monotonically_increasing_id(
        input: LocalPhysicalPlanRef,
        column_name: String,
        schema: SchemaRef,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        Self::MonotonicallyIncreasingId(MonotonicallyIncreasingId {
            input,
            column_name,
            schema,
            stats_state,
        })
        .arced()
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn hash_join(
        left: LocalPhysicalPlanRef,
        right: LocalPhysicalPlanRef,
        left_on: Vec<ExprRef>,
        right_on: Vec<ExprRef>,
        null_equals_null: Option<Vec<bool>>,
        join_type: JoinType,
        schema: SchemaRef,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        Self::HashJoin(HashJoin {
            left,
            right,
            left_on,
            right_on,
            null_equals_null,
            join_type,
            schema,
            stats_state,
        })
        .arced()
    }

    pub(crate) fn cross_join(
        left: LocalPhysicalPlanRef,
        right: LocalPhysicalPlanRef,
        schema: SchemaRef,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        Self::CrossJoin(CrossJoin {
            left,
            right,
            schema,
            stats_state,
        })
        .arced()
    }

    pub(crate) fn concat(
        input: LocalPhysicalPlanRef,
        other: LocalPhysicalPlanRef,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        let schema = input.schema().clone();
        Self::Concat(Concat {
            input,
            other,
            schema,
            stats_state,
        })
        .arced()
    }

    pub(crate) fn physical_write(
        input: LocalPhysicalPlanRef,
        data_schema: SchemaRef,
        file_schema: SchemaRef,
        file_info: OutputFileInfo,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        Self::PhysicalWrite(PhysicalWrite {
            input,
            data_schema,
            file_schema,
            file_info,
            stats_state,
        })
        .arced()
    }

    #[cfg(feature = "python")]
    pub(crate) fn catalog_write(
        input: LocalPhysicalPlanRef,
        catalog_type: daft_logical_plan::CatalogType,
        data_schema: SchemaRef,
        file_schema: SchemaRef,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        Self::CatalogWrite(CatalogWrite {
            input,
            catalog_type,
            data_schema,
            file_schema,
            stats_state,
        })
        .arced()
    }

    #[cfg(feature = "python")]
    pub(crate) fn lance_write(
        input: LocalPhysicalPlanRef,
        lance_info: daft_logical_plan::LanceCatalogInfo,
        data_schema: SchemaRef,
        file_schema: SchemaRef,
        stats_state: StatsState,
    ) -> LocalPhysicalPlanRef {
        Self::LanceWrite(LanceWrite {
            input,
            lance_info,
            data_schema,
            file_schema,
            stats_state,
        })
        .arced()
    }

    pub fn schema(&self) -> &SchemaRef {
        match self {
            Self::PhysicalScan(PhysicalScan { schema, .. })
            | Self::EmptyScan(EmptyScan { schema, .. })
            | Self::Filter(Filter { schema, .. })
            | Self::Limit(Limit { schema, .. })
            | Self::Project(Project { schema, .. })
            | Self::ActorPoolProject(ActorPoolProject { schema, .. })
            | Self::UnGroupedAggregate(UnGroupedAggregate { schema, .. })
            | Self::HashAggregate(HashAggregate { schema, .. })
            | Self::Pivot(Pivot { schema, .. })
            | Self::Sort(Sort { schema, .. })
            | Self::Sample(Sample { schema, .. })
            | Self::HashJoin(HashJoin { schema, .. })
            | Self::CrossJoin(CrossJoin { schema, .. })
            | Self::Explode(Explode { schema, .. })
            | Self::Unpivot(Unpivot { schema, .. })
            | Self::Concat(Concat { schema, .. })
            | Self::MonotonicallyIncreasingId(MonotonicallyIncreasingId { schema, .. })
            | Self::WindowPartitionOnly(WindowPartitionOnly { schema, .. })
            | Self::WindowPartitionAndOrderBy(WindowPartitionAndOrderBy { schema, .. })
            | Self::WindowPartitionAndDynamicFrame(WindowPartitionAndDynamicFrame {
                schema, ..
            }) => schema,
            Self::PhysicalWrite(PhysicalWrite { file_schema, .. }) => file_schema,
            Self::InMemoryScan(InMemoryScan { info, .. }) => &info.source_schema,
            #[cfg(feature = "python")]
            Self::CatalogWrite(CatalogWrite { file_schema, .. }) => file_schema,
            #[cfg(feature = "python")]
            Self::LanceWrite(LanceWrite { file_schema, .. }) => file_schema,
        }
    }

    pub fn estimated_memory_cost(&self) -> usize {
        todo!("Implement estimated memory cost for local physical plan");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InMemoryScan {
    pub info: InMemoryInfo,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PhysicalScan {
    pub scan_tasks: Arc<Vec<ScanTaskLikeRef>>,
    pub pushdowns: Pushdowns,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmptyScan {
    pub schema: SchemaRef,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub input: LocalPhysicalPlanRef,
    pub projection: Vec<ExprRef>,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorPoolProject {
    pub input: LocalPhysicalPlanRef,
    pub projection: Vec<ExprRef>,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Filter {
    pub input: LocalPhysicalPlanRef,
    pub predicate: ExprRef,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Limit {
    pub input: LocalPhysicalPlanRef,
    pub num_rows: i64,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Explode {
    pub input: LocalPhysicalPlanRef,
    pub to_explode: Vec<ExprRef>,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sort {
    pub input: LocalPhysicalPlanRef,
    pub sort_by: Vec<ExprRef>,
    pub descending: Vec<bool>,
    pub nulls_first: Vec<bool>,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sample {
    pub input: LocalPhysicalPlanRef,
    pub fraction: f64,
    pub with_replacement: bool,
    pub seed: Option<u64>,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonotonicallyIncreasingId {
    pub input: LocalPhysicalPlanRef,
    pub column_name: String,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnGroupedAggregate {
    pub input: LocalPhysicalPlanRef,
    pub aggregations: Vec<ExprRef>,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HashAggregate {
    pub input: LocalPhysicalPlanRef,
    pub aggregations: Vec<ExprRef>,
    pub group_by: Vec<ExprRef>,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Unpivot {
    pub input: LocalPhysicalPlanRef,
    pub ids: Vec<ExprRef>,
    pub values: Vec<ExprRef>,
    pub variable_name: String,
    pub value_name: String,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pivot {
    pub input: LocalPhysicalPlanRef,
    pub group_by: Vec<ExprRef>,
    pub pivot_column: ExprRef,
    pub value_column: ExprRef,
    pub aggregation: AggExpr,
    pub names: Vec<String>,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HashJoin {
    pub left: LocalPhysicalPlanRef,
    pub right: LocalPhysicalPlanRef,
    pub left_on: Vec<ExprRef>,
    pub right_on: Vec<ExprRef>,
    pub null_equals_null: Option<Vec<bool>>,
    pub join_type: JoinType,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrossJoin {
    pub left: LocalPhysicalPlanRef,
    pub right: LocalPhysicalPlanRef,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Concat {
    pub input: LocalPhysicalPlanRef,
    pub other: LocalPhysicalPlanRef,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PhysicalWrite {
    pub input: LocalPhysicalPlanRef,
    pub data_schema: SchemaRef,
    pub file_schema: SchemaRef,
    pub file_info: OutputFileInfo,
    pub stats_state: StatsState,
}

#[cfg(feature = "python")]
#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogWrite {
    pub input: LocalPhysicalPlanRef,
    pub catalog_type: daft_logical_plan::CatalogType,
    pub data_schema: SchemaRef,
    pub file_schema: SchemaRef,
    pub stats_state: StatsState,
}

#[cfg(feature = "python")]
#[derive(Debug, Serialize, Deserialize)]
pub struct LanceWrite {
    pub input: LocalPhysicalPlanRef,
    pub lance_info: daft_logical_plan::LanceCatalogInfo,
    pub data_schema: SchemaRef,
    pub file_schema: SchemaRef,
    pub stats_state: StatsState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowPartitionOnly {
    pub input: LocalPhysicalPlanRef,
    pub partition_by: Vec<ExprRef>,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
    pub aggregations: Vec<AggExpr>,
    pub aliases: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowPartitionAndOrderBy {
    pub input: LocalPhysicalPlanRef,
    pub partition_by: Vec<ExprRef>,
    pub order_by: Vec<ExprRef>,
    pub descending: Vec<bool>,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
    pub functions: Vec<WindowExpr>,
    pub aliases: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowPartitionAndDynamicFrame {
    pub input: LocalPhysicalPlanRef,
    pub partition_by: Vec<ExprRef>,
    pub order_by: Vec<ExprRef>,
    pub descending: Vec<bool>,
    pub frame: WindowFrame,
    pub min_periods: usize,
    pub schema: SchemaRef,
    pub stats_state: StatsState,
    pub aggregations: Vec<AggExpr>,
    pub aliases: Vec<String>,
}
