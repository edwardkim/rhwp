/*
 * jdsample.c
 *
 * This file was part of the Independent JPEG Group's software:
 * Copyright (C) 1991-1996, Thomas G. Lane.
 * libjpeg-turbo Modifications:
 * Copyright 2009 Pierre Ossman <ossman@cendio.se> for Cendio AB
 * Copyright (C) 2010, 2015-2016, 2022, 2024-2025, D. R. Commander.
 * Copyright (C) 2014, MIPS Technologies, Inc., California.
 * Copyright (C) 2015, Google, Inc.
 * Copyright (C) 2019-2020, Arm Limited.
 * For conditions of distribution and use, see the accompanying README.ijg
 * file.
 *
 * This file contains upsampling routines.
 *
 * Upsampling input data is counted in "row groups".  A row group
 * is defined to be (v_samp_factor * DCT_scaled_size / min_DCT_scaled_size)
 * sample rows of each component.  Upsampling will normally produce
 * max_v_samp_factor pixel rows from each row group (but this could vary
 * if the upsampler is applying a scale factor of its own).
 *
 * An excellent reference for image resampling is
 *   Digital Image Warping, George Wolberg, 1990.
 *   Pub. by IEEE Computer Society Press, Los Alamitos, CA. ISBN 0-8186-8944-7.
 */

//! IJG의 중심 표본 보간. 수평·수직 필터의 중간 결과를 먼저 반올림하지 않는다.
//!
//! libjpeg-turbo 3.1.2 `src/jdsample.c`의 h2v1/h1v2/h2v2 규칙을 Rust로 재표현했다.
//! 원 저작권·배포 조건은 이 패키지의 `README.ijg`와
//! `THIRD_PARTY_LICENSES.md`에 보존한다. MCU 패딩은 실제 성분 폭의 끝 표본을 복제한다.

use crate::transforms::BoundedUpSampler;

pub fn select(horizontal: usize, vertical: usize) -> Option<BoundedUpSampler> {
    match (horizontal, vertical) {
        (2, 1) => Some(horizontal_row),
        (1, 2) => Some(vertical_rows),
        (2, 2) => Some(both_rows),
        _ => None,
    }
}

fn horizontal_row(
    input: &[i16],
    _near: &[i16],
    _far: &[i16],
    _scratch: &mut [i16],
    output: &mut [i16],
    width: usize,
) {
    for (x, sample) in output.iter_mut().enumerate() {
        let center = (x / 2).min(width - 1);
        // IJG는 성분 폭이 2 이하이면 수평 fancy 대신 단순 표본 복제를 선택한다.
        *sample = if width <= 2 || x >= width * 2 {
            input[center]
        } else if x % 2 == 0 {
            (3 * input[center] + input[center.saturating_sub(1)] + 1) >> 2
        } else {
            (3 * input[center] + input[(center + 1).min(width - 1)] + 2) >> 2
        };
    }
}

fn vertical_rows(
    input: &[i16],
    near: &[i16],
    far: &[i16],
    _scratch: &mut [i16],
    output: &mut [i16],
    width: usize,
) {
    let (top, bottom) = output.split_at_mut(input.len());
    for (row, neighbor, bias) in [(top, near, 1), (bottom, far, 2)] {
        for (x, sample) in row.iter_mut().enumerate() {
            let column = x.min(width - 1);
            *sample = (3 * input[column] + neighbor[column] + bias) >> 2;
        }
    }
}

fn both_rows(
    input: &[i16],
    near: &[i16],
    far: &[i16],
    _scratch: &mut [i16],
    output: &mut [i16],
    width: usize,
) {
    let (top, bottom) = output.split_at_mut(input.len() * 2);
    for (row, neighbor) in [(top, near), (bottom, far)] {
        for (x, sample) in row.iter_mut().enumerate() {
            let center = (x / 2).min(width - 1);
            if width <= 2 {
                *sample = input[center];
                continue;
            }
            let weighted = |column| 3 * input[column] + neighbor[column];
            let (other, bias) = if x % 2 == 0 {
                (center.saturating_sub(1), 8)
            } else {
                ((center + 1).min(width - 1), 7)
            };
            *sample = (3 * weighted(center) + weighted(other) + bias) >> 4;
        }
    }
}
