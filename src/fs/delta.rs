use std::io::Read;
use crate::fs::content_reader::ContentReader;

enum State {
    NEXT,
    COPY(usize, usize),
    INSERT(usize),
    DONE
}

pub struct DeltaReader<'a> {
    base: Box<ContentReader<'a>>,
    delta: Box<ContentReader<'a>>,
    state: State,
    size: u64,
    pos: u64
}

impl<'a> DeltaReader<'a> {
    pub fn new(base: ContentReader<'a>, mut delta: ContentReader<'a>, _size: usize) -> std::io::Result<Self> {
        let _base_size =  delta.read_varint()?;
        let output_size =  delta.read_varint()?;
        Ok(Self {
            delta: Box::new(delta),
            base: Box::new(base),
            state: State::NEXT,
            size: output_size,
            pos: 0
        })
    }

    pub(crate) fn reset(&mut self) {
        self.base.reset();
        self.delta.reset();
        self.pos = 0;
    }
}

impl<'a> Read for DeltaReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            let remain = self.size - self.pos;
            match self.state {
                State::NEXT => {
                    if remain == 0 {
                        self.state = State::DONE;
                        continue
                    }
                    let mut instruction = [0u8];
                    self.delta.read_exact(&mut instruction)?;
                    let instruction = instruction[0];
                    if instruction >= 128u8 {
                        let mut offset: usize = 0;
                        for i in 0..4 {
                           if instruction & (1u8 << i) > 0 {
                               let mut value = [0u8];
                               self.delta.read_exact(&mut value)?;
                               let value = value[0] as usize;
                               offset += value << (8 * i) as usize;
                           } 
                        }
                        let mut length: usize = 0;
                        for i in 0..3 {
                            if instruction & (0b1_0000 << i) > 0 {
                               let mut value = [0u8];
                               self.delta.read_exact(&mut value)?;
                               let value = value[0] as usize;
                               length += value << (8 * i) as usize;
                           } 
                        }
                        self.state = State::COPY(offset, length)
                    } else {
                        self.state = State::INSERT((instruction & 0b0111_1111) as usize);
                    }
                },
                State::COPY(offset, length) => {
                    if buf.len() >= length {
                        self.base.forward(offset)?;
                        self.base.read_exact(&mut buf[..length])?;
                        self.state = State::NEXT;
                        self.pos += length as u64;
                        return Ok(length);
                    } else {
                        self.base.forward(offset)?;
                        self.base.read_exact(buf)?;
                        let len = buf.len();
                        self.state = State::COPY(offset + len, length - len);
                        self.pos += len as u64;
                        return Ok(len);
                    }
                },
                State::INSERT(length) => {
                    if buf.len() >= length as usize {
                        self.delta.read_exact(&mut buf[..length])?;
                        self.state = State::NEXT;
                        self.pos += length as u64;
                        return Ok(length);
                    } else {
                        self.base.read_exact(buf)?;
                        let len = buf.len();
                        self.state = State::INSERT(length - len);
                        self.pos += len as u64;
                        return Ok(len);
                    }
                },
                State::DONE => {
                    return Ok(0);
                }
            }
        }
    }
}