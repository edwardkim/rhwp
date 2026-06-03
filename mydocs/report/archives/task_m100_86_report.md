# Task #86 최종 결과보고서: HWP 썸네일 자동 추출 + data-hwp-thumbnail 연동

## 개요

HWP/HWPX 파일 내장 미리보기 이미지(PrvImage)를 추출하여, 웹 관리자가 최소 코드로 방문자에게 HWP 문서 썸네일을 제공할 수 있도록 구현하였다.

## 수행 결과

| 단계 | 내용 | 커밋 |
|------|------|------|
| 1단계 | WASM API + 라이브러리 함수 | `2edbbca` |
| 2단계 | CLI `rhwp thumbnail` 명령 | `087f953` |
| 3단계 | Chrome 확장 연동 | `220adb3` |
| 추가 | content-script 캐시 | `864705e` |
| 추가 | 프리페치 + 디바운스 | `d6881a6` |
| 추가 | 뷰포트 오버플로우 방지 | `1dfe829` |
| 추가 | 카드/썸네일 크기 축소 | `21a4ba1` |
| 추가 | HWPX(ZIP) 지원 | `cf8530c` (Rust) + `af04ebb` (Chrome) |

## 산출물

### Rust 라이브러리

- `parser::extract_thumbnail_only(data)` — HWP(CFB) + HWPX(ZIP) 모두 지원
- `parser::ThumbnailResult` — format, data, width, height
- `model::PreviewImageFormat::Png` 변형 추가

### WASM API

- `extractThumbnail(data)` — JSON 반환 `{ format, base64, dataUri, width, height }`

### CLI

```bash
rhwp thumbnail sample.hwp                    # PNG 파일 저장
rhwp thumbnail sample.hwpx                   # HWPX도 지원
rhwp thumbnail sample.hwp --base64           # base64 stdout
rhwp thumbnail sample.hwp --data-uri         # data:image/... stdout
rhwp thumbnail sample.hwp -o thumb.png       # 지정 경로 저장
```

### Chrome 확장

- `sw/thumbnail-extractor.js` — CFB + ZIP 파싱 (JS, WASM 불필요)
- HWPX ZIP deflate 해제: `DecompressionStream('raw')` 네이티브 API
- 2단계 캐싱: content-script + Service Worker
- 프리페치: 페이지 로드 1초 후 모든 HWP 링크 백그라운드 추출 (동시 3개)
- 호버 디바운스 300ms
- 뷰포트 오버플로우 방지 (위/아래/좌우)

## 발견 사항

### PrvImage 포맷 (정오표 #27)

| HWP 버전 | PrvImage 포맷 |
|---------|-------------|
| 최신 (한컴 오피스 2022+) | **PNG** |
| 구 버전 | **GIF** |
| 스펙 기재 | BMP 또는 GIF (PNG 미기재) |

`mydocs/tech/hwp_spec_errata.md` #27에 기록.

### HWPX 썸네일 경로

| 포맷 | 컨테이너 | 썸네일 경로 |
|------|---------|-----------|
| HWP | CFB (OLE2) | `/PrvImage` 스트림 |
| HWPX | ZIP | `Preview/PrvImage.png` 엔트리 |

### DecompressionStream API

브라우저 네이티브 deflate 해제 API. JS 라이브러리 대비 수십 배 빠름. `mydocs/tech/browser_decompression_stream.md`에 기록.

## 검증

| 파일 | 포맷 | 크기 | 용량 |
|------|------|------|------|
| biz_plan.hwp | PNG | 724×1024 | 12,097 bytes |
| shift-return.hwp | GIF | 177×250 | 1,264 bytes |
| kps-ai.hwp | PNG | 724×1024 | 53,065 bytes |
| table-vpos-01.hwpx | PNG | 724×1024 | 60,290 bytes |

- `cargo build` + `cargo test` 785 passed
- Chrome 확장 빌드 + 라이브 서버 테스트 완료
- DevTools 프로파일링: HWPX ZIP deflate 구간 순식간 처리

## 웹 관리자 최종 사용 시나리오

```html
<!-- 이것만으로 호버 시 썸네일 자동 표시 -->
<a href="doc.hwp" data-hwp="true">문서.hwp</a>
```

## 이슈

- https://github.com/edwardkim/rhwp/issues/86
