---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-09
---

# Task #3820 Stage 84 — 76076 p81 중첩 행 owner 예산 분석

## 판정 기준과 재현

- 입력: `samples/76076_regulatory_analysis.hwp`
- 독립 기준: `samples/issue1891/76076_regulatory_analysis-2024.pdf` (한컴 2024 PDF)
- 현재 binary: `target/pr-review/release-test/rhwp` (HEAD `380891a5b` 뒤 빌드)

다음 직접 대조를 수행했다.

```sh
RHWP_BIN=target/pr-review/release-test/rhwp \
  venv/bin/python tools/fidelity_compare/fidelity_compare.py 80 81 \
  --source samples/76076_regulatory_analysis.hwp \
  --reference-pdf samples/issue1891/76076_regulatory_analysis-2024.pdf \
  --label task3820-stage83-76076-p081-p082 \
  --reference-grade '한컴 2024 기준 PDF' \
  --out-dir /tmp/rhwp-stage83-76076-visual.gaxV26
```

| 쪽 | raster diff | 기준 PDF owner | 현재 RHWP owner | 판정 |
| --- | ---: | --- | --- | --- |
| p81 | 16.45% | `일시적/반복적`, `반복적`, `근거설명`, `○ 구내운반차 … 사고` 첫 줄 | 표 `pi=842`의 row 0..2만 | **결함** |
| p82 | 5.05% | `를 예방함으로써 …`로 `근거설명`을 재개 | row 3와 `근거설명` 전체를 새로 시작 | **결함** |

`fidelity_compare --text-only --export-all-svg --layout-ledger` 전수 원장도 같은 경계를
`table_fragment_text_owner_drift`로 수집했다. reference-only p81과 SVG-only p82의 상호
이동은 39자이며, source 표 식별자는 `pi=842,ci=0,rows=5,cols=2`다. 전체 페이지 수는 PDF,
SVG, render tree 모두 82로 같으므로 페이지 수는 이 결함을 검출하거나 해소하는 근거가 아니다.

## source·RenderTree 사실

원본 `dump -p 842`는 outer RowBreak table의 row 3을 `일시적/반복적`/`반복적`으로, row 4의
우측 cell을 block 1×1 nested table(`근거설명`, 첫 문단)로 저장한다. 현재 render tree는 다음처럼
분할된다.

| page | `pi=842` visible rows | nested 설명 text |
| --- | --- | --- |
| p81 | 0, 1, 2 | 없음 |
| p82 | 3, 4 | `○ 구내운반차 …`, `사고를 예방함으로써 …`부터 전체 |

이는 기준 PDF의 p81 row 3 + row 4 첫 line / p82 row 4 second line onward와 다르다. p82의
`를 예방함으로써 …` first token은 기준 PDF에서 owner가 바뀐 뒤의 continuation임을 PDF text layer와
review PNG로 함께 확인했다. 문자 multiset만으로 확정하지 않았다.

## typeset budget 경로

`RHWP_DIAG_SPLITSCAN=1 RHWP_DIAG_SCAN=1 RHWP_TABLE_DRIFT=1`으로 p81을 render-tree export한
결과는 아래와 같다.

```text
TABLE_DRIFT pi=842 ... mt_rows=[23.3,23.3,23.3,23.3,239.3]
TABLE_CUT_DRIFT pi=842 ... cut_rows=[20.3,20.3,20.3,20.3,216.7]
TABLE_SPLIT_AVAIL pi=842 cursor_row=0 cont=false ... avail_for_rows=62.3
DIAG_SCAN UNSPLITTABLE r=3 consumed=60.9 row_total=20.3 rest=1.3
TABLE_SPLIT_RESULT pi=842 cursor_row=0 end_row=3 ...
TABLE_SPLIT_AVAIL pi=842 cursor_row=3 cont=true ... avail_for_rows=971.3
```

즉 source row 3은 20.3px이고 p81에 표 첫 세 행을 배치한 뒤 scan 예산은 1.3px으로 계산된다.
그래서 row 3 자체가 시작되지 않고 row 4의 `nested_table_mixed_fragment_heights()` 및 parent
RowCut 전파 경로도 이 경계에서는 사용되지 않는다. p82의 continuation이 row 3부터 시작하는 직접
원인이다.

선행 표 `pi=831`의 PDF vector table border와 RHWP RenderTree bbox를 같은 96 DPI 좌표로
환산하면 다음과 같다.

| 대상 | 시작 | 끝 | 높이 | 차이 |
| --- | ---: | ---: | ---: | ---: |
| 한컴 2024 PDF | 240.1px | 870.2px | 630.2px | 기준 |
| 현재 RHWP | 215.8px | 900.6px | 684.9px | **+54.7px** |

`pi=831` 마지막 row의 1×1 child에서 RHWP는 16개의 비어 있지 않은 `TextLine`을 만들고,
PDF는 15개의 줄로 `…해당한다는 의견`을 마지막 한 줄에 함께 둔다. 이 wrap 차이는 높이 차이의
일부지만, `p842` 시작 위치만으로 계산한 deficit와 동치인지 다음 측정에서 확인해야 한다.

중요하게도 source `dump -p 831`의 table common size는 `47624×47324 HU`이고, diagnostic의
declared height는 634.7px이다. 이는 한컴 PDF border 630.2px와 약 4.5px 차이지만, 현재
`TABLE_DRIFT`는 `measured/effective=684.9px`를 선택한다. 따라서 p842 부족의 직접 원인은
다음 표의 tolerance가 아니라, **고정 높이 RowBreak parent `pi=831`가 줄 측정 결과로 약 50px
과대 확장되는 선택 경로**다. 단순 declared-height 강제는 표 내부 text clip을 재발시킬 수 있으므로,
PDF와 같은 15줄 wrap·inner 1×1 child height가 먼저 재현되는지 확인한 뒤에만 적용한다.

검증 당시 worktree에는 `native_rowbreak_parent_short_child` gate를 추가한 **미검증** 코드 변경이
있었다. 그러나 p81의 scan은 row 3을 시작하기도 전에 끝나므로, 이 gate가 `pi=842` row 4만을
분해해 고친다는 주장은 성립하지 않는다. 또한 같은 모양의 `pi=831`에도 적용되므로, 먼저 focused
회귀로 안전성을 확인하고 p831 measure/fragment에 미치는 실제 효과를 측정한다. 성공 여부와
무관하게 이 변경을 기준 PDF 정합의 증거로 취급하지 않는다.

p842의 row-cut tolerance를 전역으로 늘리거나 기준 PDF를 baseline으로 갱신해 row 3을 억지로
넣으면 다른 RowBreak 경계를 왜곡할 수 있으므로 금지한다.

## 미검증 gate 확인 결과 (폐기)

변경 직후 사용자 지정 first gate를 실행했다.

```text
CARGO_TARGET_DIR=target/pr-review CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test issue_2430_cell_rewrap_threshold -- --nocapture

2 passed; 0 failed
```

그 뒤 같은 HEAD의 `fidelity_compare`와 split diagnostic을 재실행했다. p81/p82의 39자
`table_fragment_text_owner_drift`는 **그대로**였고, `pi=842`도 여전히
`end_row=3`, `page_avail=62.3`, `rest=1.3`이었다. 따라서 focused regression 통과는 이 변경의
기존 계약만 보존했다는 의미일 뿐, 목표 결함을 고쳤다는 증거가 아니다.

이 gate는 `frags.len() > 1`인 child에서만 이어지는 분해 경로를 여는데, 현재 p81은 parent scan이
row 3 전에 종료한다. 관측상 이 경로는 fragment owner를 바꾸지 못했다. 변경은 다음 Cargo gate 전에
제거했으며, `issue_2430_cell_rewrap_threshold`를 재실행해 2/2 통과를 확인했다. 이제
`pi=831`의 과대 measured height 또는 parent fragment available-height 원장을 바로 고친다.

## 기존 declared-tail helper의 도달 여부

코드에는 이미 `fit_measured_table_nested_tail_to_declared_height()`가 있고 주석도 76076의
비용/편익 표를 대상으로 한다. `pi=831`의 source 구조는 helper의 표면 조건(7행 RowBreak,
빈 host, 마지막 행의 1×1 child, 50.2px 축소 필요)에 맞는다. 그럼에도 current `effective_height`가
684.9px인 것은 (a) `TopAndBottom`/host predicate 전에 탈락했거나, (b) child의 실제 IR 속성이
helper의 block predicate와 다르거나, (c) fit 결과가 format 이후 다른 measured table로 덮인 경우다.

다음 코드 변경은 이 셋을 `RHWP_DIAG_NESTED_TAIL=1`의 관측 전용 로그로 구분하는 것으로만
제한한다. 이 로그는 behaviour를 바꾸지 않으며, 로그로 도달 실패를 확정한 뒤에만 helper predicate
또는 row-height 산정의 최소 보정을 설계한다.

## paginator–renderer height parity 결론

관측 로그는 helper가 탈락한 것이 아니라, paginator에서 정상 적용됐음을 보였다.

```text
DIAG_NESTED_TAIL pi=831 float=true shrunk=true empty_host=true \
  native_rowbreak_candidate=true applied=true
raw rows   = [23.28, 23.28, 23.28, 23.28, 23.28, 44.08, 524.37]
fitted rows= [23.28, 23.28, 23.28, 23.28, 23.28, 44.08, 470.51]
TABLE_DRIFT effective=631.0px, declared=634.7px
```

그 결과 paginator는 `pi=842`에 `page_avail=116.1px`를 주고 row 0..3을 p81에 배치한다.
그러나 renderer의 `layout_table()`은 raw `MeasuredTable` 행 높이(684.9px)를 다시 사용해 p81
`pi=831` border를 `y=215.8..900.6`으로 paint하고, p842도 `y=933.7`에서 그린다. 즉 pagination은
fitted height, rendering은 raw height라는 **서로 다른 row-height 원장**을 사용한다.

이 때문에 현재 p81→82 owner candidate는 39자에서 29자로 줄었다. `일시적/반복적` 행은 paginator
기준으로 p81에 넘어왔지만 renderer의 실제 p81에는 row 4의 `근거설명`과 첫 nested line이 없다.
`fidelity_compare`가 이를 잡아냈고, raster diff도 p81 16.67%, p82 4.84%다. 따라서 다음 수정은
pagination만 더 당기거나 row 4를 bleed하는 것이 아니라, native HWP5의 동일 declared-tail contract를
renderer의 `resolve_row_heights`에도 적용해 **paint height를 paginator와 동일하게** 만드는 것이다.

그 parity 뒤에도 p81에 row 4 첫 line이 없으면, 그때에만 `pi=842`의 1×1 child를 parent RowBreak
fragment 단위로 분해하는 별도 좁은 경로를 추가한다. 이 둘을 한 번에 일반 tolerance로 합치지 않는다.

## parity 뒤 남은 child fragment 결함

renderer parity 보정 후 p81의 `pi=831` bbox는 `y=215.8..846.8`, 높이는 631.0px가 되어
Hancom PDF 630.2px와 정합했다. p842도 올바른 후속 위치 `y=879.9`에서 시작한다. 그러나 p842의
first fragment는 row 0..3(93.6px)에서 끝난다.

```text
p842 page_avail=116.1px
rows 0..3 consumed=81.2px
remaining=34.9px
row 4 (1×1 child 전체) cut height=212.3px  -> atomic, start하지 않음
```

기준 PDF는 이 34.9px 안에 row 4 label `근거설명`과 child 첫 line
`○ 구내운반차 안전조치를 통해 근로자와 부딪히는 등의 사고`를 소유한다. 그러므로 남은 문제는
parent의 height가 아니라 **한 페이지 안에 들어오는 short parent의 1×1 child가 atom으로 남는
cell-unit granularity**다.

새 gate는 다음을 모두 요구한다.

1. native HWP5, non-TAC, TopAndBottom, RowBreak, 실제 다행 parent;
2. 마지막 child가 non-TAC 1×1 block table이고 여러 fragment를 만들 수 있음;
3. 그 child의 physical height가 parent declared height보다 큼.

세 번째 조건은 `pi=842`(child tail 212.3px, parent declared 114.5px)은 통과시키되,
child가 parent 안에 정상적으로 들어가는 `pi=831`(parent declared 634.7px)은 제외한다. 따라서
이미 회복한 p831 declared-tail fit을 되돌리지 않는다. 먼저 `issue_2430`을 실행하고, 그 다음
p81/p82 owner와 `issue_3820_rowbreak_rowspan_band`를 확인한다.

## cell-unit gate의 실제 도달과 빠진 split-eligibility

위 구조로 `cell_units()`가 1×1 child의 mixed fragment를 생성하도록 연 뒤, 사용자 지정 first
gate를 다시 실행했다.

```text
CARGO_TARGET_DIR=target/pr-review CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test issue_2430_cell_rewrap_threshold -- --nocapture

2 passed; 0 failed
```

그러나 direct PDF 비교의 owner는 여전히 p81 reference-only / p82 SVG-only 29자였고, scan은
row 4를 atomic으로 처리했다. 원인은 다음 두 계약이 분리돼 있었기 때문이다.

1. `LayoutEngine::cell_units()`는 위 gate가 참일 때 row 4 child를 복수 unit으로 만든다.
2. 반면 `MeasuredTable::is_row_splittable()`는 `line_heights.len() > 1` 또는
   `nested_split_row_count > 1`만 인정한다. 후자는 의도적으로 **2행 이상** 중첩 표만 기록한다.
   p842 child는 1×1이므로 measured 값은 1이고, typeset scan은 `advance_row_cut()`를 호출하지
   않은 채 `UNSPLITTABLE`로 종료한다.

따라서 cell-unit을 더 잘게 만드는 것만으로는 owner가 바뀌지 않는다. 다음 보정은 tolerance,
page body height, 혹은 `MeasuredTable`의 일반 1×1 규칙을 바꾸지 않는다. 같은 native HWP5
short-parent-child 저장 구조이며 **실제로 non-spacer unit이 둘 이상인 행만** `RowBreak` scan에서
split 가능으로 승격한다. 구현 전 source gate도 parent 선언 높이보다 child physical height가 큰지를
명시적으로 확인하도록 한 곳에 모은다. 이 조건은 p831의 정상 declared-tail parent를 제외하는
필수 안전장치다.

예상되는 결과는 p81이 row 0..3 뒤 34.9px 잔여에 row 4 label과 첫 child unit을 보유하고,
p82가 다음 child unit부터 재개하는 것이다. `advance_row_cut()`의 기존 최소-top-keep과 hard-break
규칙은 그대로 사용한다. 구현 뒤 검증 순서는 (1) `issue_2430` 2/2, (2) 새 p81/p82 RenderTree
owner assertion, (3) `issue_3820_rowbreak_rowspan_band`, (4) 한컴 PDF 직접 비교다.

## 첫 split-eligibility 보정의 반증 결과

중앙 predicate에 parent-declared-vs-child-physical 조건과 multi-unit 확인을 넣고 scan에서
`MeasuredTable` 결과와 OR한 뒤, 먼저 `issue_2430_cell_rewrap_threshold`를 실행해 2/2 통과를
확인했다. 이어 현재 release-test binary로 한컴 PDF를 직접 재대조했다.

```text
p81 raster diff 16.89%
p82 raster diff  4.84%
p81→p82 rhwp_later_than_reference 29 chars
근거설명○구내운반차안전조치를통해근로자와부딪히는등의사고
```

즉 owner 이동은 변하지 않았다. 이 결과만으로는 predicate가 false인지, unit은 생성됐지만
`advance_row_cut()`가 34.9px에서 첫 unit을 거절했는지 구분할 수 없다. 다음 단계는 새
진단 출력으로 **predicate / non-spacer unit count / scan cut 결과**를 각각 관측하는 것이다.
그 관측 전에는 minimum-top-keep, hard-break, page height를 변경하거나 회귀 assertion을 추가하지
않는다.

## cut 결과: 전역 orphan 하한이 아닌 painted fragment 판정 누락

`RHWP_DIAG_SCAN=1`으로 p81 render-tree를 다시 생성해 다음을 확인했다.

```text
TABLE_SPLIT_AVAIL pi=842 ... avail_for_rows=116.1
DIAG_SCAN CUT_TRY r=4 budget=31.9 padding=3.0 consumed_h=22.7 \
  fully=false end_cut=[1, 2]
TABLE_SPLIT_RESULT pi=842 ... end_row=4 consumed=81.2
```

따라서 새 eligibility는 실제로 true이고 `advance_row_cut()`도 첫 child fragment를 선택한다. 그러나
`row_split_meets_min_top_keep()`은 기본 content-only 25px 하한과 `consumed_h=22.7px`를 비교해
이를 거절한다. `row_cut_content_height()`가 그리는 조각은 visible cell padding(3.0px)을 포함하므로
PDF에서 실제로 남는 border+label+첫 child line은 이 content-only 수치보다 크다.

수정 범위는 이미 #3738에서 검증한 painted-height orphan 판정을 이 **동일한
`native_short_parent_child_row_is_fragmentable()` 구조**에만 확장하는 것이다. 일반 RowBreak,
일반 1×1 child, hard-break reset, page-height tolerance에는 적용하지 않는다. 그 결과도
`advance_row_cut()`의 기존 `end_cut=[1,2]`를 사용하므로, 새 분할 알고리즘이나 임의 bleed가 아니라
이미 계산된 PDF-소유 첫 조각을 수용하는 변화다.

## painted-height 보정 뒤의 child continuation 재개 결함

보정 뒤 `fidelity_compare`의 p81→p82 owner-shift candidate는 사라졌지만, 이것만으로 정합을
선언할 수 없다. PDF text layer와 RHWP RenderTree를 직접 확인한 결과는 다음과 같다.

| 대상 | p81 | p82 |
| --- | --- | --- |
| 한컴 2024 PDF | `근거설명 ○ … 등의 사고` | `를 예방함으로써 …`에서 재개 |
| 현재 RHWP | `근거설명 ○ … 등의`에서 잘림 | `○ … 등의 사고를 예방함으로써 …`를 다시 그림 |

현재 p81의 parent row 4 bbox는 25.7px이고 p82 child는 첫 bullet line부터 재시작한다. 즉
`end_cut=[1,2]`는 scan에서 선택됐지만, nested child의 continuation `start_cut`으로 그 semantic
소비 범위가 정확히 전달되지 않았다. p81→82 owner candidate가 비어도 `text-report.tsv`는 p81에
`사고` 2자 reference-only, p82에 첫 bullet line 25자 SVG-only를 기록한다. raster도 p81 17.32%,
p82 4.32%로 끝나지 않았다.

다음 조사는 cell-unit의 mixed-nested fragment 순서와 `PartialTable` render에서 `start_cut`을 child
source range에 적용하는 경로만 대상으로 한다. 문단 폭, font, parent row height, 혹은 orphan
threshold는 다시 바꾸지 않는다. 회귀는 “p81에는 `…등의 사고`, p82에는 `를 예방함으로써`만”이라는
두 소유 조건을 동시에 고정해야 한다.

`mixed_nested_split_from_cut()`은 p82 continuation의 `content_offset=22.7px`를 계산해 child에
전달하지만, scalar 1×1 child layout은 `split_terminal=true`면 마지막 tail 유실 방지를 위해
fragment source cut을 전부 끈다. 이 p842 continuation은 terminal이면서도 **앞 조각에서 이미
source unit을 소비한** 형태라 그 일반 예외와 다르다. 보정은 `NestedTableSplit`에
`force_source_start_cut`을 명시적으로 싣고, 바로 이 short-parent-child predicate에서만 true로
만든다. child renderer는 그 flag일 때 `content_offset`에 해당하는 unit을 `start`로 삼아
terminal viewport도 source cut을 적용한다. flag 없는 기존 terminal tail은 종전 동작을 유지한다.

## source-unit 경계 진단 — 다음 수정 전 사실 고정

위의 terminal continuation 보정을 적용한 binary로 다시 source-unit 진단을 실행했다.

```text
DIAG_SHORT_CHILD eligible=true native=true
  parent=(rows=5,h=110.7,wrap=TopAndBottom,vert=Para,break=RowBreak)
  cell=(row=4,span=1,paras=2)
  child=(cols=1,tac=false,cells=1,paras=3,flow=196.4)
DIAG_SCAN CUT_TRY r=4 budget=31.9 padding=3.0 consumed_h=22.7
  fully=false end_cut=[1, 2]
```

이는 다음을 확정한다.

1. short-parent-child predicate 자체는 통과한다. parent의 stored height(110.7px)와 child
   source flow(196.4px)가 다르다는 조건도 실제로 충족한다.
2. p81 row 4에는 콘텐츠 22.7px만 배치되고 cut은 source unit `[1, 2]`에서 끝난다. 따라서
   p81의 마지막 PDF 단어 `사고`가 빠지고 p82의 첫 단어가 `사고`로 남는 것은 paint clip의
   우연한 누락이 아니라, 현재 source-unit 경계가 PDF의 첫 visual line 내부를 가르고 있다는
   증거다.
3. terminal continuation에 start cut을 강제한 뒤 직접 PDF 대조에서 p82의 재paint는 전체
   bullet 첫 줄에서 `사고` 두 글자로 축소됐다. 즉 `content_offset` 전달 경로는 유효하지만,
   `[1, 2]`라는 **종료 cut 자체가 한 단어 이르다**. p81/p82 raster diff는 각각 17.32%/3.96%이며,
   text ledger는 p81 reference-only `사고` 2자, p82 SVG-only `사고` 2자를 기록했다.

따라서 다음 변경 후보는 page budget, row height, terminal clip 일반 규칙이 아니다. 먼저
`nested_table_mixed_fragment_heights()`가 이 child의 첫 PDF visual line을 왜 둘 이상의 unit으로
나누는지(저장 vpos reset, hard break, paragraph fragment 중 무엇인지)를 source-unit별 height와
text로 확인한다. 그 결과가 없으면 `end_cut`을 한 unit 넓히는 보정은 다른 child에 blank/중복
또는 진짜 overflow를 만들 수 있으므로 금지한다.

### source-unit 상세 결과 — cut이 아니라 Korean wrap 경계

`RHWP_DIAG_MIXFRAG=구내운반차 안전조치`의 unit 원장은 child 첫 문단을 3개의 ordinary
line unit으로 기록했다. 각 높이는 20.80px이며 hard/stored break는 모두 false다. outer row 4의
`end_cut=[1,2]`, `consumed_h=22.7px`는 이 첫 20.80px line에 nested outer margin을 더한 값과
일치한다. 즉 p81은 child의 첫 **완전한 RHWP line unit**을 소유한다.

PDF text bbox와 같은 p81 RenderTree를 96 DPI로 환산해 직접 비교하면 다음과 같다.

| 대상 | 첫 child line의 끝 | 다음 페이지 시작 |
| --- | --- | --- |
| 한컴 2024 PDF | `… 부딪히는 등의 사고` | `를 예방함으로써 …` |
| RHWP | `… 부딪히는 등의` | `사고를 예방함으로써 …` |

따라서 `사고`는 source unit가 아니라 같은 Korean 어절 `사고를` 안의 시각 line-break 위치다.
RHWP는 이 어절을 다음 line으로 통째로 넘기는 반면 PDF는 `사고` 뒤에서 character break한다.
이 사실은 terminal start-cut을 더 넓히거나 outer table budget을 조정해서 고칠 수 없다. 그런
변경은 p81에 아직 존재하지 않는 source unit을 강제로 넣거나 p82 source를 삭제한다.

다음 원인 조사 범위는 `compose_paragraph`의 Hangul break opportunity와 cell inner-width/오른쪽
padding이다. PDF/RHWP의 first-line available width와 `사고를` glyph advance를 함께 측정해,
폭이 충분한데 break opportunity가 없는지, 또는 inner width가 좁은지를 구분한다. 이 결론 전에는
`force_source_start_cut`을 기능 수정으로 승격하지 않고, p81→82의 중복 억제 효과만 가진
진단적 중간 상태로 취급한다.

### Korean-break 설정 확인

동일 child p[0]의 `DIAG_RECOMP` 결과는 `kbu=1`, `char_break=true`다. 즉 source의
`ParaShape 123`은 이미 글자 단위 분할을 요청하며, 현재 composer도 이를 인식한다. 실제
재구성 폭은 cell inner width 474.03px(첫 줄), continuation 445.35px이고 3줄을 만든다.
따라서 `korean_break_unit`을 전역 반전하는 것은 근거가 없고, #2430 보호 대상까지 바꿀 위험이
있다. 다음 관측은 이 설정 아래에서 `split_composed_line_by_width()`가 만든 각 line의 text와
advance다. 그 결과로 (a) char-level loop가 실제 `사고` 전에서 멈추는지, (b) run advance가
PDF보다 큰지, (c) nested-cell padding/indent가 다른 경로보다 좁은지를 분리한다.

### 저장 여백 회귀와 p81 범위의 분리

이번 p81→82 문제를 해결하면서 `510HU` 저장 셀 여백을 전역으로 제거하면 안 된다. 같은
`76076_regulatory_analysis.hwp`의 p34는 이미 기준 PDF로 검증된 반례다.

- `tests/issue_2308_render_normalized_derived_state.rs`의
  `issue_2308_nested_non_tac_table_keeps_saved_horizontal_cell_margin`은 p34의 non-TAC 1×1
  nested table을 `x=213.7px`, `w=487.6px`로 고정하고, 텍스트 오른쪽 끝이 border보다 최소
  6px 안쪽이어야 한다고 검증한다.
- p34의 source도 table 기본 padding `(0,0,141,141)`과 `aim=false` cell 저장 좌우
  padding `510HU`의 조합이다. 이 예외를 끄면 nested text viewport가 우측선까지 넓어져
  사용자 관찰의 “문단이 우측선 침범”이 재발한다.
- p81 child도 같은 저장 형상을 갖지만, 이번 direct PDF oracle은 **p81의 첫 visual line에
  `… 등의 사고`를 남기고 p82가 `를 예방…`으로 시작한다**는 owner/line-wrap 계약이다.
  따라서 p34 보호 규칙의 폐기나 table 폭 전역 확장은 p81의 후보 수정이 아니다.

현재 p81 SVG의 해당 child는 원명 `한양중고딕`으로 조판된다. 이미
`text_measurement.rs`에는 이 원명 전용 PDF space advance(550/1024em) 보정이 있어, 단순
공백 폭을 다시 바꾸는 것도 독립 검증 없이 금지한다. 다음 구현 후보는 다음 둘 중 하나로만
제한한다.

1. parent short-child fragment가 사용할 source-unit line을 PDF와 같은 폭·glyph advance로
   재구성하는 경로를 고친다. 이 경우 p34의 non-fragment nested-cell padding 계약은 그대로
   둔다.
2. 실제로 p81의 row-4 viewport가 p34와 다른 적용 조건을 가져야 한다는 PDF/RenderTree
   증거가 생길 때에만, native HWP5·TopAndBottom·RowBreak·short-child gate 안에서 별도의
   padding context를 만든다. p34의 일반 non-TAC table에는 적용하지 않는다.

다음 코드 수정 전에는 ① `issue_2308_nested_non_tac_table_keeps_saved_horizontal_cell_margin`
보호 조건, ② p81의 `사고` owner, ③ p82의 `를 예방` 시작을 같은 회귀로 함께 고정한다.

### p81 직접 glyph 좌표 대조 — 여백과 공백을 분리할 근거

한컴 PDF의 p81을 `pdftocairo -svg`로 벡터화하고, RHWP SVG 좌표를 같은 point 단위로
환산했다(RHWP SVG는 96dpi이므로 `px × 0.75`). 대상 첫 줄의 순서는 두 출력 모두
`○ 구내운반차 안전조치를 통해 근로자와 부딪히는 등의 …`다.

| 관측 | 한컴 PDF | RHWP 현재 | 차이 |
| --- | ---: | ---: | ---: |
| bullet 시작 | 156.960pt | 163.945pt | RHWP가 +6.985pt 오른쪽 |
| `구` 시작 | 176.640pt | 186.445pt | RHWP가 +9.805pt 오른쪽 |
| 일반 한글 advance | 12.960pt 부근 | 13.000pt | 거의 동일 |
| `차` 뒤 첫 공백 advance | 6.839pt | 9.358pt | RHWP가 +2.519pt 넓음 |
| 현재/기준 line tail | `… 등의` / `… 등의 사고` | `… 등의` | 기준은 두 음절을 더 소유 |

공백은 이 줄에 여러 번 있으므로, 2.519pt의 누적과 9.805pt의 시작 offset을 합치면 단순
한글 glyph 폭 차이가 아니라 **(a) RowBreak child 조각의 좌측/우측 여백 적용과 (b) 이 문단의
공백 advance 적용**이 함께 line decision을 바꾼다는 설명이 성립한다. 반면 한글 glyph advance
자체는 0.04pt 정도로 거의 같아 전역 한글 폭 보정은 근거가 없다.

다음 진단은 이 정확한 문단의 resolved `font_size`, `letter_spacing`, `ratio`, `condense`와
cell padding context를 한 번에 출력한다. 그 결과가 PDF의 6.839pt 공백에 대응하는 설정인지
확인한 뒤, `한양중고딕` 전역 metric을 바꿀지 또는 native short-parent-child fragment 전용
context로 한정할지를 결정한다. 이 진단은 동작을 바꾸지 않는다.

### resolved style와 저장 폭 projection의 결론

진단 결과 p81 첫 줄은 `align=Justify`, `kbu=1`, `condense=0`, `font=한양중고딕`,
`font-size=17.33px(13pt)`, `letter-spacing=0`, `ratio=1.0`이다. 즉 추가 condense·자간·장평이
없으므로 p81의 공백 차이는 source style의 숨은 보정이 아니다. Justify paint는 이미 결정된
첫 줄을 cell 폭에 배분할 뿐, `사고`를 넣을 수 있는 recompose width 자체를 만들지 않는다.

source 구조를 p34와 대조한 결과도 범위를 더 좁힌다.

- p34 `pi=325`의 `근거설명` child는 **13 문단**, p34 `pi=336` child는 1문단이다. 둘 다
  일반 1×1 non-TAC nested-table geometry이며 #2308의 저장폭·510HU margin 계약을 유지해야 한다.
- p81 `pi=842` child만 3문단이고, native `TopAndBottom + Para + RowBreak` parent의 마지막
  행이며, control-only host 뒤에 reset-only 빈 문단이 있다. 이는 이미
  `native_short_parent_child_fragment_eligible()`가 분리한 short-child 구조다.
- p81 parent 우측 cell의 저장 폭은 38,245HU, child 저장 폭은 36,572HU(약 95.6%)다. 현재
  `RenderNormalizationOverlay`의 `NESTED_STRETCH_MIN_RATIO=1.0`은 near-fit child를 항상
  저장폭으로만 조판·가운데 배치한다. 그 결과 첫 줄의 실제 recompose inner width는 474.03px이고,
  margin을 제거해도 관측상 487.63px에서 `사`까지만 들어간다.

PDF glyph start(156.960pt)는 parent right cell의 좌측 경계와 510HU의 small margin을 유지한
위치와 일치하고, PDF line tail은 parent cell 폭을 쓰는 경우에만 `사고`를 수용할 여유를 얻는다.
따라서 다음 기능 수정 후보는 **일반 near-fit projection의 복원**이 아니라, 위 short-child
구조에만 child effective width를 owner cell width로 투영하는 것이다. 이것은 p34 13문단 child와
무관하며, p81/p82 source-unit cut이 같은 projected width로 측정·paint되어야 한다.

구현 전 추가 계약:

1. projection gate는 `RowBreak`, `TopAndBottom`, `VertRelTo::Para`, 마지막 non-rowspan host,
   1×1 non-TAC child, control-only host/reset-only tail, child paragraphs `<=3`,
   `source_width < owner_width` 및 95% 이상 near-fit을 모두 요구한다. p34의 1문단 child도
   이 조건 일부를 만족하므로, parent/child stored viewport 관계로 추가 분리가 필요하다.
   **초기 3배 가설은 아래 p33 반증에서 폐기했다**. 최종 조건은 `child.common.height >
   owner.common.height`이며, 실제 값은 p842 12,846HU > 8,304HU, p34 `pi=336`
   9,350HU <= 19,400HU다.
2. p34의 saved-width/margin test와 p81 p82 owner assertion을 같은 focused fixture test에 둔다.
3. `NESTED_STRETCH_MIN_RATIO` 일반 상수는 되돌리지 않는다. 이 구조 밖의 near-fit table은
   여전히 저장 폭을 보존한다.

### 첫 width-projection 후보의 회귀 반증

위 contract를 구현한 직후, 사용자 지정 first Cargo gate는 통과했다.

```text
issue_2430_cell_rewrap_threshold
2 passed; 0 failed
```

새 RenderNormalization 단위 test도 generic near-fit / long-owner 반례 / short-owner positive를
5/5로 통과했다. 그러나 같은 source fixture의 기존 PDF geometry gate는 실패했다.

```text
issue_2308_saved_nested_width_keeps_fragment_geometry
page 33 nested fragment: expected y=351.1 h=649.3,
got y=377.8 h=649.3
```

이는 p34 `pi=325` 13문단 child 및 p34 `pi=336` long owner만 제외하는 것으로 충분하지
않음을 뜻한다. p33에서 같은 height를 유지한 채 시작 y만 +26.7px 이동했으므로, candidate가
다른 preceding RowBreak parent의 line/fragment height에 영향을 주었을 가능성이 높다. 이 결과는
새 projection의 정당화가 아니며, 현 단계에서 성공 기준을 충족하지 못했다.

다음 수정 전에는 overlay가 실제 projection한 **source owner paragraph, host row, child paragraph
count, owner/child stored height, near-fit ratio**를 관측해 p33 candidate를 `pi=842`와 분리한다.
관측 결과로 차이를 확정하기 전에는 기준 geometry를 갱신하거나 p33 test를 완화하지 않는다.

첫 관측은 owner paragraph 식별자를 누락한 채 세 candidate만 출력했다.

```text
owner_rows=5 owner_h=19400 child=(paras=1,h=9350,w=36572)
owner_rows=5 owner_h=24456 child=(paras=3,h=14406,w=36572)
owner_rows=5 owner_h= 8304 child=(paras=3,h=12846,w=36572)
```

여기서 `h`는 dump에 보이는 child cell height(3,280HU)가 아니라 nested table의
`common.height`다. 따라서 이전의 `p34 pi=336: 19,400 / 3,280 = 5.91` exclusion 근거는
잘못된 높이 필드를 사용한 것이다. 이 관측만으로 p33 candidate를 특정하거나 gate를 조정하면
안 된다. 다음 관측은 `pi`, child 첫 text, 그리고 table `common.height`를 함께 출력해 세 후보의
source owner를 정확히 식별한다.

식별자를 포함한 재관측 결과는 다음과 같다.

| owner | parent common height | child common height | child 문단 | 판정 |
| --- | ---: | ---: | ---: | --- |
| `pi=336` | 19,400HU | 9,350HU | 1 | p34 long-owner 반례 |
| `pi=511` | 24,456HU | 14,406HU | 3 | **p33 geometry 회귀의 원인** |
| `pi=842` | 8,304HU | 12,846HU | 3 | 목표 p81 |

즉 p33을 움직인 `pi=511`은 parent viewport 안에 child common table height가 들어가고,
목표 `pi=842`만 child common table height가 parent declared height를 넘는다. 이 직접 저장
조건은 `p842`의 fragment 대상 여부와도 일치하며, 앞의 잘못된 “parent가 child의 3배 이내” 조건보다
정확하다. 다음 수정은 **`child.common.height > owner.common.height`** 로만 projection을 좁힌다.
`pi=511`/`pi=336`을 확실히 제외한 뒤 p33 geometry gate와 p81 PDF를 다시 대조한다.

조건 교체 후 overlay 관측은 실제로 `pi=842` 한 건만 출력했다. 그 상태에서도 p33 geometry
gate는 같은 `y=377.8px` 실패를 냈다. p842는 p81보다 뒤의 source paragraph이므로 p33의 위치를
직접 바꿀 수 없다. 따라서 이 gate 실패는 현재 worktree에 이미 있던 height/layout 변경의 baseline
가능성이 있다. 다음 단계에서는 projection을 일시 억제한 동일 worktree gate를 실행해 이 가정을
검증한다. 결과가 어느 쪽이든 p33 assertion을 삭제·갱신하지 않는다.

일시 억제 실행(`RHWP_DISABLE_SHORT_ROWBREAK_WIDTH_PROJECTION=1`)에서도 p33은 완전히 같은
`y=377.8px, h=649.3px`로 실패했다. 따라서 이 source diff만으로는 p33 regression을 만들지
않는다는 반증을 얻었다. 억제 상태에서는 p81 전용 assertion이 예상대로 실패했고, 기본 상태에서는
p81 assertion은 통과했다. 현재 남은 p33 failure는 다른 미커밋 height/layout 변경의 baseline으로
별도 추적하되, full regression을 통과했다고 주장하거나 p33 assertion을 약화하지 않는다.

## Stage 84 수정 전 계약

1. 먼저 p81의 `pi=831` measured row height와 PDF line wrap을 분해하여, p842의 62.3px budget이
   상위 table의 실제 과대 flow인지 혹은 fragment overhead 중복인지 구분한다.
2. 근거가 확정되기 전 `ROWBREAK_*_TOLERANCE`, page body height, row 3 강제 bleed를 변경하지 않는다.
3. 수정 시에는 기준 PDF의 39자 owner를 render-tree assertion으로 고정한다. p81은 row 3과
   row 4의 첫 nested line을 포함하고, p82에는 그 line을 재paint하지 않으며 `사고를 예방함으로써`로
   재개해야 한다.

## 구현 중간 원장 — 측정/배치 대칭과 short child 판정

`fit_measured_table_nested_tail_to_declared_height()`는 p831의 앞 행 경계를 유지한 채
마지막 nested tail만 선언 높이로 맞춘다. 이를 `TypesetEngine`에만 적용하면 pagination은
약 50px을 회수하지만 SVG `LayoutEngine`은 기존 684.9px 높이로 표를 계속 paint한다. 따라서
`resolve_row_heights_with_common_fit()`도 동일한 native HWP5·TopAndBottom·RowBreak·전 행
rowspan=1 gate와 동일 helper를 적용해야 한다. 이 대칭 적용 뒤 p842의 p81 시작 y는 933.7px에서
879.9px로 올라와 row 3까지는 기준처럼 p81 owner가 되었다. 그러나 row 4의 `근거설명`과 child
문장은 여전히 p82에 원자로 남아 자동 후보는 39자에서 29자로만 감소했다. 이는 완료가 아니다.

source dump를 다시 확인한 결과, p842 row 4 우측 cell은 `Table` control만 있는 empty host 뒤에
vpos=0 empty reset paragraph 하나를 갖는다. 1×1 child의 stored `common.height`는 3280HU
(약 43.7px)지만 `nested_table_mixed_fragment_heights()`의 실제 source-unit flow 합은 약 196px이다.
parent의 declared height는 8304HU(약 110.7px)다. 그러므로 `calc_nested_table_height()`처럼 stored
viewport만 사용하면 short-child gate가 false가 되어 child를 atomic으로 남긴다. gate와 paginator의
split-eligibility는 같은 mixed-fragment flow 합을 판단에 사용해야 한다.

이 경로는 다음으로 좁힌다.

- native HWP5의 paragraph-relative TopAndBottom RowBreak parent;
- 마지막, non-rowspan cell의 empty table host와 reset-only trailing empty paragraph;
- non-TAC 1×1 child 중 3문단 이하 short child;
- child source-unit flow가 parent declared height를 넘고 fragment가 둘 이상인 경우.

이 조건은 p831의 7문단 `산식 설명` child를 제외한다. p81의 first child source unit을 p81에
배치하고 p82가 그 다음 source unit에서 재개하는지 다음 binary/PDF comparison으로만 판정한다.
페이지 수, 단순 text count, 또는 기존 SVG는 성공 근거가 아니다.

## HWPCTRL 경계

[`webhwpctrl_compat_development.md`](../manual/webhwpctrl_compat_development.md)를 함께 확인했다.
이번 변경은 existing HWP의 SVG/PDF layout 및 source-unit owner 보정이며, HWPCTRL API scenario나
fixture를 생성·갱신하지 않는다. 따라서 이 단계의 독립 정답은 한컴 2024 PDF다. 향후 HWPCTRL
fixture/Oracle 값을 변경해야 할 경우에만 Windows 한글 2022 COM live Oracle을 문서별 직렬로
실행하고 `oracleMode`를 별도 기록한다. macOS WASM 결과를 새로운 HWPCTRL Oracle로 승격하지 않는다.
4. 사용자 지시대로 미검증 변경 뒤에는 `tests/issue_2430_cell_rewrap_threshold.rs`를 **첫 Cargo
   검증**으로 실행한다. 그 다음 `issue_3820_rowbreak_rowspan_band`, 새 p81/p82 regression,
   PDF 대조 순으로 수행한다.
