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
