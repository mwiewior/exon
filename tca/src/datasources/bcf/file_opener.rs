use std::sync::Arc;

use datafusion::{error::DataFusionError, physical_plan::file_format::FileOpener};
use futures::{StreamExt, TryStreamExt};
use tokio_util::io::StreamReader;

use super::{batch_reader::BatchReader, config::BCFConfig};

/// A file opener for BCF files.
pub struct BCFOpener {
    config: Arc<BCFConfig>,
}

impl BCFOpener {
    /// Create a new BCF file opener.
    pub fn new(config: Arc<BCFConfig>) -> Self {
        Self { config }
    }
}

impl FileOpener for BCFOpener {
    fn open(
        &self,
        file_meta: datafusion::physical_plan::file_format::FileMeta,
    ) -> datafusion::error::Result<datafusion::physical_plan::file_format::FileOpenFuture> {
        let config = self.config.clone();

        Ok(Box::pin(async move {
            let get_result = config.object_store.get(file_meta.location()).await?;

            let stream_reader = Box::pin(get_result.into_stream().map_err(DataFusionError::from));

            let stream_reader = StreamReader::new(stream_reader);
            let batch_reader = BatchReader::new(stream_reader, config).await?;

            let batch_stream = batch_reader.into_stream();

            Ok(batch_stream.boxed())
        }))
    }
}
