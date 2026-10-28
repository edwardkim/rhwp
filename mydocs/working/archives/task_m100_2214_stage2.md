# Task M100 #2214 Stage 2 완료보고서 — warm cache 및 cell-flow 회귀 계약 고정

## 0. 판정 요약

- **Stage 판정**: 완료
- **production 변경**: 없음
- **GREEN 계약**:
  - HWP/HWPX 28자 입력은 target paragraph 상대 flow advance 불변
  - 순차 입력 1~43번째는 `0HU`, 44번째만 `+1920HU`, 45~50번째는 다시 `0HU`
  - pagination 없는 layout-cache clear만으로 tree/cursor는 174까지 복구
  - #2185 한 글자 기준선과 Studio 206개 단위 테스트 유지
- **의도된 RED 계약**:
  - warm deferred 44자 뒤 model 174 / tree 129 / cursor 첫 줄 fallback
  - 실제 flag-invariant deferred insert가 target cell을 evict하지 않아 membership `(true,true,true)`
  - 실제 empty nested-host insert가 owner cells를 evict하거나 cached flag를 true로 갱신하지 않음
  - cached owner flag=true인 local false→true branch에서 global clear가 owner sibling을 제거
  - global clear가 flag 불변 branch의 sibling/unrelated cache를 모두 제거
  - owner flag 변화 branch에서도 global clear가 unrelated cache를 제거
- **경계 정확성 판단**: stale/cache-only/full-flush 세 상태 모두 115쪽 structured cut chain이
  gap/overlap 없이 이어진다. cache-only는 chain을 바꾸지 않고, full flush 뒤 HWP/HWPX 모두
  0~114쪽 fragment가 바뀐다. 따라서 즉시 tree/cursor coherence와 flow-boundary geometry
  확정을 분리해야 한다.
- **다음 단계**: Stage 3 scoped cell/table cache eviction과 `cellFlowChanged` Rust result
  구현 승인 대기

## 1. 기준 환경

| 항목 | 값 |
|------|----|
| 작업 브랜치 | `issue-2214-page-local-repaint` |
| 최신 devel | `upstream/devel@c7864c62` |
| merge commit | `e1bb4ab7` |
| 보존된 Stage 1 | `f0cb99f0` |
| 보존된 계획 보정 | `572fa256` |
| worktree | `/private/tmp/rhwp-task2214` |
| WASM | 6,662,474 bytes |
| WASM SHA-256 | `41d675bebe3c981903ef7c0ab67b0e38393c379a215f12693901d57e73f2cb92` |
| Studio | Vite 8.1.4, Chrome headless, viewport 1280×900, DPR 1 |

최신 devel은 rebase하지 않고 merge했다. 이로써 두 기존 커밋 SHA와 승인 이력을 그대로
보존했다. 예상 중복 파일은 `mydocs/orders/20260712.md` 하나였으며, upstream 당일 기록 전체에
#2214 당시 행만 합쳐 보존했다.

## 2. Stage 2 변경 범위

| 파일 | 역할 | production 영향 |
|------|------|-----------------|
| `tests/issue_2214_page_local_repaint.rs` | warm public-behavior RED와 1~50 flow 기준선 | 없음 |
| `tests/issue_2214_cache_matrix_probe.rs` | HWP/HWPX 30-case cold/warm 진단 | 없음 (`#[ignore]`) |
| `src/wasm_api/tests.rs` | cache-only/full-flush 및 115쪽 structured cut-chain 격리 | 없음 (`#[cfg(test)]`, `#[ignore]`) |
| `src/renderer/layout.rs` | table-wide flag 실제 scan 횟수 test-only counter | 없음 (`#[cfg(test)]`) |
| `src/renderer/layout/table_layout.rs` | 실제 deferred 호출부와 flag 불변/변화 cache scope RED | 없음 (`#[cfg(test)]`, `#[ignore]`) |
| `rhwp-studio/e2e/issue-2214-page-local-repaint.test.mjs` | Stage 2 출력 격리, 원인 라벨 정정 | 진단 runner만 변경 |

`invalidate_page_tree_cache_from`, `clear_layout_caches`, deferred insert와 Studio input pipeline의
production 코드는 수정하지 않았다.

## 3. native 계약

### 3.1 cell-flow 기준선 — GREEN

`issue_2214_cell_flow_transition_baseline`은 HWP/HWPX에서 통과했다.

| 입력 | line starts | key별 상대 flow 변화 | 향후 신호 |
|------|-------------|----------------------|-----------|
| 28자 batch | `[0,44,84,122]` | `0HU` | `cellFlowChanged=false` |
| sequential 1~43 | 44번째 직전까지 4줄 | 매 key `0HU` | 매 key `false` |
| sequential 44 | `[0,44,84,122,129]` | `+1920HU` | `true` |
| sequential 45~50 | 같은 5줄 | 매 key `0HU` | 매 key `false` |

따라서 line count나 line-start 변화 자체가 아니라 target paragraph의 상대 flow advance를
pagination 경계 신호로 사용한다.

### 3.2 warm public behavior — 의도된 RED

실제 Studio 순서처럼 page 0을 warm하고 44자를 deferred 입력한 뒤 path-near를 첫 observer로
호출했다.

| 형식 | model | tree max | cursor | bounds h | 판정 |
|------|------:|---------:|--------|---------:|------|
| HWP | 174 | 129 | page 0 `(84.1,238.7)` fallback | 945.9 | RED |
| HWPX | 174 | 129 | page 0 `(84.1,238.7)` fallback | 945.9 | RED |

desired assertion은 tree max 174, path-near `(569.7,341.1)`, height 16,
`cellOverflowed=false`다. 테스트는 두 형식의 실제값을 모두 수집한 뒤 exit 101로 실패한다.
그 전에 page count 115, 전 115쪽 pagination fragment 불변, page 0 `end_cut=[37]`,
`cellBounds.h=945.9`를 단언한다. 따라서 잘못된 즉시 pagination이 tree/cursor만 맞춰도
false-green이 되지 않는다. Stage 3 scoped eviction 뒤 같은 assertion을 변경하지 않고
GREEN으로 전환한다.

### 3.3 cache-only 원인 격리 — GREEN

crate-internal probe는 warm 44자 뒤 pagination 없이 `invalidate_page_tree_cache()`를 호출해
page/layer/LayoutEngine cache만 비웠다.

| 상태 | tree max | direct cursor | cut | bounds h |
|------|---------:|---------------|----:|---------:|
| warm stale | 129 | `(84.1,238.7)` | 37 | 945.9 |
| cache clear only | 174 | `(569.7,341.9)` | 37 | 945.9 |
| full flush | 174 | `(569.7,341.9)` | 38 | 971.5 |

HWP/HWPX가 동일했다. debug 문자열 대신 crate-internal `PaginationResult`의
`PageItem::PartialTable`을 구조적으로 읽었다. 각 상태에서 page별 target fragment가 정확히
1개였고, split fragment의 `end_cut`은 다음 page의 `start_cut`과 같았으며 row 범위·cut
component도 역행하지 않았다. cache-only 전후 115쪽 구조는 완전히 같았고, full flush 뒤에는
두 형식 모두 **115/115쪽 전체 구조가 변경**됐지만 새 chain의 연속성은 유지됐다.

별도로 115쪽 render tree 전체에서 exact cell path의 UTF-16 TextRun 범위를 모았다. stale은
`0..129`, cache-only와 full flush는 모두 `0..174`를 gap/overlap 없이 이었고 cache-only range
목록은 flush oracle과 완전히 같았다. 따라서 cut-unit 연속성과 실제 target text range
연속성을 서로 대신하지 않고 각각 검증했다.

이 결과는 다음 두 계약을 동시에 확정한다.

1. warm `cell_units_cache`가 visible tree/cursor 결함의 직접 원인이다.
2. flow 경계에서 bounds와 115쪽 pagination geometry를 확정하려면 full flush가 필요하다.

### 3.4 실제 deferred 호출부 scope — 의도된 RED

실제 HWP/HWPX fixture에서 target cell과 같은 표 sibling을 `cell_units()`로 warm했다. flow가
바뀌지 않는 첫 한 글자와 43자를 미리 입력한 뒤의 44번째 한 글자를 각각 실제
`insert_text_in_cell_native_deferred_pagination()`으로 삽입했다. table, target cell, sibling cell
포인터는 모두 안정적이었고 owner table-wide flag도 pre/post `false`로 불변이었다.

membership 순서는 `(target cell, sibling cell, owner table flag)`다.

| 구분 | 값 |
|------|----|
| 현재 deferred call site (stable/44번째 공통) | `(true,true,true)`, target 재계산 false, table scan 0회 |
| desired invariant-flag contract | `(false,true,true)`, target 재계산 true, table scan 0회 |

현재 경로는 page-tree만 무효화하고 layout cache eviction을 호출하지 않는다. desired 계약은
edited cell만 evict하고 cached owner flag와 sibling을 보존한다. 이는 no-op, global clear,
owner-flag eviction 및 owner-table 전체 clear를 모두 거부한다. 추가로 target `Arc` 재계산과
sibling `Arc` identity 재사용, mutation부터 재warm까지 table-wide scan 0회를 단언한다.

### 3.5 실제 flag-changing deferred 호출부 — 의도된 RED

같은 공개 fixture의 owner table-wide flag는 `false`이며, 빈 텍스트와 nested table control을
가진 host paragraph가 존재한다. 해당 host에 실제 deferred insert로 `x`를 넣으면 local
contribution이 false→true가 되고 owner flag도 true가 되어야 한다. 모든 owner cells와 host의
nested table cell을 명시적으로 warm해 다음 계약을 둔다.

```text
(owner cell 중 남은 entry 존재, nested cell entry, owner flag cached value, nested flag entry)
```

| 구분 | 값 |
|------|----|
| 현재 deferred call site | `(true,true,Some(false),true)`, owner 재계산 false, table scan 0회 |
| desired flag-changing contract | `(false,true,Some(true),true)`, owner 재계산 true, table scan 0회 |

desired branch는 owner table의 모든 cell units를 evict하고 owner flag cache를 true로 갱신하되,
내용이 바뀌지 않은 nested table cache identity는 보존한다. full-table rescan은 사용하지 않고
edited paragraph의 local contribution 변화와 기존 cached flag로 판정해야 한다.

### 3.6 cached-true local change — 의도된 RED

한 host가 이미 visible text+nested table을 가져 cached owner flag가 `true`인 상태에서, 두 번째
empty nested host에 텍스트를 넣는 synthetic branch를 고정했다. local contribution은
false→true지만 table-wide 값은 true→true이므로 owner-wide eviction은 과잉이다.

```text
(edited cell, 다른 owner cells 전부, unrelated cell, owner flag value, unrelated flag)
```

| 구분 | 값 |
|------|----|
| 현재 global clear | `(false,false,false,None,false)`, table scan 2회 |
| desired cached-true contract | `(false,true,true,Some(true),true)`, table scan 0회 |

desired branch는 edited cell `Arc`만 재계산하고 다른 owner cells, owner flag와 unrelated cache
identity를 모두 보존한다. 이 RED는 local false→true만 보고 무조건 owner-wide clear하는
과잉 구현을 거부한다.

### 3.7 global clear의 두 scope branch — 의도된 RED

두 cell을 가진 owner table과 별도 unrelated table을 warm한 뒤 현재 global
`clear_layout_caches()`를 실행했다. 첫 fixture는 owner table-wide flag가 불변이다.

membership 순서는 다음과 같다.

```text
(edited cell, sibling cell, unrelated cell, owner table flag, unrelated table flag)
```

| 구분 | 값 |
|------|----|
| 현재 global clear | `(false,false,false,false,false)` |
| desired flag-invariant contract | `(false,true,true,true,true)` |

즉 편집 cell만 제거하고 owner flag, 같은 표 sibling cell, unrelated cell/table cache는
보존해야 한다.

두 번째 fixture는 multi-row RowBreak table의 빈 nested-table host에 텍스트를 넣어 owner flag를
`false→true`로 바꾼다. 이 flag는 모든 owner cell-unit 계산의 입력이므로 sibling 보존이
안전하지 않다. owner의 4개 cell을 모두 warm한 뒤 다음 순서로 판정한다.

```text
(owner cell 중 남은 entry 존재, unrelated cell entry, owner flag cached value, unrelated flag entry)
```

| 구분 | 값 |
|------|----|
| 현재 global clear | `(false,false,None,false)` |
| desired flag-changing contract | `(false,true,Some(true),true)` |

flag 변화 branch에서는 owner table의 **모든** cell units를 제거하고 owner flag를 true로
갱신하되 unrelated cell/table cache는 보존한다. 두 RED는 Stage 3에서 호출만 scoped API로
교체하고 assertion은 유지한다.

### 3.8 cold/warm 30-case matrix

HWP/HWPX × 15 case, 총 30개가 95.90초에 완료됐다.

| case | HWP | HWPX |
|------|-----|------|
| cold seq44 direct | max 174, `(569.7,341.9)`, 28.51ms | max 174, `(569.7,341.9)`, 27.23ms |
| prewarm seq44 direct | max 129, fallback, 1960.76ms | max 129, fallback, 2016.54ms |
| cold seq50 direct | max 180, `(629.7,341.9)`, 26.70ms | max 180, `(629.7,341.9)`, 27.56ms |
| warm30+20 direct | max 129, fallback, 1968.52ms | max 129, fallback, 1971.51ms |

batch/sequential과 direct/path-near는 결과를 바꾸지 않았고 warm 여부만 분기였다. 이 matrix는
case 결과를 JSON으로 기록하는 진단이며 현재는 case별 기대값을 hard assertion하지 않으므로,
`1 passed`를 production correctness GREEN으로 해석하지 않는다.

## 4. 최신 Studio RED

최신 WASM으로 실제 앱 로드와 keyboard 경로를 형식별 세 번 실행했다.

| 형식 | N | 일반 입력 p50 | 경계 handler | explicit flush |
|------|---|---------------|--------------|----------------|
| HWP | `[44,44,44]` | 35.99~36.26ms | 2022.64~2056.43ms | 906.9ms |
| HWPX | `[44,44,44]` | 33.71~33.94ms | 1955.77~1973.11ms | 931.2ms |

- verdict: 두 형식 모두 `RED`
- cause: `stale-layout-input`
- model/LINE_SEG: 최신
- tree/layout: flush oracle과 불일치
- pagination 없는 full-layer render: 복구하지 못함
- timeline/full-layer 대 flush: 8,644 pixels, 4.5418%, 두 형식 동일

기존 `stale-pagination-fragment` 라벨은 cold/warm 원인 격리를 과도하게 해석하므로
`stale-layout-input`으로 정정했다. 이 runner는 수동 진단용이며 Stage 4 영구 GREEN은 더 작은
contract runner로 정리한다.

## 5. 검증 결과

| 검증 | 결과 |
|------|------|
| `cargo fmt --check` | 통과 |
| Stage 2 targeted Rust compile | 통과 |
| cell-flow 1~50 baseline | 1 passed, 4.74s |
| cache-only + 115쪽 cut/UTF-16 range probe | 1 passed, 25.60s |
| warm public behavior RED | expected failure, transient 계약 통과 후 HWP/HWPX 모두 174/129 |
| actual invariant call-site RED | expected failure, stable/44번째 × 두 형식 모두 `(true,true,true)`, scan 0 |
| actual flag-changing call-site RED | expected failure, 두 형식 모두 `(true,true,Some(false),true)`, scan 0 |
| flag-invariant global-clear RED | expected failure, all five cache entries absent |
| cached-true local-change RED | expected failure, `(false,false,false,None,false)`, scan 2 |
| flag-changing global-clear RED | expected failure, `(false,false,None,false)` |
| all crate-internal #2214 ignored explicit run | 1 diagnostic passed + 5 expected RED, 25.58s |
| default #2214 native gates | integration 1 passed/1 ignored, lib 6 ignored |
| 30-case matrix | diagnostic completed, 95.90s |
| #2185 한 글자 HWP/HWPX | 1 passed, 6.90s |
| targeted clippy (`-D warnings`) | 통과 |
| Studio unit | 206/206 passed |
| latest WASM build | 통과, SHA 고정 |
| Studio HWP/HWPX 3회 RED | 결정적 재현 통과 |
| `git diff --check` | 통과 |

의도된 RED 여섯 건은 기본 test run에서 `#[ignore]`이므로 일반 게이트를 깨지 않는다. Stage 3에서
production 수정 뒤 같은 desired assertions를 non-ignored GREEN으로 승격한다.

## 6. ignored 산출물

```text
output/poc/task2214/stage2/native-matrix.json
output/poc/task2214/stage2/studio/summary.json
output/poc/task2214/stage2/studio/{hwp,hwpx}-diagnostic.json
output/poc/task2214/stage2/studio/{hwp,hwpx}/{timeline,full-layer-control,flush-control,diff}/
```

WASM, JSON, PNG, diff와 `node_modules`는 Git에 포함하지 않는다.

## 7. Stage 3 입력 계약

Stage 3은 Rust 범위에 한정한다.

1. `LayoutEngine`에 edited cell과 owner table을 받는 scoped eviction API를 추가한다.
2. full-table rescan 대신 edited paragraph의 pre/post local contribution과 cached owner flag를
   사용한다.
   - flag 불변: edited cell만 제거, owner flag/sibling/unrelated identity 보존
   - local false→true + cached owner true: table-wide 불변이므로 같은 국소 계약
   - local false→true + cached owner false: owner table의 모든 cell units 제거, owner flag=true
     갱신, unrelated/nested table identity 보존
   - 모든 branch에서 mutation부터 cache 재warm까지 table-wide scan 0회
   - table-wide predicate를 단일 helper에 두고 mutation에 별도 전수 스캔을 만들지 않음
3. mutation 전후 상대 flow advance를 비교해 `cellFlowChanged`를 deferred insert JSON에
   추가한다.
4. 처리 순서는 pre-local contribution 캡처 → mutation/reflow → 후속 vpos 재계산 → post-local
   contribution/cached flag 판정 → scoped eviction/update → page-tree invalidation이다.
5. Stage 3에서는 Studio pre-cursor flush를 아직 연결하지 않는다.

Stage 2의 scan counter는 현재 유일한 cached flag helper의 cache-miss predicate 실행을 센다.
Stage 3에서 새 direct table iteration을 추가하면 이 counter를 우회할 수 있으므로, GREEN 전환
전에 table-wide predicate를 단일 production helper로 추출하고 counter를 그 helper 내부에 둔
뒤 mutation 경로의 중복 전수 스캔 부재를 코드 구조와 테스트로 함께 확인한다.

다음 조건이면 Stage 3을 중단하고 범위를 다시 승인받는다.

- local contribution/cached flag만으로 판정할 수 없거나 owner table cell 집합을 안전하게 제거할 수 없음
- nested/ancestor cache coherence에 reverse owner index 또는 generation 설계가 필요함
- scoped eviction 뒤에도 warm direct/path-near가 exact hit하지 못함
- 115쪽 page count 또는 target tree 범위가 깨짐

## 8. 최종 PR 전 진단 정리 계약

Stage 2의 95초 matrix, cache-isolation probe와 대형 Studio runner는 원인 확정용 진단이다.
최종 PR 전에는 다음처럼 정리한다.

1. full matrix를 최종 production으로 1회 실행한다. case별 hard assertion을 추가하기 전에는
   correctness GREEN이 아니라 diagnostic completion으로만 기록한다.
2. cold/prewarm·direct/path·44/50 대표 case는 빠른 non-ignored native 회귀로 축약한다.
3. 여섯 intentional RED와 115쪽 핵심 assertion을 non-ignored GREEN으로 승격한다.
4. 대형 timeline/PNG 수집은 optional `--diagnose`로 남길 수 있지만, focused Studio E2E의
   기본 실행은 HWP/HWPX GREEN이어야 한다.
5. final acceptance에 expected RED, 필수 `#[ignore]`, `--diagnose` 전용 실행을 남기지 않는다.

## 9. 승인 요청

Stage 2의 test-only 계약과 근거를 승인받은 뒤 Stage 3 Rust production 변경을 시작한다.
승인 전에는 scoped cache API, `cellFlowChanged`, Studio input pipeline을 수정하지 않는다.
