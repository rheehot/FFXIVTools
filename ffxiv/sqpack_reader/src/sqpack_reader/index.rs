use std::io;
use std::path::Path;

use tokio::fs::File;

use super::definition::{FileSegment, FolderSegment, SqPackHeader, SqPackIndexHeader};
use crate::common::{ReadExt, SqPackFileReference};

pub struct SqPackIndex {
    pub dat_count: u32,
    folder_segments: Vec<FolderSegment>,
    file_segments: Vec<FileSegment>,
    file_segment_base: u32,
}

macro_rules! read_segment {
    ($file: expr, $segment: expr, $type: ty) => {
        read_and_parse!($file, $segment.offset, $segment.size as usize / <$type>::SIZE, $type)
    };
}

impl SqPackIndex {
    pub async fn new(path: &Path) -> io::Result<Self> {
        let mut f = File::open(path).await?;

        let sqpack_header = read_and_parse!(f, 0, SqPackHeader).await?;
        let index_header = read_and_parse!(f, sqpack_header.header_length, SqPackIndexHeader).await?;

        let folder_segments = read_segment!(f, index_header.folder_segment, FolderSegment).await?;
        let file_segments = read_segment!(f, index_header.file_segment, FileSegment).await?;

        Ok(Self {
            folder_segments,
            file_segments,
            file_segment_base: index_header.file_segment.offset,
            dat_count: index_header.dat_count,
        })
    }

    pub fn find_offset(&self, reference: &SqPackFileReference) -> io::Result<u32> {
        let folder_index = self
            .folder_segments
            .binary_search_by_key(&reference.folder_hash, |x| x.folder_hash)
            .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "No such folder"))?;
        let folder = &self.folder_segments[folder_index];

        let file_begin = (folder.file_list_offset - self.file_segment_base) as usize / FileSegment::SIZE;
        let file_end = file_begin + folder.file_list_size as usize / FileSegment::SIZE;
        let file_index = self.file_segments[file_begin..file_end]
            .binary_search_by_key(&reference.file_hash, |x| x.file_hash)
            .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "No such file"))?;
        let file = &self.file_segments[file_index + file_begin];

        Ok(file.data_offset)
    }
}