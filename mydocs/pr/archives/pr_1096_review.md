# PR #1096 검토 — render: expand CanvasKit image replay coverage

## 1. 개요

| 항목 | 내용 |
|------|------|
| PR | [#1096](https://github.com/edwardkim/rhwp/pull/1096) |
| 작성자 | seo-rii (Seohyun Lee) — CanvasKit P-series 핵심 컨트리뷰터 (P8~P18) |
| base / head | `devel` / `seo-rii:render-p18` |
| 이슈 | `Refs #536` (멀티 렌더러 지원 트래킹 — closes 아님, P 시리즈 단계 진행) |
| label | enhancement |
| mergeable / merge state | MERGEABLE / BEHIND (conflict 없음, devel 가 앞섬) |
| 변경 | +592 / -39, 5 files (TS 4 + Rust 1) |
| commits | 3 (feat + review feedback 2회 반영) |
| CI | **전부 pass** (Build & Test / CodeQL / Canvas visual diff / Analyze 모두) |

## 2. seo-rii P-series 사이클 컨텍스트

| PR | 단계 | 상태 |
|----|------|------|
| #761 | P8 — PageLayerTree schema 강화 | CLOSED (merge) |
| #769 | P9 — Skia text replay parity | CLOSED |
| #797 | P11 — Text IR v2 호환 contract | CLOSED |
| #840 | P12 — GlyphRun layer variant contract | CLOSED |
| #881 | P13 — Text IR v2 diagnostics | CLOSED |
| #916 | P14 — guarded text variants | CLOSED |
| #925 | P15 — replay policy diagnostics | CLOSED |
| #996 | P16 — browser CanvasKit direct renderer | CLOSED |
| #1057 | P17 — CanvasKit direct replay contract 강화 | CLOSED (merge) |
| **#1096** | **P18 — image replay coverage 확장** | **본 PR** |

본 P18 은 P17 의 direct replay contract 후속 — CanvasKit image replay 가 layer JSON
의 image geometry payload (crop / fillMode / transform) 를 실제 소비하도록 확장.

## 3. 본질 분석

### 3.1 본 PR 의 범위 (PR 본문 + 코드 정독)

1. **CanvasKit image replay 확장**: layer `crop`, `originalSize`, `fillMode`, `transform`
   payload 직접 소비 (이전: bbox 만 사용)
2. **image cache key 강화**: `imageRef` 만 보지 않고 base64 fingerprint 포함 — 같은 ref 의
   다른 payload 충돌 방지
3. **순수 함수 분리**: CanvasKit image helper → `image-replay.ts` 신규 파일 — crop/source
   rect, fill-mode anchor, cache-key 등 회귀 테스트 가능 영역
4. **`getCanvasKitReplayPlan` 분류 정교화**: 단순 image = direct item, image effect/
   brightness/contrast 는 `imageEffect:*` diagnostics 로 남김 (직접 적용 비범위)

### 3.2 명시적 non-goals

- grayscale/blackWhite/pattern8x8 pixel filter direct 적용
- RawSvg/WMF/OLE object replay 확장
- TextRun effect/glyph payload replay
- renderer sweep / WebGPU / performance / PDF

→ scope 한정 명확. 본 PR 은 image geometry payload 만 직접 소비.

### 3.3 호환성 보장 (PR 본문 명시)

- 기본 renderer = Canvas2D (변경 없음)
- CanvasKit = opt-in path (변경)
- image effect/brightness/contrast 는 결과 변경 안 함 — diagnostics 만 추가
- public native Skia/PDF API 변경 없음

→ **회귀 위험 매우 낮음** — CanvasKit opt-in path 만 확장, default Canvas2D 무영향.

## 4. 변경 파일 정독

| 파일 | 추가 | 영역 |
|------|------|------|
| `rhwp-studio/src/core/types.ts` | +3 | `LayerImageOp` 에 `originalSize` / `crop` / `transform` 필드 추가 |
| `rhwp-studio/src/view/canvaskit/image-replay.ts` | +138 신규 | 순수 함수 helper (cache key + source rect + fill-mode anchor) |
| `rhwp-studio/src/view/canvaskit-renderer.ts` | +198/-X | `renderImage` 본체 + cache key + transform 적용 |
| `rhwp-studio/tests/render-backend.test.ts` | +40 | 회귀 테스트 (crop, source rect, fill-mode, cache key) |
| `src/renderer/canvaskit_policy.rs` | +252 | replay item 분류 + diagnostics |

### 4.1 types.ts 정합성

```typescript
export interface LayerImageOp {
  // ... 기존 ...
  fillMode?: string;
  originalSize?: { width: number; height: number };  // 신규
  crop?: { left: number; top: number; right: number; bottom: number };  // 신규
  // ...
  transform?: LayerPathTransform;  // Task #1067 의 LayerPathTransform 재사용
}
```

`LayerPathTransform` 재사용 — Task #1067 의 정합 인터페이스. base 가 #1067 머지 후 시점 (mergeable 확인).

### 4.2 신규 helper image-replay.ts

순수 함수 영역 — `canvasKitImageCacheKey` (fnv1a32 fingerprint), `canvasKitImageSourceRect`
(crop → source rect), fill-mode anchor 등. 회귀 테스트 가능 + 다른 PR 영역과 충돌 없음.

`HWPUNIT_PER_PIXEL = 75` 명시 (96 DPI 정합) — Rust 측 hwpunit_to_px 와 정합.

### 4.3 canvaskit_policy.rs 정독 영역

+252 lines — replay item 분류 (direct vs deterministic detail). image effect/brightness/
contrast 는 deterministic detail (CanvasKit 가 처리 못하는 영역 명시) — 후속 PR 영역 분리.

## 5. PR 작성자 검증 (PR 본문)

- `npm --prefix rhwp-studio test`
- `npm --prefix rhwp-studio run build`
- `cargo test canvaskit --lib`
- `cargo fmt --check`
- `git diff --cached --check`

## 6. PR 사이클의 review feedback 반영

3 commits 중 2 개가 review feedback 반영:
- `feat: expand CanvasKit image replay coverage` (초안)
- `fix: address CanvasKit image replay review feedback`
- `fix: tighten CanvasKit image replay diagnostics`

→ 메인테이너 review 후 보정 흔적 — 본 PR 의 정정 영역이 review 단계 거침.

## 7. 위험 분석

| 위험 | 평가 |
|------|------|
| canvaskit-renderer.ts 가 #1067 의 renderPath 정정과 같은 파일 | mergeable=MERGEABLE, LayerPathTransform 재사용 — base 가 #1067 이후, conflict 없음 |
| `LayerImageOp` 인터페이스 변경 | optional field 추가만 (`?:`), 호환적 |
| canvaskit_policy.rs +252 lines (큰 영역) | Rust 측 정책 — diagnostics 분류만, 실제 동작 변경 부재 (PR 본문 보증) |
| 새 helper 파일 image-replay.ts | 신규 파일, 다른 영역 영향 없음 + 회귀 테스트 동반 |
| default Canvas2D 영역 영향 | 본 PR 가 CanvasKit opt-in path 만 변경 — Canvas2D 무영향 |

## 8. 검증 계획 (메인테이너 영역)

| 항목 | 명령 |
|------|------|
| Rust lib | `cargo test --release --lib canvaskit` |
| Rust 전체 lib | `cargo test --release --lib` |
| clippy | `cargo clippy --release --lib -- -D warnings` |
| fmt | `cargo fmt --all --check` |
| rhwp-studio test | `cd rhwp-studio && npm test` |
| rhwp-studio build | `cd rhwp-studio && npm run build` |
| CI 전부 pass 확인 (이미 통과) | GitHub Actions |

## 9. 처리 권장

- **merge 권장** (검증 통과 후) — 본질 정확, 회귀 영역 한정 (CanvasKit opt-in path 만),
  scope 명시적 (non-goals 명확), CI 전부 pass, review feedback 2회 반영
- merge 방식: merge commit (3 commits 보존 — 작성자 + review iteration 영역 가시화)
- close 후 archives: `mydocs/pr/archives/pr_1096_*.md`

## 10. 메모리 룰 정합

- `feedback_contributor_cycle_check` — seo-rii P-series 핵심 컨트리뷰터, P8~P18 누적
- `feedback_pr_supersede_chain` — P-series 단계적 진행 (개별 merge 패턴)
- `feedback_pr_comment_tone` — 반복 컨트리뷰터, 차분한 사실 중심 merge 메시지
- `feedback_release_sync_check` — devel merge 전 origin/devel 동기화 확인
- `feedback_push_full_test_required` — lib + tests + clippy + fmt + npm 모두 통과

## 11. 작업지시자 승인 요청

1. 본 검토 (merge 권장) 승인 여부
2. 검증 영역 (Rust lib + clippy + fmt + npm test/build) 권장 수용 여부
3. merge 방식 (merge commit, 3 commits 보존) 결정
