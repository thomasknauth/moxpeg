use std::io::Write;

struct BmpFileHeader {
    file_size: u32,
    offset: u32
}

impl BmpFileHeader {
    fn serialize(&self) -> [u8; 14] {
        let mut ret = [0; 14];
        ret[0] = 'B' as u8;
        ret[1] = 'M' as u8;
        ret[2..6].copy_from_slice(&self.file_size.to_le_bytes());
        ret[10..14].copy_from_slice(&self.offset.to_le_bytes());
        ret
    }
}

#[derive(Debug,Default,Copy,Clone)]
struct BmpCoreHeader {
    size: u32,
    width: u16,
    height: u16,
    nr_planes: u16,
    bits_per_pixel: u16
}

impl BmpCoreHeader {
    fn serialize(&self) -> [u8; 12] {
        let mut ret = [0; 12];
        ret[0..4].copy_from_slice(&self.size.to_le_bytes());
        ret[4..6].copy_from_slice(&self.width.to_le_bytes());
        ret[6..8].copy_from_slice(&self.height.to_le_bytes());
        ret[8..10].copy_from_slice(&self.nr_planes.to_le_bytes());
        ret[10..12].copy_from_slice(&self.bits_per_pixel.to_le_bytes());
        ret
    }
}

pub struct BmpImage {
}

impl BmpImage {

    pub fn write<W: Write>(self, width: i32, height: i32, pixels: &Vec<u8>, writer: &mut W) -> std::io::Result<()> {
        let bytes_per_pixel = 3i32;
        let mut padding_bytes = 4 - ((width * bytes_per_pixel) % 4);
        if padding_bytes == 4 {
            padding_bytes = 0;
        }
        let bmp_file_header_size = 14i32;
        let bmp_dib_header_size = 12i32;
        let mut bmp_file_header = BmpFileHeader {file_size: 0, offset: 0};
        bmp_file_header.file_size = (bmp_file_header_size + bmp_dib_header_size + (width + padding_bytes) * height).try_into().unwrap();
        bmp_file_header.offset = u32::try_from(bmp_file_header_size + bmp_dib_header_size).unwrap();
        writer.write_all(&bmp_file_header.serialize())?;

        let mut bmp_core_header = BmpCoreHeader::default();
        bmp_core_header.size = 12;
        bmp_core_header.width = width.try_into().unwrap();
        bmp_core_header.height = height.try_into().unwrap();
        bmp_core_header.bits_per_pixel = (bytes_per_pixel * 8).try_into().unwrap();
        bmp_core_header.nr_planes = 1;
        writer.write_all(&bmp_core_header.serialize())?;

        let padding = vec![0u8; padding_bytes.try_into().unwrap()];

        // Pixel data stored bottom-to-top, hence .rev()
        for nr_row in (0..height).rev() {
            let start_idx: usize = usize::try_from(nr_row * width * bytes_per_pixel).unwrap();
            let end_idx: usize = start_idx + usize::try_from(width * bytes_per_pixel).unwrap();

            writer.write_all(&pixels[start_idx..end_idx])?;
            writer.write_all(&padding)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::OpenOptions;

    #[test]
    fn simple_bmp() {
        let pixels: Vec<u8> = vec![255,0,0,0,255,0,0,0,255];
        let bmp = BmpImage {};

        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .open("my.bmp").unwrap();

        bmp.write(3, 1, &pixels, &mut f);
    }
}
