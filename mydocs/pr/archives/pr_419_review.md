# PR #419 검토 — PageLayerTree generation API 도입

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#419](https://github.com/edwardkim/rhwp/pull/419) |
| 작성자 | [@seo-rii](https://github.com/seo-rii) (Seohyun Lee) — **신규 외부 컨트리뷰터** |
| 이슈 | [#364](https://github.com/edwardkim/rhwp/issues/364) (closes), 작성자 본인 PR #165 (CLOSED) 의 분할 재제출 |
| base / head | `devel` ← `seo-rii:render-p1` |
| 변경 규모 | +3,837 / -584, 16 files (9 commits, merge commit 1 제외 본질 8) |
| mergeable | `MERGEABLE` (BEHIND) |
| 검토 일자 | 2026-04-29 |

## 컨트리뷰터 정황

이슈 #364 작성자/assignee 는 @postmelee 이나 PR 작성자는 @seo-rii — **두 컨트리뷰터 합의 기록을 메인테이너가 확인** (작업지시자 직접 확인). PR #165 (CLOSED, "feat: introduce skia renderer") 작성자도 @seo-rii 로, 본 PR 은 PR #165 의 1단계 분할 재제출. `closes #364` 는 같은 방향 후속 정합 정황.

## 본질

이슈 #364 의 메인테이너 안내 7 항목 정확 대응:

| # | 안내 | 작성자 대응 |
|---|------|-----------|
| 1 | schema outline | ✅ PR 본문 "Schema outline" — kind / paint op type / text / shape 모두 명시 |
| 2 | schemaVersion 정책 | ✅ "additive 유지, 호환되지 않게 바뀌면 schemaVersion 올림" |
| 3 | 변환 정확성 보존 | ✅ paint::builder 단위 테스트 (preserves_structural_groups_and_clips_for_backend_replay 등) |
| 4 | PR 범위 좁히기 | ✅ "이번 PR에서 하지 않는 것" 명시 (Skia/CanvasKit/ThorVG/C ABI/Canvas 전환/parity 보장 모두 제외) |
| 5 | surface 우선순위 | ✅ Rust native + WASM 만 (C ABI 후속) |
| 6 | 회귀 테스트 | ✅ paint 단위 테스트 8건 + 광범위 byte 단위 무회귀 검증 |
| 7 | enhancement 라벨 | (메인테이너 처리, 머지 시 자동 close) |

## 변경 파일

### 신규 추가 (paint 모듈 + transition adapter)

| 파일 | 라인 수 | 역할 |
|------|--------|------|
| `src/paint/builder.rs` | 741 | LayerBuilder (PageRenderTree → PageLayerTree 변환) |
| `src/paint/json.rs` | 994 | JSON 직렬화 (schemaVersion 1, unit "px", coordinateSystem "page-top-left") |
| `src/paint/layer_tree.rs` | 182 | PageLayerTree, LayerNode 타입 |
| `src/paint/mod.rs` | 20 | 모듈 노출 |
| `src/paint/paint_op.rs` | 80 | PaintOp 타입 (textRun, line, image, equation 등) |
| `src/paint/profile.rs` | 9 | RenderProfile |
| `src/paint/resources.rs` | 6 | ResourceArena |
| `src/renderer/layer_renderer.rs` | 9 | layer renderer 마커 trait |
| `src/renderer/svg_layer.rs` | 335 | **opt-in transition adapter** (PageLayerTree → 임시 PageRenderTree → 기존 SvgRenderer) |

### 기존 파일 수정 (최소 변경)

| 파일 | 변경 | 정황 |
|------|------|-----|
| `src/lib.rs` | +1 | `pub mod paint;` |
| `src/renderer/mod.rs` | +2 | `pub mod layer_renderer; pub mod svg_layer;` |
| `src/document_core/queries/rendering.rs` | +43 | `build_page_layer_tree`, `get_page_layer_tree_native` |
| `src/wasm_api.rs` | +1290 / -554 | **본질은 `get_page_layer_tree` 1개만 추가**, 나머지 99% 가 rustfmt |
| `src/renderer/layout/integration_tests.rs` | +93 / -24 | 테스트 보강 |
| `README.md` / `README_EN.md` | +14 / -1 각 | 문서 갱신 |

### 기존 렌더러 본체 변경 (피델리티 핵심)

| 파일 | 본 PR 변경 |
|------|----------|
| `src/renderer/svg.rs` | **0 라인** |
| `src/renderer/canvas.rs` | **0 라인** |
| `src/renderer/web_canvas.rs` | **0 라인** |
| `src/renderer/render_tree.rs` | **0 라인** |
| `src/renderer/layout.rs` | **0 라인** |

## 처리 방향

**옵션 A — 본질 8 commits 분리 cherry-pick** (작성자 attribution 보존, PR #406/#408/#410 와 같은 패턴).

PR 의 9 commits 중 `3f28042` (Merge devel into render-p1) 만 제외하고 나머지 8 commits cherry-pick.

## dry-run cherry-pick 결과

`local/pr419` 브랜치 (`local/devel` 분기) 에서 8 commits cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `6f973d6` (← `51ce84f`) | @seo-rii | feat: add layered paint tree scaffolding |
| `fc3572e` (← `194ee10`) | @seo-rii | feat: add layered svg replay path |
| `cd523f4` (← `575b0da`) | @seo-rii | feat: expose layer tree json for browser backends |
| `646cb33` (← `ac68c30`) | @seo-rii | docs: document multi-renderer backends |
| `0e17be0` (← `42dc3db`) | @seo-rii | fix: tighten layer tree replay contract |
| `f4d2385` (← `0647802`) | @seo-rii | fix: preserve layer node metadata in json |
| `a8749f3` (← `11f2ac8`) | @seo-rii | fix: align layer tree api with issue 364 |
| `a27896b` (← `e2015fe`) | @seo-rii | fix: carry layer output options |

cherry-pick 결과: 충돌 없이 자동 적용.

## 검증 (cherry-pick 후)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1062 passed** (1050 → +12 신규 paint 모듈 테스트) |
| `cargo test --lib paint::` | ✅ 8 passed (paint::builder + paint::json) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 48s, 4,184,496 bytes (+67,567, +1.64%) |

## 광범위 byte 단위 무회귀 검증

10 샘플 / 309 페이지 SVG 비교 (devel baseline ↔ PR #419 legacy 경로):

| 샘플 | 페이지 |
|------|------|
| aift.hwp | 77 |
| biz_plan.hwp | 6 |
| exam_kor.hwp | 24 |
| exam_math.hwp | 20 |
| k-water-rfp.hwp | 28 |
| kps-ai.hwp | 80 |
| 2025년 기부·답례품 실적 지자체 보고서_양식.hwpx | 30 |
| synam-001.hwp | 35 |
| equation-lim.hwp | 1 |
| exam_eng.hwp | 8 |
| **합계** | **309** |

**결과: 309/309 byte 단위 동일 (100%)** ✅

추가로 layer-svg opt-in 경로 (`RHWP_RENDER_PATH=layer-svg`) 도 legacy 와 byte 단위 동일 검증 (대표 샘플 2건).

## 시각 판정 정황

본 PR 은 **API 추가 (변환 경로) + opt-in 검증 adapter** 패턴. 화면 출력 변화 없음 — 309/309 byte 단위 동일이 결정적 검증.

→ **시각 판정 불필요** (PR #405, #411, #400 와 같은 패턴 — API 노출, 화면 변화 없음).

## 피델리티 영향 분석

별도 보고서 [mydocs/pr/pr_419_fidelity_analysis.md](mydocs/pr/pr_419_fidelity_analysis.md) 에 4 단계 증거 정리:

1. 정적 코드 분석 — 기존 5 렌더러 파일 변경 0 라인, paint 미의존
2. wasm_api.rs 본질 변경 — 메서드 수 251 → 252 (1개만 신규)
3. 환경변수 가드 — layer-svg 는 `RHWP_RENDER_PATH=layer-svg` 일 때만 활성화
4. byte 단위 비교 — 309/309 동일

작업지시자 분석 보고서 확인 후 머지 결정.

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1062 + svg_snapshot 6/6 + clippy 0 + WASM + byte 단위 무회귀 |
| 시각 판정 게이트 (push 전 필수) | ✅ API 추가 패턴이라 시각 판정 불필요 (309/309 byte 동일) |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr419` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |
| PR 댓글 톤 | ✅ |

## 다음 단계

1. 본 보고서 + 분석 보고서 + 오늘할일 갱신 commit
2. `local/pr419` → `local/devel` → `devel` 머지 + push
3. PR #419 close + 작성자 댓글 (이슈 #364 자동 close)

## 참고

- PR: [#419](https://github.com/edwardkim/rhwp/pull/419)
- 이슈: [#364](https://github.com/edwardkim/rhwp/issues/364)
- 작성자 origin PR: [#165](https://github.com/edwardkim/rhwp/pull/165) (CLOSED, Skia renderer)
- 피델리티 분석: `mydocs/pr/pr_419_fidelity_analysis.md`
