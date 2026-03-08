use std::io::{self, BufRead, Read, Seek, SeekFrom};

use imagesize::{ImageSize, blob_size, reader_size};

const DIMENSION_CHUNK_SIZE: usize = 32 * 1024;
const DIMENSION_PREFIX_SIZE: usize = 1024;

pub fn read_dimensions<R: Read>(mut reader: R) -> io::Result<[u32; 2]> {
    let mut prefix = vec![0_u8; DIMENSION_PREFIX_SIZE];
    let bytes_read = reader.read(&mut prefix)?;
    prefix.truncate(bytes_read);

    if let Ok(size) = blob_size(&prefix) {
        return to_dimensions(size);
    }

    let size = reader_size(CachedReadSeeker::with_buffer(reader, prefix, bytes_read < DIMENSION_PREFIX_SIZE))
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to read image dimensions: {err}")))?;

    to_dimensions(size)
}

fn to_dimensions(size: ImageSize) -> io::Result<[u32; 2]> {
    let width = u32::try_from(size.width)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("Image width {} exceeds u32", size.width)))?;
    let height = u32::try_from(size.height)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("Image height {} exceeds u32", size.height)))?;

    Ok([width, height])
}

struct CachedReadSeeker<R> {
    reader: R,
    buffer: Vec<u8>,
    position: usize,
    reached_eof: bool,
}

impl<R: Read> CachedReadSeeker<R> {
    fn with_buffer(reader: R, buffer: Vec<u8>, reached_eof: bool) -> Self {
        let mut cached = Vec::with_capacity(buffer.len().max(DIMENSION_CHUNK_SIZE));
        cached.extend_from_slice(&buffer);
        Self { reader, buffer: cached, position: 0, reached_eof }
    }

    fn ensure_available(&mut self, end: usize) -> io::Result<()> {
        while !self.reached_eof && self.buffer.len() < end {
            let mut chunk = [0_u8; DIMENSION_CHUNK_SIZE];
            let bytes_read = self.reader.read(&mut chunk)?;
            if bytes_read == 0 {
                self.reached_eof = true;
                break;
            }
            self.buffer.extend_from_slice(&chunk[..bytes_read]);
        }

        Ok(())
    }

    fn read_to_end(&mut self) -> io::Result<()> {
        while !self.reached_eof {
            self.ensure_available(self.buffer.len() + DIMENSION_CHUNK_SIZE)?;
        }

        Ok(())
    }
}

impl<R: Read> Read for CachedReadSeeker<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        self.ensure_available(self.position + buf.len())?;
        let available = self.buffer.len().saturating_sub(self.position);
        let bytes_to_copy = available.min(buf.len());
        if bytes_to_copy == 0 {
            return Ok(0);
        }

        buf[..bytes_to_copy].copy_from_slice(&self.buffer[self.position..self.position + bytes_to_copy]);
        self.position += bytes_to_copy;
        Ok(bytes_to_copy)
    }
}

impl<R: Read> BufRead for CachedReadSeeker<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.ensure_available(self.position + 1)?;
        Ok(&self.buffer[self.position..])
    }

    fn consume(&mut self, amt: usize) {
        self.position = self.position.saturating_add(amt).min(self.buffer.len());
    }
}

impl<R: Read> Seek for CachedReadSeeker<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let target = match pos {
            SeekFrom::Start(offset) => i128::from(offset),
            SeekFrom::Current(offset) => self.position as i128 + i128::from(offset),
            SeekFrom::End(offset) => {
                self.read_to_end()?;
                self.buffer.len() as i128 + i128::from(offset)
            }
        };

        if target < 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Cannot seek before start of stream"));
        }

        let target = usize::try_from(target)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Seek target exceeds usize"))?;
        self.ensure_available(target)?;
        if target > self.buffer.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Cannot seek past end of stream"));
        }

        self.position = target;
        u64::try_from(self.position)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Seek position exceeds u64"))
    }
}
