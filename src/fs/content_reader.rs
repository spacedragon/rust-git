use crate::fs::pack_file::PackZlibReader;
use crate::fs::delta::DeltaReader;
use std::io::{Read};
use crate::fs::loose_file::LooseFileReader;
use crate::fs::SeekRead;

pub enum Source<'a> {
    FromPack(PackZlibReader<'a>),
    FromLooseFile(LooseFileReader),
    Delta(DeltaReader<'a>)
}

pub struct ContentReader<'a> {
    pub(crate) source: Source<'a>,
    pos: usize
}

impl <'a> ContentReader<'a> {
    pub fn attach_base(self, base:ContentReader<'a>, size: usize) -> std::io::Result<Self> {
        let source = Source::Delta(DeltaReader::new(base, self, size)?);
        Ok(ContentReader {
            source,
            pos: 0
        })
    }

    pub fn from_pack(pack_reader: PackZlibReader<'a>) -> Self {
        let source = Source::FromPack(pack_reader);
        ContentReader {
            source,
            pos: 0
        }
    }

    pub fn from_loose_file(file_reader: Box<dyn SeekRead>, offset: usize, size: usize) -> Self {
        let reader = LooseFileReader::new(file_reader, offset, size);
        let source = Source::FromLooseFile(reader);
        ContentReader {
            source,
            pos: 0
        }
    }

    pub fn forward(&mut self, offset: usize) -> std::io::Result<usize> {

        if offset < self.pos {
            self.reset();
        }

        let forward = (offset - self.pos) as u64;
        if forward > 0 {
            std::io::copy(&mut self.by_ref().take((forward) as u64), &mut std::io::sink())?;
        }
        Ok(self.pos)

    }
    pub fn read_varint(&mut self) -> std::io::Result<u64> {
        let mut byte = self.read_byte()?;
        let mut result = u64::from(byte & 0b0111_1111);
        let mut shift = 7;
        while byte > 128u8 {
            byte = self.read_byte()?;
            result += u64::from(byte & 0b0111_1111) << shift;
            shift += 7;
        }
        Ok(result)
    }
    pub fn read_byte(&mut self) -> std::io::Result<u8> {
        let mut byte = [0u8];
        self.read_exact(&mut byte)?;
        Ok(byte[0])
    }

    pub(crate) fn reset(&mut self){
        match &mut self.source {
            Source::FromLooseFile(reader) => {
                reader.reset();
            }
            Source::FromPack(reader) => {
                reader.reset()
            }
            Source::Delta(reader) => {
                reader.reset()
            }
        };
        self.pos = 0;
    }
}

impl <'a> Read for ContentReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let size = match &mut self.source {
            Source::FromLooseFile(reader) => {
                reader.read(buf)?
            }
            Source::FromPack(reader) => {
                reader.read(buf)?
            }
            Source::Delta(reader) => {
                reader.read(buf)?
            }
        };
        self.pos += size;
        Ok(size)
    }
}