// Copyright 2023 WHERE TRUE Technologies.
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

use std::{any::Any, str::FromStr, sync::Arc};

use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use async_trait::async_trait;
use datafusion::{
    datasource::{
        listing::{ListingTableConfig, ListingTableUrl, PartitionedFile},
        physical_plan::FileScanConfig,
        TableProvider,
    },
    error::{DataFusionError, Result},
    execution::context::SessionState,
    logical_expr::{expr::ScalarUDF, TableProviderFilterPushDown, TableType},
    physical_plan::{empty::EmptyExec, ExecutionPlan, Statistics},
    prelude::Expr,
};
use futures::{StreamExt, TryStreamExt};
use noodles::{bam::lazy::Record, core::Region};
use object_store::ObjectStore;
use tokio_util::io::StreamReader;

fn infer_region_from_scalar_udf(scalar_udf: &ScalarUDF) -> Option<Region> {
    if scalar_udf.fun.name.as_str() == "bam_region_filter" {
        if scalar_udf.args.len() == 2 || scalar_udf.args.len() == 4 {
            match &scalar_udf.args[0] {
                Expr::Literal(l) => {
                    let region_str = l.to_string();
                    let region = Region::from_str(region_str.as_str()).ok()?;
                    Some(region)
                }
                _ => None,
            }
        } else {
            None
        }
    } else {
        None
    }
}

#[derive(Debug, Clone)]
/// Configuration for a BAM listing table
pub struct ListingBAMTableConfig {
    inner: ListingTableConfig,

    options: Option<ListingBAMTableOptions>,
}

impl ListingBAMTableConfig {
    /// Create a new BAM listing table configuration
    pub fn new(table_path: ListingTableUrl) -> Self {
        Self {
            inner: ListingTableConfig::new(table_path),
            options: None,
        }
    }

    /// Set the options for the BAM listing table
    pub fn with_options(self, options: ListingBAMTableOptions) -> Self {
        Self {
            options: Some(options),
            ..self
        }
    }
}

use crate::{
    datasources::{
        hive_partition::filter_matches_partition_cols,
        indexed_file_utils::{augment_partitioned_file_with_byte_range, IndexedFile},
        sam::SAMSchemaBuilder,
    },
    physical_plan::object_store::pruned_partition_list,
};

use super::{indexed_scanner::IndexedBAMScan, BAMScan};

#[derive(Debug, Clone)]
/// Listing options for a BAM table
pub struct ListingBAMTableOptions {
    file_extension: String,

    indexed: bool,

    table_partition_cols: Vec<(String, DataType)>,

    tag_as_struct: bool,
}

impl Default for ListingBAMTableOptions {
    fn default() -> Self {
        Self {
            file_extension: String::from("bam"),
            table_partition_cols: Vec::new(),
            indexed: false,
            tag_as_struct: false,
        }
    }
}

impl ListingBAMTableOptions {
    /// Set the table partition columns
    pub fn with_table_partition_cols(self, table_partition_cols: Vec<(String, DataType)>) -> Self {
        Self {
            table_partition_cols,
            ..self
        }
    }

    /// Infer the schema for the table
    pub async fn infer_schema(
        &self,
        state: &SessionState,
        table_path: &ListingTableUrl,
    ) -> datafusion::error::Result<(SchemaRef, Vec<usize>)> {
        if !self.tag_as_struct {
            let partition_fields = self
                .table_partition_cols
                .iter()
                .map(|f| Field::new(&f.0, f.1.clone(), false))
                .collect::<Vec<_>>();

            let builder = SAMSchemaBuilder::default().with_partition_fields(partition_fields);

            let (schema, file_projection) = builder.build();
            return Ok((Arc::new(schema), file_projection));
        }

        let store = state.runtime_env().object_store(table_path)?;

        let mut files = exon_common::object_store_files_from_table_path(
            &store,
            table_path.as_ref(),
            table_path.prefix(),
            self.file_extension.as_str(),
            None,
        )
        .await;

        let mut schema_builder = SAMSchemaBuilder::default();

        while let Some(f) = files.next().await {
            let f = f?;

            let get_result = store.get(&f.location).await?;

            let stream_reader = Box::pin(get_result.into_stream().map_err(DataFusionError::from));
            let stream_reader = StreamReader::new(stream_reader);
            let mut reader = noodles::bam::AsyncReader::new(stream_reader);

            reader.read_header().await?;
            reader.read_reference_sequences().await?;

            let mut record = Record::default();

            reader.read_lazy_record(&mut record).await?;

            let data = record.data();
            let data: noodles::sam::record::Data = data.try_into()?;

            schema_builder = schema_builder.with_tags_data_type_from_data(&data)?;
        }

        let partition_fields = self
            .table_partition_cols
            .iter()
            .map(|f| Field::new(&f.0, f.1.clone(), false))
            .collect::<Vec<_>>();

        schema_builder = schema_builder.with_partition_fields(partition_fields);

        let (schema, file_projection) = schema_builder.build();

        Ok((Arc::new(schema), file_projection))
    }

    /// Update the indexed flag
    pub fn with_indexed(mut self, indexed: bool) -> Self {
        self.indexed = indexed;
        self
    }

    /// Update the tag_as_struct flag
    pub fn with_tag_as_struct(mut self, tag_as_struct: bool) -> Self {
        self.tag_as_struct = tag_as_struct;
        self
    }

    async fn create_physical_plan_with_region(
        &self,
        conf: FileScanConfig,
        region: Arc<Region>,
    ) -> datafusion::error::Result<Arc<dyn ExecutionPlan>> {
        let scan = IndexedBAMScan::new(conf, region);
        Ok(Arc::new(scan))
    }

    async fn create_physical_plan(
        &self,
        conf: FileScanConfig,
    ) -> datafusion::error::Result<Arc<dyn ExecutionPlan>> {
        let scan = BAMScan::new(conf);
        Ok(Arc::new(scan))
    }
}

#[derive(Debug, Clone)]
/// A BAM listing table
pub struct ListingBAMTable {
    table_paths: Vec<ListingTableUrl>,

    table_schema: SchemaRef,

    /// The file projection
    file_projection: Vec<usize>,

    options: ListingBAMTableOptions,
}

impl ListingBAMTable {
    /// Create a new BAM listing table
    pub fn try_new(
        config: ListingBAMTableConfig,
        table_schema: Arc<Schema>,
        file_projection: Vec<usize>,
    ) -> Result<Self> {
        Ok(Self {
            table_paths: config.inner.table_paths,
            table_schema,
            file_projection,
            options: config
                .options
                .ok_or_else(|| DataFusionError::Internal(String::from("Options must be set")))?,
        })
    }

    /// File Schema
    pub fn file_schema(&self) -> Result<SchemaRef> {
        let file_schema = &self.table_schema.project(&self.file_projection)?;
        Ok(Arc::new(file_schema.clone()))
    }
}

#[async_trait]
impl TableProvider for ListingBAMTable {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        Arc::clone(&self.table_schema)
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> Result<Vec<TableProviderFilterPushDown>> {
        tracing::debug!("Filters for BAM table provider: {:?}", filters);
        Ok(filters
            .iter()
            .map(|f| match f {
                Expr::ScalarUDF(s) if s.fun.name.as_str() == "bam_region_filter" => {
                    if s.args.len() == 2 || s.args.len() == 4 {
                        tracing::debug!("Pushing down region filter");
                        TableProviderFilterPushDown::Exact
                    } else {
                        tracing::debug!("Unsupported number of arguments for region filter");
                        filter_matches_partition_cols(f, &self.options.table_partition_cols)
                    }
                }
                _ => filter_matches_partition_cols(f, &self.options.table_partition_cols),
            })
            .collect())
    }

    async fn scan(
        &self,
        state: &SessionState,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let object_store_url = if let Some(url) = self.table_paths.get(0) {
            url.object_store()
        } else {
            return Ok(Arc::new(EmptyExec::new(false, Arc::new(Schema::empty()))));
        };

        let object_store = state.runtime_env().object_store(object_store_url.clone())?;

        let regions = filters
            .iter()
            .filter_map(|f| {
                if let Expr::ScalarUDF(s) = f {
                    infer_region_from_scalar_udf(s)
                } else {
                    None
                }
            })
            .collect::<Vec<Region>>();

        if regions.is_empty() && self.options.indexed {
            return Err(DataFusionError::Plan(
                "INDEXED_BAM table type requires a region filter. See the 'bam_region_filter' function.".to_string(),
            ));
        }

        if regions.len() > 1 {
            return Err(DataFusionError::Plan(
                "Only one region filter is supported".to_string(),
            ));
        }

        if regions.is_empty() {
            let file_list = pruned_partition_list(
                state,
                &object_store,
                &self.table_paths[0],
                filters,
                self.options.file_extension.as_str(),
                &self.options.table_partition_cols,
            )
            .await?
            .try_collect::<Vec<_>>()
            .await?;

            let inner_size = 1;
            let file_groups: Vec<Vec<PartitionedFile>> = file_list
                .chunks(inner_size)
                .map(|chunk| chunk.to_vec())
                .collect();

            let file_scan_config = FileScanConfig {
                object_store_url,
                file_schema: self.file_schema()?,
                file_groups,
                statistics: Statistics::default(),
                projection: projection.cloned(),
                limit,
                output_ordering: Vec::new(),
                table_partition_cols: self.options.table_partition_cols.clone(),
                infinite_source: false,
            };

            let plan = self.options.create_physical_plan(file_scan_config).await?;

            return Ok(plan);
        }

        let mut file_list = pruned_partition_list(
            state,
            &object_store,
            &self.table_paths[0],
            filters,
            self.options.file_extension.as_str(),
            &self.options.table_partition_cols,
        )
        .await?;

        let mut byte_ranges = Vec::new();

        let region = regions[0].clone();

        while let Some(f) = file_list.next().await {
            let f = f?;

            let file_byte_range = augment_partitioned_file_with_byte_range(
                object_store.clone(),
                &f,
                &region,
                &IndexedFile::Bam,
            )
            .await?;

            byte_ranges.extend(file_byte_range);
        }

        let file_scan_config = FileScanConfig {
            object_store_url: object_store_url.clone(),
            file_schema: self.file_schema()?,
            file_groups: vec![byte_ranges],
            statistics: Statistics::default(),
            projection: projection.cloned(),
            limit,
            output_ordering: Vec::new(),
            table_partition_cols: self.options.table_partition_cols.clone(),
            infinite_source: false,
        };

        let table = self
            .options
            .create_physical_plan_with_region(file_scan_config, Arc::new(region.clone()))
            .await?;

        return Ok(table);
    }
}
