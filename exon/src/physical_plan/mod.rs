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

use std::fmt::Display;

#[derive(Debug)]
struct InvalidRegionError;

impl Display for InvalidRegionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid expression for region")
    }
}

impl std::error::Error for InvalidRegionError {}

/// A physical expression that represents a chromosome.
pub mod chrom_physical_expr;

/// A physical expression that represents a genomic interval.
pub mod interval_physical_expr;

/// A physical expression that represents a region, e.g. chr1:100-200.
pub mod region_physical_expr;

/// Utilities for working with object stores.
pub mod object_store;
