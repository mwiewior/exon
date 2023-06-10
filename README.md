<h1 align="center">
    <img src=".github/images/logo.png" width="200px" alt="Exon" />
</h1>

Exon is an analysis toolkit for life-science applications. It features:

* Support for over 10 (and counting) file formats from bioinformatics, proteomics, and others
* Seamless transition between local and object storage (e.g. S3)
* Arrow FFI primitives for multi-language support
* SQL based access to bioinformatics data -- general DML and some DDL support

Please note Exon was recently excised from a larger library, so please be patient as we work to clean up after that. If you have a comment or question in the meantime, please file an issue.

- [Installation](#installation)
- [Usage](#usage)
- [File Formats](#file-formats)
- [Benchmarks](#benchmarks)
  - [VCF](#vcf)
  - [FASTA](#fasta)

## Installation

Exon is available via crates.io. To install, add the following to your `Cargo.toml`:

```toml
[dependencies]
exon = "*"
```

## Usage

Exon is designed to be used as a library. For example, to read a FASTA file:

```rust
use exon::context::ExonSessionExt;

use datafusion::prelude::*;
use datafusion::error::Result;

let ctx = SessionContext::new();

let df = ctx.read_fasta("test-data/datasources/fasta/test.fasta", None).await?;
```

Please see the [rust docs](https://docs.rs/exon) for more information.

## File Formats

| Format    | Compression(s) | Inferred Extension(s) |
| --------- | -------------- | --------------------- |
| BAM       | -              | .bam                  |
| BCF       | -              | .bcf                  |
| BED       | gz, zstd       | .bed                  |
| FASTA     | gz, zstd       | .fasta, .fa, .fna     |
| FASTQ     | gz, zstd       | .fastq, .fq           |
| GENBANK   | gz, zstd       | .gbk, .genbank, .gb   |
| GFF       | gz, zstd       | .gff                  |
| HMMDOMTAB | gz, zstd       | .hmmdomtab            |
| MZML      | gz, zstd       | .mzml[^2]             |
| SAM       | -              | .sam                  |
| VCF       | gz[^1]         | .vcf                  |


[^1]: Uses bgzip not gzip.
[^2]: mzML also works.

## Benchmarks

Set of benchmarks for various file formats. All benchmarks were run on a 2022 MacBook Air.

### VCF

Reading VCF files.

<!-- copied from the output of cargo bench -->
<img src=".github/images/vcf-benches.svg" width="500px" alt="Exon" />

### FASTA

Reading uniprot sequences from a FASTA file.

<!-- copied from the output of cargo bench -->
<img src=".github/images/uniprot-benches.svg" width="500px" alt="Exon" />
