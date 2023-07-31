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

use std::{any::Any, sync::Arc};

use arrow::datatypes::SchemaRef;
use async_trait::async_trait;
use datafusion::{
    datasource::{
        file_format::{file_type::FileCompressionType, FileFormat},
        physical_plan::FileScanConfig,
    },
    execution::context::SessionState,
    physical_plan::{ExecutionPlan, PhysicalExpr, Statistics},
};
use object_store::{ObjectMeta, ObjectStore};

use crate::optimizer;

use super::{hmm_dom_tab_config::schema, hmm_dom_tab_scanner::HMMDomTabScan};

#[derive(Debug)]
/// Implements a datafusion `FileFormat` for HMMDomTab files.
pub struct HMMDomTabFormat {
    /// The compression type of the file.
    file_compression_type: FileCompressionType,
}

impl HMMDomTabFormat {
    /// Create a new HMMDomTab format.
    pub fn new(file_compression_type: FileCompressionType) -> Self {
        Self {
            file_compression_type,
        }
    }
}

impl Default for HMMDomTabFormat {
    fn default() -> Self {
        Self {
            file_compression_type: FileCompressionType::UNCOMPRESSED,
        }
    }
}

#[async_trait]
impl FileFormat for HMMDomTabFormat {
    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn infer_schema(
        &self,
        _state: &SessionState,
        _store: &Arc<dyn ObjectStore>,
        _objects: &[ObjectMeta],
    ) -> datafusion::error::Result<SchemaRef> {
        Ok(schema().into())
    }

    async fn infer_stats(
        &self,
        _state: &SessionState,
        _store: &Arc<dyn ObjectStore>,
        _table_schema: SchemaRef,
        _object: &ObjectMeta,
    ) -> datafusion::error::Result<Statistics> {
        Ok(Statistics::default())
    }

    async fn create_physical_plan(
        &self,
        state: &SessionState,
        conf: FileScanConfig,
        _filters: Option<&Arc<dyn PhysicalExpr>>,
    ) -> datafusion::error::Result<Arc<dyn ExecutionPlan>> {
        let config = state.config();
        let target_partitions = config.target_partitions();

        let repartition_file_scans = config.options().optimizer.repartition_file_scans;

        if target_partitions == 1 || !repartition_file_scans {
            let scan = HMMDomTabScan::new(conf.clone(), self.file_compression_type.clone());
            Ok(Arc::new(scan))
        } else {
            let mut scan_config = conf.clone();

            scan_config.file_groups = optimizer::repartitioning::regroup_file_partitions(
                scan_config.file_groups,
                target_partitions,
            );

            let scan = HMMDomTabScan::new(scan_config, self.file_compression_type.clone());

            Ok(Arc::new(scan))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::tests::test_listing_table_url;

    use super::HMMDomTabFormat;
    use datafusion::{
        datasource::listing::{ListingOptions, ListingTable, ListingTableConfig},
        prelude::SessionContext,
    };

    #[tokio::test]
    async fn test_listing() {
        let ctx = SessionContext::new();
        let session_state = ctx.state();

        let table_path = test_listing_table_url("hmmdomtab");

        let hmm_format = Arc::new(HMMDomTabFormat::default());
        let lo = ListingOptions::new(hmm_format.clone()).with_file_extension(".hmmdomtab");

        let resolved_schema = lo.infer_schema(&session_state, &table_path).await.unwrap();

        let config = ListingTableConfig::new(table_path)
            .with_listing_options(lo)
            .with_schema(resolved_schema);

        let provider = Arc::new(ListingTable::try_new(config).unwrap());
        let df = ctx.read_table(provider.clone()).unwrap();

        let mut row_cnt = 0;
        let bs = df.collect().await.unwrap();
        for batch in bs {
            row_cnt += batch.num_rows();
        }
        assert_eq!(row_cnt, 100)
    }
}
