// Copyright (c) 2018 Intel Corporation
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! 공개 Intel MediaSDK 정수 IDCT의 안전한 scalar 번역.
//!
//! 원문: https://github.com/Intel-Media-SDK/MediaSDK/blob/
//! 7a72de33a15d6e7cdb842b12b901a003f7154f0a/contrib/ipp/src/pjdecdctcn.c
//! 행별 정규화·반올림과 열별 포화 연산을 보존한다. 문서별 보정값은 없다.
//! 입력은 decoder가 역양자화한 8×8 블록이며 MCU 입력은 수정하지 않는다.

const TABLES: [[i16; 32]; 4] = [
    [
        16384, 21407, 16384, 8867, -16384, 21407, 16384, -8867, 16384, -8867, 16384, -21407, 16384,
        8867, -16384, -21407, 22725, 19266, 19266, -4520, 4520, 19266, 19266, -22725, 12873,
        -22725, 4520, -12873, 12873, 4520, -22725, -12873,
    ],
    [
        22725, 29692, 22725, 12299, -22725, 29692, 22725, -12299, 22725, -12299, 22725, -29692,
        22725, 12299, -22725, -29692, 31521, 26722, 26722, -6270, 6270, 26722, 26722, -31521,
        17855, -31521, 6270, -17855, 17855, 6270, -31521, -17855,
    ],
    [
        21407, 27969, 21407, 11585, -21407, 27969, 21407, -11585, 21407, -11585, 21407, -27969,
        21407, 11585, -21407, -27969, 29692, 25172, 25172, -5906, 5906, 25172, 25172, -29692,
        16819, -29692, 5906, -16819, 16819, 5906, -29692, -16819,
    ],
    [
        19266, 25172, 19266, 10426, -19266, 25172, 19266, -10426, 19266, -10426, 19266, -25172,
        19266, 10426, -19266, -25172, 26722, 22654, 22654, -5315, 5315, 22654, 22654, -26722,
        15137, -26722, 5315, -15137, 15137, 5315, -26722, -15137,
    ],
];
const ROW_ROUND: [i32; 8] = [65536, 2901, 2260, 1704, 1024, 455, 512, 373];
fn sat(x: i32) -> i16 {
    x.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16
}
fn add(a: i16, b: i16) -> i16 {
    a.saturating_add(b)
}
fn sub(a: i16, b: i16) -> i16 {
    a.saturating_sub(b)
}
fn high(a: i16, b: i16) -> i16 {
    ((i32::from(a) * i32::from(b)) >> 16) as i16
}
fn pair(a: i16, b: i16, c: i16, d: i16) -> i32 {
    i32::from(a)
        .wrapping_mul(i32::from(b))
        .wrapping_add(i32::from(c).wrapping_mul(i32::from(d)))
}
fn idct(dequantized: &[i32; 64]) -> [u8; 64] {
    let mut rows = [[0i16; 8]; 8];
    for r in 0..8 {
        let x: [i16; 8] = std::array::from_fn(|c| dequantized[r * 8 + c] as i16);
        let even = [x[0], x[2], x[0], x[2], x[4], x[6], x[4], x[6]];
        let odd = [x[1], x[3], x[1], x[3], x[5], x[7], x[5], x[7]];
        let t = &TABLES[match r {
            0 | 4 => 0,
            1 | 7 => 1,
            2 | 6 => 2,
            _ => 3,
        }];
        for i in 0..4 {
            let j = (i + 2) % 4;
            let a = pair(even[i * 2], t[i * 2], even[i * 2 + 1], t[i * 2 + 1])
                .wrapping_add(ROW_ROUND[r])
                .wrapping_add(pair(
                    even[j * 2],
                    t[8 + j * 2],
                    even[j * 2 + 1],
                    t[8 + j * 2 + 1],
                ));
            let b =
                pair(odd[i * 2], t[16 + i * 2], odd[i * 2 + 1], t[16 + i * 2 + 1]).wrapping_add(
                    pair(odd[j * 2], t[24 + j * 2], odd[j * 2 + 1], t[24 + j * 2 + 1]),
                );
            rows[r][i] = sat(a.wrapping_add(b) >> 11);
            rows[r][7 - i] = sat(a.wrapping_sub(b) >> 11);
        }
    }
    let mut out = [0u8; 64];
    for c in 0..8 {
        let x: [i16; 8] = std::array::from_fn(|r| rows[r][c]);
        let t3 = add(high(x[3], -21746), x[3]);
        let t5 = add(high(x[5], -21746), x[5]);
        let tm765 = add(t5, x[3]);
        let tm465 = sub(x[5], t3);
        let t1 = high(x[1], 13036);
        let t7 = high(x[7], 13036);
        let tp765 = add(x[1], t7);
        let tp465 = sub(t1, x[7]);
        let t7 = add(tp765, tm765);
        let tp65 = sub(tp765, tm765);
        let t4 = add(tp465, tm465);
        let tm65 = sub(tp465, tm465);
        let t2 = high(x[2], 27146);
        let t6 = high(x[6], 27146);
        let tm03 = add(x[2], t6);
        let tm12 = sub(t2, x[6]);
        let t5 = sub(tp65, tm65);
        let t6 = add(tp65, tm65);
        let t5 = add(high(t5, -19195), t5);
        let t6 = add(high(t6, -19195), t6);
        let tp03 = add(x[0], x[4]);
        let tp12 = sub(x[0], x[4]);
        let t0 = add(tp03, tm03);
        let t3 = sub(tp03, tm03);
        let t1 = add(tp12, tm12);
        let t2 = sub(tp12, tm12);
        let y = [
            add(t0, t7),
            add(t1, t6),
            add(t2, t5),
            add(t3, t4),
            sub(t3, t4),
            sub(t2, t5),
            sub(t1, t6),
            sub(t0, t7),
        ];
        for r in 0..8 {
            out[r * 8 + c] = (i32::from(y[r] >> 6) + 128).clamp(0, 255) as u8;
        }
    }
    out
}

pub(super) fn transform(input: &mut [i32; 64], output: &mut [i16], stride: usize) {
    let samples = idct(input);
    for (row, values) in samples.chunks_exact(8).enumerate() {
        for (column, sample) in values.iter().enumerate() {
            output[row * stride + column] = i16::from(*sample);
        }
    }
}
