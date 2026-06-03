# Task #275 최종 결과보고서 — WASM canvas 렌더러 OLE RawSvg/Placeholder 처리 복구

**이슈**: [#275 WASM canvas 렌더러가 OLE 개체(RawSvg/Placeholder) 처리 누락 — bitmap.hwp/한셀OLE.hwp 빈 페이지](https://github.com/edwardkim/rhwp/issues/275)
**브랜치**: `local/task275` (3 커밋: 8eab580, 76fed63, 23b6905)
**마일스톤**: v1.0.0 (M100)
**기간**: 2026-04-24 (하루 내 완료)

## 1. 문제

`samples/bitmap.hwp`, `samples/한셀OLE.hwp` 를 rhwp-studio 에서 열면 파일 로드는 성공(1페이지 표시)하지만 **본문이 완전히 빈 페이지**로 렌더. 네이티브 CLI (`rhwp export-svg`) 는 정상 렌더 (46KB SVG, BMP 이미지 포함).

### 근본 원인

두 파일 모두 첫 문단에 **OLE 컨트롤** 존재 (bitmap.hwp: 150×84mm BMP, 한셀OLE.hwp: 106×14mm 한셀 시트).

`src/renderer/layout/shape_layout.rs:983-1094` `ShapeObject::Ole` 처리가 OLE 컨테이너를 다음 중 하나의 렌더 노드로 변환:
- `RenderNodeType::RawSvg` — OOXML 차트 SVG / EMF 변환 SVG / 네이티브 BMP·PNG·JPEG (`<image data:...>`)
- `RenderNodeType::Placeholder` — 모든 추출 실패 시 폴백

두 렌더러의 노드 처리 차이:

| 노드 타입 | `src/renderer/svg.rs` | `src/renderer/web_canvas.rs` |
|-----------|----|----|
| `RawSvg` | 처리 O | **arm 부재 → `_ =>` 로 빠짐 (암묵 무시)** |
| `Placeholder` | 처리 O | **arm 부재 → `_ =>` 로 빠짐 (암묵 무시)** |

## 2. 해결

### 2.1 변경 요약

| 파일 | 변경 | 라인 |
|------|------|------|
| `src/renderer/mod.rs` | `pub mod svg_fragment;` 등록 | +1 |
| `src/renderer/svg_fragment.rs` (신규) | SVG 조각 파서 유틸 + 단위 테스트 19건 | +247 |
| `src/renderer/web_canvas.rs` | Placeholder arm, RawSvg arm (A+B 경로), detect_image_mime_type SVG 확장 | +51 |
| `mydocs/plans/task_m100_275.md` (신규) | 수행계획서 | +82 |
| `mydocs/plans/task_m100_275_impl.md` (신규) | 구현계획서 | +195 |
| `mydocs/working/task_m100_275_stage1.md` (신규) | 단계1 보고서 | +66 |
| `mydocs/working/task_m100_275_stage2.md` (신규) | 단계2 보고서 | +114 |
| `mydocs/working/task_m100_275_stage3.md` (신규) | 단계3 보고서 | +121 |

### 2.2 설계 결정

1. **공용 헬퍼를 별도 모듈로 분리** (`svg_fragment.rs`):
   - `web_canvas` 모듈이 wasm32 전용이라 네이티브 테스트 불가 → 헬퍼 분리로 네이티브 단위 테스트 확보
   - `find_svg_attr_value`, `try_parse_single_image_data_url`, `decode_base64_data_url`, `is_svg_prefix`, `wrap_svg_fragment`

2. **`RawSvg` 이중 경로 (A/B)**:
   - **A 경로** (`<image data:...>` 단일 요소, BMP/PNG/JPEG): 직접 디코드 → `draw_image` 호출
   - **B 경로** (복합 SVG, EMF/OOXML): `wrap_svg_fragment` 로 `<svg>` 문서화 → `draw_image` 호출 (SVG MIME 감지 경유)
   - 둘 다 기존 `draw_image` 의 IMAGE_CACHE · HtmlImageElement async 로드 · 재렌더 파이프라인 공유 → 별도 `draw_svg` 함수 불필요

3. **`detect_image_mime_type` 확장** (draw_svg 분리 회피):
   - `is_svg_prefix` 매치 시 `image/svg+xml` MIME 반환 → 기존 `draw_image` 가 자동으로 `data:image/svg+xml;base64,...` data URL 생성

4. **viewBox 좌표계**: `<svg>` 래퍼의 viewBox 와 width/height 를 bbox 와 동일하게 설정 → 조각 내부의 페이지 절대좌표 = drawImage 위치 (1:1 단순 투영)

5. **Placeholder arm 은 svg.rs 와 동등 출력**:
   - `StrokeDash::Dash` ([6, 3]) + fill_rect + stroke_rect
   - 폰트 크기 `clamp(min(w, h) * 0.06, 12, 28)` — svg.rs 와 동일 공식
   - text-align / baseline 기본값 복원으로 다른 노드 영향 차단

## 3. 검증 결과

### 3.1 단위 테스트

`cargo test --lib svg_fragment`: **19 passed / 0 failed**
- `find_attr_*` 3건 (단어 경계, 기본, 부재)
- `parse_single_image_*` 6건 (xlink:href, href, 공백, 복합 SVG 거부, 비-data URL 거부, href 부재)
- `decode_data_url_*` 3건 (PNG, 비-base64 거부, malformed 거부)
- `wrap_svg_fragment_*` 2건 (기본, 이스케이프 보존)
- `is_svg_prefix_*` 5건 (직접, XML 선언, PNG 거부, HTML 거부, 256B 창)

### 3.2 회귀

`cargo test --lib`: **968 passed / 14 failed / 1 ignored**
- +19 신규, 949 → 968
- 14 failed: baseline 사전 실패 (cfb_writer/wasm_api 직렬화 roundtrip), 단계별 stash 비교로 확인
- 변화 없음

### 3.3 빌드

- `cargo check --lib --target wasm32-unknown-unknown`: clean
- `wasm-pack build --target web`: 성공
- `pkg/rhwp_bg.wasm`: 4,043,989 → **4,051,019 bytes (+7,030 bytes)**
- `npx tsc --noEmit` (rhwp-studio): clean

### 3.4 시각 검증 (puppeteer + headless Chrome)

**A 경로 (네이티브 이미지)** — 이슈 재현 샘플 해결:

| 파일 | 변경 전 | 변경 후 |
|------|---------|---------|
| `samples/bitmap.hwp` | 빈 페이지 | **비트맵 손글씨 이미지 정상 렌더** |
| `samples/한셀OLE.hwp` | 빈 페이지 | **노란 스프레드시트 이미지 정상 렌더** |

**B 경로 (복합 SVG)** — 단계 3 에서 shape_layout.rs 임시 가드로 강제 재현:
- 원본 `<image>` 를 `<g><rect stroke=red/><image/><text>B-PATH</text></g>` 복합 SVG 로 교체
- **빨간 사각형 테두리 + "B-PATH" 라벨 + 내부 이미지 모두 동시 렌더 확인**
- 원복 후 diff clean

**Placeholder 경로** — 단계 3 에서 shape_layout.rs 임시 가드로 강제 재현:
- FORCE_PLACEHOLDER 로 OLE 추출 건너뜀
- **회색 배경 (#f0f0f0) + 점선 테두리 (#707070, [6,3]) + 중앙 라벨 "OLE 개체 (BinData #1)" 렌더 확인** — svg.rs 출력과 동등
- 원복 후 diff clean

**회귀** (기존 샘플): `biz_plan.hwp`, `form-002.hwpx` 렌더 변화 없음.

## 4. 파급 효과

이번 수정으로 **WASM canvas 에서 정상 렌더되는 OLE 유형**:

1. 네이티브 이미지 임베드 OLE (BMP/PNG/JPEG/GIF 포함) — A 경로
2. EMF 프리뷰가 있는 OLE — B 경로
3. OOXML 차트 (Task #195 단계 8 연계) — B 경로 (HWPX ooxml_chart 직접 경로 + CFB 내부 ooxml_chart)
4. 모든 추출 실패 시 Placeholder 폴백

**실측 확인**: A 경로 (bitmap/한셀OLE) + B 경로·Placeholder (강제 재현). 실제 EMF/OOXML 차트 샘플이 `samples/` 에 없어 production 실 샘플 검증은 커뮤니티 제보 대기.

## 5. 범위 외 (후속 이슈 후보)

- **A 경로 성능 최적화**: 현재 A 경로는 base64 디코드 → `draw_image` 내부에서 재인코딩 중. 캐시 히트 후에는 문제 없으나 첫 로드에서 약간의 중복 작업. 큰 이미지에서 체감 시 최적화 고려
- **EMF/OOXML 실제 샘플 편입**: `samples/` 에 대표 파일 추가하면 정식 e2e 테스트 작성 가능
- **`RawSvgNode` 데이터 모델 개선**: 현재 SVG 문자열을 담는 방식. 구조화된 노드 트리로 변환하면 canvas 네이티브 path 명령으로 직접 그릴 수 있어 async 로드 없이 동기 렌더 가능 (M101+ 범위)

## 6. 커밋 이력

```
23b6905  Task #275 단계3: RawSvg 복합 SVG 경로 + detect_image_mime_type SVG 확장
76fed63  Task #275 단계2: RawSvg <image> 단일 경로 (OLE 네이티브 이미지 복구)
8eab580  Task #275 단계1: web_canvas 에 Placeholder arm 추가
```

## 7. 승인 요청

- 최종 결과보고서 + 오늘할일 갱신 커밋 허가 요청
- 승인 시 `local/task275` → `devel` merge (`--no-ff`) + GitHub Issue #275 close

## 8. 다음 작업

작업지시자 지시 대기.
