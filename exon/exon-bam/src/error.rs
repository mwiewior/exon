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

use std::error::Error;

use arrow::error::ArrowError;

#[derive(Debug)]
pub enum ExonBAMError {
    PositionConversionError(String),
}

impl Error for ExonBAMError {}

impl std::fmt::Display for ExonBAMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExonBAMError::PositionConversionError(e) => {
                write!(f, "Error converting position: {}", e)
            }
        }
    }
}

impl From<ExonBAMError> for ArrowError {
    fn from(e: ExonBAMError) -> Self {
        ArrowError::ExternalError(Box::new(e))
    }
}
