# Task #902 v2 Stage 33 — WMF inline SVG 임베딩 (sandbox 우회)

## 배경

PR #918 의 Phase 3 + 옵션 2 (woff2 base64 SVG embed) 적용 후에도, hwp3-sample16.hwp 페이지 18 의 박스 안 텍스트가 정확히 배치되지 않는 회귀가 남아 있다 (작업지시자 보고).

### 진단 결과

- 페이지 18 은 WMF 1개를 포함, WMF 내부에 1599 tspans + 1 path + 1 polygon. 페이지 20 은 WMF 없음 → 정상.
- 현재 파이프라인: `convert_wmf_to_svg(data)` → SVG 바이트 → base64 → `<image href="data:image/svg+xml;base64,...">` 로 outer SVG 또는 Canvas2D 에 임베드.
- rsvg-convert (libsrvg) 로 standalone 렌더링 시: 박스 안 텍스트 정확히 배치됨 — SVG 자체는 정확.
- 브라우저 (rhwp-studio) 의 `<img src=data:svg>` 또는 SVG `<image href=data:svg>` 는 **sandboxed image** 로 처리되어 inner SVG 의 `@font-face` 가 일부 환경에서 적용되지 않거나 metrics 불일치 → 텍스트 mispositioning.
- inner SVG 의 `<text>` 요소 font-family chain 에 `NanumGothicEmbedded` 가 빠져 있고, CSS rule `text, tspan { font-family: ... !important }` 으로 override 시도 — sandbox 에서 신뢰 불가.

### 해결 방향

WMF → SVG 결과물을 **inline `<svg>` nested element** 로 outer SVG/DOM 에 직접 삽입하여 sandbox 제약 우회.

## 작업 범위

### 영향 파일

| 파일 | 변경 내용 |
|------|---------|
| `src/renderer/svg.rs` (line 1141 wasm branch + 2358 두 번째 branch) | WMF → SVG 결과를 nested `<svg>` 로 inline. base64/data URL 미사용. |
| `src/wmf/converter/svg/mod.rs` | nested SVG 친화 출력 옵션 (xmlns 생략 가능 / @font-face → outer 로 hoist 가능 / id 충돌 방지 prefix). |
| `src/renderer/web_canvas.rs` (line 2026~) | Canvas2D 경로 — WMF 영역만 별도 DOM `<svg>` overlay 로 렌더 (Canvas2D 외부). 또는 keep `<image>` 경로 + 별도 후속 이슈로 분리. |

### 단계

**Stage 33-A — SVG export inline 화**
- `convert_wmf_to_inline_svg(data) -> Option<(viewBox, body_xml)>` 신설 (또는 `convert_wmf_to_svg` 결과를 파싱).
- `svg.rs` 의 image-as-data-URL 경로를 nested `<svg x=... y=... width=... height=... viewBox=... preserveAspectRatio="none">{body}</svg>` 으로 교체.
- 검증: `rhwp export-svg hwp3-sample16.hwp -p 17` 결과를 rsvg-convert / 브라우저로 렌더링 → 박스/텍스트 정합 확인. golden_svg 회귀 점검.

**Stage 33-B — Canvas2D (rhwp-studio) 경로 결정**
- 옵션 1: Canvas2D 위에 별도 DOM SVG overlay 삽입 (JS 레이어 추가, rhwp-studio 수정)
- 옵션 2: 현 `<image>` 유지 + 후속 이슈로 분리
- 옵션 3: SVG 결과물을 `rhwp-studio` JS 에서 `new DOMParser().parseFromString(...)` 으로 파싱하여 페이지 SVG 에 직접 삽입

작업지시자 결정 후 진행.

## 위험 / 회귀

- **id 충돌**: inner SVG 의 `id="elem23"` 류가 outer 와 충돌 가능 → prefix (`wmf{N}_elem23`) 도입.
- **viewBox 정합**: nested `<svg>` 는 자체 viewBox 좌표계 가지므로 outer x/y/width/height + preserveAspectRatio 만 신경 쓰면 됨.
- **CSS 격리**: outer 의 CSS 가 inner text 에 영향. 반대로 inner `<style>` 이 outer 에 누출 가능 → `<style scoped>` 미지원, namespace prefix 로 회피.
- **golden_svg 영향**: SVG 구조 변경으로 모든 WMF 포함 페이지 golden 재생성 필요.

## 검증

1. `cargo build --release` + `cargo test`
2. `rhwp export-svg samples/hwp3-sample16.hwp -p 17` → outer + nested SVG 구조 확인
3. rsvg-convert 로 outer SVG 렌더링 → 박스/텍스트 정확
4. 브라우저 (Safari/Chrome) 에서 outer SVG 직접 표시 → 박스/텍스트 정확
5. (Stage 33-B 이후) rhwp-studio 로 페이지 18 표시 → 박스/텍스트 정확

## 예상 분량

- 코어 변경: svg.rs ~40 lines, mod.rs ~20 lines (옵션 함수 추가)
- 테스트: 골든 SVG 재생성 + 시각 검증

작업지시자 승인 후 단계별 진행.
