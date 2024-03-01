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

use datafusion::{
    datasource::{function::TableFunctionImpl, TableProvider},
    error::{DataFusionError, Result},
    execution::context::SessionContext,
    logical_expr::Expr,
};
use exon_fasta::FASTASchemaBuilder;

use crate::{
    config::ExonConfigExtension,
    datasources::{
        fasta::table_provider::{
            ListingFASTATable, ListingFASTATableConfig, ListingFASTATableOptions,
        },
        ScanFunction,
    },
    ExonRuntimeEnvExt,
};

/// A table function that returns a table provider for a FASTA file.
pub struct FastaScanFunction {
    ctx: SessionContext,
}

impl FastaScanFunction {
    /// Create a new `FastaScanFunction`.
    pub fn new(ctx: SessionContext) -> Self {
        Self { ctx }
    }
}

impl TableFunctionImpl for FastaScanFunction {
    fn call(&self, exprs: &[Expr]) -> Result<Arc<dyn TableProvider>> {
        let listing_scan_function = ScanFunction::try_from(exprs)?;

        futures::executor::block_on(async {
            self.ctx
                .runtime_env()
                .exon_register_object_store_url(listing_scan_function.listing_table_url.as_ref())
                .await
        })?;

        let state = self.ctx.state();

        let exon_settings = state
            .config()
            .options()
            .extensions
            .get::<ExonConfigExtension>()
            .ok_or(DataFusionError::Execution(
                "Exon settings must be configured.".to_string(),
            ))?;

        let fasta_schema = FASTASchemaBuilder::default()
            .with_large_utf8(exon_settings.fasta_large_utf8)
            .build();

        let listing_table_options =
            ListingFASTATableOptions::new(listing_scan_function.file_compression_type);

        let listing_table_config = ListingFASTATableConfig::new(
            listing_scan_function.listing_table_url,
            listing_table_options,
        );

        let listing_table = ListingFASTATable::try_new(listing_table_config, fasta_schema)?;

        Ok(Arc::new(listing_table))
    }
}
