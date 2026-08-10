---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-09
---

# Task #3820 Stage 95 — HWPX 임시 suffix reflow의 cache patch 신호

## 시작 근거

Stage 94는 HWPX 짧은 edit의 stored LineSeg 보존(#2185)과 실제 줄 수 변화의 full reflow
boundary(#2214/#2424)를 함께 보정했다. 전체 release-test에서 다음 하나가 남았다.

```text
issue3137_focused_cell_geometry_matches_exact_rect
hwpx tail input 58 patch signal: expected true, actual false
```

58번째 입력은 HWPX의 실제 61번째 flow boundary보다 앞이다. 따라서 최종 조판은 네 줄이고
focused page tree는 patchable이어야 한다. Stage 94의 suffix 재래핑은 마지막 줄 start만
`122 → 123`으로 이동시켰지만 줄 수·높이는 바꾸지 않았다. 이 **동일 줄 수의 내부 drift**가
cache patch의 line identity를 깨고 `focusedPageTreePatched=false`를 만들었다.

## 재현 및 원인 판정

2026-08-09에 다음 focused regression을 실행했다.

```text
CARGO_TARGET_DIR=target/pr-review CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --lib \
  wasm_api::tests::issue3137_focused_cell_geometry_matches_exact_rect \
  -- --exact --nocapture
```

HWPX tail 입력 54--57은 모두 `cellFlowChanged=false` 및
`focusedPageTreePatched=true`였다. 58번째 입력은 다음처럼 실패했다.

```json
{"cellFlowChanged":false,"charOffset":188,"focusedPageTreePatched":false,"ok":true}
```

`cellFlowChanged`는 `relative_paragraph_flow_advance`의 전후 차이만 비교하므로 실제
네 줄 높이가 유지됐다는 뜻이다. 58번째 뒤 paragraph의 LineSeg는
`[0, 44, 84, 123]` (각 `text_start`)였고, Stage 93에서 고정한 저장 경계의 마지막 값
`122`에서 하나 이동해 있었다. 줄 수는 바뀌지 않았으므로 Stage 94의 “local helper가 새
줄을 만들 때만 full reflow 후보를 확인” 조건은 실행되지 않았다. 따라서 focused caret의
동일-line signature 판정이 실패해 page-tree patch가 보수적으로 중단됐다.

즉 이는 page-tree cache의 결함이나 58번째의 실제 flow boundary가 아니라, **실제 flow가
유지되는 HWPX suffix edit이 마지막 LineSeg start를 이동시킨 결함**이다. 첫 후보 보정은
줄 수만으로 이 변화를 감지한다는 가정이 틀려 #3137을 통과시키지 못했다.

## 보정 기준

- suffix helper가 실제로 prefix를 보존한 HWPX edit에서만, LineSeg geometry가 바뀌었는지
  확인한다. prefix가 무효한 full-reflow fallback은 보존한다.
- geometry가 바뀐 경우에만 full reflow를 *후보 계산*으로 실행한다.
- 후보의 줄 수가 기존과 달라지면 그 후보 LineSeg를 적용한다. 이것이 #2214의 61번째
  growth와 #2424의 5→4 shrink다.
- 후보의 줄 수가 기존과 같으면 저장 LineSeg를 유지한다. 텍스트는 이미 편집됐지만
  LineSeg geometry와 page-tree patch 계약은 그대로다.
- HWP 경로와 prefix가 무효한 full-reflow fallback은 변경하지 않는다.

### 두 번째 후보 보정의 반례

전체 LineSeg 배열이 다르면 저장 배열 전체를 복원한 두 번째 보정은 #3137의 첫 stable
tail insert에서 실패했다. patch는 생성됐지만 delta 적용 x가 `239.2466…`이고 fresh
render x는 `239.0`이었다. 이는 cache 계약이 요구하는 마지막 focused line 외의
LineSeg까지 되돌려 기존 text-layout 조정과 충돌했다는 반례다.

따라서 최종 기준은 더 좁다.

- deferred tail patch가 참조하는 **마지막 focused LineSeg**만 비교한다.
- 첫 HWPX edit처럼 `text_start`는 같아도 line metric/tag가 달라지는 경우에는 suffix
  helper 결과를 그대로 유지한다. focused signature가 달라져 cache patch는 자연스럽게
  fallback하며, fresh exact cursor rect가 다음 revision의 기준이 된다. 이 경우 full
  reflow를 강제하면 #2185의 저장 LineSeg `[0,44,84,122]`가 `[0,45,87,125]`로 회귀한다.
- 마지막 LineSeg의 `text_start`만 달라지고 나머지 metric/tag가 같을 때만 HWPX
  false-positive 후보로 분류한다.
- 그 후보의 줄 수가 불변이면 현재 마지막 seg의 `text_start`만 저장값으로 되돌린다.
  비-focused LineSeg와 정상적으로 정규화된 metric은 건드리지 않는다.
- 후보 줄 수가 감소하거나 증가하면 실제 flow boundary이므로 후보 전체를 적용한다.

## #3137 56번째 cache guard 원인

56번째 입력 뒤에도 `focused_before`/`focused_after`는 모두 line index 3, start 122,
동일 LineSeg signature이고 `cellFlowChanged=false`였다. 최종 LineSeg도
`[0,44,84,122]`로 보존됐다. 그럼에도 patch가 false였던 직접 원인은
`try_patch_cached_focused_cell_tail_line`의 자연 폭 guard였다.

- cached cell layout의 `available_width`: 약 551.893px
- HWPX 저장 마지막 LineSeg `segment_width`: 44,008 HU = 약 586.773px (96dpi)

56번째의 plain TextRun은 전자의 폭 guard는 넘지만 후자의 저장 line 폭에는 아직 들어간다.
따라서 HWPX의 비합성(authoritative) LineSeg에 한정하여 cache guard의 허용 폭을
`segment_width`로 읽어야 한다. Render tree의 TextRun style 자체는 기존 cached style을
유지하고, 이 값은 tail patch 허용 여부에만 쓴다. HWP·합성 LineSeg·다른 text 경로는
기존 cell layout 폭을 유지한다.

## 현재 보정과 검증 순서

- HWPX 저장 prefix의 첫 metric/tag 정규화는 helper 결과를 보존한다. `text_start`와
  줄 수가 변하지 않으므로 #2185 저장 경계 계약을 깨지 않는다.
- 마지막 start만 이동하거나 줄 수가 실제로 달라질 때만 full-reflow 후보를 계산한다.
  후보 줄 수가 같으면 저장 LineSeg를 복원하고, 증가·감소하면 후보를 적용한다.
- cache tail patch는 HWPX 비합성 저장 LineSeg의 `segment_width`까지 허용하되, cached
  TextRun style·geometry 계산은 바꾸지 않는다. 따라서 같은 줄의 edit만 patch하며 실제
  flow boundary에서는 기존 invalidation으로 돌아간다.

`issue3137_focused_cell_geometry_matches_exact_rect`는 이 보정 뒤 `1 passed; 0 failed`로
통과했다. #2185, #2214, #2424, #2020, #2430을 순차 재검증한 뒤 전체 release-test를 실행한다.

## 수정 뒤 focused 결과

같은 전용 target에서 아래 순서를 모두 exit `0`으로 완료했다.

| gate | 결과 |
| --- | --- |
| `issue_2430_cell_rewrap_threshold` | `2 passed; 0 failed` |
| `issue_2185_korean_break_unit` | `1 passed; 0 failed` — HWP/HWPX 저장 경계·원본 형식 왕복·115쪽 |
| `issue3137_focused_cell_geometry_matches_exact_rect` | `1 passed; 0 failed` — HWPX cache patch와 fresh tree 일치 |
| `issue2214_scoped_cache_coherence_preserves_transient_pagination` | `1 passed; 0 failed` — HWPX 61번째 boundary·115 fragments |
| `issue_2214_page_local_repaint` | `3 passed; 0 failed` — HWP=56/HWPX=61 boundary와 56·62 cursor/LineSeg를 형식별 고정 |
| `issue_2214_cache_matrix_probe` | `1 passed; 0 failed` — cold/warm·direct/path cache 행렬 |
| `issue_2424_pagination_subphase_probe` | `2 passed; 0 failed` — 형식별 boundary 뒤 모두 115 fragment `pending → complete` |
| `issue2424_resumable_delete_commits_only_after_final_fragment` | `1 passed; 0 failed` — 5→4 shrink |
| `issue_2020` | `4 passed; 0 failed` — receipt PDF contract 포함 |

이 단계에서 변경한 것은 production reflow/cache 보정과 그 보정의 source-format-aware
회귀 계약이다. fixture, PDF oracle, page-count baseline은 수정하지 않았다. 전체
`cargo test --profile release-test --tests`의 최종 exit code와 summary를 다음으로 기록한다.

## #2214 integration gate의 stale 56회 가정

`tests/issue_2214_page_local_repaint.rs`는 HWP와 HWPX 모두 56번째 입력에서 fifth line과
`cellFlowChanged=true`를 기대했다. 그러나 다음 두 독립 근거가 HWPX 61회를 가리킨다.

1. Stage 90/93의 한컴 2020 adapter-save oracle 기록: HWP 56 / HWPX 61.
2. 현재 `wasm_api::tests::issue2214_scoped_cache_coherence_preserves_transient_pagination`
   는 format별 boundary helper(56/61)를 사용하며 통과했다. HWPX 61번째의 target end는
   191(원 130 + 61), HWP는 186(원 130 + 56)으로 확인됐다.

따라서 page-local integration gate의 format-unaware expectation은 회귀를 잡는 기준이 아니라
HWPX oracle을 HWP 값으로 되돌리는 stale baseline이다. test는 HWP 56 / HWPX 61로 분기하고,
각 56·62 representative cursor와 LineSeg 기대값은 실제 current oracle에서 다시 계측해
명시한다. fixture/PDF baseline은 변경하지 않는다.

### 2026-08-09 현재 구현 계측

보정된 현재 소스에서 `issue_2214_page_local_repaint`를 `--nocapture`로 실행해, 기존
56회 단일 기대값이 실제로 깨지는 위치를 계측했다. HWP 결과는 기존 수치와 일치했다.

| 형식 | 56회 뒤 LineSeg starts | deferred path caret | cold direct caret |
| --- | --- | --- | --- |
| HWP | `[0, 44, 84, 122, 129]` | `(x=573.9, y=344.8)` | `(x=573.9, y=345.6)` |
| HWPX | `[0, 44, 84, 122]` | `(x=671.6, y=319.2)` | `(x=671.6, y=320.0)` |

두 경우 모두 page `0`, height `16.0`, transient cell height `945.9`, overflow `false`였다.
그러므로 HWPX 56회에서 HWP의 fifth-line caret를 요구하던 assertion은 단순 허용치 문제가
아니라 형식 고유 oracle을 반대로 만드는 잘못된 계약이다. 이에 56회와 62회의 기대
LineSeg/caret를 형식별 helper로 명시하고, flow delta 및 `changed_inputs`도 HWP=56,
HWPX=61로 고정했다.

같은 실행에서 62회 HWPX도 fifth line으로 전환했고, starts는
`[0, 45, 87, 125, 129]`, path caret은 HWP와 같은 `(x=621.5, y=344.8)`이었다.
따라서 5행 상태 자체도 형식별로 고정한다. HWP의 `[0, 44, 84, 122, 129]`를 HWPX에
강요하지 않으며, HWPX의 `[0, 45, 87, 125, 129]`만 정답으로 둔다.

## #2424 local probe의 동일한 stale boundary

`issue_2424_pagination_subphase_probe`는 실제 제품 게이트가 아니라 ignored local
performance diagnostic이지만, HWP/HWPX 모두 `0..56`까지만 입력한 뒤 56번째가
`cellFlowChanged=true`이고 resumable begin이 `pending`이라고 강제하고 있었다.

현재 실행에서 HWP는 기존처럼 56회에서 boundary를 만나 `pending`으로 들어갔다. 반면
HWPX 56회는 앞서 계측한 대로 아직 4행이고 `cellFlowChanged=false`이므로 deferred
pagination job이 없으며 begin의 `fallback`은 정상 결과다. 이것은 cache/path 결함이 아니라
**아직 HWPX boundary까지 입력하지 않은 diagnostic fixture 오류**다.

따라서 probe도 source-format boundary helper(HWP=56, HWPX=61)를 써서 입력 횟수와
flow signal을 계산한다. boundary에 도달한 두 형식 모두 115개 fragment의 `pending → complete`
resumable 경로를 확인한다. timing 자체에는 assertion을 추가하지 않는다.

수정 뒤 같은 명령을 다시 실행해 `issue_2214_page_local_repaint`의 세 계약이 모두
`3 passed; 0 failed`임을 확인했다. 이 gate는 fixture/PDF baseline을 변경하지 않고,
한컴 adapter-save PDF가 구분한 원본 형식별 전환점만 보존한다.

성능 진단용 ignored probe
`issue_2424_profile_boundary_full_pagination_subphases`도 HWP=56/HWPX=61 경계로
정정해 명시 실행했고, HWP·HWPX 모두 deferred flush 뒤 115쪽을 유지하며
`1 passed; 0 failed`였다. 타이밍 수치는 환경 의존이므로 계약에 포함하지 않았다.

## 계획

1. #3137을 단독 재현해 `cellFlowChanged`, page-tree cache invalidation, final LineSeg 순서를
   확인한다.
2. deferred cell edit에서 동일 줄 수의 임시 suffix reflow가 실제 LineSeg를 바꾸지 않도록,
   후보와 최종 geometry를 분리하는 최소 보정을 적용한다.
3. #2430 후 #3137/#2185/#2214/#2424/#2020 및 전체 release-test를 재실행한다.

## 완료 기준

- HWPX의 same-line tail edit는 focused page patch를 유지하고, 실제 줄 수 변화 때만
  무효화된다.
- #2185의 stored LineSeg 보존과 #2214/#2424의 61번째 boundary가 유지된다.
- 전체 release-test가 `0 failed`다.
