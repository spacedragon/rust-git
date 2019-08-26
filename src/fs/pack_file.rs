use crate::fs::pack_idx::PackIdx;
use nom::IResult;
use nom::bytes::complete::{tag, take};
use nom::number::complete::be_u32;
use crate::errors::*;
use crate::model::id::Id;
use crate::model::object::{GitObject, ObjectType, ObjectHeader};
use std::io::{BufReader, Read, Write};
use flate2::read::ZlibDecoder;
use crate::fs::locator::Locator;

use crate::model::tree::parse_id;
use std::cmp::min;
use crate::fs::content_reader::ContentReader;
use std::mem;

pub struct PackFile {
    mmap: Box<dyn AsRef<[u8]>>,
    idx: Option<PackIdx>,
    version: u32,
    count: u32,
}

impl PackFile {
    pub fn try_from(mmap: Box<dyn AsRef<[u8]>>) -> Result<Self> {
        let input = (*mmap).as_ref();
        let (_, (version, count)) = parse_header(input)
            .map_err(|_| ErrorKind::InvalidPackfile)?;
        let len = input.len();
        let bytes = &input[(len - 20)..len];
        let _id = Id::new(bytes);
        Ok(PackFile {
            mmap,
            version,
            count,
            idx: None,
        })
    }
    pub fn load_idx(&mut self, idx: PackIdx) {
        self.idx = Some(idx)
    }
    pub fn version(&self) -> u32 {
        self.version
    }
    pub fn count(&self) -> u32 {
        self.count
    }
    pub fn id(&self) -> Id {
        let input = (*(self.mmap)).as_ref();
        let len = input.len();
        let bytes = &input[(len - 20)..len];
        Id::new(bytes)
    }

    pub fn idx(&self)  -> &Option<PackIdx> {
        &self.idx
    }

    pub fn find_object(&self, id: &Id) -> Option<GitObject> {
        if let Some(idx) = &self.idx {
            if let Some((id, offset)) = idx.lookup(&id) {
                let (locator, object_type, object_length)
                    = self.read_object(offset).expect("parse object failed");
                let header = ObjectHeader {
                    object_type,
                    length: object_length,
                };
                return Some(GitObject::new(&id.clone(), header, locator));
            }
        }
        None
    }

    pub fn write_object(&self, offset: usize, range: (usize, usize), writer: &mut dyn Write) -> Result<u64> {
        let mmap = (*(self.mmap)).as_ref();
        let input = &mmap[offset..];
        let mut reader = BufReader::new(
            ZlibDecoder::new(BufReader::new(input))
        );
        if range.0 > 0 {
            std::io::copy(&mut reader.by_ref().take(range.0 as u64) , &mut std::io::sink())?;
        }
        let size = std::io::copy(&mut reader.take(range.1 as u64), writer)?;
        Ok(size)
    }

    pub fn read_object(&self, from_offset: usize) -> Result<(Locator, ObjectType, usize)> {
        let mmap = (*(self.mmap)).as_ref();
        let input = &mmap[from_offset..];
        match parse_object_header(input) {
            Ok((input, (pack_object_type, object_length))) => {
                let object_type = match pack_object_type {
                    PackObjectType::COMMIT => ObjectType::COMMIT,
                    PackObjectType::TAG => ObjectType::TAG,
                    PackObjectType::TREE => ObjectType::TREE,
                    _ => ObjectType::BLOB,
                };
                let locator = match pack_object_type {
                    PackObjectType::OFS_DELTA => {
                        let (input, delta_offset) = parse_offset(input)
                            .map_err(|_|ErrorKind::ParseError)?;
                        let data_offset = input.as_ptr() as usize - mmap.as_ptr() as usize;
                        Locator::PackOfs(self.id(), data_offset, from_offset - delta_offset)
                    },
                    PackObjectType::REF_DELTA => {
                        let (input, ref_id) = parse_id(input)
                            .map_err(|_|ErrorKind::ParseError)?;
                        let data_offset = input.as_ptr() as usize - mmap.as_ptr() as usize;
                        Locator::PackRef(self.id(), data_offset, ref_id)
                    }
                    _ => {
                        let data_offset = input.as_ptr() as usize - mmap.as_ptr() as usize;
                        Locator::Packfile(self.id(), data_offset)
                    }
                };
                Ok((locator, object_type, object_length))
            },
            Err(_e) => Err(ErrorKind::ParseError.into())
        }
    }

    pub fn read_object_content(&self, from_offset: usize, size: usize) -> Result<ContentReader> {
        let pack_reader = PackZlibReader::new(self, from_offset, size);
        Ok(ContentReader::from_pack(pack_reader))
    }
}

pub struct PackZlibReader<'a> {
    reader: ZlibDecoder<&'a [u8]>,
    pos: usize,
    input: &'a [u8],
    size: usize,
}

impl <'a> PackZlibReader<'a> {
    fn new(pack: &'a PackFile, offset: usize, size: usize) -> Self {
        let mmap = (*(pack.mmap)).as_ref();
        let input = &mmap[offset..];
        let reader = ZlibDecoder::new(input);
        Self {
            input,
            reader,
            size,
            pos: 0
        }
    }
    pub(crate) fn reset(&mut self) {
        let input = self.input;
        mem::replace(&mut self.reader, ZlibDecoder::new(input));
        self.pos = 0;
    }
}

impl <'a> Read for PackZlibReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let remain = self.size - self.pos;
        if remain > 0 {
            let len = min(remain, buf.len());
            let len = self.reader.read(&mut buf[..len])?;
            self.pos += len;
            Ok(len)
        } else {
            Ok(0)
        }
    }
}


fn parse_header(input: &[u8]) -> IResult<&[u8], (u32, u32)> {
    let (input, _) = tag("PACK")(input)?;
    let (input, version) = be_u32(input)?;
    let (input, count) = be_u32(input)?;
    Ok((input, (version, count)))
}

fn parse_object_header(input: &[u8]) -> IResult<&[u8], (PackObjectType, usize)> {
    let _header_bytes: Vec<u8> = vec![];
    let _size = 0u64;
    let (mut input, byte) = take(1u8)(input)?;
    let mut byte = byte[0];
    let object_type = (byte & 0b0111_0000) >> 4;
    let object_type = match object_type {
        1 => PackObjectType::COMMIT,
        2 => PackObjectType::TREE,
        3 => PackObjectType::BLOB,
        4 => PackObjectType::TAG,
        6 => PackObjectType::OFS_DELTA,
        7 => PackObjectType::REF_DELTA,
        _ => return Err(
            nom::Err::Failure(nom::error::make_error(input, nom::error::ErrorKind::IsNot))
        )
    };
    let mut size = (byte & 0b0000_1111) as usize;
    let mut shift = 4;
    while byte >= 128u8 {
        let (rest, b) = take(1u8)(input)?;
        byte = b[0];
        input = rest;
        let value = (byte & 0b0111_1111) as usize;
        size |= value << shift;
        shift += 7;
    }

    Ok((input, (object_type, size)))
}

fn parse_offset(input: &[u8]) -> IResult<&[u8], usize> {
    let (mut input, byte) = take(1u8)(input)?;
    let mut byte = byte[0];    
    let mut offset = (byte & 0b0111_1111) as usize;
    while byte >= 128u8 {
        offset += 1;
        offset <<= 7;
        let (rest, b) = take(1u8)(input)?;
        byte = b[0];
        input = rest;
        let value = (byte & 0b0111_1111) as usize;
        offset += value;
    }
    Ok((input, offset))
}

enum PackObjectType {
    COMMIT = 1,
    TREE = 2,
    BLOB = 3,
    TAG = 4,
    OFS_DELTA = 6,
    REF_DELTA = 7,
}

#[cfg(test)]
mod tests {
    #[test]
    fn read_file() {
//        let dir = Path::new("/Users/draco/workspace/elastic/kibana/data/code/repos/github.com/JetBrains/intellij-community/objects/pack");
//        let idx = dir.join("pack-37258d77206ddb14788b8df17a9c9bdd362d0c75.idx");
//        let os = OsFs;
//        let reader = os.read_file(idx).expect("read file failed.");
//        let idx: PackIdx = reader.try_into().expect("parse idx failed");
//        assert_eq!(idx.version(), 2);
//        let id = Id::from_str("ea8b4605b3505a7c6abbe532e627076b388ccd31").expect("");
//        let offset = idx.find_offset(&id);
//        println!("{:?}", offset);
    }
}