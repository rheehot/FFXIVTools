use std::io;
use std::path::Path;

use super::data::SqPackData;
use super::index::SqPackIndex;
use super::reference::SqPackFileReference;

#[derive(Eq, PartialEq, Hash, Default, Clone)]
pub struct SqPackArchiveId {
    pub root: u8,
    pub ex: u8,
    pub part: u8,
}

pub struct SqPackArchive {
    pub index: SqPackIndex,
    pub data: Vec<SqPackData>,
}

impl SqPackArchive {
    pub fn new(index_path: &Path) -> io::Result<SqPackArchive> {
        let index_path_str = index_path.to_str().unwrap();
        let base_path = index_path_str.trim_end_matches(".index");
        let index = SqPackIndex::new(index_path)?;

        let data = (0..index.dat_count)
            .map(|x| {
                let path_str = format!("{}.dat{}", base_path, x);
                Ok(SqPackData::new(Path::new(&path_str))?)
            })
            .collect::<io::Result<Vec<_>>>()?;
        Ok(SqPackArchive { index, data })
    }

    pub fn read_file(&mut self, reference: &SqPackFileReference) -> io::Result<Vec<u8>> {
        let file_offset = self.index.find_offset(reference)?;

        let dat_index = (file_offset & 0x0f) >> 1;
        let offset = (file_offset & 0xffff_fff0) << 3;

        Ok(self.data[dat_index as usize].read(offset as usize)?)
    }
}
