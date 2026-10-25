# Third-Party Licenses

rhwp 프로젝트가 사용하는 서드파티 라이브러리 및 리소스의 라이선스 목록이다.

아랍어 문맥 측정의 결합 속성 표는 Unicode 16.0.0의
[ArabicShaping.txt](https://www.unicode.org/Public/16.0.0/ucd/ArabicShaping.txt)에서 생성했다.
Copyright © 2024 Unicode, Inc.이며 [Unicode License v3](assets/licenses/Unicode-3.0.txt)를 적용한다.
원문 SHA-256은 `764f420cedfc8b43d9fec251c957a5d55fc45d40f6573f162990ed1dce7e36e0`이다.

Print JPEG 인코더의 원래 라이선스·저작권 고지는 `vendor/rhwp-jpeg-encoder/LICENSE-*`에 있다.
Print JPEG 디코더의 선택 변환은 공개 Intel MediaSDK 정수 IDCT(MIT)를 Rust로 번역하고,
IJG 보간 규칙을 재표현한다. Intel 저작권·MIT 고지는
`src/renderer/image_resolver/print_jpeg_idct.rs`, IJG 원문과 배포 조건은
`vendor/rhwp-zune-jpeg/README.ijg`, 해당 구현은 `vendor/rhwp-zune-jpeg/src/ijg_upsampler.rs`와
최종 JPEG 재생의 `vendor/rhwp-zune-jpeg/src/ijg_replay.rs`,
변경 설명은 `vendor/rhwp-zune-jpeg/RHWP_PATCHES.md`에 있다.
This software is based in part on the work of the Independent JPEG Group.

기준 파일:

- `Cargo.toml` / `Cargo.lock` (`rhwp` v0.9.0)
- `rhwp-studio/package.json` / `rhwp-studio/package-lock.json`
- `rhwp-chrome/package.json` / `rhwp-chrome/package-lock.json`
- `rhwp-firefox/package.json` / `rhwp-firefox/package-lock.json`
- `rhwp-vscode/package.json` / `rhwp-vscode/package-lock.json`
- `assets/fonts/FONTS.md`

---

## Rust 크레이트

### 직접 의존성

| 크레이트 | 버전 | 라이선스 | 저장소 | 비고 |
|---------|------|---------|--------|------|
| aes | 0.9.2 | MIT OR Apache-2.0 | RustCrypto/block-ciphers | HWPX 암호화 |
| base64 | 0.23.1 | MIT OR Apache-2.0 | marshallpierce/rust-base64 | Base64 인코딩 |
| blake3 | 1.8.6 | CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception | BLAKE3-team/BLAKE3 | 해시/진단 |
| byteorder | 1.5.0 | Unlicense OR MIT | BurntSushi/byteorder | 바이너리 endian 처리 |
| cbc | 0.2.1 | MIT OR Apache-2.0 | RustCrypto/block-modes | HWPX 암호화 모드 |
| cfb | 0.14.0 | MIT | mdsteele/rust-cfb | OLE Compound File |
| cipher | 0.5.2 | MIT OR Apache-2.0 | RustCrypto/traits | 암호 trait |
| codepage | 0.1.2 | Apache-2.0 OR MIT | hsivonen/codepage | 코드페이지 처리 |
| console_error_panic_hook | 0.1.7 | Apache-2.0/MIT | rustwasm/console_error_panic_hook | WASM panic hook |
| crc32fast | 1.5.0 | MIT OR Apache-2.0 | srijs/rust-crc32fast | HWP3 암호 키 검증 |
| des | 0.9.0 | MIT OR Apache-2.0 | RustCrypto/block-ciphers | HWP3 암호화 |
| ed25519-dalek | 2.2.0 | BSD-3-Clause | dalek-cryptography/curve25519-dalek | capsule 서명 |
| embedded-io | 0.7.1 | MIT OR Apache-2.0 | rust-embedded/embedded-hal | IO trait |
| encoding_rs | 0.8.35 | (Apache-2.0 OR MIT) AND BSD-3-Clause | hsivonen/encoding_rs | 문자 인코딩 |
| flate2 | 1.1.9 | MIT OR Apache-2.0 | rust-lang/flate2-rs | 압축/해제 |
| getrandom | 0.4.3 | MIT OR Apache-2.0 | rust-random/getrandom | 서명 키 난수 생성 |
| hmac | 0.13.0 | MIT OR Apache-2.0 | RustCrypto/MACs | HWPX 무결성 |
| image | 0.25.10 | MIT OR Apache-2.0 | image-rs/image | BMP/JPEG/PNG 디코딩 |
| rhwp-zune-jpeg | 0.5.15-rhwp.1 | (MIT OR Apache-2.0 OR Zlib) AND IJG | etemesi254/zune-image, IJG | 요청별 할당 예산·인스턴스별 Print 변환; 기본 PDF import decoder는 원래 변환 유지 |
| rhwp-jpeg-encoder | 0.7.1-rhwp.1 | (MIT OR Apache-2.0) AND IJG | vstroebel/jpeg-encoder | Print의 공통 스칼라 JPEG 인코딩; 원본·변경 내역은 `vendor/rhwp-jpeg-encoder/` |
| js-sys | 0.3.102 | MIT OR Apache-2.0 | rustwasm/wasm-bindgen | WASM JS interop |
| office_oxide | 0.1.9 | MIT OR Apache-2.0 | yfedoseev/office_oxide | `pdf-import` optional 호환성 제약; PDF 전이 의존성 |
| paste | 1.0.15 | MIT OR Apache-2.0 | dtolnay/paste | 매크로 보조 |
| pbkdf2 | 0.13.0 | MIT OR Apache-2.0 | RustCrypto/password-hashes | HWPX 키 파생 |
| pcx | 0.2.5 | MIT OR Apache-2.0 OR WTFPL | kryptan/pcx | PCX 이미지 디코딩 |
| pdf-writer | 0.12.1 | MIT OR Apache-2.0 | typst/pdf-writer | PDF 출력 |
| rhwp-pdf-import-backend | 0.3.78-rhwp.1 | MIT OR Apache-2.0 | edwardkim/rhwp | `pdf-import` optional; `pdf_oxide 0.3.78` 기반 내부 보안 패치, 코드 import alias `pdf_oxide` |
| quick-xml | 0.41.0 | MIT | tafia/quick-xml | HWPX XML 파싱 |
| resvg | 0.47.0 | Apache-2.0 OR MIT | linebender/resvg | Native/WASM 공통 WMF Print surface, native-skia SVG raster |
| roxmltree | 0.21.1 | MIT OR Apache-2.0 | RazrFalcon/roxmltree | XML 검증·정규화 |
| serde | 1.0.229 | MIT OR Apache-2.0 | serde-rs/serde | 직렬화 |
| serde_json | 1.0.151 | MIT OR Apache-2.0 | serde-rs/json | JSON 직렬화 |
| sha1 | 0.11.0 | MIT OR Apache-2.0 | RustCrypto/hashes | HWPX 암호 키 파생 |
| sha2 | 0.11.0 | MIT OR Apache-2.0 | RustCrypto/hashes | 서명·CAS 해시 |
| skia-safe | 0.99.0 | MIT | rust-skia/rust-skia | native-skia PNG/PDF backend (optional) |
| snafu | 0.9.2 | MIT OR Apache-2.0 | shepmaster/snafu | 에러 처리 |
| strum | 0.28.0 | MIT | Peternator7/strum | enum derive |
| subsecond | 0.7.10 | MIT OR Apache-2.0 | DioxusLabs/dioxus | 개발 핫패치 (optional) |
| subsetter | 0.2.6 | MIT OR Apache-2.0 | typst/subsetter | 폰트 subset |
| rhwp-jpeg-encoder | 0.7.1-rhwp.1 (upstream 0.7.1) | (MIT OR Apache-2.0) AND IJG | vstroebel/jpeg-encoder, rhwp 보정 | Print JPEG의 성분 순서·채널별 sampling·정수 DCT/YCbCr 전달. 원본 및 패치 고지 유지 |
| rhwp-svg2pdf | 0.13.0-rhwp.1 (upstream 0.13.0) | MIT OR Apache-2.0 | typst/svg2pdf, rhwp 보정 | SVG → PDF. 원저작자·패치 고지 유지 |
| svgtypes | 0.16.1 | Apache-2.0 OR MIT | linebender/svgtypes | SVG 값 파싱 |
| ttf-parser | 0.25.1 | MIT OR Apache-2.0 | harfbuzz/ttf-parser | 폰트 파싱 |
| unicode-properties | 0.1.4 | MIT OR Apache-2.0 | unicode-rs/unicode-properties | 유니코드 문자 분류 |
| unicode-bidi | 0.3.18 | MIT OR Apache-2.0 | servo/unicode-bidi | 텍스트 표시 방향과 논리 구간의 시각 원점 |
| unicode-bidi-mirroring | 0.4.0 | MIT OR Apache-2.0 | https://github.com/RazrFalcon/unicode-bidi-mirroring |
| unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 | unicode-rs/unicode-segmentation | grapheme/word segmentation |
| unicode-width | 0.2.2 | MIT OR Apache-2.0 | unicode-rs/unicode-width | 문자 폭 계산 |
| usvg | 0.45.1 / 0.47.0 | Apache-2.0 OR MIT | linebender/resvg | Native SVG 파싱 / 공통 WMF surface의 resvg 파싱 |
| wasm-bindgen | 0.2.125 | MIT OR Apache-2.0 | rustwasm/wasm-bindgen | WASM binding |
| wasm-bindgen-test | 0.3.75 | MIT OR Apache-2.0 | rustwasm/wasm-bindgen | WASM 테스트 |
| web-sys | 0.3.102 | MIT OR Apache-2.0 | rustwasm/wasm-bindgen | Web API binding |
| zip | 8.6.0 | MIT | zip-rs/zip2 | HWPX ZIP 처리 |

### PDF 코덱의 내부 보정판 (#8183·#8174)

현재 #8174 v2-S02의 기본 빌드는 다음 코덱을 포함한다. `pdf-jbig2`는 JBIG2와 내부 MMR 처리용
CCITT를, `pdf-jpeg2000`은 JPEG 2000을 선택한다. backend의 전체 rendering은 켜지 않는다.

| 내부 패키지 | 원본 버전 | 라이선스 | 보완 범위 |
| --- | --- | --- | --- |
| `rhwp-hayro-jbig2` `0.3.0-rhwp.1` | `hayro-jbig2` `0.3.0` | Apache-2.0 OR MIT | 사전·segment·bitmap의 예약과 반복 작업 |
| `rhwp-hayro-ccitt` `0.3.0-rhwp.1` | `hayro-ccitt` `0.3.0` | Apache-2.0 OR MIT | JBIG2 MMR의 changing-line 저장과 디코딩 작업 |
| `rhwp-hayro-jpeg2000` `0.4.0-rhwp.1` | `hayro-jpeg2000` `0.4.0` | Apache-2.0 OR MIT | tile·component·wavelet 저장과 디코딩 작업 |
| `rhwp-codec-resource` `0.1.0` | RHWP 자체 코드 | MIT | 동기 요청의 fallible 예약·논리 작업량 연결 |

원저작자는 Laurenz Stampfl이며 원본은 [LaurenzV/hayro](https://github.com/LaurenzV/hayro)이다.
각 `vendor/rhwp-hayro-*/`의 `NOTICE`, `LICENSE-*`, `UPSTREAM_FILES.json`에 원본 고지와
공식 `.crate` archive SHA-256·파일별 원본 해시를 보존했다. JPEG 2000의 ICC 자료 고지는
`vendor/rhwp-hayro-jpeg2000/assets/README.md`와 `assets/LICENSE.txt`를 함께 배포한다.
이는 RHWP 보정판이며 공식 hayro 릴리스가 아니다. 패키지는 게시 전 로컬 후보이고
실제 crates.io 게시 승인·이름 확정은 #8174의 절차를 따른다.

### 전체 Rust 의존성 요약

전체 의존성은 현재 `Cargo.toml` / `Cargo.lock`과 선택한 feature·target을 함께 기준으로 확인한다.
`cargo metadata --locked --features pdf-import --filter-platform wasm32-unknown-unknown` 등으로
해당 구성의 resolve graph를 확인하며, lockfile에 존재한다는 이유만으로 최종 배포물에 포함된다고
간주하지 않는다. 과거 workspace의 package 개수는 현재 의존성 총수로 사용하지 않는다.

주요 라이선스군:

- MIT / Apache-2.0 dual license 계열
- MIT 단독
- Apache-2.0 OR MIT
- BSD-2-Clause / BSD-3-Clause
- BSD-3-Clause AND MIT (`brotli`, PDF 전이 의존성)
- Zlib
- Unlicense
- 0BSD
- ISC
- Unicode-DFS-2016
- CC0-1.0 / Apache-2.0 WITH LLVM-exception (`blake3`)
- WTFPL 대체 선택지 포함 (`pcx`, MIT/Apache 선택 가능)

> 모든 Rust 의존성은 MIT, Apache-2.0, BSD, Zlib, Unlicense, 0BSD, ISC 등 permissive license 계열이거나
> permissive option을 포함하며, rhwp의 MIT 라이선스와 호환된다.

### PDF 입력 의존성의 고지

`pdf-import`는 기본 포함이며, 이를 통해 `rhwp-pdf-import-backend =0.3.78-rhwp.1`과
`office_oxide =0.1.9`를 모두 `default-features = false`로 사용한다.
`office_oxide` 직접 버전 제약은 `pdf_oxide 0.3.78`과의 컴파일 호환성을 위한 것이며,
rhwp가 Office 입력 형식 지원을 추가했다는 뜻은 아니다.
내부 backend는 Brotli private scratch를 요청별로 제한하기 위해
`brotli-decompressor =6.0.1`도 직접 사용한다. 해당 crate의 라이선스 표기는
`BSD-3-Clause/MIT`이며 기존 `brotli 9.0.0`의 `BSD-3-Clause AND MIT`와 구별한다.
감사한 Native/WASM 구성에서는 `std`만 활성화하고 `unsafe`/FFI feature는 활성화하지 않는다.

내부 backend는 메인테이너의 별도 승인에 따라 `pdf_oxide 0.3.78`을 보존한 보안 패치다.
크기 최적화용 소스 추출은 하지 않는다. 수정판을 공식 PDFOxide release로 표시하지 않으며,
원본·변경 범위·배포 제한은 [패치 기록](vendor/rhwp-pdf-import-backend/RHWP_PATCHES.md),
원본 파일별 hash는 `vendor/rhwp-pdf-import-backend/UPSTREAM_FILES.json`에 보존한다.
원래 MIT/Apache 라이선스·NOTICE·상표 정책도 해당 directory에 유지한다.
내부 Rust dependency는 `rlib`만 생성한다. 원본 C API·writer 소스는 보존하지만 이 내부
package를 독립 cdylib/staticlib 제품으로 배포하는 계약은 제공하지 않는다.
`publish = false`이며 외부 게시·배포와 게시된 rhwp library 소비자 지원은 아직 완료하지 않았다.

아래 고지는 해당 버전 crate 배포본의 `NOTICE`와 `LICENSE-MIT` 원문이다.
두 crate의 재배포에는 dual license 중 MIT 선택지를 적용한다. 이 고지와 각 전이 의존성의
라이선스·저작권 고지는 실제 Native/WASM 배포물의 문서 또는 고지 자료에도 함께 제공해야 한다.
제품명·패키지명·로고의 상표 사용 권한은 소스 코드 라이선스와 별개다.

#### PDFOxide 0.3.78 NOTICE

```text
PDFOxide
Copyright (c) 2025-2026 Yury Fedoseev and the PDFOxide project contributors

PDFOxide is the product; "pdf_oxide" is the name of its packages/crate.

This product is licensed under the terms of the MIT license OR the
Apache License, Version 2.0, at your option. See LICENSE-MIT and
LICENSE-APACHE for the full license texts.

Copyright in the PDFOxide code base is held by Yury Fedoseev and the project's
contributors; contributions are made under the project's Contributor License
Agreement (see CLA.md) and/or the Developer Certificate of Origin (see
CONTRIBUTING.md), under which contributors retain ownership of their
contributions and license them to the project.

"PDFOxide" (the product name) and "pdf_oxide" (the package name), together with
any associated logo, are trademarks of Yury Fedoseev and are not covered by the
source-code license. See TRADEMARKS.md.

Portions of this software incorporate third-party open-source components; their
respective licenses are listed in the dependency manifests (Cargo.toml /
Cargo.lock and the per-binding package manifests).
```

#### PDFOxide 0.3.78 MIT License

```text
Copyright (c) 2025-present Yury Fedoseev

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
```

#### OfficeOxide 0.1.9 NOTICE

```text
OfficeOxide
Copyright (c) 2026 Yury Fedoseev and the OfficeOxide project contributors

OfficeOxide is the product; "office_oxide" is the name of its packages/crate.

This product is licensed under the terms of the MIT license OR the
Apache License, Version 2.0, at your option. See LICENSE-MIT and
LICENSE-APACHE for the full license texts.

Copyright in the OfficeOxide code base is held by Yury Fedoseev and the project's
contributors; contributions are made under the project's Contributor License
Agreement (see CLA.md) and/or the Developer Certificate of Origin (see
CONTRIBUTING.md), under which contributors retain ownership of their
contributions and license them to the project.

"OfficeOxide" (the product name) and "office_oxide" (the package name), together with
any associated logo, are trademarks of Yury Fedoseev and are not covered by the
source-code license. See TRADEMARKS.md.

Portions of this software incorporate third-party open-source components; their
respective licenses are listed in the dependency manifests (Cargo.toml /
Cargo.lock and the per-binding package manifests).
```

#### OfficeOxide 0.1.9 MIT License

```text
Copyright (c) 2026 Yury Fedoseev

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
```

#### PDFOxide 동봉 폰트

`pdf_oxide 0.3.78` 소스에는 DejaVu Sans Regular/Bold가 `include_bytes!`로 선언되어 있다.
이 자산이 포함된 배포에 대비해 crate의 `src/fonts/assets/LICENSE-DejaVu` 고지를 보존한다.
소스에 존재하는 것과 최종 WASM에 포함되는지는 구분하며, 포함 여부를 크기 측정만으로 단정하지 않는다.
별도 `cjk-form-fonts` / `cjk-render-fallback` 기능의 Droid Sans Fallback·Noto Emoji는
현재 활성화하지 않는다. 해당 기능 도입 시 별도 폰트 고지도 함께 추가한다.

```text
DejaVu Fonts — Copyright (c) 2003 by Bitstream, Inc. All Rights Reserved.
Bitstream Vera is a trademark of Bitstream, Inc.

Permission is hereby granted, free of charge, to any person obtaining a copy
of the fonts accompanying this license ("Fonts") and associated documentation
files (the "Font Software"), to reproduce and distribute the Font Software,
including without limitation the rights to use, copy, merge, publish,
distribute, and/or sell copies of the Font Software, and to permit persons to
whom the Font Software is furnished to do so, subject to the following
conditions:

The above copyright and trademark notices and this permission notice shall be
included in all copies of one or more of the Font Software typefaces.

The Font Software may be modified, altered, or added to, and in particular the
designs of glyphs or characters in the Fonts may be modified and additional
glyphs or characters may be added to the Fonts, only if the fonts are renamed
to names not containing either the words "Bitstream" or the word "Vera".

THE FONT SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT OF COPYRIGHT, PATENT,
TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL BITSTREAM OR THE GNOME FOUNDATION
BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, INCLUDING ANY GENERAL,
SPECIAL, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL DAMAGES, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF THE USE OR INABILITY TO
USE THE FONT SOFTWARE OR FROM OTHER DEALINGS IN THE FONT SOFTWARE.
```

---

## npm 패키지

### npm 배포 패키지

| 패키지 | 버전 | 라이선스 | 용도 |
|--------|------|---------|------|
| @rhwp/core | 0.8.6 | MIT | WASM parser/renderer API |
| @rhwp/editor | 0.8.6 | MIT | iframe 기반 웹 에디터 wrapper |

### rhwp-studio

| 패키지 | Lock 버전 | 라이선스 | 용도 |
|--------|-----------|---------|------|
| canvaskit-wasm | 0.42.0 | BSD-3-Clause | CanvasKit 렌더링 backend |
| @noble/hashes | 2.4.0 | MIT | 렌더 리소스 키와 문서 digest |
| @types/chrome | 0.2.5 | MIT | Chrome API 타입 |
| pixelmatch | 7.2.0 | ISC | 시각 diff 픽셀 비교 |
| pngjs | 7.0.0 | MIT | 시각 diff PNG 처리 |
| puppeteer-core | 25.5.0 | Apache-2.0 | E2E 테스트 / CDP 연결 |
| typescript | 7.0.2 | Apache-2.0 | TypeScript 컴파일 |
| vite | 8.2.1 | MIT | 개발 서버 + 빌드 |
| vite-plugin-pwa | 1.3.0 | MIT | PWA 빌드 보조 |
| workbox-window | 7.4.1 | MIT | Service worker runtime 보조 |

### rhwp-chrome / rhwp-firefox

| 패키지 | Lock 버전 | 라이선스 | 용도 |
|--------|-----------|---------|------|
| typescript | 7.0.2 | Apache-2.0 | TypeScript 컴파일 |
| vite | 8.2.1 | MIT | 확장 번들 빌드 |

### rhwp-vscode

| 패키지 | Lock 버전 | 라이선스 | 용도 |
|--------|-----------|---------|------|
| @types/node | 18.15.13 | MIT | Node.js 타입 |
| @types/vscode | 1.82.0 | MIT | VS Code API 타입 |
| @typescript/native | 7.0.2 | Apache-2.0 | TypeScript native preview 컴파일러 |
| copy-webpack-plugin | 14.0.0 | MIT | Webpack 파일 복사 |
| null-loader | 4.0.1 | MIT | Webpack 로더 |
| ts-loader | 9.6.2 | MIT | Webpack TypeScript 로더 |
| typescript | 6.0.2 | Apache-2.0 | TypeScript 컴파일 |
| webpack | 5.109.2 | MIT | 번들러 |
| webpack-cli | 7.2.2 | MIT | Webpack CLI |
| @noble/hashes | 2.4.0 | MIT | 문서 digest |
| canvaskit-wasm | 0.42.0 | BSD-3-Clause | 자동 선택 CanvasKit backend |

### rhwp-shared

`rhwp-shared`는 현재 별도 외부 npm 의존성이 없는 shared source package다.

---

## 웹 폰트 및 폰트 리소스

`assets/fonts/`에 포함된 canonical 폰트 목록이다. 저작권 보호가 필요한 한컴/Microsoft 폰트는 Git에
포함하지 않으며, 세부 파일 목록과 폴백 관계는 `assets/fonts/FONTS.md`를 참조한다.

| 폰트/리소스 | 라이선스 | 출처 | 비고 |
|-------------|---------|------|------|
| Pretendard (9종) | SIL Open Font License 1.1 | github.com/orioncactus/pretendard | Sans-serif fallback |
| Noto Serif KR (2종) | SIL Open Font License 1.1 | Google Fonts | Serif fallback |
| Noto Sans KR (3종) | SIL Open Font License 1.1 | Google Fonts | Sans-serif fallback |
| Noto Sans Math Regular 2.001 | SIL Open Font License 1.1 | Google LLC / Ubuntu `fonts-noto-core`; [공식 라이선스](https://github.com/google/fonts/blob/main/ofl/notosansmath/OFL.txt) | CanvasKit 수학 기호 fallback; [라이선스 전문](assets/fonts/NotoSansMath-OFL.txt) 동봉 |
| Nanum Myeongjo (3종) | SIL Open Font License 1.1 | Google Fonts | Serif fallback |
| Nanum Gothic (3종) | SIL Open Font License 1.1 | Google Fonts | Sans-serif fallback |
| Nanum Gothic Coding (2종) | SIL Open Font License 1.1 | Google Fonts | Monospace fallback |
| Gowun Batang (2종) | SIL Open Font License 1.1 | Google Fonts | Serif fallback |
| Gowun Dodum | SIL Open Font License 1.1 | Google Fonts | Sans-serif fallback |
| D2 Coding (2종) | SIL Open Font License 1.1 | NAVER | Monospace fallback |
| Spoqa Han Sans | SIL Open Font License 1.1 | github.com/spoqa/spoqa-han-sans | Sans-serif fallback |
| RHWP Old Hangul subset | SIL Open Font License 1.1 | Adobe / Google | 옛한글 PUA fallback, `SourceHanSerifK-OFL.txt` 동봉 |
| Latin Modern Math 1.959 | GUST Font License | B. Jackowski, P. Strzelczyk, P. Pianowski / TeX 사용자 그룹 | 수식 fallback 및 적분·곱셈·근호·중괄호·연산자·대문자 윤곽; [라이선스 전문](assets/fonts/LatinModernMath-GUST-FONT-LICENSE.txt) 동봉 |
| Cafe24 써라운드 | 카페24 무료 배포 | fonts.cafe24.com | 장식체 |
| Cafe24 슈퍼매직 | 카페24 무료 배포 | fonts.cafe24.com | 장식체 |
| 해피니스 산스 (Happiness Sans, 4종) | SIL Open Font License 1.1 | [현대백화점](https://thehyundaifont.com/) | 공식 배포 WOFF2 원본. [동봉 라이선스 PDF](ttfs/redistributable/happiness-sans/HapinessSans_License.pdf)에 저작권·Reserved Font Name·OFL 전문 수록 |

테스트 폰트를 공개 GitHub 저장소에 포함하는 것도 재배포이므로, 제품 번들 여부와 관계없이
원본의 저작권·라이선스·수정본 명명 조건이 적용된다. 저장소의 MIT 라이선스가 외부 폰트의
재배포 권한을 대신하지 않는다.

| 경로 | 출처·라이선스 | 비고 |
| --- | --- | --- |
| `tests/fixtures/fonts/RHWPEquation*.ttf` (3개) | Adobe Noto Sans KR 수정본, SIL OFL 1.1 | [출처·변형 설명](tests/fixtures/fonts/RHWPEquationFonts.LICENSE.md), [전문](tests/fixtures/fonts/RHWPEquationFonts.OFL.txt). 내부 family는 `RHWP Equation Contract`; 문서의 요청 이름은 렌더러의 명시적 대체 설정으로 연결 |
| `tests/fixtures/native_font_family_names/family-contract.ttf` | Adobe Noto Sans KR 수정본, SIL OFL 1.1 | [출처](tests/fixtures/native_font_family_names/README.md), [전문](tests/fixtures/native_font_family_names/OFL.txt) |
| `tests/fixtures/native_font_styles/*.ttf` (2개) | Adobe Noto Sans KR 수정본, SIL OFL 1.1 | [출처·재생성](tests/fixtures/native_font_styles/README.md), [전문](tests/fixtures/native_font_styles/OFL.txt) |
| 나머지 `tests/fixtures/fonts/`의 TTF·TTC (7개) | 저장소 생성기의 직접 작성 윤곽선, MIT | 각 `*.LICENSE.md`에 생성기·용도 기록. 외부 폰트 수정본과 구별 |

`assets/fonts/NotoSansMath-Regular.woff2`는 Ubuntu 설치본의 전체 TTF를 WOFF2로 변환한
파일이다. 이 버전의 `Copyright 2018 Google LLC. All Rights Reserved.`와 SIL OFL 1.1
정보를 폰트 내부 및 동봉 라이선스에 보존한다. 소프트웨어와 함께 재배포하며 폰트 자체는
SIL OFL 1.1을 유지한다. 원본 경로·SHA-256·변환 방법은 [폰트 목록](assets/fonts/FONTS.md)에 기록한다.

`src/renderer/equation/integral_outline.rs`,
`src/renderer/equation/multiplication_outline.rs`,
`src/renderer/equation/radical_outline.rs`,
`src/renderer/equation/brace_outline.rs`,
`src/renderer/equation/brace_stretch_data.rs`,
`src/renderer/equation/round_stretch_data.rs`,
`src/renderer/equation/relation_outline.rs`,
`src/renderer/equation/operator_outline.rs`의 윤곽 상수는 동봉된 Latin Modern Math
1.959의 `integral`·`multiply`·`radical`·`braceleft`·`plus`·`minus`·`equal` glyph에서
추출한 공개 윤곽이다. `brace_stretch_data.rs`는 중괄호의 MATH 크기별 variant와
`uni23A7`·`uni23A8`·`uni23A9`·`braceleft.ex` 조립 부품의 윤곽·연결 메트릭도 포함한다.
`round_stretch_data.rs`는 공개 둥근 괄호의 `uni239B`·`uni239C`·`uni239D`
MATH 조립 부품의 윤곽·연결 메트릭을 포함한다.
원본 SHA-256과 추출 방법을 해당 파일에 기록한다. 원본 Copyright 2012--2014
B. Jackowski, P. Strzelczyk, P. Pianowski를 보존하고, 해당 윤곽 데이터는
GUST Font License를 유지한다. rhwp는 글꼴 파일을 개명·수정하는 대신 이 윤곽의
좌표를 공통 수식 ink 영역으로 변환한다. 한컴 사설 글꼴의 윤곽은 포함하지 않는다.

`relation_outline.rs`는 같은 공개 원본의 `less`·`greater`·`lessequal`·`greaterequal`·
`parenleft`·`parenright` glyph를 추출한 관계 기호와 둥근 괄호 윤곽이다.
저작권·GUST Font License를 그대로 유지한다. 사설 face에서 소비하는 bbox는
메트릭이며 사설 윤곽 데이터가 아니다.

`src/renderer/equation/accent_outline.rs`의 화살촉은 같은 공개 원본의 MATH 조립
부품 `arrowright.rt`, `arrowboth.lft`, `arrowboth.rt`에서 파생했다. 원 저작권과
GUST Font License를 유지하며 사설 글꼴의 윤곽·바이트는 포함하지 않는다.

`assets/fonts/LatinModernMath-VariableOutline-subset.otf`는 같은 공개 원본의
정체 A~Z/a~z와 이탤릭 a~z 윤곽을 포함한 13,488 byte 파생 OpenType CFF 글꼴이다. 원본 저작권·
라이선스를 보존하고 family·full·PostScript 및 CFF 이름을 변경했다.
[파생 manifest](assets/fonts/MANIFEST-RHWP-LMM-VariableOutline.txt)에 원본·
파생 SHA와 변경 범위를 기록한다. `src/renderer/equation/variable_outline.rs`는
이 공개 윤곽을 파싱하며, 사설 글꼴에서는 em 프레임의 메트릭만 소비한다.

---

## 도구

| 도구 | 라이선스 | 용도 |
|------|---------|------|
| Docker | Apache-2.0 | WASM 빌드 환경 |
| wasm-pack | MIT OR Apache-2.0 | WASM 패키징 |
| Chrome DevTools Protocol | BSD-3-Clause | E2E 테스트 / 브라우저 자동화 |
| GitHub Actions | GitHub Terms | CI/CD |
| npm Trusted Publishing / OIDC | npm Terms | npm 배포 |

---

## 직접 포팅한 알고리즘

공통 Print 장치 면적 표본(`src/renderer/print_image_sampling.rs`)은 공개 PDFium
`chromium/7999`의 CStretchEngine 고정소수점 산술 계약을 적용한다. 원래 PDFium Authors와
Foxit Software의 저작권 고지 및 BSD-3-Clause 조건은 `licenses/pdfium/LICENSE`에 보존한다.
이 항목은 Cargo 크레이트 의존성이 아니다. 출처는 `licenses/pdfium/README.md`에 있다.

### Volexity hwp-extract

- 출처: [volexity/hwp-extract](https://github.com/volexity/hwp-extract), commit
  `e5f8b5e1590dee973630666e687e919fa70da2e2`
- 참조 파일: `src/hwp_extract/encrypt.py`
- 적용 위치: `src/parser/crypto.rs`의 HWP5 EncryptVersion 4 키 파생 및 비트 단위
  AES-CFB 복호화
- 라이선스: BSD-3-Clause

Modified BSD License

_Copyright © `2024`, `Volexity, Inc`_

_All rights reserved._

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright
   notice, this list of conditions and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright
   notice, this list of conditions and the following disclaimer in the
   documentation and/or other materials provided with the distribution.
3. Neither the name of the `Volexity, Inc` nor the
   names of its contributors may be used to endorse or promote products
   derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL `Volexity, Inc` BE LIABLE FOR ANY
DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

---

## 참조한 오픈소스 프로젝트 (스펙·설계 참조)

rhwp는 아래 프로젝트들의 **코드를 직접 복사하지 않으며**, 공개된 스펙 정보(enum 값·속성 기본값·태그 이름·검증 규칙 등)만 참조한다.
Apache-2.0 라이선스 고지 의무는 본 문서, 관련 기술 문서, 그리고 각 참조 파일의 헤더 주석으로 충족한다.

| 프로젝트 | 라이선스 | 참조 범위 | rhwp 위치 |
|---------|---------|----------|-----------|
| [hancom-io/hwpx-owpml-model](https://github.com/hancom-io/hwpx-owpml-model) | Apache-2.0 © 2022 Hancom Inc. | HWPX enum 정의, 속성 기본값, 태그 전체 집합, canonical 속성·자식 순서 | `src/serializer/hwpx/canonical_defaults.rs`, `src/serializer/hwpx/{header,table,shape,picture}.rs`, `mydocs/tech/hwpx_hancom_reference.md` |
| [hancom-io/dvc](https://github.com/hancom-io/dvc) | Apache-2.0 © 2022 Hancom Inc. | HWPX 검증 규칙 JSON 스키마, errorCode 체계 | `mydocs/tech/hwpx_dvc_reference.md` |

---

## 라이선스 호환성

rhwp는 **MIT 라이선스**로 배포된다.

- MIT, Apache-2.0, BSD, Zlib, Unlicense, 0BSD, ISC, CC0 — MIT와 호환되는 permissive license 계열
- `encoding_rs`의 BSD-3-Clause 조항은 고지 의무를 요구하며, 이 문서로 충족
- `pcx`는 WTFPL 대체 선택지를 포함하지만 MIT/Apache-2.0 선택지가 있어 rhwp MIT 배포와 호환
- 오픈 폰트는 SIL OFL 1.1 또는 GUST Font License 등 재배포 가능한 라이선스만 Git에 포함
- 저작권 보호 대상 폰트(한컴, Microsoft 등)는 Git에 포함하지 않으며 `ttfs/FONTS.md`, `assets/fonts/FONTS.md`에서 목록과 폴백만 관리

---

*이 문서는 `Cargo.toml`, `Cargo.lock`, 각 `package.json`/`package-lock.json`, `assets/fonts/FONTS.md` 기준으로 현행화되었으며, 의존성 업데이트 시 함께 갱신해야 한다.*

배포 폰트의 파일별 SHA·원본 저작권·라이선스 전문은
[fonts/licenses](assets/fonts/licenses/README.md)에 함께 보존한다. 제품 폰트 37개와
공개 테스트·Native 원본 18개를 [목록](assets/fonts/licenses/font-notices.json)에 등록하며,
등록되지 않은 공개 폰트 또는 배포 고지 누락은 `scripts/frontend-font-assets.test.mjs`가 검출한다.
상용 글꼴의 문서 요청 이름은 대체 설정에서 처리하고, 파생 폰트의 내부 이름으로 사용하지 않는다.

### 기본 ICC CMM

`pdf-icc`는 기본 feature이며 Native/WASM에서 같은 순수 Rust CMM인 `rhwp-qcms`를
사용한다. qcms 0.3.0의 내부 보정판이며 Mozilla Foundation·Marti Maria 등의 원 저작권과
MIT 전문은 [COPYING](vendor/rhwp-qcms/COPYING)에 보존한다. 원본 archive·VCS commit과
요청별 CLUT 예약·작업량 변경은 [패치 기록](vendor/rhwp-qcms/RHWP_PATCHES.md)에 있다.
공식 qcms 릴리스나 이미 게시된 crate로 표시하지 않는다. 실제 포장·독립 소비는 #8174 S05에서
최종 dependency graph로 재검증한다.

## Nimbus Roman 계량 자료

`src/renderer/font_metric_nimbus_roman.rs`와 `data/font-metrics/nimbus-roman/`은
URW Base35의 글꼴 프로그램에서 추출한 이름·Unicode 대응·전진폭 계량 자료다.
원본 프로그램 SHA-256과 생성 방법은 해당 디렉터리의 `source.json`·`README.md`에 있다.
글꼴 프로그램 바이트와 윤곽선은 이 자료에 포함하지 않는다. 원본의 고지는 패키지에 동봉한
`data/font-metrics/nimbus-roman/URW-FONT-NOTICE.txt`에 보존한다.
