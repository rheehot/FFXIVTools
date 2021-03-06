use alloc::vec::Vec;
use core::mem::size_of;

use bytes::Bytes;
use miniz_oxide::inflate::decompress_to_vec;

use util::{cast, round_up};

#[repr(C)]
struct BlockHeader {
    pub header_size: u32,
    _unk: u32,
    pub compressed_length: u32, // 32000 if not compressed
    pub uncompressed_length: u32,
}

#[repr(C)]
struct CompressedFileHeader {
    uncompressed_size: u32,
    header_size: u32,
    block_count: u32,
}

pub struct SqPackRawFile {
    uncompressed_size: u32,
    header: Bytes,
    blocks: Vec<Bytes>,
}

impl SqPackRawFile {
    pub fn from_compressed_file(data: Vec<u8>) -> Self {
        let data = Bytes::from(data);
        let file_header = cast::<CompressedFileHeader>(&data);

        let header = data.slice(size_of::<CompressedFileHeader>()..size_of::<CompressedFileHeader>() + file_header.header_size as usize);

        let begin = size_of::<CompressedFileHeader>() + file_header.header_size as usize;
        let blocks = (0..file_header.block_count)
            .scan(begin, |offset, _| {
                let block_size = Self::get_block_size(&data[*offset..*offset + size_of::<BlockHeader>()]);
                let block = data.slice(*offset..*offset + block_size);

                *offset += round_up(block_size, 4usize);

                Some(block)
            })
            .collect::<Vec<_>>();

        Self {
            uncompressed_size: file_header.uncompressed_size,
            header,
            blocks,
        }
    }

    #[cfg(feature = "std")]
    pub fn from_blocks(uncompressed_size: u32, header: Bytes, blocks: Vec<Bytes>) -> Self {
        Self {
            uncompressed_size,
            header,
            blocks,
        }
    }

    pub fn into_decoded(self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.uncompressed_size as usize + self.header.len());
        result.extend(self.header);
        if result.len() == 4 {
            result.resize(result.len() + 0x40, 0); // mdl files has 0x44 bytes of header
        }

        for block in self.blocks {
            Self::decode_block_into(&block, &mut result);
        }

        result
    }

    #[cfg(feature = "std")]
    pub fn into_compressed(self) -> Vec<u8> {
        use core::iter;

        let mut result = Vec::with_capacity(self.uncompressed_size as usize + size_of::<CompressedFileHeader>());
        result.extend(self.uncompressed_size.to_le_bytes().iter());
        result.extend((self.header.len() as u32).to_le_bytes().iter());
        result.extend((self.blocks.len() as u32).to_le_bytes().iter());

        for block in self.blocks {
            let block_size = Self::get_block_size(&block);
            result.extend(&block[0..block_size]);

            let rounded_size = round_up(block_size, 4);
            result.extend(iter::repeat(0).take(rounded_size - block_size));
        }

        result
    }

    fn get_block_size(block: &[u8]) -> usize {
        let header = cast::<BlockHeader>(&block);

        if header.compressed_length >= 32000 {
            header.header_size as usize + header.uncompressed_length as usize
        } else {
            header.header_size as usize + header.compressed_length as usize
        }
    }

    fn decode_block_into(block: &[u8], result: &mut Vec<u8>) {
        let header = cast::<BlockHeader>(&block);

        if header.compressed_length >= 32000 {
            result.extend(&block[header.header_size as usize..header.header_size as usize + header.uncompressed_length as usize]);
        } else {
            let data = &block[header.header_size as usize..header.header_size as usize + header.compressed_length as usize];

            result.extend(decompress_to_vec(data).unwrap());
        }
    }
}
