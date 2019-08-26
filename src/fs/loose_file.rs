use crate::fs::{SeekRead, Forwardable};
use flate2::read::ZlibDecoder;
use std::io::{SeekFrom, Read, Seek};
use std::mem;

pub struct LooseFileReader {
    reader: ZlibDecoder<Box<dyn SeekRead>>,
    init_file_pos: u64,
    init_offset: usize,
    pos: u64,
    size: usize
}
pub struct Empty;

impl Seek for Empty {
    fn seek(&mut self, _pos: SeekFrom) -> std::io::Result<u64> {
        Ok(0)
    }
}

impl Read for Empty {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize>  {
        Ok(0)
    }
}

impl LooseFileReader {
    pub fn new(mut file_reader: Box<dyn SeekRead>, init_offset: usize, size: usize) -> Self {
        let init_file_pos = file_reader.seek(SeekFrom::Current(0)).expect("file reader can't seek?");
        let mut reader = ZlibDecoder::new(file_reader);
        if init_offset > 0 {
            reader.forward(init_offset as u64).expect("");
        }
        Self {
            reader,
            init_file_pos,
            init_offset,
            pos: 0,
            size
        }
    }

    pub(crate) fn reset(&mut self) {
        let empty : Box<dyn SeekRead> = Box::new(Empty);
        let empty_reader = ZlibDecoder::new(empty);
        let reader = mem::replace(&mut self.reader, empty_reader);
        let mut file_reader = reader.into_inner();
        file_reader.seek(SeekFrom::Start(self.init_file_pos)).expect("");
        let new_reader =  ZlibDecoder::new(file_reader);
        mem::replace(&mut self.reader, new_reader);
        if self.init_offset > 0 {
            self.reader.forward(self.init_offset as u64).expect("");
        }
        self.pos = 0;
    }
}

impl Read for LooseFileReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

impl Seek for LooseFileReader {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(pos) =>  pos,
            SeekFrom::End(pos) => { (self.size as u64).wrapping_add(pos as u64)   },
            SeekFrom::Current(pos) => { self.pos.wrapping_add(pos as u64) }
        };
        if new_pos > self.pos {
            self.forward(new_pos - self.pos)?
        } else {
            self.reset();
            self.forward(new_pos)?;
        }
        Ok(self.pos)
    }
}