use sha1::Sha1;
use std::io::Read;


pub struct Sha1Reader<T> {
    reader: T,
    sha1: Sha1
}

impl<T: Read> Sha1Reader<T> {
    pub fn new(reader: T) -> Self{
        Sha1Reader {
            reader,
            sha1: Sha1::new()
        }
    }
    pub fn digest(&self) -> [u8; 20] {
        self.sha1.digest().bytes()
    }
}

impl<T: Read> Read for Sha1Reader<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let result = self.reader.read(buf)?;
        self.sha1.update(buf);
        Ok(result)
    }
}