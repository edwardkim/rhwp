# PR #996 검토 — render: add browser CanvasKit direct renderer

- 작성일: 2026-05-19
- 컨트리뷰터: [@seo-rii](https://github.com/seo-rii) (Seohyun Lee)
- PR: https://github.com/edwardkim/rhwp/pull/996
- base/head: `devel` ← `seo-rii:render-p16` (cross-repo fork)
- 연결 이슈: Refs #536 (P16, render 백엔드 시리즈)
- 규모: +1291 / -19, 11 files (rhwp-studio TypeScript 전용, Rust 소스 무변경)
- 라벨: enhancement

## 1. PR 정보 확인

| 항목 | 값 |
|------|----|
| mergeable | **CONFLICTING** (`rhwp-studio/package-lock.json` 단 1개, add-only) |
| 본질 커밋 | 단일 `2599e2ee` "feat: add browser CanvasKit direct renderer" |
| CI | (확인 필요 — Build & Test / CodeQL) |
| base 동기화 | PR branch base `e5931c67`, 현 devel `bbda5285` (devel 선행) |

## 2. 컨트리뷰터 사이클 점검 (`feedback_contributor_cycle_check`)

@seo-rii = **render 백엔드 핵심 컨트리뷰터, 15 PR 사이클**:

`#165` skia → `#419` PageLayerTree API → `#456` Canvas PageLayerTree 라우팅 → `#498` canvas visual diff → `#599` native Skia PNG → `#626` 수식 replay → `#720` raw SVG replay → `#761` PageLayerTree schema → `#769` Skia text parity → `#797` Text IR v2 contract → `#840` GlyphRun layer variant → `#881` Text IR v2 diagnostics → `#916` guarded text variants → `#925` **P15 CanvasKit replay policy diagnostics** → `#996` **P16 CanvasKit direct renderer (본 PR)**

직전 단계 #925(P15)는 CanvasKit replay를 diagnostics-only API로 개방. 본 P16은 Studio에서 실제 browser CanvasKit backend를 opt-in 실행하는 foundation.

## 3. 변경 내용 분석

### 신규 파일

| 파일 | 역할 |
|------|------|
| `src/view/render-backend.ts` (+126) | backend/mode/surface/profile resolver. URL query + localStorage opt-in. 미지원 값 진단 (`unsupportedReason`) |
| `src/view/canvaskit-renderer.ts` (+608) | `PageLayerTree` JSON 직접 replay. canvaskit-wasm dynamic import 분리. unsupported op는 Canvas2D로 덮지 않고 diagnostics 기록 |
| `src/view/canvaskit/policy.ts` (+20) | `canvaskitClipRightPad` — compat+fastPreview+body/tableCell 한정 4px 우측 패딩. **케이스별 명시 가드** |
| `tests/render-backend.test.ts` (+74) | resolver 단위 + canvaskit-renderer 소스 정적 검사 (Canvas2D overlay 미사용 계약 강제) |

### 수정 파일

- `core/types.ts` (+273): browser `PageLayerTree` 타입 추가
- `core/wasm-bridge.ts` (+53): `getPageLayerTreeObject(page, profile)` bridge
- `main.ts` (+42): backend 해석 → CanvasKit dynamic import → 실패 시 Canvas2D 폴백 → CanvasView 주입
- `view/canvas-view.ts` (+32), `view/page-renderer.ts` (+57): backend 분기 (canvaskit일 때만 `renderPageCanvasKit`), 생성자 기본값 하위 호환
- `package.json`: `canvaskit-wasm@^0.41.1` 1개 추가

## 4. 검토 의견

### 강점

1. **하위 호환 100%** — 생성자 기본값 `backend='canvas2d'`, 기본 경로는 CanvasKit 번들 미로드 (dynamic import). PR 본문 Compatibility 약속과 코드 일치.
2. **opt-in 게이팅 견고** — URL query / localStorage 미지정 시 항상 Canvas2D. 미지원 값은 throw 없이 진단 후 안전 폴백. storage 예외(private 컨텍스트) try/catch 처리.
3. **본질을 테스트로 고정** — "CanvasKit을 Canvas2D-assisted preview로 두지 않는다"는 PR 본질을 소스 정적 검사 회귀 테스트로 강제 (`getContext('2d')`/`rhwpOverlay`/`renderPageToCanvas` 부재).
4. **케이스별 명시 가드** (`feedback_hancom_compat_specific_over_general` 정합) — `policy.ts` 주석이 명시적으로 "not a general clip inflation policy; broader parity rules belong with later coverage work". 일반화 위험 회피, 알려진 legacy text overflow 케이스만 미러.
5. **점진 분리 운영 철학 정합** (`feedback_small_batch_release_strategy`) — P15(diagnostics) → P16(foundation), full parity는 non-goal로 명시 분리. opt-in·하위호환 신규 모듈.
6. **법적 안전** — `canvaskit-wasm@0.41.1` = **BSD-3-Clause** (Google Skia, Chrome 동일 엔진). 프로젝트 MIT와 호환.
7. **렌더러 경로 인지** (`feedback_image_renderer_paths_separate`) — 기존 Canvas2D overlay 경로(svg/web_canvas/json)를 건드리지 않고 별도 backend로 추가. 기존 경로 회귀 표면 없음.

### 충돌 분석

- `package-lock.json` **add-only 충돌 1블록** (라인 2618-2637): devel HEAD에 `@types/yauzl`(canvaskit-wasm 전이 의존)·`@webgpu/types` 엔트리 부재 → PR이 추가. `npm install` lockfile 재생성으로 자동 해결되는 기계적 충돌.
- **본질 소스 파일(types.ts/wasm-bridge.ts/main.ts/canvas-view.ts/page-renderer.ts)은 cherry-pick auto-merge 전부 성공** — 의미적 충돌 0건.

### 확인 필요 사항

1. `npm --prefix rhwp-studio test` (resolver + 소스 정적 검사) GREEN
2. `npm --prefix rhwp-studio run build` (Vite 번들, dynamic chunk 분리) 성공
3. `cargo test --release` 전체 GREEN (Rust 무변경이나 회귀 가드)
4. 작업지시자 시각 판정 — `?renderer=canvaskit` opt-in 시 정상 렌더 + 기본 경로(Canvas2D) 무회귀

## 5. 처리 옵션

- **옵션 A (수용 — 권고)**: 본질 커밋 `2599e2ee` cherry-pick + package-lock 재생성 + 자기 검증(npm test/build + cargo test) + 작업지시자 시각 판정 + no-ff merge. 근거: 하위호환 100%, opt-in, 케이스별 가드, 본질 테스트 고정, render 핵심 컨트리뷰터 P-시리즈 일관, 충돌은 기계적 lockfile만.
- **옵션 B (수정 요청)**: 별도 결함/우려 발견 시. 현재 코드 품질·격리 우수하여 해당 없음 판단.
- **옵션 C (close)**: 본질 결함 시. 해당 없음.

## 6. 메모리 룰 정합

- `feedback_contributor_cycle_check` — @seo-rii 15 PR render 시리즈, P15→P16 연속 점검
- `feedback_hancom_compat_specific_over_general` — policy.ts 케이스별 명시 가드 (권위 사례)
- `feedback_small_batch_release_strategy` — opt-in·하위호환 신규 모듈 점진 분리
- `feedback_image_renderer_paths_separate` — 기존 경로 무변경, 별도 backend 추가
- `feedback_visual_judgment_authority` — 시각 판정 게이트 (작업지시자)

## 7. 권고

**옵션 A** 권고. 자기 검증(npm test/build + cargo test) 통과 + 작업지시자 시각 판정 통과 조건부 cherry-pick no-ff merge.
