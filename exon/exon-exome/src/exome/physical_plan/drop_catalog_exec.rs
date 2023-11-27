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

use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use datafusion::{
    error::DataFusionError,
    physical_plan::{stream::RecordBatchStreamAdapter, DisplayAs, ExecutionPlan, Partitioning},
};

use futures::stream;

use crate::{
    exome_catalog_manager::{Change, CreateCatalog, ExomeCatalogManager},
    exome_config::ExomeConfigExtension,
    OrganizationName,
};

use super::CHANGE_SCHEMA;

#[derive(Debug, Clone)]
pub struct DropCatalogExec {
    name: String,
    library_id: String,
}

impl DropCatalogExec {
    pub fn new(name: String, library_id: String) -> Self {
        Self { name, library_id }
    }

    pub async fn drop_catalog(
        self,
        organization_name: OrganizationName,
        manager: Arc<ExomeCatalogManager>,
    ) -> Result<RecordBatch, DataFusionError> {
        let changes = vec![Change::CreateCatalog(CreateCatalog::new(
            self.name.clone(),
            self.library_id.clone(),
        ))];

        manager
            .apply_changes(organization_name, changes)
            .await
            .map_err(|e| DataFusionError::Execution(format!("Error applying changes: {}", e)))?;

        Ok(RecordBatch::new_empty(CHANGE_SCHEMA.clone()))
    }
}

impl DisplayAs for DropCatalogExec {
    fn fmt_as(
        &self,
        _t: datafusion::physical_plan::DisplayFormatType,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        write!(f, "DropCatalogExec")
    }
}

impl ExecutionPlan for DropCatalogExec {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn schema(&self) -> datafusion::arrow::datatypes::SchemaRef {
        CHANGE_SCHEMA.clone()
    }

    fn output_partitioning(&self) -> Partitioning {
        Partitioning::UnknownPartitioning(1)
    }

    fn output_ordering(&self) -> Option<&[datafusion::physical_expr::PhysicalSortExpr]> {
        todo!()
    }

    fn children(&self) -> Vec<std::sync::Arc<dyn ExecutionPlan>> {
        vec![]
    }

    fn with_new_children(
        self: std::sync::Arc<Self>,
        _children: Vec<std::sync::Arc<dyn ExecutionPlan>>,
    ) -> datafusion::error::Result<std::sync::Arc<dyn ExecutionPlan>> {
        Ok(Arc::new(DropCatalogExec {
            name: self.name.clone(),
            library_id: self.library_id.clone(),
        }))
    }

    fn execute(
        &self,
        partition: usize,
        context: std::sync::Arc<datafusion::execution::TaskContext>,
    ) -> datafusion::error::Result<datafusion::physical_plan::SendableRecordBatchStream> {
        if partition != 0 {
            return Err(DataFusionError::Execution(
                "DropCatalogExec only supports 1 partition".to_string(),
            ));
        }

        let session_config = context.session_config();

        let exome_catalog_manager = match session_config.get_extension::<ExomeCatalogManager>() {
            Some(exome_catalog_manager) => exome_catalog_manager,
            None => {
                return Err(DataFusionError::Execution(
                    "ExomeCatalogManager not found".to_string(),
                ))
            }
        };

        let exome_config = match session_config
            .options()
            .extensions
            .get::<ExomeConfigExtension>()
        {
            Some(exome_config) => exome_config,
            None => {
                return Err(DataFusionError::Execution(
                    "ExomeConfigExtension not found".to_string(),
                ))
            }
        };

        let this = self.clone();
        let stream = stream::once(
            this.drop_catalog(exome_config.exome_organization(), exome_catalog_manager),
        );

        Ok(Box::pin(RecordBatchStreamAdapter::new(
            CHANGE_SCHEMA.clone(),
            stream,
        )))
    }
}
