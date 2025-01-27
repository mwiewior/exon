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
    common::Statistics,
    datasource::{
        file_format::file_compression_type::FileCompressionType,
        physical_plan::{FileScanConfig, FileStream},
    },
    error::Result,
    physical_plan::{
        metrics::ExecutionPlanMetricsSet, DisplayAs, DisplayFormatType, ExecutionPlan,
        PlanProperties, SendableRecordBatchStream,
    },
};
use exon_vcf::VCFConfig;

use crate::datasources::{vcf::VCFOpener, ExonFileScanConfig};

#[derive(Debug, Clone)]
/// Implements a datafusion `ExecutionPlan` for VCF files.
pub struct VCFScan {
    /// The base configuration for the file scan.
    base_config: FileScanConfig,

    /// The projected schema for the scan.
    projected_schema: SchemaRef,

    /// The compression type of the file.
    file_compression_type: FileCompressionType,

    /// Metrics for the execution plan.
    metrics: ExecutionPlanMetricsSet,

    /// The plan properties cache.
    properties: PlanProperties,

    /// The statistics for the scan.
    statistics: Statistics,
}

impl VCFScan {
    /// Create a new VCF scan.
    pub fn new(
        base_config: FileScanConfig,
        file_compression_type: FileCompressionType,
    ) -> Result<Self> {
        let (projected_schema, statistics, properties) = base_config.project_with_properties();

        Ok(Self {
            base_config,
            projected_schema,
            file_compression_type,
            metrics: ExecutionPlanMetricsSet::new(),
            properties,
            statistics,
        })
    }

    /// Return the base configuration for the scan.
    pub fn base_config(&self) -> &FileScanConfig {
        &self.base_config
    }
}

impl DisplayAs for VCFScan {
    fn fmt_as(&self, _t: DisplayFormatType, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "VCFScan: {:?}", self.base_config)
    }
}

impl ExecutionPlan for VCFScan {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn properties(&self) -> &PlanProperties {
        &self.properties
    }

    fn statistics(&self) -> datafusion::error::Result<Statistics> {
        Ok(self.statistics.clone())
    }

    fn name(&self) -> &str {
        "VCFScan"
    }

    fn repartitioned(
        &self,
        target_partitions: usize,
        _config: &datafusion::config::ConfigOptions,
    ) -> Result<Option<Arc<dyn ExecutionPlan>>> {
        if target_partitions == 1 || self.base_config.file_groups.is_empty() {
            return Ok(None);
        }

        let file_groups = self.base_config.regroup_files_by_size(target_partitions);

        let mut new_plan = self.clone();
        new_plan.base_config.file_groups = file_groups;

        new_plan.properties = new_plan.properties.with_partitioning(
            datafusion::physical_plan::Partitioning::UnknownPartitioning(
                new_plan.base_config.file_groups.len(),
            ),
        );

        Ok(Some(Arc::new(new_plan)))
    }

    fn schema(&self) -> SchemaRef {
        tracing::trace!("VCF schema: {:#?}", self.projected_schema);
        Arc::clone(&self.projected_schema)
    }

    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
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

        let file_schema = Arc::clone(&self.base_config.file_schema);
        let config = VCFConfig::new(object_store, file_schema)
            .with_batch_size(batch_size)
            .with_projection(self.base_config().file_projection());

        let opener = VCFOpener::new(Arc::new(config), self.file_compression_type);
        let stream = FileStream::new(&self.base_config, partition, opener, &self.metrics)?;

        Ok(Box::pin(stream) as SendableRecordBatchStream)
    }
}
