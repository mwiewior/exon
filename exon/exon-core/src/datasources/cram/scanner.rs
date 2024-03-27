// Copyright 2024 WHERE TRUE Technologies.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{any::Any, sync::Arc};

use arrow::datatypes::SchemaRef;
use datafusion::{
    datasource::physical_plan::{FileScanConfig, FileStream},
    physical_plan::{
        metrics::ExecutionPlanMetricsSet, DisplayAs, DisplayFormatType, ExecutionPlan,
        Partitioning, SendableRecordBatchStream,
    },
};
use exon_cram::CRAMConfig;

use crate::datasources::ExonFileScanConfig;

use super::file_opener::CRAMOpener;

#[derive(Debug, Clone)]
/// Implements a datafusion `ExecutionPlan` for CRAM files.
pub struct CRAMScan {
    /// The base configuration for the file scan.
    base_config: FileScanConfig,

    /// Projected schema for the scan.
    projected_schema: SchemaRef,

    /// The FASTA reference to use.
    reference: Option<String>,

    /// Metrics for the execution plan.
    metrics: ExecutionPlanMetricsSet,
}

impl CRAMScan {
    pub fn new(file_scan_config: FileScanConfig, reference: Option<String>) -> Self {
        let projected_schema = if let Some(p) = &file_scan_config.projection {
            Arc::new(file_scan_config.file_schema.project(p).unwrap())
        } else {
            file_scan_config.file_schema.clone()
        };

        Self {
            base_config: file_scan_config,
            projected_schema,
            reference,
            metrics: ExecutionPlanMetricsSet::new(),
        }
    }

    pub fn base_config(&self) -> &FileScanConfig {
        &self.base_config
    }
}

impl DisplayAs for CRAMScan {
    fn fmt_as(&self, _t: DisplayFormatType, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CRAMScan")
    }
}

impl ExecutionPlan for CRAMScan {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repartitioned(
        &self,
        target_partitions: usize,
        _config: &datafusion::config::ConfigOptions,
    ) -> datafusion::error::Result<Option<Arc<dyn ExecutionPlan>>> {
        if target_partitions == 1 {
            return Ok(None);
        }

        let file_groups = self.base_config.regroup_files_by_size(target_partitions);

        let mut new_plan = self.clone();
        if let Some(repartitioned_file_groups) = file_groups {
            tracing::info!(
                "Repartitioned {} file groups into {}",
                self.base_config.file_groups.len(),
                repartitioned_file_groups.len()
            );
            new_plan.base_config.file_groups = repartitioned_file_groups;
        }

        Ok(Some(Arc::new(new_plan)))
    }

    fn schema(&self) -> SchemaRef {
        tracing::trace!("CRAM schema: {:#?}", self.projected_schema);
        self.projected_schema.clone()
    }

    fn output_partitioning(&self) -> datafusion::physical_plan::Partitioning {
        Partitioning::UnknownPartitioning(self.base_config.file_groups.len())
    }

    fn output_ordering(&self) -> Option<&[datafusion::physical_expr::PhysicalSortExpr]> {
        None
    }

    fn children(&self) -> Vec<Arc<dyn ExecutionPlan>> {
        vec![]
    }

    fn with_new_children(
        self: Arc<Self>,
        _children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> datafusion::error::Result<Arc<dyn ExecutionPlan>> {
        Ok(self)
    }

    fn execute(
        &self,
        partition: usize,
        context: Arc<datafusion::execution::context::TaskContext>,
    ) -> datafusion::error::Result<datafusion::physical_plan::SendableRecordBatchStream> {
        let object_store = context
            .runtime_env()
            .object_store(&self.base_config.object_store_url)?;

        let batch_size = context.session_config().batch_size();

        let config = CRAMConfig::new(
            object_store,
            self.base_config.file_schema.clone(),
            self.reference.clone(),
        )
        .with_batch_size(batch_size)
        .with_projection(self.base_config().file_projection());

        let opener = CRAMOpener::new(Arc::new(config));
        let stream = FileStream::new(&self.base_config, partition, opener, &self.metrics)?;

        Ok(Box::pin(stream) as SendableRecordBatchStream)
    }
}