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

use arrow::{
    array::{ArrayRef, GenericListBuilder, GenericStringBuilder, Int32Builder, Int64Builder},
    error::ArrowError,
    error::Result,
};
use exon_common::ExonArrayBuilder;
use noodles::sam::alignment::{
    record::{cigar::op::Kind, Cigar},
    RecordBuf,
};
use noodles::sam::Header;

use crate::{SAMConfig, TagsBuilder};

/// Builds an vector of arrays from a SAM file.
pub struct SAMArrayBuilder {
    names: GenericStringBuilder<i32>,
    flags: Int32Builder,
    references: GenericStringBuilder<i32>,
    starts: Int64Builder,
    ends: Int64Builder,
    mapping_qualities: GenericStringBuilder<i32>,
    cigar: GenericStringBuilder<i32>,
    mate_references: GenericStringBuilder<i32>,
    sequences: GenericStringBuilder<i32>,
    quality_scores: GenericListBuilder<i32, Int64Builder>,

    tags: TagsBuilder,

    projection: Vec<usize>,

    rows: usize,

    header: Header,
}

impl SAMArrayBuilder {
    /// Creates a new SAM array builder.
    pub fn create(header: Header, sam_config: Arc<SAMConfig>) -> Self {
        let tags_builder = sam_config
            .file_schema
            .field_with_name("tags")
            .map_or(TagsBuilder::default(), |field| {
                TagsBuilder::try_from(field.data_type()).unwrap()
            });

        let projection = sam_config.projection();

        let quality_scores = GenericListBuilder::<i32, Int64Builder>::new(Int64Builder::new());

        Self {
            names: GenericStringBuilder::<i32>::new(),
            flags: Int32Builder::new(),
            references: GenericStringBuilder::<i32>::new(),
            starts: Int64Builder::new(),
            ends: Int64Builder::new(),
            mapping_qualities: GenericStringBuilder::<i32>::new(),
            cigar: GenericStringBuilder::<i32>::new(),
            mate_references: GenericStringBuilder::<i32>::new(),
            sequences: GenericStringBuilder::<i32>::new(),
            quality_scores,

            tags: tags_builder,

            projection,

            rows: 0,

            header,
        }
    }

    /// Returns the number of records in the builder.
    pub fn len(&self) -> usize {
        self.rows
    }

    /// Returns whether the builder is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Appends a record to the builder.
    pub fn append(&mut self, record: &RecordBuf) -> Result<()> {
        for col_idx in self.projection.iter() {
            match col_idx {
                0 => {
                    if let Some(name) = record.name() {
                        let name = std::str::from_utf8(name.as_ref())?;
                        self.names.append_value(name);
                    } else {
                        self.names.append_null();
                    }
                }
                1 => {
                    let flag_bits = record.flags().bits();
                    self.flags.append_value(flag_bits as i32);
                }
                2 => {
                    let reference_name = match record.reference_sequence(&self.header) {
                        Some(Ok((name, _))) => Some(std::str::from_utf8(name)?),
                        Some(Err(_)) => None,
                        None => None,
                    };
                    self.references.append_option(reference_name);
                }
                3 => {
                    self.starts
                        .append_option(record.alignment_start().map(|v| v.get() as i64));
                }
                4 => {
                    self.ends
                        .append_option(record.alignment_end().map(|v| v.get() as i64));
                }
                5 => {
                    self.mapping_qualities
                        .append_option(record.mapping_quality().map(|v| v.get().to_string()));
                }
                6 => {
                    let mut cigar_to_print = Vec::new();

                    // let cigar_string = cigar.iter().map(|c| c.to_string()).join("");
                    for op_result in record.cigar().iter() {
                        let op = op_result?;

                        let kind_str = match op.kind() {
                            Kind::Deletion => "D",
                            Kind::Insertion => "I",
                            Kind::HardClip => "H",
                            Kind::SoftClip => "S",
                            Kind::Match => "M",
                            Kind::SequenceMismatch => "X",
                            Kind::Skip => "N",
                            Kind::Pad => "P",
                            Kind::SequenceMatch => "=",
                        };

                        cigar_to_print.push(format!("{}{}", op.len(), kind_str));
                    }

                    let cigar_string = cigar_to_print.join("");
                    self.cigar.append_value(cigar_string);
                }
                7 => {
                    let mate_reference_name = match record.mate_reference_sequence(&self.header) {
                        Some(Ok((name, _))) => Some(std::str::from_utf8(name)?),
                        Some(Err(_)) => None,
                        None => None,
                    };
                    self.mate_references.append_option(mate_reference_name);
                }
                8 => {
                    let sequence = record.sequence().as_ref();
                    self.sequences.append_value(std::str::from_utf8(sequence)?);
                }
                9 => {
                    let quality_scores = record.quality_scores().as_ref();
                    let slice_i8: &[i8] = unsafe {
                        std::slice::from_raw_parts(
                            quality_scores.as_ptr() as *const i8,
                            quality_scores.len(),
                        )
                    };

                    let slice_i64 = slice_i8.iter().map(|v| *v as i64).collect::<Vec<_>>();

                    self.quality_scores.values().append_slice(&slice_i64);
                    self.quality_scores.append(true);
                }
                10 => {
                    // This is _very_ similar to BAM, may not need body any more
                    let data = record.data();
                    self.tags.append(data)?;
                }
                _ => {
                    return Err(ArrowError::InvalidArgumentError(format!(
                        "Invalid column index {} for SAM",
                        col_idx
                    )))
                }
            }
        }

        self.rows += 1;

        Ok(())
    }

    /// Finishes the builder and returns an vector of arrays.
    pub fn finish(&mut self) -> Vec<ArrayRef> {
        let mut arrays: Vec<ArrayRef> = Vec::new();

        for col_idx in self.projection.iter() {
            match col_idx {
                0 => arrays.push(Arc::new(self.names.finish())),
                1 => arrays.push(Arc::new(self.flags.finish())),
                2 => arrays.push(Arc::new(self.references.finish())),
                3 => arrays.push(Arc::new(self.starts.finish())),
                4 => arrays.push(Arc::new(self.ends.finish())),
                5 => arrays.push(Arc::new(self.mapping_qualities.finish())),
                6 => arrays.push(Arc::new(self.cigar.finish())),
                7 => arrays.push(Arc::new(self.mate_references.finish())),
                8 => arrays.push(Arc::new(self.sequences.finish())),
                9 => arrays.push(Arc::new(self.quality_scores.finish())),
                10 => arrays.push(Arc::new(self.tags.finish())),
                _ => panic!("Invalid column index {} for SAM", col_idx),
            }
        }

        arrays
    }
}

impl ExonArrayBuilder for SAMArrayBuilder {
    /// Finishes building the internal data structures and returns the built arrays.
    fn finish(&mut self) -> Vec<ArrayRef> {
        self.finish()
    }

    /// Returns the number of elements in the array.
    fn len(&self) -> usize {
        self.len()
    }
}
