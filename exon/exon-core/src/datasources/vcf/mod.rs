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

//! VCF data source.
//!
//! This module provides functionality for working with VCF files as a data source.

mod file_opener;
mod indexed_scanner;
mod scanner;
mod schema_builder;
mod table_provider;

pub use self::indexed_scanner::IndexedVCFScanner;
pub use self::scanner::VCFScan;
pub use self::schema_builder::VCFSchemaBuilder;
pub use self::table_provider::ListingVCFTable;
pub use self::table_provider::ListingVCFTableOptions;
pub use crate::datasources::vcf::file_opener::unindex_file_opener::VCFOpener;

mod udtf;
pub use self::udtf::VCFIndexedScanFunction;
pub use self::udtf::VCFScanFunction;
