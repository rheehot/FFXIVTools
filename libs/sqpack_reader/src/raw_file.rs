#[cfg(feature = "std")]
use std::iter;

use compression::prelude::DecodeExt;
use compression::prelude::Deflater;
use nom::number::complete::le_u32;
use nom::{do_parse, named};

#[cfg(feature = "std")]
use bytes::BufMut;
use bytes::{Bytes, BytesMut};

use util::{parse, round_up};

struct BlockHeader {
    pub header_size: u32,
    pub compressed_length: u32, // 32000 if not compressed
    pub uncompressed_length: u32,
}

impl BlockHeader {
    const SIZE: usize = 16;

    #[rustfmt::skip]
    named!(pub parse<Self>,
        do_parse!(
            header_size:            le_u32  >>
            /* unk: */              le_u32  >>
            compressed_length:      le_u32  >>
            uncompressed_length:    le_u32  >>
            (Self {
                header_size,
                compressed_length,
                uncompressed_length,
            })
        )
    );
}

struct CompressedFileHeader {
    uncompressed_size: u32,
    header_size: u32,
    block_count: u32,
}

impl CompressedFileHeader {
    const SIZE: usize = 12;

    #[rustfmt::skip]
    named!(pub parse<Self>,
        do_parse!(
            uncompressed_size:  le_u32  >>
            header_size:        le_u32  >>
            block_count:        le_u32  >>
            (Self {
                uncompressed_size,
                header_size,
                block_count,
            })
        )
    );
}

pub struct SqPackRawFile {
    uncompressed_size: u32,
    header: Bytes,
    blocks: Vec<Bytes>,
}

impl SqPackRawFile {
    pub fn from_compressed_file(data: Bytes) -> Self {
        let file_header = parse!(&data, CompressedFileHeader);

        let header = data.slice(CompressedFileHeader::SIZE..CompressedFileHeader::SIZE + file_header.header_size as usize);

        let begin = CompressedFileHeader::SIZE + file_header.header_size as usize;
        let blocks = (0..file_header.block_count)
            .scan(begin, |offset, _| {
                let block_size = Self::get_block_size(&data[*offset..*offset + BlockHeader::SIZE]);
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

    #[cfg(feature = "std")]
    pub fn from_contiguous_block(uncompressed_size: u32, header: Bytes, block_data: Bytes, block_sizes: Vec<u16>) -> Self {
        let blocks = Self::iterate_blocks(block_data, block_sizes).collect::<Vec<_>>();

        Self {
            uncompressed_size,
            header,
            blocks,
        }
    }

    #[cfg(feature = "std")]
    pub fn from_contiguous_blocks(uncompressed_size: u32, header: Bytes, contiguous_blocks: Vec<(Bytes, Vec<u16>)>) -> Self {
        let blocks = contiguous_blocks
            .into_iter()
            .flat_map(|(block_data, block_sizes)| Self::iterate_blocks(block_data, block_sizes))
            .collect::<Vec<_>>();

        Self {
            uncompressed_size,
            header,
            blocks,
        }
    }

    pub fn into_decoded(self) -> Bytes {
        let mut result = BytesMut::with_capacity(self.uncompressed_size as usize + self.header.len());
        result.extend(self.header);

        for block in self.blocks {
            Self::decode_block_into(&block, &mut result);
        }

        result.freeze()
    }

    #[cfg(feature = "std")]
    pub fn into_compressed(self) -> Bytes {
        let mut result = BytesMut::with_capacity(self.uncompressed_size as usize + CompressedFileHeader::SIZE);
        result.put_u32_le(self.uncompressed_size);
        result.put_u32_le(self.header.len() as u32);
        result.put_u32_le(self.blocks.len() as u32);

        for block in self.blocks {
            let block_size = Self::get_block_size(&block);
            result.extend(&block[0..block_size]);

            let rounded_size = round_up(block_size, 4);
            result.extend(iter::repeat(0).take(rounded_size - block_size));
        }

        result.freeze()
    }

    #[cfg(feature = "std")]
    fn iterate_blocks(block_data: Bytes, block_sizes: Vec<u16>) -> impl Iterator<Item = Bytes> {
        block_sizes.into_iter().scan(0usize, move |offset, block_size| {
            let result = block_data.slice(*offset..);
            *offset += block_size as usize;

            Some(result)
        })
    }

    fn get_block_size(block: &[u8]) -> usize {
        let header = parse!(&block, BlockHeader);

        if header.compressed_length >= 32000 {
            header.header_size as usize + header.uncompressed_length as usize
        } else {
            header.header_size as usize + header.compressed_length as usize
        }
    }

    fn decode_block_into(block: &[u8], result: &mut BytesMut) {
        let header = parse!(&block, BlockHeader);

        if header.compressed_length >= 32000 {
            result.extend(&block[header.header_size as usize..header.header_size as usize + header.uncompressed_length as usize]);
        } else {
            let data = &block[header.header_size as usize..header.header_size as usize + header.compressed_length as usize];

            result.extend(data.iter().cloned().decode(&mut Deflater::new()).collect::<Result<Vec<_>, _>>().unwrap());
        }
    }
}