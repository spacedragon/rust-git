use std::convert::TryFrom;
use std::io::Read;
use crate::errors::*;
use crate::fs::SeekRead;
use nom::IResult;
use nom::multi::count;
use nom::number::complete::{be_u32, be_u64};
use crate::model::id::Id;
use crate::model::tree::parse_id;
use crate::fs::checksum::Sha1Reader;

use std::cmp::Ordering;

pub struct PackIdxV1 {
    fanout: [u32; 255],
    objects: Vec<Id>,
    offsets: Vec<u32>,
    pack_id: Id,
}

pub struct PackIdxV2 {
    fanout: [u32; 255],
    objects: Vec<Id>,
    offsets: Vec<u32>,
    crcs: Vec<u32>,
    large_offsets: Vec<u64>,
    pack_id: Id,
}

pub enum PackIdx {
    V1(PackIdxV1),
    V2(PackIdxV2),
}

impl PackIdx {
    pub fn pack_id(&self) -> &Id {
        match self {
            PackIdx::V1(a) => &a.pack_id,
            PackIdx::V2(a) => &a.pack_id,
        }
    }

    pub fn version(&self) -> u8 {
        match self {
            PackIdx::V1(_) => 1,
            PackIdx::V2(_) => 2,
        }
    }

    pub fn find_offset(&self, id: &Id) -> Option<u32> {
        let (fanout, offsets, objects) = match self {
            PackIdx::V1(a) => (&a.fanout, &a.offsets, &a.objects),
            PackIdx::V2(a) => (&a.fanout, &a.offsets, &a.objects),
        };
        let first_byte = id.bytes()[0] as usize;
        let mut lo = if first_byte > 0 {
            fanout[first_byte - 1] as usize
        } else { 0usize };
        let mut hi = fanout[first_byte] as usize;

        loop {
            let mid = ((hi + lo) / 2) as usize;
            let mid_id = &objects[mid];
            match id.partial_cmp(mid_id) {
                Some(Ordering::Less) => { hi = mid }
                Some(Ordering::Greater) => { lo = mid + 1 }
                Some(Ordering::Equal) => return Some(offsets[mid]),
                _ => return None,
            }
        }
    }
}

fn verify_checksum(f: &mut Sha1Reader<Box<dyn SeekRead>>) -> Result<()> {
    let sha1_checksum = f.digest();
    let mut checksum = [0u8; 20];
    f.read_exact(&mut checksum)?;
    if sha1_checksum != checksum {
        return Err(ErrorKind::ChecksumMismatch.into());
    }
    Ok(())
}

fn get_pack_id(f: &mut Sha1Reader<Box<dyn SeekRead>>) -> Result<Id> {
    let mut buf = [0u8; 20];
    f.read_exact(&mut buf)?;
    Ok(Id::new(&buf))
}

impl TryFrom<Box<dyn SeekRead>> for PackIdx where {
    type Error = Error;

    fn try_from(f: Box<dyn SeekRead>) -> Result<Self> {
        let mut f = Sha1Reader::new(f);
        let mut magic = [0u8; 4];
        f.read_exact(&mut magic)?;
        let mut version = [0u8; 4];
        f.read_exact(&mut version)?;

        if &magic != b"\xfftOc" {
            return parse_idx_v1(&mut f, magic, version);
        }

        if version != [0u8, 0u8, 0u8, 2u8] {
            return Err(ErrorKind::UnsupportedPackIndexVersion.into());
        }
        parse_idx_v2(&mut f)
    }
}


#[inline]
fn parse_offset(input: &[u8]) -> IResult<&[u8], u32> {
    be_u32(input)
}

#[inline]
fn parse_large_offset(input: &[u8]) -> IResult<&[u8], u64> {
    be_u64(input)
}

#[inline]
fn parse_crc(input: &[u8]) -> IResult<&[u8], u32> {
    be_u32(input)
}

fn parse_entry(input: &[u8]) -> IResult<&[u8], (u32, Id)> {
    let (input, offset) = parse_offset(input)?;
    let (input, id) = parse_id(input)?;
    Ok((input, (offset, id)))
}

fn parse_entries(input: &[u8], size: usize) -> IResult<&[u8], (Vec<u32>, Vec<Id>)> {
    let mut rest = input;
    let mut objects = Vec::with_capacity(size);
    let mut offsets = Vec::with_capacity(size);
    for i in 0..size {
        let (input, (offset, id)) = parse_entry(rest)?;
        objects[i] = id;
        offsets[i] = offset;
        rest = input;
    }
    Ok((input, (offsets, objects)))
}

fn parse_fanout(input: &[u8]) -> IResult<&[u8], ([u32; 255], usize)> {
    let mut fanout = [0u32; 255];
    let mut rest = input;
    for i in fanout.iter_mut() {
        let (input, value) = be_u32(rest)?;
        *i = value;
        rest = input;
    }
    let (input, size) = be_u32(rest)?;
    Ok((input, (fanout, size as usize)))
}


fn parse_idx_v1(mut f: &mut Sha1Reader<Box<SeekRead>>, magic: [u8; 4], version: [u8; 4]) -> Result<PackIdx> {
    // parse v1
    let mut fanout_buf = [0u8; 256 * 4];
    fanout_buf[0..4].clone_from_slice(&magic);
    fanout_buf[4..8].clone_from_slice(&version);
    f.read_exact(&mut fanout_buf[8..])?;
    let (_, (fanout, size)) = parse_fanout(&fanout_buf)
        .map_err(|_| ErrorKind::ParseError)?;
    let mut buf = Vec::with_capacity(24 * size);
    f.read_exact(buf.as_mut_slice())?;
    let (_, (offsets, objects)) = parse_entries(buf.as_slice(), size)
        .map_err(|_| ErrorKind::ParseError)?;
    let mut buf = [0u8; 20];
    f.read_exact(&mut buf)?;
    let pack_id = get_pack_id(&mut f)?;
    verify_checksum(&mut f)?;
    Ok(PackIdx::V1(PackIdxV1 {
        pack_id,
        fanout,
        offsets,
        objects,
    }))
}


fn parse_idx_v2(mut f: &mut Sha1Reader<Box<SeekRead>>) -> Result<PackIdx> {
// parse v2
    let mut fanout_buf = [0u8; 256 * 4];
    f.read_exact(&mut fanout_buf)?;
    let (_, (fanout, size)) = parse_fanout(&fanout_buf)
        .map_err(|_| ErrorKind::ParseError)?;
    let mut buf = vec![0u8; 20 * size];
    f.read_exact(&mut buf)?;
    let (_, objects) = count(parse_id, size)(&buf)
        .map_err(|_| ErrorKind::ParseError)?;
    let mut buf = vec![0u8; 4 * size];
    f.read_exact(&mut buf)?;
    let (_, crcs) = count(parse_crc, size)(&buf)
        .map_err(|_| ErrorKind::ParseError)?;
    f.read_exact(&mut buf)?;
    let (_, offsets) = count(parse_offset, size)(buf.as_slice())
        .map_err(|_| ErrorKind::ParseError)?;
    let large_offset_count: usize = offsets.iter().filter(|o| *o & 0x8000_0000 > 0).count();
    let large_offsets: Vec<u64> = if large_offset_count > 0 {
        let mut buf = vec![0u8; 8 * size];
        f.read_exact(&mut buf)?;
        let (_, offsets) = count(parse_large_offset, large_offset_count)(&buf)
            .map_err(|_| ErrorKind::ParseError)?;
        offsets
    } else {
        vec![]
    };
    let pack_id = get_pack_id(&mut f)?;
    verify_checksum(&mut f)?;
    Ok(PackIdx::V2(PackIdxV2 {
        pack_id,
        fanout,
        offsets,
        crcs,
        objects,
        large_offsets,
    }))
}

