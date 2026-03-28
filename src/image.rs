use std::io::{self, BufRead, Read, Seek, SeekFrom};

use imagesize::{blob_size, reader_size};

const PREFIX_SIZE: usize = 1024;
const CHUNK_SIZE: usize = 32 * 1024;

pub fn read_dimensions<R: Read>(mut reader: R) -> io::Result<(u32, u32)> {
    let mut prefix = vec![0u8; PREFIX_SIZE];
    let n = reader.read(&mut prefix)?;
    prefix.truncate(n);

    if let Ok(size) = blob_size(&prefix) {
        return Ok((size.width as u32, size.height as u32));
    }

    let size = reader_size(CachedReadSeeker::new(reader, prefix, n < PREFIX_SIZE))
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to read image dimensions: {err}")))?;

    Ok((size.width as u32, size.height as u32))
}

struct CachedReadSeeker<R> {
    reader: R,
    buffer: Vec<u8>,
    position: usize,
    eof: bool,
}

impl<R: Read> CachedReadSeeker<R> {
    fn new(reader: R, buffer: Vec<u8>, eof: bool) -> Self {
        Self { reader, buffer, position: 0, eof }
    }

    fn ensure_available(&mut self, end: usize) -> io::Result<()> {
        while !self.eof && self.buffer.len() < end {
            let mut chunk = [0u8; CHUNK_SIZE];
            let n = self.reader.read(&mut chunk)?;
            if n == 0 {
                self.eof = true;
            } else {
                self.buffer.extend_from_slice(&chunk[..n]);
            }
        }
        Ok(())
    }

    fn read_all(&mut self) -> io::Result<()> {
        while !self.eof {
            self.ensure_available(self.buffer.len() + CHUNK_SIZE)?;
        }
        Ok(())
    }
}

impl<R: Read> Read for CachedReadSeeker<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.ensure_available(self.position + buf.len())?;
        let n = buf.len().min(self.buffer.len().saturating_sub(self.position));
        buf[..n].copy_from_slice(&self.buffer[self.position..self.position + n]);
        self.position += n;
        Ok(n)
    }
}

impl<R: Read> BufRead for CachedReadSeeker<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.ensure_available(self.position + 1)?;
        Ok(&self.buffer[self.position..])
    }

    fn consume(&mut self, amt: usize) {
        self.position = (self.position + amt).min(self.buffer.len());
    }
}

impl<R: Read> Seek for CachedReadSeeker<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let target = match pos {
            SeekFrom::Start(n) => n as i64,
            SeekFrom::Current(n) => self.position as i64 + n,
            SeekFrom::End(n) => {
                self.read_all()?;
                self.buffer.len() as i64 + n
            }
        };

        if target < 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "seek before start"));
        }

        let target = target as usize;
        self.ensure_available(target)?;
        if target > self.buffer.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "seek past end"));
        }

        self.position = target;
        Ok(self.position as u64)
    }
}
