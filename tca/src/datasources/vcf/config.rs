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

use arrow::datatypes::SchemaRef;
use object_store::ObjectStore;

/// Configuration for a VCF datasource.
pub struct VCFConfig {
    /// The number of records to read at a time.
    pub batch_size: usize,
    /// The object store to use.
    pub object_store: Arc<dyn ObjectStore>,
    /// The file schema to use.
    pub file_schema: Arc<arrow::datatypes::Schema>,
    /// Any projections to apply to the resulting batches.
    pub projection: Option<Vec<usize>>,
}

impl VCFConfig {
    /// Create a new VCF configuration.
    pub fn new(object_store: Arc<dyn ObjectStore>, file_schema: SchemaRef) -> Self {
        Self {
            batch_size: crate::datasources::DEFAULT_BATCH_SIZE,
            object_store,
            file_schema,
            projection: None,
        }
    }

    /// Set the batch size.
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    /// Set the projection.
    pub fn with_projection(mut self, projection: Vec<usize>) -> Self {
        self.projection = Some(projection);
        self
    }
}
