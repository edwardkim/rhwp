---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-09
---

# Task #3820 Stage 86 — 전체 회귀의 WASM pagination coherence 실패 분석

## 실행 사실

Stage 85의 short-child content-box 변경 뒤 다음 전체 gate를 끝까지 실행했다.

```sh
CARGO_TARGET_DIR=target/pr-review CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --tests
```

명시적 결과는 `3370 passed; 5 failed; 13 ignored`다. 따라서 전체 회귀가 통과했다고
간주하지 않는다.

| 실패 test | 관측 |
| --- | --- |
| `wasm_api::tests::issue2214_scoped_cache_coherence_preserves_transient_pagination` | `hwp: input 56 flow signal`: `Some(false)` vs expected `Some(true)` |
| `wasm_api::tests::issue2424_new_edit_stales_old_job_and_sync_flush_restarts_latest_revision` | status `fallback` vs expected `pending` |
| `wasm_api::tests::issue2424_resumable_delete_commits_only_after_final_fragment` | HWP delete 후 fifth line이 남음 |
| `wasm_api::tests::issue2424_resumable_pagination_commits_only_after_final_fragment` | begin status `fallback` vs expected `pending` |
| `wasm_api::tests::issue3137_focused_cell_geometry_matches_exact_rect` | `hwp: tail input 56 flow signal`: `Some(false)` vs expected `Some(true)` |

모두 native HWP의 incremental pagination/cache state를 전제로 한다. Stage 85의 pure render
normalization overlay는 fresh `DocumentCore`의 page render에서만 적용하지만, 현재 worktree에는
`typeset.rs`, `height_measurer.rs`, `table_layout.rs`, `table_partial.rs`의 미커밋 변경도 함께
있다. 따라서 이 다섯 실패를 Stage 85 projection의 회귀라고 단정할 근거는 없다.

## 분리 가설과 금지 범위

1. HWP5 flow signal이 false가 된 공통 원인은 `RenderNormalizationOverlay` 캐시 key/재사용이거나
   `PartialTable` continuation state일 수 있다. 먼저 `issue2214`/`issue3137`을 단독으로 재현해
   HWP와 HWPX case의 첫 divergence를 대조한다.
2. `issue2424`의 `pending`→`fallback`은 실제 layout failure인지, stale render-normalization/cache
   revision을 invalidation하지 않은 결과인지 확인한다. fallback 값을 테스트에 맞추도록 강제하지
   않는다.
3. PDF p81/p82 owner correctness를 위해 Stage 85 projection을 전역으로 끄거나 510HU margin
   보존을 풀지 않는다. p33 geometry assertion도 삭제·갱신하지 않는다.

다음 source 변경 전에는 각 failing test를 **하나씩** 실행해 failure 로그와 DocumentCore revision,
layout profile, row fragment owner를 기록한다. 원인이 확인되면 해당 state transition만 보정하고,
각 변경 뒤 첫 Cargo 검증은 `issue_2430_cell_rewrap_threshold` 2/2로 수행한다.

## #2214 fixture 구조와 p81 변경의 비적용 확인

`issue1949_giant_cell_nested_tables_perf.hwp`의 target은 section 0 / parent paragraph 0 / table
control 2 / cell 2 / cell paragraph 5다. source table은 `3×1 RowBreak`, target paragraph는
stored 4 lines(`vpos=9480,11400,13320,15240`)이며 nested table control이 없다.

`cellFlowChanged`는 `text_editing.rs`에서 `reflow_cell_paragraph()` 전후의
`relative_paragraph_flow_advance()` 값만 비교해 결정된다. 따라서 다음 Stage 85 변경은 이
fixture의 input 56 signal을 직접 바꾸지 않는다.

- `pi=842` 한정 render normalization width/content-box projection: target 구조 불일치
- 마지막 1×1 nested child measured-tail fit: target에 nested child 없음
- RowBreak short-child cell-unit split/terminal source cut: target에 nested child 없음

실제 flow threshold 자체가 56에서 57 이상으로 움직였는지, 혹은 stale `line_segs`/memo가
재사용되는지 아직 확정되지 않았다. 다음은 target tuple에만 `flow_advance_before/after`,
`line_segs.len()`, text length를 출력하는 관측 전용 diagnostic이다. 이 diagnostic은 mutation,
flow 판정, cache invalidation을 바꾸지 않으며, 기록 뒤 제거한다.

관측 결과 input 1부터 실패 지점 input 56까지 모두 같았다.

```text
input 1:  len 130→131, flow Some(7680)→Some(7680), lines 4→4
...
input 56: len 185→186, flow Some(7680)→Some(7680), lines 4→4
```

그러므로 실패는 pending descriptor나 resumable job의 후속 상태가 아니라, 그 전단의
`reflow_cell_paragraph()`가 fifth line을 만들지 않은 데서 시작한다. #2424의 `fallback`과 delete
실패는 이 false flow signal의 연쇄 결과다. 다음 분석은 reflow가 사용한 cell inner width,
resolved font advance, composed line count와 source LINE_SEG의 4-line storage를 나란히 기록해
“test threshold가 stale인가”와 “reflow width/metric이 넓어졌는가”를 구분한다.

## 2026-08-09 — reflow 경로와 다음 관측 설계

코드 경로를 다시 확인했다. deferred insert는 `replace_text_in_cell_native_impl()`에서 대상
문단을 수정한 뒤 아래 순서로 진행한다.

1. `cell_metrics_for_control()`이 target cell의 `width=44790HU`, padding `141HU`를 읽는다.
2. `reflow_cell_paragraph()`이 px 변환, cell 내 문단의 좌우 paragraph margin 차감을 거쳐
   `reflow_line_segs()`에 `final_width`를 넘긴다.
3. `reflow_line_segs()`는 그 폭으로 `fill_lines()`를 실행하고 새 LINE_SEG의 수·vertical_pos를
   만든다.
4. `relative_paragraph_flow_advance()`가 마지막 LINE_SEG 끝을 비교해 `cellFlowChanged`를
   결정한다.

따라서 현재 56번째 입력의 false는 cache invalidation 단계가 아니라 step 2 또는 step 3의
폭/글리프 측정 결과가 네 줄에 머무르는 현상이다. 기존 test의 “56번째에서 4→5줄” 주석은
**검증할 가설**일 뿐 정답으로 취급하지 않는다. 반대로 지금의 four-line 결과도 정답으로
가정하지 않는다.

다음 관측은 환경 변수 `RHWP_DIAG_DEFERRED_CELL_FLOW`가 있을 때 이 fixture target에 한해
`cell_width`, padding, paragraph margin, `final_width`, 재래핑 뒤 LINE_SEG의 UTF-16 start를
기록한다. mutation, line-break 규칙, cache state에는 영향을 주지 않는다. 이 값으로 (a) 저장
HWP metrics를 써야 하는 폭 해석 오류인지, (b) glyph advance/line-break의 threshold 오류인지
구분한다. 관측을 마친 즉시 진단 코드는 제거하고 다시 지정된 #2430 2/2 gate를 확인한다.

### 관측 결과

HWP fixture의 input 1부터 56까지 reflow에 전달된 값은 모두 아래와 같았다.

```text
cell_width_hu=44790
padding_hu=(141, 141)
cell_width_px=597.2000
available_width_px=593.4400
paragraph_margin_px=(6.6667, 0.0000)
final_width_px=586.7733
LINE_SEG starts=[0, 45, 87, 125], count=4
```

input 56에서도 `len=185→186`, LINE_SEG starts와 count가 전혀 바뀌지 않고 flow가
`Some(7680)→Some(7680)`이었다. 즉 56회 경계는 float round-off나 deferred cache를 거쳐
우연히 사라진 것이 아니라, 현재 `fill_lines()`가 이 폭에서 마지막 줄을 계속 수용한 결과다.

아직 할 수 없는 결론은 두 가지다.

- **기존 56회 회귀의 기대값을 갱신하지 않는다.** 다른 측정 구현 또는 저장 HWP의 true text
  viewport를 확인하기 전에는 test가 stale인지 판단할 수 없다.
- **현재 폭을 즉시 줄이지 않는다.** 44790HU는 parsed cell width이고, 임의 축소는 실제 PDF와
  일치하는 다른 긴 셀 문단을 망가뜨릴 수 있다.

다음 관측은 같은 final width에서 다섯째 줄이 처음 생기는 추가 입력 수를 clone에만 적용해
측정한다. 그 임계가 57 근처면 반올림/advance 문제를, 크게 멀면 target의 viewport/원본
LINE_SEG 해석 불일치를 우선 조사한다.

### clone threshold 결과

같은 final width(`586.7733px`)의 clone에 끝자리 `1`을 더 넣어 측정한 결과는 다음과 같다.

| 편집 직후 길이 | 다섯째 줄까지 추가로 필요한 `1` | 다섯째 줄이 생기는 최종 길이 |
| ---: | ---: | ---: |
| 180 | 11 | 191 |
| 181 | 10 | 191 |
| 182 | 9 | 191 |
| 183 | 8 | 191 |
| 184 | 7 | 191 |
| 185 | 6 | 191 |
| 186 | 5 | 191 |

따라서 현재 알고리즘의 실제 경계는 56번째 입력(길이 186)이 아니라 **61번째 입력(길이
191)** 이다. 5자 차이는 padding HU 한 번의 반올림으로 설명될 수 있는 규모가 아니다.
다음 가설은 숫자 `1`의 resolved font family/size/ratio와 `fill_lines()`가 사용하는 실제 advance가
기존 `#2430` 주석의 0.497em 전제와 다시 달라졌는지다. 이 값을 관측하기 전에는 폭 보정이나
test 기대값 변경을 하지 않는다.

## 2026-08-09 — upstream line-break 변경과의 인과 분리

`#2430`의 56회 expectation은 commit `1727cfc20`에서 한컴 COM PDF ladder 실측을 근거로
44→56으로 이관됐다. 현재 HEAD는 그 뒤의 `ba99fad54` (`#3822`, overlong token을 이전 break
뒤에서 새 줄에 다시 fit 검사)와 `#4046` native/WASM measurer 공통화 등을 포함한다. 현재
worktree diff에는 font metric data나 line-breaking algorithm의 기능 변경이 없으므로, Stage 85의
table projection과 별개로 **upstream #3822가 이 particular boundary를 바꿨는지** 먼저 분리한다.

이를 위해 normal 동작에는 전혀 영향이 없는 진단 환경변수로, #3822 이전의 “이전 break 뒤
current token을 다시 fit 검사하지 않고 누적” 분기만 같은 HWP target에서 재현한다. 결과가
56으로 돌아오면 이전 test expectation은 #3822의 효과를 반영하지 못한 것이다. 결과가 여전히
61이면 glyph metric/viewport 가설을 계속 조사한다. 어느 쪽이든 PDF/COM의 편집 결과가 없는
상태에서 normal path나 expectation은 변경하지 않는다.

### #3822 counterfactual 결과

`RHWP_DIAG_PRE3822_POSTBREAK=1`로 위 이전 분기만 emulation해도 clone threshold는 동일하게
길이 191(56번째 입력 뒤 5자 추가)였다. 따라서 `#3822`의 token 재검사는 이 five-character
drift의 원인이 아니다. 진단 분기는 제거한다.

남은 유력 경로는 다음 둘이다.

1. `resolved_to_text_style()`가 target의 appended ASCII `1`에 적용하는 family/size/ratio/spacing
   또는 embedded metric이 #2430 ladder 전제와 다르다.
2. COM ladder가 실제로 확인한 편집 뒤 LINE_SEG boundary와, 현재 HWP fixture의 cell inner
   viewport/paragraph margin 해석이 다르다.

다음 관측은 마지막 appended `1`의 resolved style과 unrounded advance 및 마지막 line의 누적
HWPUNIT 폭을 기록한다. 이 두 값을 #2430의 0.497em 실측과 비교한 뒤에만 HWP viewport나
metric table의 수정을 검토한다.

### resolved metric 결과

대상에 실제 적용된 appended `1`의 값은 `char_style_id=48`, `한양신명조`, `16.0px`,
장평 `1.0`, 자간 `0.0`, unrounded advance `7.9467px = 0.4967em`이었다. 이는 #2430 COM
ladder의 0.497em과 일치한다. 그러므로 metric table을 넓혀 56회 boundary를 억지로 되돌리는
수정은 근거가 없고, PDF fidelity도 훼손할 수 있으므로 금지한다.

반면 첫 deferred edit 뒤 reflow가 만든 line starts는 `[0, 45, 87, 125]`다. `#2185`의 실물
fixture 보존 근거에는 같은 giant-cell target의 저장 LINE_SEG starts가 `[0, 44, 84, 122]`로
기록되어 있다. 즉 남은 가설은 **current reflow가 첫 stable edit에서 저장 줄 경계를 얼마나
그리고 왜 이동시키는가**다. 다음 관측은 mutation 직전의 stored starts와 첫 reflow 뒤 starts를
동시에 기록한다. source boundary와 다르면, width나 font metric 전역 보정이 아니라 이
native-HWP cell의 stored LINE_SEG 보존/재래핑 전환 규칙을 좁게 고친다.

## 2026-08-09 — 한컴 HWP 저장 oracle 검증 계획

`#2430`의 56회 expectation과 현재 reflow의 61회 결과 중 어느 쪽이 실제 HWP 편집 결과인지
Rust 내부 line count만으로 결정하지 않는다. 다음 순서로 **같은 원본 HWP에서 실제 저장 파일을
만들어 한컴 2020 출력과 대조**한다.

1. `issue1949_giant_cell_nested_tables_perf.hwp`의 target cell paragraph 5 끝(offset 130)에
   ASCII `1`을 각각 56회와 61회 넣은 HWP를 `export_hwp_native()`로 별도 산출한다. 이 산출은
   source fixture를 덮어쓰지 않고, 작업 증적 디렉터리만 사용한다.
2. 각 산출본은 HWP 2020 변환 경로로 PDF화한다. 그 PDF에서 target cell의 실제 행 수와
   4→5행 전환을 읽는다. rhwp가 자기 산출물을 재파싱한 결과는 보조 근거일 뿐 oracle이 아니다.
3. 56회에서 한컴도 4행이면 `#2430`의 fixture expectation이 stale일 수 있으므로, test의
   수정 전 exact COM probe와 현재 fixture/글꼴 버전을 다시 대조한다. 56회에서 한컴이 5행이면
   current native-HWP reflow가 저장 LINE_SEG 전환을 놓친 것이므로, 첫 edit의 `[0,44,84,122]`
   → `[0,45,87,125]` 이동을 최소 범위에서 보정한다.

이 검증 전에는 테스트 기대값, 전역 글꼴 advance, cell width/padding을 변경하지 않는다. HWP
산출을 위한 임시 test/diagnostic이 필요하면 이 절에 기록하고, 증적 생성 뒤 source에서 제거한다.

### 한컴 2020 oracle 결과

환경변수 한정 test 경로로 원본 target cell paragraph 5의 offset 130에 `1`을 각각 56회와
61회 삽입한 HWP를 만들었다. 산출 HWP의 SHA-256은 각각
`714ff71d…aeba52215` 및 `1b7936d…4fe119c1b`이다. 두 파일은 한컴 2020 `PrintToPDFEx`
경로로 직접 PDF화했으며, 결과는 다음과 같다.

| 입력 | 한컴 2020 PDF 관측 | 판정 |
| --- | --- | --- |
| 56회 | 원래 네 번째 줄의 `적용한` 뒤 `다.111…`가 **새 다섯 번째 줄**로 출력 | `#2430`의 56회 boundary가 맞음 |
| 61회 | 같은 다섯 번째 줄에 `1` 다섯 글자만 더 출력 | 이미 56회에서 전환됨 |

두 PDF 모두 A4 115쪽이며 SHA-256은 `a2325062…e983a1d99`(56회),
`823148c7…30891c870`(61회)다. 파일은
`pdf/task_m100_3820_stage86_wasm_boundary_oracle/` 아래에 보관한다.

이 결과는 현재 Rust reflow의 61회 판단이 틀렸고, test expectation을 61로 바꾸는 것은
금지해야 함을 확정한다. 또한 resolved digit advance와 cell width는 이미 한컴 출력과 독립적으로
맞았으므로, 원인은 **전역 폭/폰트 값이 아니라 native HWP incremental edit가 기존 줄 경계를
보존하지 않고 문단 전체를 다시 나누는 방식**이다.

56회 삽입 전 저장 HWP의 starts는 `[0,44,84,122]`이다. 현재 full reflow는 첫 삽입부터
`[0,45,87,125]`로 바꾸어 편집 위치(130)보다 앞선 세 줄도 이동시킨다. 그 결과 마지막 줄의
앞부분 Korean glyph 세 개(약 48px)가 사라져 56개의 숫자를 한 줄에 더 수용하고, five-line
transition이 61까지 늦어진다. 한컴 oracle은 저장된 앞선 boundary를 유지한 채 **편집이 포함된
line(122)부터** 재래핑한다는 쪽을 지지한다.

다음 source 변경은 전체 `reflow_line_segs()`의 metric/width를 바꾸지 않는다. 셀 텍스트
삽입·삭제 호출에만 edit offset을 전달하여, 유효한 native-HWP LINE_SEG가 있고 edit이 첫 줄이
아니면 edit 직전 line들은 그대로 유지하고 edit 포함 line부터 suffix를 재래핑한다. 첫 줄 편집,
HWPX/합성 LINE_SEG, table operation·formatting처럼 명시 edit offset이 없는 호출은 기존 full
reflow를 그대로 사용한다. 구현 뒤 기존 `#2214/#2424/#3137`이 다섯 번째 줄을 일관되게 보는지와
별도 prefix-preservation 회귀를 확인한다.

### 첫 보정 실행과 HWPX 분리

위 범위로 native HWP edit에 suffix reflow를 적용한 뒤 `issue2214`를 다시 실행했다. HWP case는
56번째 input에서 flow signal이 true가 되어 transient/flush oracle까지 통과했다. test가 이어서
검사한 HWPX case는 여전히 56번째에서 false였다. 즉 HWP의 원인은 확정·보정됐지만, HWPX를
"합성 LINE_SEG이므로 full reflow 유지"로 둔 초기 범위는 gate 계약을 충족하지 못한다.

`#2430`의 기존 근거는 HWP와 HWPX 모두 같은 56회 COM ladder였으므로, 다음에는 HWPX 원본도
동일 edit 뒤 HWP adapter 저장본을 한컴 2020 PDF로 내보낸다. PDF에서도 56회가 다섯째 줄이면
HWPX 저장 LINE_SEG도 유효한 prefix로 취급하되, HWPX에만 존재하는 adapter/합성 marker는
제외하는 구조 조건을 좁힌다. PDF가 61회를 지지하면 HWPX expectation만 별도 재검토한다.
이 외부 oracle 전에는 HWPX 범위를 넓히거나 test expectation을 바꾸지 않는다.

### HWPX 한컴 2020 oracle 결과와 gate 정정 범위

HWPX 원본의 56회·61회 edit도 HWP adapter 저장본으로 만들어 같은 한컴 2020 `PrintToPDFEx`
경로로 비교했다.

| HWPX 입력 | 한컴 2020 PDF 관측 | 판정 |
| --- | --- | --- |
| 56회 | `용한다.111…`가 넷째 줄에 남음 | 아직 4줄 |
| 61회 | `다.111…`가 새 다섯째 줄로 전환 | **61회** boundary |

두 PDF는 모두 A4 115쪽이고 SHA-256은 `09ff2091…010e46d4`(56회),
`05cec394…ccb45f3a1`(61회)다. HWPX는 HWP와 실제 editable layout이 다르므로 56이라는
공통 expectation이 stale이었다. 즉 HWPX full reflow를 HWP prefix-preservation 로직으로
확장하는 것은 oracle과 반대이며 수행하지 않는다.

따라서 정정은 test fixture가 source-format별 실제 boundary를 표현하도록 하는 데 한정한다.
HWP는 56회, HWPX는 61회로 각각 입력·cursor offset·delete boundary를 계산한다. pagination
state machine 자체의 pending/complete/fragment count 기대값은 공통으로 유지한다. 이 변경은
회귀를 약화하는 것이 아니라, 기존 단일 56 값이 가렸던 실제 한컴 HWPX 61회 behavior를
검증 계약으로 고정하는 것이다.

## 구현 전 계약 — source-format별 boundary와 정리 범위

이번 수정은 아래 세 가지를 동시에 수행한다.

1. `issue2214`·`issue2424`·`issue3137`의 loop count, cursor offset, 삭제 기준을 `hwp=56`,
   `hwpx=61`이라는 하나의 test helper에서 계산한다. HWPX를 56회에서 억지로 flow-change로
   만들거나, HWP의 prefix 보존 경로를 HWPX까지 확장하지 않는다.
2. native HWP 셀 편집만 `reflow_line_segs_after_cell_text_edit()`를 통해 저장 LINE_SEG
   prefix를 보존한다. HWPX는 `source_format != Hwp` 조건에서 기존 full reflow를 유지한다.
   현재 구현도 이 조건을 가지고 있으므로 동작 변경이 아니라 근거와 주석을 oracle에 맞춘다.
3. HWP/HWPX 56/61 artifact를 만들기 위한 환경변수 기반 test I/O와 flow diagnostic은 이미
   oracle 역할을 마쳤으므로 제거한다. 정답지 PDF와 SHA-256, 재현 fixture·offset은 이 문서와
   `pdf/task_m100_3820_stage86_wasm_boundary_oracle/`에 남는다.

수정 직후 첫 Cargo 실행은 작업 규약대로
`issue_2430_cell_rewrap_threshold`이며, 성공 조건은 명시적 `2 passed; 0 failed`다. 그 다음
다섯 failing WASM test를 개별 실행한다. 이 순서는 test 기대값을 먼저 통과시키는 것이 아니라,
새 HWP suffix reflow가 기존 2430 경계를 훼손하지 않았음을 먼저 확인하는 안전장치다.

## 1차 targeted gate 결과와 잔여 #3137 분리

수정 뒤 첫 명령은 규약대로 다음이었다.

```sh
CARGO_TARGET_DIR=target/pr-review CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test issue_2430_cell_rewrap_threshold -- --nocapture
```

결과는 명시적으로 `2 passed; 0 failed`였다. 이어서 이전 전체 gate의 다섯 실패를 각각
실행한 결과는 다음과 같다.

| test | 결과 | 확인 내용 |
| --- | --- | --- |
| `issue2214_scoped_cache_coherence_preserves_transient_pagination` | 통과 | HWP max=186, HWPX max=191; 두 형식 모두 transient/flush cut 115개와 downstream 113개 변경을 유지 |
| `issue2424_resumable_pagination_commits_only_after_final_fragment` | 통과 | HWP/HWPX 모두 fragment 115개를 끝 step에서만 commit |
| `issue2424_resumable_delete_commits_only_after_final_fragment` | 통과 | 형식별 마지막 boundary 문자 삭제 후 full-pagination oracle과 일치 |
| `issue2424_new_edit_stales_old_job_and_sync_flush_restarts_latest_revision` | 통과 | HWP 56회 boundary 뒤 최신 revision restart 유지 |
| `issue3137_focused_cell_geometry_matches_exact_rect` | 실패 | `focused page tree cache`가 없다는 assertion failure |

따라서 `#2214/#2424`의 기존 다섯 실패 중 네 건은 HWP/HWPX 경계 contract 불일치에서
해소됐고, `#3137`은 별개 cache 관찰 실패로 분리한다. 현재 assertion은
`assert_cached_line_matches_fresh()`에서 무조건 cache slot 0을 읽는다. 반면 patch helper는
실제 target page의 cache slot을 찾아 갱신한다. 먼저 HWP case에서 patch의 `page_index`와 cache의
점유 slot을 관측하여, (a) cache slot 0이라는 test 가정만 틀린 것인지, (b) patch가 true를
보고하고 실제 cache를 잃는 product 결함인지 판정한다. 이 관측은 제품 동작을 변경하지 않는
test 진단이며, 결론 후 제거한다.

### cache slot 관측 결과

target patch가 `pageIndex=0`이라는 가정은 틀렸다. helper가 반환한 실제 patch page를 사용해
cache와 fresh tree를 대조하면 HWP의 `insert-a`와 IME replace는 통과한다. 그러나 이어지는
HWP `delete-backward`는 `focusedPagePatch` 자체를 반환하지 않는다. 이는 slot 선택 오류가 아니라
same-line tail delete fast-path가 `None`으로 fallback한 결과다. 이 fallback이 geometry 보존을
위해 필요한 것인지, suffix reflow 뒤에도 안전하게 patch할 수 있는데 helper predicate가 지나치게
좁은 것인지는 아직 미확정이다. 다음은 mutation JSON을 포함한 failure로 predicate를 식별한다.

`delete-backward`의 실제 반환은
`{"cellFlowChanged":false,"charOffset":131,"focusedPageTreePatched":false}`다. 따라서
pagination이나 output fidelity를 바꾸는 문제가 아니라, `focused_cursor_delta_x()`의 same-line
signature gate 또는 cached tail-line candidate gate가 fast-path를 차단한 것이다. 다음 관측은
`focused_cursor_delta_x()`에 환경변수 한정 debug를 넣어 line index/start/signature와 x만 기록한다.
서명 불일치이면 full invalidation이 올바른 fallback이므로 test를 그 계약으로 고치고, 서명이 같은데
candidate만 거절되면 cached tree patch predicate의 product 결함으로 다룬다.

### same-line predicate 관측 결과

HWP의 첫 입력은 stored LINE_SEG의 `column_start/tag`가 정규화되어 signature가 달라지므로 이미
exact fallback 대상이다. 한 run의 delete에서는 마지막 줄 `text_start`도 `122 → 125`로 바뀌어
fallback이 선택됐다. 이는 `cellFlowChanged=false`가 문단 높이만 불변이라는 뜻이지 line content
boundary까지 같다는 뜻은 아님을 보여 준다.

원인은 삭제 뒤 `char_offsets`에는 end-of-text caret 항목이 없다는 점이었다. 기존 prefix 선택은
`char_offsets[edit_offset]`만 읽어 이 경우 `None`이 되고 full reflow를 선택했다. 저장 prefix의
UTF-16 text end는 token boundary이므로, `edit_offset == text_len`에서는 그 end를 사용해 마지막
stored LINE_SEG를 찾도록 보정한다. 이로써 HWP delete도 line start 122를 보존하고 cache patch가
가능해진다. HWPX는 source-format 분기로 기존 full reflow/fallback을 유지한다.

따라서 #3137은 HWP/HWPX append·IME replace에서 actual cache patch와 fresh-tree equality를
강제한다. HWPX의 paragraph layout은 full reflow를 유지하지만, 그 결과가 같은 visual line이면
cached page-tree tail patch 자체는 안전하며 실제로 반환된다. 따라서 HWPX의 61회 **layout
boundary**와 focused cache patch policy를 혼동하지 않는다.

end backspace는 prefix 재래핑 뒤 local line signature가 같은 경우 patch를, 달라진 경우 full
invalidation을 선택할 수 있다. `#2424`의 5→4 deletion처럼 prefix start 선택이 꼭 달라져야 하는
경우가 있으므로 특정 boolean은 수용 기준이 아니다. #3137은 patch가 있으면 fresh tree와 정확히
같은지를, fallback이면 page-tree cache가 비어 있고 fresh cursor의 cell bounds가 유지되는지를
각각 검사한다. 환경변수 diagnostic은 제거했다.

## end deletion의 empty-final-line 경계

위 end-of-text 보정 뒤 `issue2424_resumable_delete_commits_only_after_final_fragment`의 HWP case가
실패했다. 56회 입력으로 생긴 fifth line의 `text_start`가 삭제 후 text UTF-16 end와 같을 때,
`rposition(seg.text_start <= end)`는 이미 비어 버린 fifth line을 affected line으로 선택한다.
그 결과 prefix에 그 line이 남고 suffix가 빈 줄을 다시 만들므로 `cellFlowChanged=false`가 되어
실제 5→4 전환을 놓친다.

이는 앞 절의 end caret 해석을 되돌릴 근거가 아니다. end deletion에는 서로 다른 두 경우가 있다.

- 마지막 줄의 start가 `end`보다 작다: 그 줄은 실제 문자를 가지므로 계속 suffix reflow의 시작점이다
  (`#3137` IME backspace).
- 마지막 줄의 start가 정확히 삭제 후 `end`와 같다: 삭제된 문자만 있던 empty-final-line이므로
  직전 `< end` 줄부터 다시 나눠야 한다 (`#2424` 5→4 boundary).

수정은 `edit_offset == text_len`의 end deletion에만 strict `< end`를 사용한다. insertion과 중간
edit은 기존 `<=`를 유지한다. 수정 뒤 첫 #2430 2/2 gate와 #2424 delete, #3137, #2214을 차례로
다시 확인한다.

### strict end 후보의 1차 반증

strict `< end`를 적용한 뒤에도 #2424 HWP delete가 `cellFlowChanged=false`로 남았다. 따라서
fifth line이 단순히 `text_start == end`인 empty line이라는 가설은 충분하지 않다. 다음 관측은
56회 입력·flush 직후와 delete 직후의 실제 LINE_SEG start 배열을 test failure에 포함한다. 이 값으로
마지막 line의 실제 source 범위와 suffix reflow 결과를 확인한 뒤, prefix 선택 또는 empty suffix
처리 중 어느 쪽을 보정할지 결정한다.

### 현재 결론 — end deletion과 cache fallback 분리

실제 관측을 반영해 end deletion은 UTF-16 text end를 찾되, 마지막 실제 줄의 start가 end보다
작으면 그 직전 줄부터 suffix를 다시 채우도록 보정했다. 이렇게 해야 boundary 문자를 지운 뒤
마지막 줄이 직전 줄에 합쳐지는 `#2424`의 5→4 전환을 보존한다. 빈 마지막 줄의 start가 end와
같을 때는 그 빈 줄을 prefix에 보존하지 않는다.

이 변경 뒤 첫 규정 gate `issue_2430_cell_rewrap_threshold`는 명시적으로 `2 passed; 0 failed`를
다시 확인했다. 이어서 `#2214`, `#2424` pagination/delete/new-edit, `#3137`을 현재 source로
실행해 모두 통과했다. `#3137`의 end backspace는 같은 문단 높이라도 local line signature가
달라질 수 있으므로 무조건 cache patch를 강제하지 않는다. patch가 선택된 경우에는 해당
`pageIndex`의 cached tree와 fresh tree가 같음을, fallback이면 모든 cached page tree가 비워지고
fresh cursor의 `cellBounds`가 보존됨을 검사한다.

source-format별 한컴 PDF oracle, HWP prefix 재래핑 범위, HWPX full-reflow 경계, end deletion
및 cache 안전성의 최종 분석·targeted 결과는 다음 stage 문서로 이어진다.
[`task_m100_3820_stage89_hwpx_incremental_prefix_reflow.md`](task_m100_3820_stage89_hwpx_incremental_prefix_reflow.md)
