use std::{str::FromStr, sync::Arc};

use datafusion::{
    datasource::file_format::{file_type::FileCompressionType, FileFormat},
    error::DataFusionError,
};

use super::{
    bam::BAMFormat, bcf::BCFFormat, bed::BEDFormat, fasta::FASTAFormat, fastq::FASTQFormat,
    genbank::GenbankFormat, gff::GFFFormat, hmmdomtab::HMMDomTabFormat, sam::SAMFormat,
    vcf::VCFFormat,
};

#[cfg(feature = "mzml")]
use super::mzml::MzMLFormat;

/// The type of file.
pub enum TCAFileType {
    /// FASTA file format.
    FASTA,
    /// FASTQ file format.
    FASTQ,
    /// VCF file format.
    VCF,
    /// BCF file format.
    BCF,
    /// GFF file format.
    GFF,
    /// BAM file format.
    BAM,
    /// SAM file format.
    SAM,
    /// Genbank file format.
    GENBANK,
    /// HMMER file format.
    HMMER,
    /// BED file format.
    BED,

    /// mzML file format.
    #[cfg(feature = "mzml")]
    MZML,
}

impl FromStr for TCAFileType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_uppercase();

        match s.as_str() {
            "FASTA" | "FA" | "FNA" => Ok(Self::FASTA),
            "FASTQ" | "FQ" => Ok(Self::FASTQ),
            "VCF" => Ok(Self::VCF),
            "BCF" => Ok(Self::BCF),
            "GFF" => Ok(Self::GFF),
            "BAM" => Ok(Self::BAM),
            "SAM" => Ok(Self::SAM),
            #[cfg(feature = "mzml")]
            "MZML" => Ok(Self::MZML),
            "GENBANK" | "GBK" | "GB" => Ok(Self::GENBANK),
            "HMMDOMTAB" => Ok(Self::HMMER),
            "BED" => Ok(Self::BED),
            _ => Err(()),
        }
    }
}

impl TCAFileType {
    /// Get the file format for the given file type.
    pub fn get_file_format(
        self,
        file_compression_type: FileCompressionType,
    ) -> Result<Arc<dyn FileFormat>, DataFusionError> {
        match self {
            Self::BAM => Ok(Arc::new(BAMFormat::default())),
            Self::BCF => Ok(Arc::new(BCFFormat::default())),
            Self::BED => Ok(Arc::new(BEDFormat::new(file_compression_type))),
            Self::FASTA => Ok(Arc::new(FASTAFormat::new(file_compression_type))),
            Self::FASTQ => Ok(Arc::new(FASTQFormat::new(file_compression_type))),
            Self::GENBANK => Ok(Arc::new(GenbankFormat::new(file_compression_type))),
            Self::GFF => Ok(Arc::new(GFFFormat::new(file_compression_type))),
            Self::HMMER => Ok(Arc::new(HMMDomTabFormat::new(file_compression_type))),
            Self::SAM => Ok(Arc::new(SAMFormat::default())),
            Self::VCF => Ok(Arc::new(VCFFormat::new(file_compression_type))),
            #[cfg(feature = "mzml")]
            Self::MZML => Ok(Arc::new(MzMLFormat::new(file_compression_type))),
        }
    }
}

/// Infer the file type from the file extension.
pub fn infer_tca_format(path: &str) -> Result<Arc<dyn FileFormat>, DataFusionError> {
    let mut exts = path.rsplit('.');
    let mut splitted = exts.next().unwrap_or("");

    let file_compression_type =
        FileCompressionType::from_str(splitted).unwrap_or(FileCompressionType::UNCOMPRESSED);

    if file_compression_type.is_compressed() {
        splitted = exts.next().unwrap_or("");
    }

    let file_type = TCAFileType::from_str(splitted).map_err(|_| {
        DataFusionError::Execution(format!(
            "Unable to infer file type from file extension: {}",
            path
        ))
    })?;

    let file_format = file_type.get_file_format(file_compression_type)?;

    Ok(file_format)
}
