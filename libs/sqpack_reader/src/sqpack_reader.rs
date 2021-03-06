mod archive;
mod archive_container;
mod data;
mod definition;
mod index;

use alloc::boxed::Box;
use std::io;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use futures::{
    stream::{FuturesUnordered, TryStreamExt},
    FutureExt,
};
use hashbrown::HashMap;
use log::debug;

use crate::archive_id::SqPackArchiveId;
use crate::error::Result;
use crate::package::{BatchablePackage, Package};
use crate::reference::SqPackFileReference;

use archive::SqPackArchive;
use archive_container::SqPackArchiveContainer;

pub struct SqPackReader {
    archives: SqPackArchiveContainer,
}

impl SqPackReader {
    pub fn new(base_dir: &Path) -> io::Result<Self> {
        Ok(Self {
            archives: SqPackArchiveContainer::new(base_dir)?,
        })
    }

    pub async fn archive(&self, archive_id: SqPackArchiveId) -> io::Result<Arc<SqPackArchive>> {
        self.archives.get_archive(archive_id).await
    }

    pub async fn read_as_compressed(&self, path: &str) -> Result<Vec<u8>> {
        debug!("Reading {}", path);

        let reference = SqPackFileReference::new(path);
        let archive = self.archive(reference.archive_id).await?;
        let result = archive.read_as_compressed(reference.hash.folder, reference.hash.file).await;

        if result.is_err() {
            debug!("No such file {}", path);
        }
        result
    }
}

#[async_trait]
impl Package for SqPackReader {
    async fn read_file_by_reference(&self, reference: &SqPackFileReference) -> Result<Vec<u8>> {
        let archive = self.archive(reference.archive_id).await?;

        let result = archive.read_file(reference.hash.folder, reference.hash.file).await;

        #[cfg(debug_assertions)]
        if result.is_err() {
            debug!("No such file {}", reference.path);
        }

        result
    }
}

#[async_trait]
impl BatchablePackage for SqPackReader {
    async fn read_files(&self, references: &[&SqPackFileReference]) -> Result<HashMap<SqPackFileReference, Vec<u8>>> {
        references
            .iter()
            .map(|reference| self.read_file_by_reference(reference).map(move |x| Ok(((*reference).to_owned(), x?))))
            .collect::<FuturesUnordered<_>>()
            .try_collect::<HashMap<_, _>>()
            .await
    }
}
