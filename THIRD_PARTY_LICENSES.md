# Third-Party Licenses

rhwp 프로젝트가 사용하는 서드파티 라이브러리 및 리소스의 라이선스 목록이다.

기준 파일:

- `Cargo.toml` / `Cargo.lock` (`rhwp` v0.7.17)
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
| base64 | 0.22.1 | MIT OR Apache-2.0 | marshallpierce/rust-base64 | Base64 인코딩 |
| blake3 | 1.8.5 | CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception | BLAKE3-team/BLAKE3 | 해시/진단 |
| byteorder | 1.5.0 | Unlicense OR MIT | BurntSushi/byteorder | 바이너리 endian 처리 |
| cfb | 0.14.0 | MIT | mdsteele/rust-cfb | OLE Compound File |
| codepage | 0.1.2 | Apache-2.0 OR MIT | hsivonen/codepage | 코드페이지 처리 |
| console_error_panic_hook | 0.1.7 | Apache-2.0/MIT | rustwasm/console_error_panic_hook | WASM panic hook |
| embedded-io | 0.7.1 | MIT OR Apache-2.0 | rust-embedded/embedded-hal | IO trait |
| encoding_rs | 0.8.35 | (Apache-2.0 OR MIT) AND BSD-3-Clause | hsivonen/encoding_rs | 문자 인코딩 |
| flate2 | 1.1.9 | MIT OR Apache-2.0 | rust-lang/flate2-rs | 압축/해제 |
| image | 0.25.10 | MIT OR Apache-2.0 | image-rs/image | BMP/JPEG/PNG 디코딩 |
| js-sys | 0.3.102 | MIT OR Apache-2.0 | rustwasm/wasm-bindgen | WASM JS interop |
| paste | 1.0.15 | MIT OR Apache-2.0 | dtolnay/paste | 매크로 보조 |
| pcx | 0.2.5 | MIT OR Apache-2.0 OR WTFPL | kryptan/pcx | PCX 이미지 디코딩 |
| pdf-writer | 0.12.1 | MIT OR Apache-2.0 | typst/pdf-writer | PDF 출력 |
| quick-xml | 0.40.1 | MIT | tafia/quick-xml | HWPX XML 파싱 |
| resvg | 0.45.1 | Apache-2.0 OR MIT | linebender/resvg | native-skia SVG raster (optional) |
| serde | 1.0.228 | MIT OR Apache-2.0 | serde-rs/serde | 직렬화 |
| serde_json | 1.0.150 | MIT OR Apache-2.0 | serde-rs/json | JSON 직렬화 |
| skia-safe | 0.99.0 | BSD-3-Clause | rust-skia/rust-skia | native-skia PNG/PDF backend (optional) |
| snafu | 0.9.1 | MIT OR Apache-2.0 | shepmaster/snafu | 에러 처리 |
| strum | 0.28.0 | MIT | Peternator7/strum | enum derive |
| subsetter | 0.2.6 | MIT OR Apache-2.0 | typst/subsetter | 폰트 subset |
| svg2pdf | 0.13.0 | MIT OR Apache-2.0 | typst/svg2pdf | SVG → PDF |
| ttf-parser | 0.25.1 | MIT OR Apache-2.0 | harfbuzz/ttf-parser | 폰트 파싱 |
| unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 | unicode-rs/unicode-segmentation | grapheme/word segmentation |
| unicode-width | 0.2.2 | MIT OR Apache-2.0 | unicode-rs/unicode-width | 문자 폭 계산 |
| usvg | 0.45.1 | Apache-2.0 OR MIT | linebender/resvg | SVG 파싱 |
| wasm-bindgen | 0.2.125 | MIT OR Apache-2.0 | rustwasm/wasm-bindgen | WASM binding |
| wasm-bindgen-test | 0.3.75 | MIT OR Apache-2.0 | rustwasm/wasm-bindgen | WASM 테스트 |
| web-sys | 0.3.102 | MIT OR Apache-2.0 | rustwasm/wasm-bindgen | Web API binding |
| zip | 8.6.0 | MIT | zip-rs/zip2 | HWPX ZIP 처리 |

### 전체 Rust 의존성 요약

`Cargo.lock` 기준 package entry는 190개이며, 이 중 root package `rhwp`를 제외한 외부 Rust 크레이트는 189개다.

주요 라이선스군:

- MIT / Apache-2.0 dual license 계열
- MIT 단독
- Apache-2.0 OR MIT
- BSD-2-Clause / BSD-3-Clause
- Zlib
- Unlicense
- 0BSD
- ISC
- Unicode-DFS-2016
- CC0-1.0 / Apache-2.0 WITH LLVM-exception (`blake3`)
- WTFPL 대체 선택지 포함 (`pcx`, MIT/Apache 선택 가능)

> 모든 Rust 의존성은 MIT, Apache-2.0, BSD, Zlib, Unlicense, 0BSD, ISC 등 permissive license 계열이거나
> permissive option을 포함하며, rhwp의 MIT 라이선스와 호환된다.

---

## npm 패키지

### npm 배포 패키지

| 패키지 | 버전 | 라이선스 | 용도 |
|--------|------|---------|------|
| @rhwp/core | 0.7.13 | MIT | WASM parser/renderer API |
| @rhwp/editor | 0.7.13 | MIT | iframe 기반 웹 에디터 wrapper |

### rhwp-studio

| 패키지 | Lock 버전 | 라이선스 | 용도 |
|--------|-----------|---------|------|
| canvaskit-wasm | 0.41.1 | BSD-3-Clause | CanvasKit 렌더링 backend |
| @types/chrome | 0.1.42 | MIT | Chrome API 타입 |
| puppeteer-core | 25.0.4 | Apache-2.0 | E2E 테스트 / CDP 연결 |
| typescript | 6.0.3 | Apache-2.0 | TypeScript 컴파일 |
| vite | 8.0.14 | MIT | 개발 서버 + 빌드 |
| vite-plugin-pwa | 1.3.0 | MIT | PWA 빌드 보조 |
| workbox-window | 7.4.1 | MIT | Service worker runtime 보조 |

### rhwp-chrome / rhwp-firefox

| 패키지 | Lock 버전 | 라이선스 | 용도 |
|--------|-----------|---------|------|
| typescript | 5.9.3 | Apache-2.0 | TypeScript 컴파일 |
| vite | 6.4.2 | MIT | 확장 번들 빌드 |

### rhwp-vscode

| 패키지 | Lock 버전 | 라이선스 | 용도 |
|--------|-----------|---------|------|
| @types/node | 20.19.37 | MIT | Node.js 타입 |
| @types/vscode | 1.110.0 | MIT | VS Code API 타입 |
| copy-webpack-plugin | 14.0.0 | MIT | Webpack 파일 복사 |
| null-loader | 4.0.1 | MIT | Webpack 로더 |
| ts-loader | 9.5.4 | MIT | Webpack TypeScript 로더 |
| typescript | 5.9.3 | Apache-2.0 | TypeScript 컴파일 |
| webpack | 5.105.4 | MIT | 번들러 |
| webpack-cli | 6.0.1 | MIT | Webpack CLI |

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
| Nanum Myeongjo (3종) | SIL Open Font License 1.1 | Google Fonts | Serif fallback |
| Nanum Gothic (3종) | SIL Open Font License 1.1 | Google Fonts | Sans-serif fallback |
| Nanum Gothic Coding (2종) | SIL Open Font License 1.1 | Google Fonts | Monospace fallback |
| Gowun Batang (2종) | SIL Open Font License 1.1 | Google Fonts | Serif fallback |
| Gowun Dodum | SIL Open Font License 1.1 | Google Fonts | Sans-serif fallback |
| D2 Coding (2종) | SIL Open Font License 1.1 | NAVER | Monospace fallback |
| Spoqa Han Sans | SIL Open Font License 1.1 | github.com/spoqa/spoqa-han-sans | Sans-serif fallback |
| Source Han Serif K Old Hangul subset | SIL Open Font License 1.1 | Adobe / Google | 옛한글 PUA fallback, `SourceHanSerifK-OFL.txt` 동봉 |
| Latin Modern Math | GUST Font License | GUST / TeX ecosystem | 수식 fallback |
| Cafe24 써라운드 | 카페24 무료 배포 | fonts.cafe24.com | 장식체 |
| Cafe24 슈퍼매직 | 카페24 무료 배포 | fonts.cafe24.com | 장식체 |
| 행복고딕 (Happiness Sans, 4종) | 무료 배포 | 행복나눔재단 | Sans-serif fallback |

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
