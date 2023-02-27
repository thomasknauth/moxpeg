use std::fs::File;
use std::io::{SeekFrom};
use super::iso11172_stream;

/// Provide a Reader that strips away system level packets and only
/// returns video level data. This encapsulates parsing of system
/// level packets and simplifies the decoder.
pub struct MpegVideoStream {
    cursor: std::io::Cursor<Vec<u8>>
}

impl std::io::Read for MpegVideoStream {

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.cursor.read(buf)
    }
}

impl std::io::Seek for MpegVideoStream {

    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.cursor.seek(pos)
    }
}

impl MpegVideoStream {

    pub fn new(f: &mut File) -> MpegVideoStream {
        let mut buf = vec![];
        iso11172_stream(f, &mut buf).unwrap();
        Self {cursor: std::io::Cursor::<Vec<u8>>::new(buf)}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Seek};

    #[test]
    fn test1() {
        let mut file = File::open("tests/bjork-v2-short-2.mpg").unwrap();
        let mut vs = MpegVideoStream::new(&mut file);
        let mut buf = [0; 8];
        vs.read(&mut buf).unwrap();
        // Can only seek backwards.
        // assert_eq!(vs.seek(SeekFrom::Current(1)).unwrap_err().kind(),
        //            std::io::ErrorKind::Unsupported);
        vs.seek(SeekFrom::Current(-4)).unwrap();
        let mut buf2 = [0; 4];
        vs.read(&mut buf2).unwrap();
    }
}
