/*
 * jidctint.c
 * This file was part of the Independent JPEG Group's software:
 * Copyright (C) 1991-1998, Thomas G. Lane.
 * Modification developed 2002-2018 by Guido Vollbeding.
 * libjpeg-turbo Modifications:
 * Copyright (C) 2015, 2020, 2022, D. R. Commander.
 *
 * jdcolor.c
 * This file was part of the Independent JPEG Group's software:
 * Copyright (C) 1991-1997, Thomas G. Lane.
 * Modified 2011 by Guido Vollbeding.
 * libjpeg-turbo Modifications:
 * Copyright 2009 Pierre Ossman <ossman@cendio.se> for Cendio AB
 * Copyright (C) 2009, 2011-2012, 2014-2015, 2022, 2024, D. R. Commander.
 * Copyright (C) 2013, Linaro Limited.
 * For conditions of distribution and use, see the accompanying README.ijg file.
 */

//! libjpeg-turbo 3.1.2의 8비트 ISLOW IDCT와 최종 JPEG 색 변환.
//! `jidctint.c`와 `jdcolor.c`를 안전한 정수 연산으로 재표현했다.
//! 기본 decoder와 원본 사진의 Print 변환은 이 함수를 자동 선택하지 않는다.

fn kernel(s: [i64; 8]) -> [i64; 8] {
    let z1 = (s[2] + s[6]) * 4433;
    let t2 = z1 - s[6] * 15137;
    let t3 = z1 + s[2] * 6270;
    let t0 = (s[0] + s[4]) << 13;
    let t1 = (s[0] - s[4]) << 13;
    let a = t0 + t3;
    let d = t0 - t3;
    let b = t1 + t2;
    let c = t1 - t2;
    let mut o0 = s[7];
    let mut o1 = s[5];
    let mut o2 = s[3];
    let mut o3 = s[1];
    let z1 = o0 + o3;
    let z2 = o1 + o2;
    let z3 = o0 + o2;
    let z4 = o1 + o3;
    let z5 = (z3 + z4) * 9633;
    o0 *= 2446;
    o1 *= 16819;
    o2 *= 25172;
    o3 *= 12299;
    let z1 = z1 * -7373;
    let z2 = z2 * -20995;
    let z3 = z3 * -16069 + z5;
    let z4 = z4 * -3196 + z5;
    o0 += z1 + z3;
    o1 += z2 + z4;
    o2 += z2 + z3;
    o3 += z1 + z4;
    [
        a + o3,
        b + o2,
        c + o1,
        d + o0,
        d - o0,
        c - o1,
        b - o2,
        a - o3,
    ]
}

/// 역양자화한 8×8 블록을 소비한다. MCU shortcut이 재사용하는 입력은 덮어쓰지 않는다.
pub fn idct(input: &mut [i32; 64], output: &mut [i16], stride: usize) {
    let mut workspace = [0i64; 64];
    // 13비트 계수·첫 pass 2비트·최종 3비트 정규화는 공개 ISLOW 계약이다.
    // i64 중간값으로 손상된 큰 역양자화 입력에서도 산술 overflow를 피한다.
    for c in 0..8 {
        let result = kernel(core::array::from_fn(|r| i64::from(input[r * 8 + c])));
        for r in 0..8 {
            workspace[r * 8 + c] = (result[r] + (1 << 10)) >> 11;
        }
    }
    for r in 0..8 {
        let result = kernel(core::array::from_fn(|c| workspace[r * 8 + c]));
        for c in 0..8 {
            output[r * stride + c] = (((result[c] + (1 << 17)) >> 18) + 128).clamp(0, 255) as i16;
        }
    }
}

/// 보간이 끝난 YCbCr을 IJG의 16비트 BT.601 반올림으로 RGB에 덮어쓴다.
/// RGB·회색·CMYK 출력에는 적용하지 않는다.
pub fn ycbcr_to_rgb(pixels: &mut [u8]) {
    for p in pixels.chunks_exact_mut(3) {
        let y = i32::from(p[0]);
        let cb = i32::from(p[1]) - 128;
        let cr = i32::from(p[2]) - 128;
        p[0] = (y + ((91881 * cr + 32768) >> 16)).clamp(0, 255) as u8;
        p[1] = (y + ((-22554 * cb - 46802 * cr + 32768) >> 16)).clamp(0, 255) as u8;
        p[2] = (y + ((116130 * cb + 32768) >> 16)).clamp(0, 255) as u8;
    }
}
