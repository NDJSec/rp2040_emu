use std::io;
use std::io::{ErrorKind, Read};
use byteorder::{LittleEndian, ReadBytesExt};
use bitflags::bitflags;

bitflags! {
    pub struct UF2Flags: u32 {
        const NOT_MAIN_FLASH = 0x00000001;
        const FILE_CONTAINER = 0x00001000;
        const MD5_CHECKSUM = 0x00004000;
        const EXTENSION_TAGS = 0x00008000;
    }
}

pub struct Uf2Block {
    magic_start0: u32,
    magic_start1: u32,
    flags: UF2Flags,
    target_addr: u32,
    payload_size: u32,
    block_num: u32,
    num_blocks: u32,
    family_id: u32, // Also fileSize
    data: [u8; 476],
    magic_end: u32
}

pub struct UF2File {
    pub(crate) blocks: Vec<Uf2Block>
}

impl Uf2Block {
    const MAGIC0: u32 = 0x0A324655;
    const MAGIC1: u32 = 0x9E5D5157;
    const MAGIC_END: u32 = 0xAB16F30;

    pub fn parse_uf2_block<R: Read>(reader: &mut R) -> io::Result<Self> {
        let magic_start0 = reader.read_u32::<LittleEndian>()?;
        let magic_start1 = reader.read_u32::<LittleEndian>()?;

        if magic_start0 != Self::MAGIC0 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Invalid magic_start0: expected 0x{:08X}, got 0x{:08X}",
                    Self::MAGIC0, magic_start0
                )
            ));
        }

        if magic_start1 != Self::MAGIC1 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Invalid magic_start1: expected 0x{:08X}, got 0x{:08X}",
                        Self::MAGIC1, magic_start1
                )
            ));
        }

        let raw_flags = reader.read_u32::<LittleEndian>()?;
        let flags = UF2Flags::from_bits_truncate(raw_flags);
        let target_addr = reader.read_u32::<LittleEndian>()?;
        let payload_size = reader.read_u32::<LittleEndian>()?;
        let block_num = reader.read_u32::<LittleEndian>()?;
        let num_blocks = reader.read_u32::<LittleEndian>()?;
        let family_id = reader.read_u32::<LittleEndian>()?;
        let mut data = [0u8; 476];
        reader.read_exact(&mut data)?;

        let magic_end = reader.read_u32::<LittleEndian>()?;
        if magic_end != Self::MAGIC_END {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Invalid magic_end: expected 0x{:08X}, got 0x{:08X}",
                        Self::MAGIC_END, magic_end
                )
            ));
        }


        Ok(Self {
            magic_start0,
            magic_start1,
            flags,
            target_addr,
            payload_size,
            block_num,
            num_blocks,
            family_id,
            data,
            magic_end
        })
    }
}

impl UF2File {
    pub(crate) fn parse_file<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut blocks = Vec::new();

        while let Ok(block) = Uf2Block::parse_uf2_block(reader) {
            blocks.push(block);
        }

        if blocks.is_empty() {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "No valid UF2 blocks found",
            ));
        }

        Ok(Self { blocks })
    }

    pub(crate) fn total_payload_size(&self) -> usize {
        self.blocks.iter().map(|block| block.payload_size as usize).sum()
    }

    pub(crate) fn verify(&self) -> bool {
        self.blocks
            .iter()
            .enumerate()
            .all(|(i, block)| block.block_num as usize == i)
    }
}

