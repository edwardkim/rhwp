# PR #419 처리 보고서 — PageLayerTree generation API 도입

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#419](https://github.com/edwardkim/rhwp/pull/419) |
| 작성자 | [@seo-rii](https://github.com/seo-rii) (Seohyun Lee) |
| 이슈 | [#364](https://github.com/edwardkim/rhwp/issues/364) (closes) |
| 처리 결정 | **cherry-pick 머지** (본질 8 commits 분리, merge commit 제외) |
| 처리 일자 | 2026-04-29 |

## 처리 절차

### Stage 0: 검토 + 컨트리뷰터 정황 확인

- 이슈 #364 작성자/assignee @postmelee, PR 작성자 @seo-rii — 두 컨트리뷰터 합의 기록을 메인테이너가 직접 확인 후 진행 결정
- @seo-rii 의 PR #165 (CLOSED, Skia renderer) 의 1단계 분할 재제출 정황
- 메인테이너 안내 7 항목 (이슈 #364) 정확 대응 확인

### Stage 1: cherry-pick

`local/pr419` 브랜치 (`local/devel` 분기) 에서 본질 8 commits cherry-pick — 작성자 attribution 보존 (Merge commit `3f28042` 제외):

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

### Stage 2: 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1062 passed** (+12 신규 paint 테스트, 회귀 0건) |
| `cargo test --lib paint::` | ✅ 8 passed |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 48s, 4,184,496 bytes (+67,567) |

### Stage 3: 광범위 byte 단위 무회귀 검증

10 샘플 / 309 페이지 SVG 비교 (devel baseline ↔ PR #419 legacy 경로):

**결과: 309/309 byte 단위 동일 (100%)** ✅

layer-svg opt-in 경로 (`RHWP_RENDER_PATH=layer-svg`) 도 legacy 와 byte 단위 동일 검증.

### Stage 4: 피델리티 영향 분석 (작업지시자 직접 요청)

별도 보고서 `mydocs/pr/pr_419_fidelity_analysis.md` 에 4 단계 증거 정리:

1. **정적 코드 분석**: 기존 5 렌더러 파일 (svg.rs, canvas.rs, web_canvas.rs, render_tree.rs, layout.rs) 본 PR 변경 **0 라인**. paint 모듈을 기존 렌더러 본체가 의존하지 않음.
2. **wasm_api.rs 본질 변경**: 251 → 252 메서드 (`get_page_layer_tree` 1개 추가). 나머지는 rustfmt.
3. **환경변수 가드**: layer-svg 경로는 `RHWP_RENDER_PATH=layer-svg` 일 때만 활성화. 기본 동작은 legacy 그대로.
4. **byte 단위 비교**: 10 샘플 309 페이지 SVG 모두 byte 단위 동일.

작업지시자 분석 보고서 확인 후 **머지 결정**.

## 변경 요약

### 신규 모듈 — paint (PageLayerTree generation API)

| 파일 | 역할 |
|------|------|
| `src/paint/builder.rs` (741) | LayerBuilder — PageRenderTree → PageLayerTree 변환 |
| `src/paint/json.rs` (994) | JSON 직렬화 (schemaVersion 1, unit "px", coordinateSystem "page-top-left") |
| `src/paint/layer_tree.rs` (182) | PageLayerTree, LayerNode 타입 |
| `src/paint/paint_op.rs` (80) | PaintOp (textRun, line, image, equation 등) |
| `src/paint/{mod,profile,resources}.rs` (35) | 보조 |

### 신규 transition adapter (opt-in)

| 파일 | 역할 |
|------|------|
| `src/renderer/svg_layer.rs` (335) | PageLayerTree → 임시 PageRenderTree → 기존 SvgRenderer (검증용) |
| `src/renderer/layer_renderer.rs` (9) | layer renderer 마커 trait |

### 기존 파일 최소 변경

- `src/lib.rs`: `pub mod paint;` 1 라인
- `src/renderer/mod.rs`: 모듈 노출 2 라인
- `src/document_core/queries/rendering.rs`: 신규 API +43 라인 (`build_page_layer_tree`, `get_page_layer_tree_native`)
- `src/wasm_api.rs`: `getPageLayerTree(pageNum)` 1 메서드 추가, 나머지 99% 가 rustfmt
- `src/renderer/layout/integration_tests.rs`: 테스트 보강 +93 / -24
- README.md / README_EN.md: 문서 갱신

### 기존 렌더러 본체 변경

**0 라인** (svg.rs, canvas.rs, web_canvas.rs, render_tree.rs, layout.rs).

## Schema 정책 (PR 본문)

- top-level: `schemaVersion: 1`, `resourceTableVersion: 1`, `unit: "px"`, `coordinateSystem: "page-top-left"`
- layer node kind: `group`, `clipRect`, `leaf`
- paint op type: `pageBackground`, `textRun`, `footnoteMarker`, `line`, `rectangle`, `ellipse`, `path`, `image`, `equation`, `formObject`, `placeholder`, `rawSvg`
- text: glyph run 대신 string + style + positions (한글/폰트 fallback/italic/형광펜 보존)
- shape/path: fill/stroke/pattern/gradient/opacity/shadow/lineStyle/connectorEndpoints 포함
- transform: 2D matrix 대신 rotation, horzFlip, vertFlip
- 변경 정책: additive 유지, 호환되지 않는 변경 시 schemaVersion 증가

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1062 + svg_snapshot 6/6 + clippy 0 + WASM + byte 단위 무회귀 |
| 시각 판정 게이트 (push 전 필수) | ✅ API 추가 패턴 (309/309 byte 동일) |
| output 폴더 가이드라인 | ✅ `output/svg/pr419-{regression-baseline,regression-test,legacy,layer}` |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr419` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |

## 다음 단계

1. 본 보고서 + 검토 + 분석 보고서 + 오늘할일 갱신 commit
2. `local/pr419` → `local/devel` → `devel` 머지 + push
3. PR #419 close + 작성자 댓글 (이슈 #364 자동 close)

## 참고

- 검토 문서: `mydocs/pr/pr_419_review.md`
- 피델리티 분석: `mydocs/pr/pr_419_fidelity_analysis.md`
- PR: [#419](https://github.com/edwardkim/rhwp/pull/419)
- 이슈: [#364](https://github.com/edwardkim/rhwp/issues/364)
- 작성자 origin PR: [#165](https://github.com/edwardkim/rhwp/pull/165) (CLOSED, 분할 재제출의 1단계)
