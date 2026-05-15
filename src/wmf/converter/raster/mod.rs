/* -*- Mode: rust; tab-width: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
/*
 * [Task #902 v2 Stage 12] WMF raster Player — LibreOffice emfio 포팅 baseline.
 *
 * This file incorporates algorithms derived from LibreOffice's emfio module:
 *   https://cgit.freedesktop.org/libreoffice/core/tree/emfio/source/reader/wmfreader.cxx
 *   https://cgit.freedesktop.org/libreoffice/core/tree/emfio/source/reader/mtftools.cxx
 *
 * The algorithm references retain attribution per LibreOffice's MPL 2.0 license.
 * This Rust adaptation is provided under rhwp's MIT license; original algorithm
 * derivations follow MPL 2.0 file-level reciprocity.
 *
 * License Notice (LO source):
 *   This Source Code Form is subject to the terms of the Mozilla Public
 *   License, v. 2.0. http://mozilla.org/MPL/2.0/
 */

//! WMF raster renderer — tiny-skia + fontdue 기반의 자체 완결 WMF 렌더링.
//!
//! 기존 [`SVGPlayer`](super::SVGPlayer) 는 WMF 를 SVG 로 변환 후 외부 SVG 렌더러
//! (browser / resvg) 에 의존. 본 모듈은 WMF records 를 직접 pixel canvas 에
//! 렌더링하여 폰트/렌더링 quality 를 자체 제어.
//!
//! 알고리즘 출처: LibreOffice emfio (MPL 2.0)
//! - [`emfreader.cxx`] WMF binary parsing
//! - [`mtftools.cxx`] DrawText, DrawPolyPolygon, DrawPolygon, DrawText 등 렌더링
//!
//! 본 Rust 포팅은 LO 알고리즘의 의미를 보존하되 Rust idiom 으로 작성.

mod player;
mod state;

pub use self::player::RasterPlayer;
