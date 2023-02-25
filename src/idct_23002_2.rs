// The IDCT code is a manual translation of the C [1, 2] code to Rust.
//
// [1] http://www.reznik.org/software.html
// [2] http://www.reznik.org/software/ISO-IEC-23002-2.zip

// This module is an alternative IDCT implementation. It is only used
// during tests to compare results against another IDCT variant.
#![cfg(test)]

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
const SCALE: [i32; 8*8] = [
  A, B, C, D, A, D, C, B,
  B, E, F, G, B, G, F, E,
  C, F, H, I, C, I, H, F,
  D, G, I, J, D, J, I, G,
  A, B, C, D, A, D, C, B,
  D, G, I, J, D, J, I, G,
  C, F, H, I, C, I, H, F,
  B, E, F, G, B, G, F, E
];

#[allow(non_snake_case)]
#[cfg(test)]
pub fn idct_23002_2(P: &mut [i32; 64]) {

    let mut block: [i32; 8*8] = [0; 8*8];
    let mut block2: [i32; 8*8] = [0; 8*8];

    /* multiplier-based scaling:
     *  - can be moved outside the transform, executed for non-zero coeffs only,
     *    or absorbed in quantization step. */
    for i in 0..64 {
        block[i] = SCALE[i] * i32::from(P[i]);
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
