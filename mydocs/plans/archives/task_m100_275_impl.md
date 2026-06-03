# 구현계획서 — Task #275

**이슈**: [#275](https://github.com/edwardkim/rhwp/issues/275)
**브랜치**: `local/task275`
**수행계획서**: [`task_m100_275.md`](./task_m100_275.md) (승인 완료)

## 1. 실측 확인 결과 (RawSvg 조각 형식)

수행계획서 승인 후 실제 SVG 조각 출력 형태를 확인:

### A) 네이티브 이미지 경로 (shape_layout.rs:1059-1062)

```html
<image x=".." y=".." width=".." height=".." preserveAspectRatio="xMidYMid meet"
       xlink:href="data:image/png;base64,..." href="data:image/png;base64,..."/>
```

**단일 `<image>` 요소.** 속성 파싱만으로 기존 `draw_image(data_bytes, x, y, w, h)` 재사용 가능.

### B) EMF 변환 경로 (`src/emf/mod.rs:75`)

```html
<g transform="matrix(a,b,c,d,e,f)">
  <rect .../> <path .../> <ellipse .../> <polyline .../> <text .../>
  ...
</g>
```

**복합 SVG. `<g transform>` 루트 + 다수 요소.**

### C) OOXML 차트 경로 (`src/ooxml_chart/renderer.rs:64`)

```html
<g class="hwp-ooxml-chart">
  <rect .../> <text .../> <path .../> <line .../>
  ...
</g>
```

**복합 SVG. `<g>` 루트 + 다수 요소.**

B, C 는 HTML canvas 에 직접 그리려면 완전한 SVG 파서가 필요하므로 **SVG→Image 변환 경로**로 통합 처리한다.

## 2. 구현 전략 확정

### 2.1 `RenderNodeType::Placeholder` (단순)

`web_canvas.rs` 의 `render_node` match 에 arm 추가:
```rust
RenderNodeType::Placeholder(ph) => {
    // 1. 배경 rect (fill_color)
    // 2. 테두리 (stroke_color, 1px, dashed [4, 3])
    // 3. 중앙 라벨 (sans-serif 12px, color #333, text-anchor middle, baseline middle)
}
```
`svg.rs` 의 Placeholder 출력을 참고하여 동등한 시각 표현. 동기.

### 2.2 `RenderNodeType::RawSvg` — 디스패치 + 두 경로

```rust
RenderNodeType::RawSvg(raw) => {
    if let Some(parsed) = try_parse_single_image_svg(&raw.svg) {
        // A 경로: 단일 <image>
        self.draw_image(&parsed.data, parsed.x, parsed.y, parsed.w, parsed.h);
    } else {
        // B/C 경로: SVG 조각을 <svg> 로 래핑 → Blob URL → HtmlImageElement 비동기 로드
        self.draw_raw_svg_async(&raw.svg, &node.bbox);
    }
}
```

#### A 경로 (`try_parse_single_image_svg`)

- 정규식 또는 수동 파싱으로 `<image ... href="data:(mime);base64,(data)" .../>` 추출
- `xlink:href` 우선, 없으면 `href`
- 실패 시 `None` → B 경로로 폴백
- 반환: `{ data: Vec<u8>, x, y, w, h, mime }`

#### B 경로 (`draw_raw_svg_async`)

- 조각을 `<svg xmlns="..." xmlns:xlink="..." width=".." height=".." viewBox="x y w h">` 로 래핑
  - width/height/viewBox 는 node.bbox 기반
- 전체 SVG 문자열을 data URL (`data:image/svg+xml;charset=utf-8,<encoded>`) 로 인코딩
- 기존 `draw_image` 와 동일한 `HtmlImageElement` 패턴 사용:
  - IMAGE_CACHE 해시 기반 캐시 (SVG 문자열 해시)
  - cache hit: 즉시 drawImage
  - cache miss: placeholder 영역 유지 + Image 생성 + onload 에서 drawImage + 캐시 저장 → 재렌더 트리거 필요
- **재렌더 트리거**: 기존 `draw_image` 구현을 살펴보고 동일 경로 활용 (raw_svg 도 결국 같은 async Image 로딩 문제)

### 2.3 참조: 기존 draw_image 비동기 패턴

`web_canvas.rs:1521-1568` 의 `IMAGE_CACHE` + `HtmlImageElement` + `onload` 패턴을 그대로 따라간다. 단, 캐시 키는:
- 이미지: 바이트 해시
- SVG 래핑: SVG 문자열 해시 (`std::hash::Hasher` 로 u64)

## 3. 단계 구성 (4단계)

### 단계 1 — Placeholder arm + 테스트 인프라

**변경**:
- `src/renderer/web_canvas.rs` — `Placeholder` match arm (+~30줄)

**테스트**:
- `tests/web_canvas_render.rs` (신규 or 기존 모듈) — `#[cfg(target_arch = "wasm32")]` 이 아닌 native stub 은 어려우므로 **단위 테스트는 생략**하고 SVG snapshot 비교로 간접 검증
- 대신 `svg.rs` 의 Placeholder 출력과 수동 시각 비교 (스크린샷)

**검증**:
- `cargo build --lib` clean
- WASM 재빌드 후 Placeholder 경로 재현 케이스 필요 — **현재 샘플 중 실제로 Placeholder 로 떨어지는 파일이 없음** (bitmap/한셀OLE 은 native_image 로 성공)
- 대안: 일시적으로 `shape_layout.rs` 의 OLE 처리에 주석 처리하여 Placeholder 로 강제 떨어뜨리고 스크린샷 검증 → 원복

**완료 기준**:
- `web_canvas.rs` Placeholder arm 구현 완료
- 강제 Placeholder 케이스에서 rect + 라벨 정상 렌더 확인

### 단계 2 — RawSvg `<image>` 단일 경로 (A)

**변경**:
- `src/renderer/web_canvas.rs` — `try_parse_single_image_svg` 유틸 함수
- `src/renderer/web_canvas.rs` — `RawSvg` match arm (A 경로만, B 는 다음 단계)

**테스트**:
- `try_parse_single_image_svg` 단위 테스트 (tests 모듈 in-file `#[cfg(test)]`):
  - 정상 케이스: `<image x="1" y="2" width="3" height="4" xlink:href="data:image/png;base64,AAA"/>`
  - href only, xlink:href only, 둘 다
  - 비-image 조각 (`<g>...</g>`) → None 반환
  - base64 payload 디코딩 실패 → None 반환

**검증**:
- `cargo test --lib try_parse_single_image_svg` pass
- WASM 재빌드 후:
  - `samples/bitmap.hwp` → 이미지 정상 표시 (e2e 스크린샷 비교)
  - `samples/한셀OLE.hwp` → 이미지 정상 표시

**완료 기준**:
- 두 재현 샘플이 canvas 에 이미지와 함께 렌더됨
- 기존 회귀 (`biz_plan.hwp`, `form-002.hwpx`) 변화 없음

### 단계 3 — RawSvg 일반 경로 (B/C) + async 로드

**변경**:
- `src/renderer/web_canvas.rs` — `draw_raw_svg_async` 함수
  - SVG 래핑 유틸 (`wrap_svg_fragment`)
  - SVG 문자열 해시 → IMAGE_CACHE 공유
  - `draw_image` 와 동일한 Image onload → 재렌더 트리거
- `RawSvg` arm 의 else 분기 활성화

**테스트**:
- `wrap_svg_fragment` 단위 테스트:
  - 입력: `<g transform="matrix(1,0,0,1,0,0)"><rect .../></g>`, bbox=(10,20,100,50)
  - 출력: `<svg xmlns="..." xmlns:xlink="..." width="100" height="50" viewBox="10 20 100 50">...</svg>`

**검증**:
- 합성 테스트 문서: EMF 를 가진 OLE 샘플이 없으므로 **`tests/` 에 소형 HWP 픽스처 추가 or 기존 단위 테스트로 `emf::convert_to_svg` 출력이 wrap 후 유효한 SVG 인지 확인**
- 실제 E2E: 가능하면 OOXML 차트가 있는 HWP 를 제보 받아 확인 (없으면 단계 4 에서 합성 E2E)

**완료 기준**:
- EMF/OOXML 조각이 canvas 에 그림 형태로 출력됨 (or 최소 blob URL 로드 성공 확인)
- 재렌더 트리거로 async 로드 후 이미지가 실제로 canvas 에 그려짐

### 단계 4 — WASM 재빌드 + E2E + 정리 + 최종 보고서

**작업**:
1. Docker WASM 재빌드 (`docker compose --env-file .env.docker run --rm wasm`)
2. `rhwp-studio` dev 서버 재시작 (이미 구동 중)
3. 정식 E2E 테스트로 전환:
   - `rhwp-studio/e2e/ole-render.test.mjs` 신규 (or debug-load-bug.test.mjs 정식화)
   - `samples/bitmap.hwp`, `samples/한셀OLE.hwp` 를 `rhwp-studio/public/samples/` 에 정식 편입 (or 현재 `_bitmap.hwp`, `_hancell_ole.hwp` 접두어 제거)
4. 세션 중 임시 파일 정리:
   - `rhwp-studio/public/samples/_bitmap.hwp`, `_hancell_ole.hwp` → 정식 편입 or 삭제
   - `rhwp-studio/e2e/debug-load-bug.test.mjs` → 정식 테스트로 전환 or 삭제
   - 루트의 `first-readme.txt`, `preview.log` → 삭제
5. 회귀 스모크: 기존 HWP 5.0/HWPX 몇 건 로드 확인
6. 최종 보고서 작성: `mydocs/report/task_m100_275_report.md`
7. `mydocs/orders/20260424.md` 에 Task #275 항목 추가

**검증**:
- `cargo test --lib` pass (기존 963 + 신규 N 건)
- `cargo clippy --lib -- -D warnings` clean
- `tsc --noEmit` (rhwp-studio) clean
- E2E: bitmap/한셀OLE 정상 렌더 스크린샷 첨부

**완료 기준**:
- 이슈 #275 재현 시나리오 해결
- 회귀 없음
- 작업지시자 승인 → 이슈 close + `local/task275` → `devel` merge

## 4. 위험 및 대응

### 4.1 비동기 렌더 첫 프레임

B 경로 (SVG→Image) 는 onload 비동기이므로 첫 렌더 프레임에서 해당 영역은 빈 채로 남음. 기존 `draw_image` 도 동일 문제를 어떻게 풀고 있는지 단계 2 구현 전에 먼저 확인한다.
- 추측: 아마도 `onload` 안에서 CanvasView 에 재렌더 신호를 보내거나, IMAGE_CACHE 히트율로 2번째 렌더부턴 동기. 첫 로드 시 깜빡임은 감내하는 설계.

### 4.2 EMF/OOXML 실측 샘플 부재

현재 `samples/` 에 EMF 프리뷰 있는 OLE 나 OOXML 차트 샘플이 없어, 단계 3 E2E 검증은 합성 케이스 (샘플 생성 or unit test only) 로 수행. 프로덕션 케이스는 커뮤니티 제보 의존.

**완화**: 단계 3 의 구현을 **자기 검증 가능한 단위 테스트** 로 최대한 덮고, 실제 E2E 는 단계 4 에서 기존 샘플의 regression 부재만 확인.

### 4.3 XML 이스케이프

EMF 변환기 출력에 `xml_escape` 가 적용되어 있는지 확인 필요. 래핑 시 외부 `<svg>` 태그 안의 내용물은 그대로 통과 (이중 이스케이프 방지).

### 4.4 data URL 크기

base64 인코딩된 BMP 는 HWP 안에서 수백 KB ~ 수 MB 가능. data URL 크기 제한은 브라우저별 상이 (Chrome ~수 GB). 실측에서 문제되면 Blob URL 로 전환.

## 5. 변경 예상 라인 수

- `src/renderer/web_canvas.rs`: +150 ~ +250 줄
- `tests/` 또는 in-file `#[cfg(test)]`: +80 ~ +120 줄
- 문서: stage1-4 보고서 + 최종 보고서 = 4~5 파일

## 6. 승인 요청

이 구현계획서 (4단계) 로 진행해도 될지 검토 부탁드립니다. 승인되면 **단계 1 (Placeholder arm)** 부터 시작합니다.
