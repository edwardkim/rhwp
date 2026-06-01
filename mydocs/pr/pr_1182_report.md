# PR #1182 처리 보고서 — RawSvg(OLE/차트) 첫 로드 백지 렌더 수정

- **작성일**: 2026-05-31
- **PR**: #1182 → **MERGED** (devel, 머지커밋 `23742708`)
- **컨트리뷰터**: @planet6897 (핵심 컨트리뷰터)
- **연결 이슈**: #1181 (CLOSED)
- **판단**: **머지** ✅ (시각 판정 통과)

## 결정 사유

#1154 v2 (PR #1164, 동일 컨트리뷰터가 머지) 의 flow 디코드 안전망이 `PaintOp::Image`
만 대상으로 잡아 `PaintOp::RawSvg` 경로가 누락된 회귀를 정확히 보완. 한셀 OLE / 차트
OOXML / EMF 미리보기가 첫 로드 시 백지로 그려지던 문제를 해소. 일반 그림 동작은 불변.
검증 + 시각 판정 모두 통과하여 머지.

## 변경 요약 (2 파일, +54 / −35)

| 파일 | 변경 |
|------|------|
| `src/document_core/queries/rendering.rs` | `collect()`: `if let Image` → `match op`, `PaintOp::RawSvg => image_count++` 추가. Image 로직 동일 보존(순수 리팩토링). |
| `rhwp-studio/src/view/page-renderer.ts` | `prefetchFlowImages()`: `enqueue()`+dedupe Set, `data:image/...;base64` 직접 스캔으로 rawSvg 내장 data URL prefetch. 기존 image overlay 필터 보존. |

## 검증 결과

| 단계 | 명령 | 결과 |
|------|------|------|
| 1 | `cargo fmt --all --check` | ✅ OK |
| 2 | `cargo build` | ✅ 성공 |
| 3 | `cargo test --lib document_core::queries::rendering` | ✅ 6 passed |
| 4 | `cargo test --tests` | ✅ 97 스위트, 1826 passed, 0 failed |
| 5 | `cargo clippy --lib` | ✅ 경고 없음 |
| 6 | `tsc --noEmit` (rhwp-studio) | ✅ 통과 |
| 7 | WASM 빌드 + 시각 판정 | ✅ 통과 (작업지시자, 한셀OLE.hwp 첫 로드 백지 회귀 해소) |
| 8 | SVG 내보내기 확인 | ✅ OLE 미리보기 PNG `<image data:image/png;base64>` 정상 임베드 (output/svg/han-cell-ole/한셀OLE.svg) |

## 처리 절차

1. PR 정보 확인 — MERGEABLE / head BEHIND (충돌 없음). merge-base 기준 2 파일.
2. `pr_1182_review.md` 작성 → 승인.
3. 로컬 머지 시뮬레이션 브랜치에서 8단계 검증 통과.
4. GitHub UI 머지는 head BEHIND → 메인테이너 로컬 `--no-ff` 머지 + push (`347224ed..23742708`).
5. 시각 판정 통과 → 이슈 #1181 수동 클로즈 + PR 댓글.

## 비고

- SVG 경로(`svg.rs`)는 web_canvas 와 별개(RenderNode DFS)로 RawSvg 를 처리하므로
  본 PR(web_canvas IMAGE_CACHE 안전망)과 무관하게 정상 작동함을 export-svg 로 확인.
- @planet6897 의 누적 기여(#1164/#1148/#1095 머지)의 일부. 자기 PR(#1164)의 누락 보완.
