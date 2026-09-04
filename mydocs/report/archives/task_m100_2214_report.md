# Task M100 #2214 최종 결과보고서 — 거대 셀 연속 입력 표시·cursor fallback 정정

## 1. 최종 판정

#2214는 계획한 범위에서 해결됐다.

- 115쪽 거대 셀 문서의 44번째 줄 경계 입력이 Enter 없이 즉시 표시된다.
- HWP/HWPX 모두 모델, page tree, cursor, pagination cut·bounds와 Canvas가 일치한다.
- 줄 내부 stable 입력은 전체 pagination 0회를 유지하고 약 28.5~29.2ms p95로 동작한다.
- 실제 cell-flow 경계만 cursor 조회 전에 full pagination을 정확히 1회 수행한다.
- 기존 약 2초 cursor fallback은 scoped layout-cache coherence로 제거됐다.
- #2195 이후 별도 렌더 복사본이 활성화된 문서에서도 편집된 normalized cell paragraph와
  해당 cache만 동기화해 동일한 exact tree/cursor 계약을 유지한다.
- 저장·재로드와 #2185 한글 줄 나눔, #1949/#2063 대형 문서 회귀를 유지한다.
- 영구 브라우저 회귀는 npm 진입점과 CI syntax gate를 가지며, cache/pagination/history 계약은
  troubleshooting·tech·manual 문서에 재사용 가능한 형태로 남겼다.

정확성 우선 경계 flush의 약 0.9초 비용은 의도적으로 남겼다. 이를 bounded/partial paginator로
대체하는 작업은 #2193 종합 성능의 후속 범위다.

## 2. 재현 증상과 확정 원인

재현 문서의 첫 페이지 nested table cell 문단 끝에서 `1`을 반복 입력하면 44번째 입력에서
4줄이 5줄로 바뀐다. 기존 구현은 모델 text와 `LINE_SEG`를 174자까지 갱신했지만 warm page
tree가 129자에서 끝났다. 새 cursor offset을 찾지 못한 exact-search는 115쪽 전체를
fallback scan하면서 약 1.96~2.02초를 소비했고, 화면에는 새 문자와 캐럿이 즉시 나타나지
않았다.

직접 원인은 한컴 line-break semantic이나 font matrix 차이가 아니라 deferred cell edit 뒤
렌더 파생 상태가 원본 편집 상태를 따라가지 못한 cache coherence 누락이었다. 최초 구현 시점에는
`LayoutEngine::cell_units_cache`의 편집 셀 항목이 문제였고, #2195 병합 뒤에는 pagination에서
복제한 `render_normalized`의 text와 pointer-key cache가 함께 stale해지는 두 번째 층이 드러났다.

```text
deferred text mutation
→ 모델·LINE_SEG 최신
→ section/page-tree invalidation
→ warm cell_units_cache 잔존
→ stale page tree(129)와 최신 cursor offset(174) 공존
→ 115쪽 exact-miss fallback
```

pagination 없이 layout cache만 비우면 tree와 cursor는 즉시 174자로 복구됐다. #2195 이전
기준에서는 full pagination 뒤 page 0이 cut 38·bounds 971.5로 바뀌었지만, #2195 이후 선언 셀
높이가 증가분을 흡수해 deferred/flush 모두 cut 37·bounds 945.9다. 다만 page 2~114의
continuation cut 113개는 flush 뒤 재정렬된다. 따라서 문제는 두 층으로 분리해야 했다.

1. 매 stable 입력: 편집 cell/table 범위의 layout-cache coherence
2. 실제 flow advance 경계: pagination geometry를 확정하는 pre-cursor 1회 flush

Canvas static reuse, 800ms verification, batch/sequential 입력 방식과 cursor API 종류는 직접
원인에서 기각됐다.

## 3. 구현 내용

### 3.1 scoped cell-units cache coherence

deferred insert 전후의 nested-text local contribution과 cached owner-table flag를 비교한다.

- owner flag가 불변이면 edited cell cache만 제거한다.
- local false→true여도 cached owner flag가 이미 true면 edited cell만 제거한다.
- cached false→true일 때만 owner table의 직접 cell cache를 제거하고 flag를 true로 갱신한다.
- unrelated table과 nested table cache는 보존한다.

stable/cached-true 경로는 O(1)이고 table predicate 재스캔을 만들지 않는다. generic partial
invalidation에 global `clear_layout_caches()`를 추가하지 않았다.

### 3.2 원자적 `cellFlowChanged`

deferred insert는 target paragraph의 편집 전후 상대 flow advance를 비교해 additive result를
반환한다.

```text
last.vpos + line_height + line_spacing - first.vpos
```

승인된 HWP/HWPX 재현 픽스처에서 1~43번째와 45~50번째 입력은 `false`, 44번째
`+1920HU` 경계만 `true`다. line count 자체가 아니라 이 target paragraph에서 관찰한 상대
trailing advance를 신호로 사용한다.

### 3.3 Studio pre-cursor 경계 flush

WASM bridge, command/history effect와 일반·IME·iOS 입력 router가 mutation effect를 한 번만
전달·소비한다.

```text
WASM mutation
→ deferred pending 등록
→ cellFlowChanged=true일 때 full flush 1회
→ exact cursor lookup
→ 표시 무효화
```

stable deferred mutation은 page-local redraw를 유지한다. immediate mutation은 이전 pending을
정리하고 correctness 우선 full 표시 무효화로 보낸다. history merge는 현재 실행 effect를
소비하며 redo는 실제 mutation 결과를 다시 계산한다. 문서 전환은 pending, raw/IME/iOS state와
timer를 초기화한다.

### 3.4 normalized render-state coherence

#2195가 비-TAC 중첩 표 stretch를 `render_normalized` 복사본에 적용한 뒤에도 deferred edit은
pagination을 실행하지 않는다. 따라서 편집된 원본 cell paragraph를 동일 normalized path에만
복사하고, normalized edited cell에 기존 owner-flag scoped invalidation을 적용한다. 상위 문단·표·
셀 주소와 unrelated/sibling cache는 유지하며, 교체된 문단 내부 중첩 표에 #2195 stretch를 다시
적용한다. 전역 cache clear나 매 입력 normalized section 재복제는 추가하지 않았다.

## 4. 정확성 결과

### 4.1 native 구조

| 상태 | model/tree max | page 0 cut | bounds h | page count |
|------|---------------:|-----------:|---------:|-----------:|
| scoped deferred transient | 174 | 37 | 945.9 | 115 |
| 경계 full flush | 174 | 37 | 945.9 | 115 |

HWP/HWPX 모두 115개 `PartialTable` fragment의 cut chain이 gap/overlap 없이 이어졌고 exact
cell-path cursor를 반환했다. page 0의 cut/bounds는 같지만 page 2~114의 continuation cut
113개가 full pagination 뒤 재정렬됐다.

### 4.2 브라우저 화면

HWP/HWPX 각 3회에서 다음 계약이 반복 통과했다.

- 1~43번째: 4줄, bounds 945.9, flush 0
- 44번째: mutation → flush → cursor, 누적 flush 1, 5줄, bounds 945.9
- 45~50번째: 추가 flush 0, 최종 180자, 115쪽
- 43→44 합성 crop: 10,074 pixel 변화
- 44번째 뒤 2 rAF·100ms·850ms·1.6초 crop: exact SHA 동일, changed pixel 0
- IME/iOS raw stable: flush 0, flow 경계: flush 1

### 4.3 저장·기존 회귀

- #2185: HWP/HWPX 115쪽, 한 글자 `[0,44,84,122]`, `vpos=17160`, 저장·재로드 유지
- #1949: 거대 nested table 렌더 성능 회귀 통과
- #2063: 관련 대형 문서 회귀 통과

## 5. 성능 결과와 연관성

줄 경계 표시 소실과 한 글자 입력의 약 2초 지연은 같은 cache coherence 누락에서 갈라진
증상이었다. stale tree가 새 offset을 포함하지 않아 cursor fallback scan을 유발했다.

Stage 3 native prewarm seq44 direct query는 다음처럼 감소했다.

| 형식 | 수정 전 | 수정 후 | 단축 |
|------|--------:|--------:|-----:|
| HWP | 1,960.76ms | 27.78ms | 약 70.6배 |
| HWPX | 2,016.54ms | 27.00ms | 약 74.7배 |

Stage 5 실제 Studio 6회 관찰값은 다음 범위였다.

| 구간 | 관찰 범위 |
|------|-----------|
| stable operation p95 | 28.5~29.2ms |
| stable keyboard p95 | 47.1~48.3ms |
| boundary operation | 945.3~973.3ms |
| boundary full flush | 898.9~927.2ms |

즉 일반 입력의 2초 지연은 해결됐지만, 44번째 실제 flow 경계에는 정확성을 위해 약 0.9초
full pagination이 남는다. 시간은 환경 의존 관찰값이고, 영구 hard gate는 flush 횟수·순서,
exact state, 115쪽과 pixel 안정성이다.

## 6. 최종 검증

| 영역 | 결과 |
|------|------|
| Rust 전체 | 최신 devel release-test 모든 test binary 실패 0 |
| #2214 native | 3/3 passed + crate-internal structured GREEN |
| 30-case matrix | 1 passed, diagnostic completion, 83.87초 |
| Clippy | all targets/features, `-D warnings`, 경고 0 |
| Rustfmt | `cargo fmt --check` 통과, 수정 없음 |
| Studio unit | 303 passed / 0 failed |
| Studio build | 통과 |
| renderer contract | 통과 |
| focused browser | HWP/HWPX 3회씩 6/6 GREEN, raw 8/8 GREEN |
| Stage 4 source/WASM sameness | 구현 source 9개 + package 3개 hash 동일 |
| Stage 6 npm browser smoke | HWP/HWPX 1회씩 2/2 GREEN, raw 8/8 GREEN |
| Stage 6 Studio | 214 passed, build·renderer contract·E2E syntax 통과 |

30-case matrix는 case별 기대값을 아직 hard assertion으로 만들지 않았으므로 correctness
GREEN으로 과장하지 않고 진단 완료로만 기록한다. 최종 필수 계약은 non-ignored native/Studio
테스트와 기본 focused browser runner에 있다.

## 7. 변경 범위

### Rust

- `src/renderer/layout.rs`
- `src/renderer/layout/table_layout.rs`
- `src/document_core/commands/text_editing.rs`
- `src/wasm_api.rs`
- `src/wasm_api/tests.rs`
- `tests/issue_2214_page_local_repaint.rs`
- `tests/issue_2214_cache_matrix_probe.rs`

### Studio

- `rhwp-studio/src/core/wasm-bridge.ts`
- `rhwp-studio/src/engine/command.ts`
- `rhwp-studio/src/engine/history.ts`
- `rhwp-studio/src/engine/input-handler.ts`
- `rhwp-studio/src/engine/input-handler-text.ts`
- `rhwp-studio/tests/cell-flow-boundary.test.ts`
- `rhwp-studio/tests/input-edit-invalidation.test.ts`
- `rhwp-studio/e2e/issue-2214-page-local-repaint.test.mjs`

### 재발 방지·실행 진입점

- `rhwp-studio/package.json`
- `.github/workflows/render-diff.yml`
- `mydocs/troubleshootings/deferred_cell_edit_cache_coherence.md`
- `mydocs/tech/edit_action_undo_redo_architecture.md`
- `mydocs/manual/edit_command_review_checklist.md`

parser/serializer, font metric, 한컴 line-break semantic, paginator와 Canvas production renderer는
변경하지 않았다.

## 8. Stage 이력

| Stage | commit | 결과 |
|-------|--------|------|
| 1 | `f0cb99f0` | 실제 앱 시간축 재현과 최초 stale 경계 계측 |
| 계획 보정 | `572fa256` | 최신 devel 교차검증과 warm cache 원인 반영 |
| 2 | `7272115c` | warm cache RED와 flow 전환 진단 계약 |
| 3 | `14d31e0e` | scoped cache coherence와 `cellFlowChanged` GREEN |
| 4 | `8efd562f` | pre-cursor 경계 flush와 Studio 정합 |
| upstream 통합 | `d9da3b0b` | `upstream/devel@3c1cba96` 문서 전용 변경 병합 |
| 5 | `f0596ded` | 광역 게이트와 최종 보고 |
| 6 | Stage 6 최종 커밋 | E2E npm/CI 발견성과 재발 방지 문서 보강 |
| 7 | `de715cf4` | #2195 이후 normalized-state scoped coherence 보완 |
| 최신 devel 통합 | `51da6ee3` | `upstream/devel@6cfc4cec` 병합 |
| 8 | Stage 8 최종 커밋 | cursor/E2E 기준선 보정과 전체 CI 동등 검증 |

## 9. 후속 범위

- **#2193 종합 성능**: pagination, page tree와 Canvas 갱신 비용을 계속 분해한다.
- **bounded/partial paginator**: flow 경계의 약 0.9초 full pagination을 영향 범위 기반으로
  대체하는 별도 설계·정확성 작업이 필요하다.
- **normalized derived state 재설계**: mutable clone에 경로별 편집을 mirror하는 구조를
  revision 기반 derived cache 또는 overlay로 전환하는 별도 이슈가 필요하다.
- **#2215**: 드래그 selection은 별도 이슈다.
- 실제 iOS 기기의 contentEditable·가상 키보드·포커스 회귀가 남는다.
- boundary flush 실패 시 pending retry와 사용자 오류 UX는 별도 계약이 필요하다.
- 총 aggregate advance는 같지만 ordered per-line geometry가 달라지는 혼합 서식 문단은 이번
  픽스처에서 검증하지 않았다. 관련 재현이 확인되면 line/cell-unit geometry signature 또는
  보수적 dirty 신호를 별도 범위에서 설계한다.

이 작업은 한컴 line-break semantic의 완전 복제나 renderer 전면 재작성 없이, 확인된 cache
coherence와 flow-boundary 계약만 수정했다. 이슈 close는 PR #2241 merge 뒤 작업지시자 승인에
따라 별도로 수행한다.

로컬 진단 JSON/PNG/timeline과 WASM/build cache는 실행별 환경 의존 증거이므로 PR에 포함하지
않는다. 구조 기대값은 코드 assertion에, 픽셀 안정성은 재생성 가능한 E2E의 동일 실행 비교에 둔다.
