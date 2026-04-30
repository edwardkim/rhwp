# PR #456 검토 — Canvas rendering 을 PageLayerTree 경로로 전환 (P2)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#456](https://github.com/edwardkim/rhwp/pull/456) |
| 작성자 | [@seo-rii](https://github.com/seo-rii) (Seohyun Lee) — PR #419 (P1) 작성자, 본 사이클 2번째 PR |
| 이슈 | [#364](https://github.com/edwardkim/rhwp/issues/364), [#419](https://github.com/edwardkim/rhwp/pull/419), [#165](https://github.com/edwardkim/rhwp/pull/165) refs |
| base / head | `devel` ← `seo-rii:render-p2` |
| 변경 규모 | +5,495 / -608, 28 files (17 commits → 본질 6 commits) |
| mergeable | `CONFLICTING` (DIRTY) — 작성자 fork 의 PR #419 commits 누적 정황 |
| 검토 일자 | 2026-04-30 |

## 17 commits 누적 정황

| 영역 | commits | 처리 |
|------|---------|------|
| PR #419 commits (P1) | 8 commits | 이미 devel 흡수 (cherry-pick 으로 hash 다름) |
| Task #416 commits | 3 commits | 이미 devel 흡수 (메인테이너 작업) |
| **PR #456 본질 (P2)** | **6 commits** | **분리 cherry-pick 대상** |

## 본질 — PR #419 (P1) 의 후속 P2

PR #419 가 `PageRenderTree → PageLayerTree` 변환 API 추가 (P1, opt-in). 본 PR (#456) 은:
- **Canvas 렌더 경로를 PageLayerTree replay 로 전환** (P2, default)
- Canvas 가 legacy 경로 (직접 `PageRenderTree` 소비) 대신 **PageLayerTree 를 거쳐 replay**

### 변경 영역

```
src/renderer/canvas.rs           +432 / -18 (native Canvas + LayerReplay 경로)
src/renderer/web_canvas.rs       +192 / 0   (browser WASM Canvas + LayerReplay)
src/paint/builder.rs             +95 / -112 (LayerBuilder leaf children 보존 정정)
src/wasm_api.rs                  +50 / 0    (renderPageCanvas + renderPageCanvasLegacy 분리)
src/document_core/queries/rendering.rs +7 (Canvas 라우팅)
.github/workflows/ci.yml         +7 (Canvas layer parity test + WASM target check)
README.md / README_EN.md         +3 / -1 each
```

### 새 API 정황

| API | 경로 |
|-----|------|
| `renderPageCanvas` (default) | **PageLayerTree replay** |
| `renderPageToCanvas` (default) | **PageLayerTree replay** |
| `renderPageCanvasLegacy` | legacy `PageRenderTree` 직접 |
| `renderPageToCanvasLegacy` | legacy `PageRenderTree` 직접 |

### LayerBuilder 정정

`src/paint/builder.rs` 의 leaf node child subtree 보존 정정:
- Before: leaf payload 만 replay → child subtree 누락
- After: leaf 의 child subtree 도 보존하여 nested rendering 보장

## 처리 방향

**옵션 A — 본질 6 commits 분리 cherry-pick** (PR #419 와 같은 패턴).

다른 11 commits (PR #419 commits + Task #416 commits) 는 이미 devel 흡수 (cherry-pick 으로 hash 다름).

## dry-run cherry-pick 결과

`local/pr456` 브랜치 (`local/devel` 분기) — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `f1caa55` (← `5422222`) | @seo-rii | feat: replay layer trees through canvas recorder |
| `66e4e1b` (← `e5c3887`) | @seo-rii | feat: replay layer trees through web canvas |
| `4e56eb1` (← `afda14a`) | @seo-rii | feat: route public canvas through layer replay |
| `df02a3c` (← `873e038`) | @seo-rii | test: gate canvas layer parity in ci |
| `f9243ef` (← `9c4a511`) | @seo-rii | test: check wasm target in ci |
| `bbc3411` (← `f5708a7`) | @seo-rii | fix: preserve leaf children in layer lowering |

cherry-pick 결과: 충돌 없이 자동 적용.

## 검증 게이트 (cherry-pick 후)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1075 passed** (1070 → +5 신규 단위 테스트 — Canvas layer parity test) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 20s, 4,206,022 bytes (+19,741 from PR #450, paint 모듈 Canvas replay 추가) |

### 신규 단위 테스트 5건

`canvas_layer_tree_matches_legacy` — Canvas legacy 경로와 layer replay 경로 출력 비교 (parity test). PR 본문 명시 검증.

## 광범위 byte 단위 비교 — 매우 깔끔한 정황

10 샘플 / 305 페이지 SVG 비교 (devel ↔ PR #456):

| 결과 | 카운트 |
|------|------|
| byte 동일 | **305 / 305 (100%)** ✅ |
| 차이 발생 | **0** |

→ **SVG 출력 영향 0** (PR #419 와 같은 패턴 — paint 모듈 도입 / Canvas 경로 전환은 SVG legacy 경로에 0 영향).

PR 본문 명시:
> "기존 SVG 기본 출력은 여전히 legacy 경로를 사용합니다. 이 PR 에서 사용자-visible 하게 바뀌는 부분은 public Canvas API 가 PageLayerTree 를 거쳐 replay 된다는 점입니다."

→ 정확히 검증됨.

## 시각 판정 정황 (작업지시자 결정)

작업지시자 결정:
> "456 PR 까지 한 후 묶어서 시각 검증 하겠습니다."

**통합 시각 검증 진행** — PR #454 + #457 + #461 + #456 모두 머지 후 작업지시자 직접 진행.

본 PR 은 SVG byte 단위 동일 (0 차이) 이라 회귀 위험 매우 낮음. 통합 시각 검증의 핵심은:
1. **PR #454/#457/#461 의 누적 정정 통합 검증** (paragraph_layout / 글상자 / vpos-reset / 셀 leakage 등)
2. **본 PR (#456) 의 Canvas 경로 전환 검증** — rhwp-studio (WASM Canvas) 에서 default Canvas API 가 PageLayerTree replay 로 정상 동작하는지

## 본 PR 의 좋은 점

1. **PR #419 (P1) 의 정확한 후속 P2**: 외부 backend 도입 토대 + 검증 게이트 (Canvas parity test)
2. **legacy / replay 분리**: `renderPageCanvasLegacy` API 보존 → fallback / 비교 가능
3. **메인테이너 안내 7항목 (이슈 #364) 정확 대응**: 본 사이클 backend 추가 토대 완성
4. **Canvas parity test 추가**: legacy / replay 같은 출력 자동 검증 (CI 통합)
5. **SVG byte 단위 동일** (305/305): 기존 SVG 사용자에게 영향 0 — 위험 분산

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1075 + svg_snapshot 6/6 + clippy 0 + WASM + Canvas parity test |
| 시각 판정 게이트 (push 전 필수) | ⏸️ 통합 검증 (작업지시자 결정) |
| 한컴 호환은 일반화보다 케이스별 명시 가드 | (해당 없음 — 렌더링 경로 전환) |
| 작은 단위 PATCH 회전 | ✅ P1 / P2 분리 PR 정책 부합 |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr456` 에서 커밋 |

## 다음 단계

1. 본 보고서 commit
2. `local/pr456` → `local/devel` → `devel` 머지 + push
3. PR #456 close + 작성자 댓글 (#364, #419 후속 정합)
4. **통합 시각 검증 진행** (작업지시자 직접) — PR #454 + #457 + #461 + #456 누적 정정 + Canvas 경로 전환

## 참고

- PR: [#456](https://github.com/edwardkim/rhwp/pull/456)
- 직전 PR (P1): [#419](https://github.com/edwardkim/rhwp/pull/419)
- 본 사이클 누적 정정: [#454](https://github.com/edwardkim/rhwp/pull/454), [#457](https://github.com/edwardkim/rhwp/pull/457), [#461](https://github.com/edwardkim/rhwp/pull/461)
- 시리즈 origin: [#165](https://github.com/edwardkim/rhwp/pull/165) (CLOSED, Skia renderer)
- 향후 P3: pixel diff / visual regression e2e (별도 PR)
