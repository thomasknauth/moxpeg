// Resources
//
// https://web.archive.org/web/20160706034713/https://vsr.informatik.tu-chemnitz.de/~jan/MPEG/HTML/idct_discussion/Index.html
// http://dvdnav.mplayerhq.hu/dvdinfo/mpeghdrs.html
// https://tech.ebu.ch/docs/techreview/trev_266-ely.pdf
// http://www.reznik.org/papers/SPIE07_MPEG-C_IDCT.pdf

//mod bitstream_io;

// https://github.com/phoboslab/pl_mpeg

use bitstream_io::BitRead;
use std::io;
use std::io::Read;
use std::fs::OpenOptions;

const SEQUENCE_HEADER: [u8; 4] = [0x00, 0x00, 0x01, 0xB3];
const GROUP_OF_PICTURES_START_CODE: [u8; 4] = [0x00, 0x00, 0x01, 0xB8];
const PICTURE_START_CODE: [u8; 4] = [0x00, 0x00, 0x01, 0x00];

const VIDEO_INTRA_QUANT_MATRIX: [u8; 64] = [
	 8, 16, 19, 22, 26, 27, 29, 34,
	16, 16, 22, 24, 27, 29, 34, 37,
	19, 22, 26, 27, 29, 34, 34, 38,
	22, 22, 26, 27, 29, 34, 37, 40,
	22, 26, 27, 29, 32, 35, 40, 48,
	26, 27, 29, 32, 35, 40, 48, 58,
	26, 27, 29, 34, 38, 46, 56, 69,
	27, 29, 35, 38, 46, 56, 69, 83
];

const VIDEO_PREMULTIPLIER_MATRIX: [i32; 64] = [
	  32, 44, 42, 38, 32, 25, 17,  9,
	  44, 62, 58, 52, 44, 35, 24, 12,
	  42, 58, 55, 49, 42, 33, 23, 12,
	  38, 52, 49, 44, 38, 30, 20, 10,
	  32, 44, 42, 38, 32, 25, 17,  9,
	  25, 35, 33, 30, 25, 20, 14,  7,
	  17, 24, 23, 20, 17, 14,  9,  5,
	  9, 12, 12, 10,  9,  7,  5,  2
];

const _: [u8; 8] = [0; std::mem::size_of::<SequenceHeader>()];

struct SequenceHeader {
    raw: [u8; 8]
}

impl SequenceHeader {
    fn new<F: std::io::Read>(f: &mut F) -> SequenceHeader {
        let mut buf: [u8; 8] = [0; 8];
        f.read(&mut buf).expect("");
        SequenceHeader {
            //            raw: [data[0], data[1], data[2], data[3]],
            raw: buf,
        }
    }

    fn hsize(&self) -> u16 {
         (self.raw[0] as u16) << 4 + ((self.raw[1] & 0xF0) >> 4)
    }
    fn vsize(&self) -> u16 {
        (((self.raw[1] & 0x0F) as u16) << 8) + self.raw[2] as u16
    }
    fn aspect_ratio_str(&self) -> &str {
        let idx = (self.raw[3] & 0xF0) >> 4;
        let table: [&str; 5] = [
            "", "1:1", "4:3", "16:9", "2.21:1"
        ];
        table[idx as usize]
    }
    fn frame_rate(&self) -> f32 {
        let idx = self.raw[3] & 0x0F;
        let table: [f32; 4] = [
            0.0, 24000./1001., 24.0, 25.0];
        table[idx as usize]
    }
}

struct GroupOfPictures {
    raw: [u8; 4]
}

impl GroupOfPictures {
    fn new<F: std::io::Read>(f: &mut F) -> Option<GroupOfPictures> {
        let mut buf: [u8; 4] = [0; 4];
        match f.read_exact(&mut buf) {
            Ok(()) => {
                Some(GroupOfPictures {
                    raw: buf,
                })
            },
            Err(e) => None
        }
    }

    fn hour(&self) -> u8 {
        (self.raw[0] & 0b01111100u8) >> 2
    }

    fn min(&self) -> u8 {
        println!("{:x?}", self.raw);
        ((self.raw[0] & 0b00000011) << 4) + ((self.raw[1] & 0b11110000) >> 4)
    }

    fn sec(&self) -> u8 {
        ((self.raw[1] & 0b00000111) << 3) + ((self.raw[2] & 0b11100000) >> 5)
    }

    fn frame(&self) -> u8 {
        ((self.raw[2] & 0b00011111) << 1) + ((self.raw[3] & 0b10000000) >> 7)
    }
}

struct PictureHeader {
    raw: [u8; 4]
}

impl PictureHeader {

    fn new<F: std::io::Read>(f: &mut F) -> Option<PictureHeader> {
        let mut buf: [u8; 4] = [0; 4];
        match f.read_exact(&mut buf) {
            Ok(()) => {
                Some(PictureHeader {
                    raw: buf,
                })
            },
            Err(_) => None
        }
    }

    fn sequence_nr(&self) -> u16 {
        ((self.raw[0] as u16) << 2) + u16::from((self.raw[1] & 0b11000000) >> 6)
    }

    fn frame_type(&self) -> u8 {
        (self.raw[1] & 0b00111000) >> 3
    }
}

fn is_slice_start_code(b: &[u8; 4]) -> bool {
    b[0] == 0x0 && b[1] == 0x0 && b[2] == 0x01 && b[3] >= 0x01 && b[3] <= 0xAF
}

type MyBitReader<'a, T: std::io::Read> = bitstream_io::BitReader<&'a mut std::io::BufReader<T>, bitstream_io::BigEndian>;

fn parse_macroblock_address_increment<T: std::io::Read>(bs: &mut MyBitReader<T>) -> Option<u8> {
    match bs.read::<u8>(1).unwrap() {
        1 => return Some(1),
        _ => unimplemented!("other encodings not supported yet")
    }
}

fn parse_macroblock_type<T: std::io::Read>(bs: &mut MyBitReader<T>) -> Option<u8> {

    if bs.read::<u8>(1).unwrap() == 1 {
        // I-frame/picture
        return Some(0b1_0000);
    }

    if bs.read::<u8>(1).unwrap() == 1 {
        // I-frame/picture with quantizer
        return Some(0b1_0001);
    }

    unimplemented!("unsupported macro block type!");
}

// fn vlc_decode<U: std::io::Read>(table: std::collections::HashMap<T, S>, bs: &mut MyBitReader<U>) -> Option<S> {
//     None
// }

const VIDEO_ZIG_ZAG: [u8; 64] = [
	 0,  1,  8, 16,  9,  2,  3, 10,
	17, 24, 32, 25, 18, 11,  4,  5,
	12, 19, 26, 33, 40, 48, 41, 34,
	27, 20, 13,  6,  7, 14, 21, 28,
	35, 42, 49, 56, 57, 50, 43, 36,
	29, 22, 15, 23, 30, 37, 44, 51,
	58, 59, 52, 45, 38, 31, 39, 46,
	53, 60, 61, 54, 47, 55, 62, 63
];

const VIDEO_DCT_SIZE_LUMINANCE: [(i16, i16); 18] = [
	(  1 << 1,    0), (  2 << 1,    0),  //   0: x
	(       0,    1), (       0,    2),  //   1: 0x
	(  3 << 1,    0), (  4 << 1,    0),  //   2: 1x
	(       0,    0), (       0,    3),  //   3: 10x
	(       0,    4), (  5 << 1,    0),  //   4: 11x
	(       0,    5), (  6 << 1,    0),  //   5: 111x
	(       0,    6), (  7 << 1,    0),  //   6: 1111x
	(       0,    7), (  8 << 1,    0),  //   7: 1111 1x
	(       0,    8), (      -1,    0),  //   8: 1111 11x
];

const VIDEO_DCT_SIZE_CHROMINANCE: [(i16, i16); 18] = [
	(  1 << 1,    0), (  2 << 1,    0),  //   0: x
	(       0,    0), (       0,    1),  //   1: 0x
	(       0,    2), (  3 << 1,    0),  //   2: 1x
	(       0,    3), (  4 << 1,    0),  //   3: 11x
	(       0,    4), (  5 << 1,    0),  //   4: 111x
	(       0,    5), (  6 << 1,    0),  //   5: 1111x
	(       0,    6), (  7 << 1,    0),  //   6: 1111 1x
	(       0,    7), (  8 << 1,    0),  //   7: 1111 11x
	(       0,    8), (      -1,    0),  //   8: 1111 111x
];

//  Decoded values are unsigned. Sign bit follows in the stream.

const VIDEO_DCT_COEFF: [(i16, u16); 224] = [
	(  1 << 1,        0), (       0,   0x0001),  //   0: x
	(  2 << 1,        0), (  3 << 1,        0),  //   1: 0x
	(  4 << 1,        0), (  5 << 1,        0),  //   2: 00x
	(  6 << 1,        0), (       0,   0x0101),  //   3: 01x
	(  7 << 1,        0), (  8 << 1,        0),  //   4: 000x
	(  9 << 1,        0), ( 10 << 1,        0),  //   5: 001x
	(       0,   0x0002), (       0,   0x0201),  //   6: 010x
	( 11 << 1,        0), ( 12 << 1,        0),  //   7: 0000x
	( 13 << 1,        0), ( 14 << 1,        0),  //   8: 0001x
	( 15 << 1,        0), (       0,   0x0003),  //   9: 0010x
	(       0,   0x0401), (       0,   0x0301),  //  10: 0011x
	( 16 << 1,        0), (       0,   0xffff),  //  11: 0000 0x
	( 17 << 1,        0), ( 18 << 1,        0),  //  12: 0000 1x
	(       0,   0x0701), (       0,   0x0601),  //  13: 0001 0x
	(       0,   0x0102), (       0,   0x0501),  //  14: 0001 1x
	( 19 << 1,        0), ( 20 << 1,        0),  //  15: 0010 0x
	( 21 << 1,        0), ( 22 << 1,        0),  //  16: 0000 00x
	(       0,   0x0202), (       0,   0x0901),  //  17: 0000 10x
	(       0,   0x0004), (       0,   0x0801),  //  18: 0000 11x
	( 23 << 1,        0), ( 24 << 1,        0),  //  19: 0010 00x
	( 25 << 1,        0), ( 26 << 1,        0),  //  20: 0010 01x
	( 27 << 1,        0), ( 28 << 1,        0),  //  21: 0000 000x
	( 29 << 1,        0), ( 30 << 1,        0),  //  22: 0000 001x
	(       0,   0x0d01), (       0,   0x0006),  //  23: 0010 000x
	(       0,   0x0c01), (       0,   0x0b01),  //  24: 0010 001x
	(       0,   0x0302), (       0,   0x0103),  //  25: 0010 010x
	(       0,   0x0005), (       0,   0x0a01),  //  26: 0010 011x
	( 31 << 1,        0), ( 32 << 1,        0),  //  27: 0000 0000x
	( 33 << 1,        0), ( 34 << 1,        0),  //  28: 0000 0001x
	( 35 << 1,        0), ( 36 << 1,        0),  //  29: 0000 0010x
	( 37 << 1,        0), ( 38 << 1,        0),  //  30: 0000 0011x
	( 39 << 1,        0), ( 40 << 1,        0),  //  31: 0000 0000 0x
	( 41 << 1,        0), ( 42 << 1,        0),  //  32: 0000 0000 1x
	( 43 << 1,        0), ( 44 << 1,        0),  //  33: 0000 0001 0x
	( 45 << 1,        0), ( 46 << 1,        0),  //  34: 0000 0001 1x
	(       0,   0x1001), (       0,   0x0502),  //  35: 0000 0010 0x
	(       0,   0x0007), (       0,   0x0203),  //  36: 0000 0010 1x
	(       0,   0x0104), (       0,   0x0f01),  //  37: 0000 0011 0x
	(       0,   0x0e01), (       0,   0x0402),  //  38: 0000 0011 1x
	( 47 << 1,        0), ( 48 << 1,        0),  //  39: 0000 0000 00x
	( 49 << 1,        0), ( 50 << 1,        0),  //  40: 0000 0000 01x
	( 51 << 1,        0), ( 52 << 1,        0),  //  41: 0000 0000 10x
	( 53 << 1,        0), ( 54 << 1,        0),  //  42: 0000 0000 11x
	( 55 << 1,        0), ( 56 << 1,        0),  //  43: 0000 0001 00x
	( 57 << 1,        0), ( 58 << 1,        0),  //  44: 0000 0001 01x
	( 59 << 1,        0), ( 60 << 1,        0),  //  45: 0000 0001 10x
	( 61 << 1,        0), ( 62 << 1,        0),  //  46: 0000 0001 11x
	(      -1,        0), ( 63 << 1,        0),  //  47: 0000 0000 000x
	( 64 << 1,        0), ( 65 << 1,        0),  //  48: 0000 0000 001x
	( 66 << 1,        0), ( 67 << 1,        0),  //  49: 0000 0000 010x
	( 68 << 1,        0), ( 69 << 1,        0),  //  50: 0000 0000 011x
	( 70 << 1,        0), ( 71 << 1,        0),  //  51: 0000 0000 100x
	( 72 << 1,        0), ( 73 << 1,        0),  //  52: 0000 0000 101x
	( 74 << 1,        0), ( 75 << 1,        0),  //  53: 0000 0000 110x
	( 76 << 1,        0), ( 77 << 1,        0),  //  54: 0000 0000 111x
	(       0,   0x000b), (       0,   0x0802),  //  55: 0000 0001 000x
	(       0,   0x0403), (       0,   0x000a),  //  56: 0000 0001 001x
	(       0,   0x0204), (       0,   0x0702),  //  57: 0000 0001 010x
	(       0,   0x1501), (       0,   0x1401),  //  58: 0000 0001 011x
	(       0,   0x0009), (       0,   0x1301),  //  59: 0000 0001 100x
	(       0,   0x1201), (       0,   0x0105),  //  60: 0000 0001 101x
	(       0,   0x0303), (       0,   0x0008),  //  61: 0000 0001 110x
	(       0,   0x0602), (       0,   0x1101),  //  62: 0000 0001 111x
	( 78 << 1,        0), ( 79 << 1,        0),  //  63: 0000 0000 0001x
	( 80 << 1,        0), ( 81 << 1,        0),  //  64: 0000 0000 0010x
	( 82 << 1,        0), ( 83 << 1,        0),  //  65: 0000 0000 0011x
	( 84 << 1,        0), ( 85 << 1,        0),  //  66: 0000 0000 0100x
	( 86 << 1,        0), ( 87 << 1,        0),  //  67: 0000 0000 0101x
	( 88 << 1,        0), ( 89 << 1,        0),  //  68: 0000 0000 0110x
	( 90 << 1,        0), ( 91 << 1,        0),  //  69: 0000 0000 0111x
	(       0,   0x0a02), (       0,   0x0902),  //  70: 0000 0000 1000x
	(       0,   0x0503), (       0,   0x0304),  //  71: 0000 0000 1001x
	(       0,   0x0205), (       0,   0x0107),  //  72: 0000 0000 1010x
	(       0,   0x0106), (       0,   0x000f),  //  73: 0000 0000 1011x
	(       0,   0x000e), (       0,   0x000d),  //  74: 0000 0000 1100x
	(       0,   0x000c), (       0,   0x1a01),  //  75: 0000 0000 1101x
	(       0,   0x1901), (       0,   0x1801),  //  76: 0000 0000 1110x
	(       0,   0x1701), (       0,   0x1601),  //  77: 0000 0000 1111x
	( 92 << 1,        0), ( 93 << 1,        0),  //  78: 0000 0000 0001 0x
	( 94 << 1,        0), ( 95 << 1,        0),  //  79: 0000 0000 0001 1x
	( 96 << 1,        0), ( 97 << 1,        0),  //  80: 0000 0000 0010 0x
	( 98 << 1,        0), ( 99 << 1,        0),  //  81: 0000 0000 0010 1x
	(100 << 1,        0), (101 << 1,        0),  //  82: 0000 0000 0011 0x
	(102 << 1,        0), (103 << 1,        0),  //  83: 0000 0000 0011 1x
	(       0,   0x001f), (       0,   0x001e),  //  84: 0000 0000 0100 0x
	(       0,   0x001d), (       0,   0x001c),  //  85: 0000 0000 0100 1x
	(       0,   0x001b), (       0,   0x001a),  //  86: 0000 0000 0101 0x
	(       0,   0x0019), (       0,   0x0018),  //  87: 0000 0000 0101 1x
	(       0,   0x0017), (       0,   0x0016),  //  88: 0000 0000 0110 0x
	(       0,   0x0015), (       0,   0x0014),  //  89: 0000 0000 0110 1x
	(       0,   0x0013), (       0,   0x0012),  //  90: 0000 0000 0111 0x
	(       0,   0x0011), (       0,   0x0010),  //  91: 0000 0000 0111 1x
	(104 << 1,        0), (105 << 1,        0),  //  92: 0000 0000 0001 00x
	(106 << 1,        0), (107 << 1,        0),  //  93: 0000 0000 0001 01x
	(108 << 1,        0), (109 << 1,        0),  //  94: 0000 0000 0001 10x
	(110 << 1,        0), (111 << 1,        0),  //  95: 0000 0000 0001 11x
	(       0,   0x0028), (       0,   0x0027),  //  96: 0000 0000 0010 00x
	(       0,   0x0026), (       0,   0x0025),  //  97: 0000 0000 0010 01x
	(       0,   0x0024), (       0,   0x0023),  //  98: 0000 0000 0010 10x
	(       0,   0x0022), (       0,   0x0021),  //  99: 0000 0000 0010 11x
	(       0,   0x0020), (       0,   0x010e),  // 100: 0000 0000 0011 00x
	(       0,   0x010d), (       0,   0x010c),  // 101: 0000 0000 0011 01x
	(       0,   0x010b), (       0,   0x010a),  // 102: 0000 0000 0011 10x
	(       0,   0x0109), (       0,   0x0108),  // 103: 0000 0000 0011 11x
	(       0,   0x0112), (       0,   0x0111),  // 104: 0000 0000 0001 000x
	(       0,   0x0110), (       0,   0x010f),  // 105: 0000 0000 0001 001x
	(       0,   0x0603), (       0,   0x1002),  // 106: 0000 0000 0001 010x
	(       0,   0x0f02), (       0,   0x0e02),  // 107: 0000 0000 0001 011x
	(       0,   0x0d02), (       0,   0x0c02),  // 108: 0000 0000 0001 100x
	(       0,   0x0b02), (       0,   0x1f01),  // 109: 0000 0000 0001 101x
	(       0,   0x1e01), (       0,   0x1d01),  // 110: 0000 0000 0001 110x
	(       0,   0x1c01), (       0,   0x1b01),  // 111: 0000 0000 0001 111x
];


fn read_huffman<T, S>(table: &[(i16, S)], stream: &mut MyBitReader<T>) -> Option<S>
where
    T: Read,
    S: bitstream_io::Numeric
{
    let mut state: (i16, S) = (0, S::default());

    loop {
        state = table[usize::try_from(state.0 + stream.read::<i16>(1).unwrap()).unwrap()];

        if state.0 <= 0 {
            break;
        }
    }
    Some(state.1)
}

fn parse_dct_dc_size<T: std::io::Read>(table: &[(i16, i16); 18], bs: &mut MyBitReader<T>) -> Option<u8> {
    match read_huffman(table, bs) {
        Some(i) => Some(u8::try_from(i).unwrap()),
        None => None
    }
}

fn parse_block<T: std::io::Read>(bs: &mut MyBitReader<T>) -> Option<()> {
    Some(())
}

struct Plane {
    width: u16,
    height: u16,
    data: Vec<u8>
}

impl Plane {
    fn new(w: u16, h: u16) -> Plane {
        Plane {
            width: w,
            height: h,
            data: vec![0; (i32::from(w)*i32::from(h)).try_into().unwrap()]
        }
    }
}

struct Frame {
    width: u16,
    height: u16,
    y: Plane,
    cr: Plane,
    cb: Plane
}

impl Frame {
    fn new(w: u16, h: u16) -> Frame {

        let macroblock_width = (w + 15) / 16;
        let macroblock_height = (h + 15) / 16;

        Frame {
            width: w,
            height: h,

            // * 16 because there are 16 pixel per macroblock.
            y: Plane::new(macroblock_width * 16,
                          macroblock_height * 16),
            // * 8 because there are only half as manychrominance
            // pixels as luminance pixels.
            cr: Plane::new(macroblock_width * 8,
                           macroblock_height * 8),
            cb: Plane::new(macroblock_width * 8,
                           macroblock_height * 8)
        }
    }
}

// #define PLM_BLOCK_SET(DEST, DEST_INDEX, DEST_WIDTH, SOURCE_INDEX, SOURCE_WIDTH, BLOCK_SIZE, OP) do { \
// 	}} while(FALSE)

fn block_set(dest: &mut Vec<u8>,
                    mut dest_idx: usize,
                    dest_width: usize,
                    source_idx: usize,
                    source_width: usize,
                    block_size: usize,
                    op: &[u8; 64])
{
	  let mut dest_scan = dest_width - block_size;
	  let mut source_scan = source_width - block_size;
    let mut source_idx = 0;

	  for y in 0..block_size {
		    for x in 0..block_size {
			      dest[dest_idx] = op[source_idx];
			      source_idx += 1;
            dest_idx += 1;
		    }
		    source_idx += source_scan;
		    dest_idx += dest_scan;
    }
}

struct Slice {
    mb_row: i32,
    mb_col: i32,
    mb_addr: i32,
    mb_width: i32,
    mb_height: i32
}

#[inline(always)]
fn clamp(n: i32) -> u8 {
    if n > 255 {
        return 255;
    } else if n < 0 {
        return 0;
    }
    return n as u8;
}

impl Slice {

    fn new() -> Self {

        let w = 640;
        let h = 360;

        Slice {
            mb_row: 0,
            mb_col: 0,
            mb_addr: -1,
            mb_width: (w + 15) / 16,
            mb_height: (h + 15) / 16,
        }
    }

    fn parse_macroblock<T: std::io::Read>(&mut self, bs: &mut MyBitReader<T>) -> Option<()> {

        println!("parse_macroblock");
        let addr_inc = parse_macroblock_address_increment(bs).unwrap();
        let macro_type = parse_macroblock_type(bs).unwrap();

        // Can only deal with I-frames for now.
        assert!((macro_type & 0b1_0000) != 0);

        self.mb_addr += i32::from(addr_inc);
        self.mb_row   = self.mb_addr / self.mb_width;
        self.mb_col   = self.mb_addr & self.mb_width;

        println!("parse_macroblock: addr_inc={}, type={}", addr_inc, macro_type);

        let mut quantizer_scale = 1;
        if (macro_type & 0b1_0000) == 1 {
            quantizer_scale = bs.read::<u8>(5).unwrap();
            println!("quantizer_scale={}", quantizer_scale);
        }

        // Ignore motion vectors and block patterns since they are irrelevant for I-frames.

        let mut frame = Frame::new(640, 480);

        for i in 0 .. 6 {
            println!("block_index={}", i);

            let mut block_data = [0i32; 64];

            let table = if i < 4 {
                VIDEO_DCT_SIZE_LUMINANCE
            } else {
                VIDEO_DCT_SIZE_CHROMINANCE
            };

            let size_lum: u8 = parse_dct_dc_size(&table, bs).unwrap();
            println!("size_{{lum|chrom}}={}", size_lum);

            if size_lum > 0 {
                let dc_diff_coded = bs.read::<u8>(size_lum.into()).unwrap();
                let dc_diff: i16 = {
                    if dc_diff_coded & (1 << (size_lum - 1)) != 0 {
                        dc_diff_coded.into()
                    } else {
                        (-1i16 << size_lum)|i16::from(dc_diff_coded+1)
                    }};
                println!("dc_diff_coded={}, dc_diff={}", dc_diff_coded, dc_diff);
            }
            let mut n = 0;

            loop {
                let mut level = 0i32;
                let mut run = 0u8;

                let coeff = read_huffman(&VIDEO_DCT_COEFF, bs).unwrap();

                if (coeff == 0x0001) && (n > 0) && (bs.read::<u8>(1).unwrap() == 0) {
                    break;
                }

                if coeff == 0xffff {
                    unimplemented!();
                } else {
                    run = (coeff >> 8).try_into().unwrap();
                    level = (coeff & 0xff).try_into().unwrap();

                    if bs.read::<u8>(1).unwrap() == 1 {
                        level = -level;
                    }
                }

                n += run;

                if n < 0 || n > 64 {
                    panic!();
                }

                let de_zig_zagged = VIDEO_ZIG_ZAG[usize::from(n)];
                n += 1;

                level <<= 1;

                if macro_type & 0b1_0000 == 0 {
                    level += if level < 0 { -1 } else { 1 };
                }

                level = (level * i32::from(quantizer_scale) *
                         i32::from(VIDEO_INTRA_QUANT_MATRIX[usize::from(de_zig_zagged)])) >> 4;

                if (level & 1) == 0 {
                    level -= if level > 0 { 1 } else { -1 };
                }

                if level > 2047 {
                    level = 2047;
                } else if level < -2048 {
                    level = -2048;
                }

                block_data[usize::from(de_zig_zagged)] = level * VIDEO_PREMULTIPLIER_MATRIX[usize::from(de_zig_zagged)];

            }

            let mut d = match i {
                4 => &mut frame.cb.data,
                5 => &mut frame.cr.data,
                _ => &mut frame.y.data
            };
            let dw = if i < 4 { frame.y.width } else { frame.cr.width };

            let mut di = 0;
            if i < 4 {
                di = (self.mb_row * i32::from(frame.y.width) + self.mb_col) << 4;
                if (i & 1) != 0 {
                    di += 8;
                }
                if (i & 2) != 0 {
                    di += i32::from(frame.y.width << 3);
                }
            } else {
                di = ((self.mb_row * i32::from(frame.y.width)) << 2) + (self.mb_col << 3);
            }

            if macro_type & 0b1_0000 != 0 {
                if n == 1 {
                    let clamped = clamp((block_data[0] + 128) >> 8);
                    block_set(&mut d, di.try_into().unwrap(), dw.into(),
                              0, 8, 8, &[clamped; 64]);
                    block_data[0] = 0;
                } else {
                    plm_video_idct(&mut block_data);
                    let mut clamped = [0u8; 64];
                    for n in 0..64 {
                        clamped[n] = clamp(block_data[n]);
                    }
                    block_set(&mut d, di.try_into().unwrap(), dw.into(),
                              0, 8, 8, &clamped);
                    block_data = [0i32; 64];
                }
            } else {
                // Can only do I frames
                unimplemented!("");
            }
        }

        Some(())
    }
}

// Why need a bufread and not just some trait(s) as first param?
fn parse_slice<T: std::io::Read>(f: &mut std::io::BufReader<T>) -> Result<Slice, std::io::Error>  {

    let mut buf: [u8; 4] = [0; 4];
    f.read_exact(&mut buf)?;

    if !is_slice_start_code(&buf) {
        panic!("");
    }

    let mut stream: bitstream_io::BitReader<_, bitstream_io::BigEndian> = bitstream_io::BitReader::new(f);
    let other_scale: u8 = stream.read::<u8>(5).unwrap();
    println!("{}", other_scale);

    // Extra slice info
    loop {
        if stream.read::<u8>(1).unwrap() == 0b1 {
            println!("extra slice info");
            stream.read::<u8>(8).unwrap();
        } else {
            break;
        }
    }

    let mut slice = Slice::new();

    loop {

        slice.parse_macroblock(&mut stream);

        let next_bits = stream.read::<u32>(23)?;

        if next_bits == 0 {
            break;
        }
    }

    // next_start_code();

    Ok(Slice::new())
}

fn parse_picture<T: std::io::Read + std::io::Seek>(f: &mut std::io::BufReader<T>) -> io::Result<()> {
    let mut buf: [u8; 4] = [0; 4];

    f.read_exact(&mut buf)?;
    assert!(buf == PICTURE_START_CODE);

    let hdr = PictureHeader::new(f).unwrap();
    println!("seq nr: {}, frame type: {}", hdr.sequence_nr(), hdr.frame_type());

    loop {
        parse_slice(f).unwrap();

        f.read_exact(&mut buf)?;

        if !is_slice_start_code(&buf) {
            f.seek_relative(-4)?;
            break;
        }
    }

    Ok(())
}

pub fn parse_mpeg(path: &str) -> io::Result<()> {

    let f = OpenOptions::new()
        .read(true)
        .open(path).expect("Unable to open file");
    let mut reader = io::BufReader::new(f);

    let mut buf: [u8; 4] = [0; 4];

    let mut frame: Frame;

    loop {

        reader.read_exact(&mut buf)?;

        match buf {
            SEQUENCE_HEADER => {
                let seqhdr = SequenceHeader::new(&mut reader);
                println!("width: {}", seqhdr.hsize());
                println!("height: {}", seqhdr.vsize());
                println!("aspect ratio: {}", seqhdr.aspect_ratio_str());
                println!("frame rate: {}", seqhdr.frame_rate());
            },
            GROUP_OF_PICTURES_START_CODE => {
                let hdr = GroupOfPictures::new(&mut reader).unwrap();
                println!("hour: {} minute: {} sec: {} frame: {}", hdr.hour(), hdr.min(), hdr.sec(), hdr.frame());
                loop {
                    parse_picture(&mut reader);

                    reader.read_exact(&mut buf)?;

                    if buf != PICTURE_START_CODE {
                        reader.seek_relative(-4)?;
                        break;
                    }
                }
            },
            _ => {
                reader.seek_relative(-3)?;
            }
        };
    };

    Ok(())
}

// The IDCT code is a manual translation of the C [1, 2] code to Rust.
//
// [1] http://www.reznik.org/software.html
// [2] http://www.reznik.org/software/ISO-IEC-23002-2.zip
//
fn pmul_1(y: &mut i32, z: &mut i32) {
    // int y2, y3;          \
    // y2 = (y >> 3) - (y >> 7); \
    let y2: i32 = (*y >> 3) - (*y >> 7);
    // y3 = y2 - (y >> 11); \
    let y3: i32 = y2 - (*y >> 11);
    *z = y2 + (y3 >> 1);
    *y = *y - y2;
}

fn pmul_2(y: &mut i32, z: &mut i32) {
    // int y2;              \
    // y2 = (y >> 9) - y;   \
    let y2: i32 = (*y >> 9) - *y;
    *z = *y >> 1;
    *y = (y2 >> 2) - y2;
}

fn pmul_3(y: &mut i32, z: &mut i32) {
    // y2 = y + (y >> 5);   \
    let y2: i32 = *y + (*y >> 5);
    // y3 = y2 >> 2;        \
    let y3 = y2 >> 2;
    // y  = y3 + (y >> 4);  \
    *y = y3 + (*y >> 4);
    // z  = y2 - y3;        \
    *z = y2 - y3;
}

fn scaled_1d_idct(input: &mut [i32], out: &mut [i32]) {
  let [mut x0, mut x1, mut x2, mut x3, mut x4, mut x5, mut x6, mut x7]: [i32; 8];
  let [mut xa, mut xb]: [i32; 2];

  x1 = input[1];
  x3 = input[3];
  x5 = input[5];
  x7 = input[7];

  xa = x1 + x7;
  xb = x1 - x7;

  x1 = xa + x3;
  x3 = xa - x3;
  x7 = xb + x5;
  x5 = xb - x5;

  pmul_1(&mut x3, &mut xa);
  pmul_1(&mut x5, &mut xb);
  x3 = x3 - xb;
  x5 = x5 + xa;

  pmul_2(&mut x1, &mut xa);
  pmul_2(&mut x7, &mut xb);
  x1 = x1 + xb;
  x7 = x7 - xa;

  /* even part: */
  x0 = input[0];
  x2 = input[2];
  x4 = input[4];
  x6 = input[6];

  pmul_3(&mut x2, &mut xa);
  pmul_3(&mut x6, &mut xb);
  x2 = x2 - xb;
  x6 = x6 + xa;

  xa = x0 + x4;
  xb = x0 - x4;

  x0 = xa + x6;
  x6 = xa - x6;
  x4 = xb + x2;
  x2 = xb - x2;

  /* 1st stage: */
  out[0*8] = x0 + x1;
  out[1*8] = x4 + x5;
  out[2*8] = x2 + x3;
  out[3*8] = x6 + x7;
  out[4*8] = x6 - x7;
  out[5*8] = x2 - x3;
  out[6*8] = x4 - x5;
  out[7*8] = x0 - x1;
}
//const SEQUENCE_HEADER: [u8; 4] = [0x00, 0x00, 0x01, 0xB3];
const A: i32 = 1024;
const B: i32 = 1138;
const C: i32 = 1730;
const D: i32 = 1609;
const E: i32 = 1264;
const F: i32 = 1922;
const G: i32 = 1788;
const H: i32 = 2923;
const I: i32 = 2718;
const J: i32 = 2528;

/* 2D scale-factor matrix: */
const scale: [i32; 8*8] = [
  A, B, C, D, A, D, C, B,
  B, E, F, G, B, G, F, E,
  C, F, H, I, C, I, H, F,
  D, G, I, J, D, J, I, G,
  A, B, C, D, A, D, C, B,
  D, G, I, J, D, J, I, G,
  C, F, H, I, C, I, H, F,
  B, E, F, G, B, G, F, E
];

fn idct_23002_2(P: &mut [i32; 64]) {

    let mut block: [i32; 8*8] = [0; 8*8];
    let mut block2: [i32; 8*8] = [0; 8*8];

    /* multiplier-based scaling:
     *  - can be moved outside the transform, executed for non-zero coeffs only,
     *    or absorbed in quantization step. */
    for i in 0..64 {
        block[i] = scale[i] * i32::from(P[i]);
    }
    block[0] += 1 << 12;           /* bias DC for proper rounding */

    /* perform  scaled 1D IDCT for rows and columns: */
    for i in 0..8 {
        scaled_1d_idct (&mut block[i*8..], &mut block2[i..]);
    }

    for i in 0..8 {
        scaled_1d_idct (&mut block2[i*8..], &mut block[i..]);
    }

    /* right-shift and store the results: */
    for i in 0..64 {
        P[i] = (block[i] >> 13).try_into().unwrap();
    }
}

fn plm_video_idct(block: &mut [i32; 64]) {

	  let [mut b1, mut b3, mut b4, mut b6, mut b7, mut tmp1, mut tmp2, mut m0, mut x0, mut x1, mut x2, mut x3, mut x4, mut y3, mut y4, mut y5, mut y6, mut y7]: [i32; 18];

	// Transform columns
	for i in 0..8 {
		b1 = block[4 * 8 + i];
		b3 = block[2 * 8 + i] + block[6 * 8 + i];
		b4 = block[5 * 8 + i] - block[3 * 8 + i];
		tmp1 = block[1 * 8 + i] + block[7 * 8 + i];
		tmp2 = block[3 * 8 + i] + block[5 * 8 + i];
		b6 = block[1 * 8 + i] - block[7 * 8 + i];
		b7 = tmp1 + tmp2;
		m0 = block[0 * 8 + i];
		x4 = ((b6 * 473 - b4 * 196 + 128) >> 8) - b7;
		x0 = x4 - (((tmp1 - tmp2) * 362 + 128) >> 8);
		x1 = m0 - b1;
		x2 = (((block[2 * 8 + i] - block[6 * 8 + i]) * 362 + 128) >> 8) - b3;
		x3 = m0 + b1;
		y3 = x1 + x2;
		y4 = x3 + b3;
		y5 = x1 - x2;
		y6 = x3 - b3;
		y7 = -x0 - ((b4 * 473 + b6 * 196 + 128) >> 8);
		block[0 * 8 + i] = b7 + y4;
		block[1 * 8 + i] = x4 + y3;
		block[2 * 8 + i] = y5 - x0;
		block[3 * 8 + i] = y6 - y7;
		block[4 * 8 + i] = y6 + y7;
		block[5 * 8 + i] = x0 + y5;
		block[6 * 8 + i] = y3 - x4;
		block[7 * 8 + i] = y4 - b7;
	}

	// Transform rows
	for i in (0..64).step_by(8) {
		b1 = block[4 + i];
		b3 = block[2 + i] + block[6 + i];
		b4 = block[5 + i] - block[3 + i];
		tmp1 = block[1 + i] + block[7 + i];
		tmp2 = block[3 + i] + block[5 + i];
		b6 = block[1 + i] - block[7 + i];
		b7 = tmp1 + tmp2;
		m0 = block[0 + i];
		x4 = ((b6 * 473 - b4 * 196 + 128) >> 8) - b7;
		x0 = x4 - (((tmp1 - tmp2) * 362 + 128) >> 8);
		x1 = m0 - b1;
		x2 = (((block[2 + i] - block[6 + i]) * 362 + 128) >> 8) - b3;
		x3 = m0 + b1;
		y3 = x1 + x2;
		y4 = x3 + b3;
		y5 = x1 - x2;
		y6 = x3 - b3;
		y7 = -x0 - ((b4 * 473 + b6 * 196 + 128) >> 8);
		block[0 + i] = (b7 + y4 + 128) >> 8;
		block[1 + i] = (x4 + y3 + 128) >> 8;
		block[2 + i] = (y5 - x0 + 128) >> 8;
		block[3 + i] = (y6 - y7 + 128) >> 8;
		block[4 + i] = (y6 + y7 + 128) >> 8;
		block[5 + i] = (x0 + y5 + 128) >> 8;
		block[6 + i] = (y3 - x4 + 128) >> 8;
		block[7 + i] = (y4 - b7 + 128) >> 8;
	}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        parse_mpeg("/Users/thomas/code/mpeg/big-buck-bunny.mpg");
        // parse_mpeg("/Users/thomas/code/mpeg/bjork-all-is-full-of-love.mpg");
    }

    #[test]
    fn idct() {
        // Test matrix taken from https://en.wikipedia.org/wiki/JPEG#Discrete_cosine_transform
        let mut m: [i32; 8*8] = [
            -416, -33, -60,  32,  48, -40, 0, 0,
               0, -24, -56,  19,  26,   0, 0, 0,
             -42,  13,  80, -24, -40,   0, 0, 0,
             -42,  17,  44, -29,   0,   0, 0, 0,
              18,   0,   0,   0,   0,   0, 0, 0,
               0,   0,   0,   0,   0,   0, 0, 0,
               0,   0,   0,   0,   0,   0, 0, 0,
               0,   0,   0,   0,   0,   0, 0, 0
        ];
        let mut m2: [i32; 8*8] = m.clone();

        idct_23002_2(&mut m);

        let expect: [i32; 8*8] = [
            -66, -63, -71, -68, -56, -65, -68, -46,
            -71, -73, -72, -46, -20, -41, -66, -57,
            -70, -78, -68, -17, 20, -14, -61, -63,
            -63, -73, -62, -8, 27, -14, -60, -58,
            -58, -65, -61, -27, -6, -40, -68, -50,
            -57, -57, -64, -58, -48, -66, -72, -47,
            -53, -46, -61, -74, -65, -63, -62, -45,
            -47, -34, -53, -74, -60, -47, -47, -41
            ];
        assert_eq!(m, expect);

        // The plm_video_idct() implementation requires pre-scaling of
        // the input matrix to compute results similar to the
        // idct_23002_2() implementation.
        for i in 0 .. 64 {
            m2[i] *= VIDEO_PREMULTIPLIER_MATRIX[i];
        }

        plm_video_idct(&mut m2);

        let mut delta = 0i32;

        // The two IDCT implementations do not compute identical
        // results, but the cumulative element-wise difference should
        // still be small.
        for i in 0 .. 64 {
            delta += i32::abs(m2[i] - expect[i]);
        }

        assert!(delta < 5);
    }

    #[test]
    fn bitstream() {
        // let b = [0x00, 0x00, 0x01, 0xb8, 0x00, 0x08, 0x00, 0x00];

        let file = std::fs::File::open("/dev/zero").unwrap();
        let mut reader = io::BufReader::new(file);
        use bitstream_io::BitRead;
        // let buf: [u8; 4] = [0x53, 0xf8, 0x7d, 0x29];
        // let cursor = io::Cursor::new(buf);
        let mut stream: bitstream_io::BitReader<_, bitstream_io::BigEndian> = bitstream_io::BitReader::new(reader);
        let other_scale: u8 = stream.read::<u8>(5).unwrap();
        println!("{}", other_scale);

        // Extra slice info
        loop {
            if stream.read::<u8>(1).unwrap() == 0b1 {
                println!("extra slice info");
                stream.read::<u8>(8).unwrap();
            } else {
                break;
            }
        }

        // macroblock
        println!("{:?}", stream.into_unread());
    }

    #[test]
    fn test_parse_picture() {
        let mut buf: Vec<u8> = vec![];
        buf.extend(PICTURE_START_CODE);
        buf.extend(&[0,0,0,0,0,0,0,0]);
        let cursor = io::Cursor::new(buf);
        let mut reader = io::BufReader::new(cursor);
        parse_picture(&mut reader);
    }

    #[test]
    fn test_parse_slice() {
        let f = std::fs::File::open("test/one-slice").unwrap();
        let mut reader = io::BufReader::new(f);
        parse_slice(&mut reader).unwrap();
    }

    #[test]
    fn test_parse_dct_dc_size_luminance() {
        let buf = [0b1101_0000];
        let cursor = io::Cursor::new(buf);
        let mut reader = io::BufReader::new(cursor);
        let mut stream: bitstream_io::BitReader<_, bitstream_io::BigEndian> = bitstream_io::BitReader::new(&mut reader);
        assert_eq!(parse_dct_dc_size(&VIDEO_DCT_SIZE_LUMINANCE, &mut stream).unwrap(), 4);
        assert_eq!(parse_dct_dc_size(&VIDEO_DCT_SIZE_LUMINANCE, &mut stream).unwrap(), 0);
        assert_eq!(parse_dct_dc_size(&VIDEO_DCT_SIZE_LUMINANCE, &mut stream).unwrap(), 1);
    }
}
