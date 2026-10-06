---
kind: analysis
status: in_progress
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-07
---

# Task #3820 Stage 41 — #1921 p11 첫 partial fragment의 Square anchor

## Stage 40 이후의 범위 재설정

Stage 40의 vpos-only flow 대체는 39→38쪽으로 줄였지만 PDF p11의 내용 자체를 훼손해 채택하지
않았다. 이 Stage는 그 변경을 되돌린 상태에서, p11의 **첫** p98 row 2 fragment가 왜 source picture를
자기 cell보다 위에 그리는지를 먼저 분리한다.

## 확정 관측

- p11 fragment의 cursor는 `start_cut=[0]`, `end_cut=[29]`이다. 즉 이 row 2의 앞 fragment는
  이전 page에서 소비한 unit이 없고, continuation origin을 적용할 이유가 없다.
- current render tree에서 physical row 2 cell bbox는 `y=351.9..843.9`이다.
- 같은 cell에 속한 Square image bbox는 `y=123.3..214.8`, `y=332.7..574.2`다. 첫 image는 cell
  위로 228px 이상 빠져 row 0의 동영상/소개 image와 겹친다.
- PDF p11에서는 이 row의 product pictures가 row 2 cell 안에 있다. 따라서 p12/p13의 page-owner
  지연은 먼저 발생한 p11 anchor failure의 누적 결과이며, p12만 억지로 앞당겨서는 해결되지 않는다.

## 코드 경로

이 형상은 일반 `layout_table_cells`가 아니라 `layout_partial_table_cells`를 탄다. 해당 경로는
`cut_units`로 visible paragraph를 선택한 뒤 Square picture에 `anchor_y = para_y`를 주고
`compute_object_position`에 전달한다. `CellUnit`의 generic flow unit을 바꾸는 것만으로는 이 p11
anchor가 고쳐지지 않는 이유다.

## 검증 계획

1. `RHWP_DIAG_CELLPIC` 일시 진단으로 p0/p6의 `para_y_before`, `anchor_y`, vertical align/offset,
   `compute_object_position` 결과와 physical cell bbox를 기록한다.
2. 첫 fragment(`su=0`)에만 적용할 수 있는 origin/clip 오류인지, object vertical-align 해석 오류인지
   구분한다.
3. 원인에 대응하는 최소 수정 후 p11 row2 containment gate를 통과시킨다.
4. 그 다음에만 p12--p13 picture owner와 PDF page content를 다시 대조한다.

진단은 환경변수에 한정하고, 정답 PDF와 맞지 않는 page-count 감소는 성공으로 취급하지 않는다.

## 1차 진단 결과 — source offset과 renderer anchor를 분리

`RHWP_DIAG_CELLPIC=1`로 현재 구현을 p11에 한정해 실행한 결과는 다음과 같다.

| source control | partial cursor | physical cell | paragraph anchor | rendered y | stored vertical offset |
| --- | --- | --- | --- | --- | --- |
| p0 / control 0 | `[0, 29)` | `351.9..843.9` | `352.8` | `332.7` | `-1,507 HU` (`-20.1px`) |
| p0 / control 1 | `[0, 29)` | `351.9..843.9` | `352.8` | `76.5` | `-20,719 HU` (`-276.3px`) |

두 image 모두 `flow_with_text=true`, `Square`, paragraph-relative top anchor다. 따라서 앞서의
"partial renderer가 그림을 cell 위로 올린다"는 관측 자체는 맞지만, 단순한 renderer 산술 오차라고
단정할 수는 없다. source가 보유한 음수 vertical offset이 현재 paragraph anchor에 그대로 적용되고
있다.

이 결과로 다음 가설은 **기각**한다.

- 모든 Square image를 physical fragment cell 안으로 clamp하면 PDF와 같아진다는 가설. p0/control 1은
  source 상 큰 음수 offset을 갖고 있으므로, clamp는 원인 규명 전 이미지 위치만 임의로 바꾸는 조치가 된다.

다음으로 검증할 가설은 하나다.

- RowBreak 첫 fragment가 source의 full-cell vertical alignment origin을 잃고 physical slice top
  (`352.8`)을 paragraph anchor로 사용한다. 이 경우 PDF의 product image 위치와 source offset을 동시에
  설명할 수 있다. 이를 확인하려면 `effective_align`, full resolved row height, slice `inner_height`,
  measured content height를 같은 진단에 기록하고, PDF p11의 각 asset 위치와 대응시켜야 한다.

## 회귀 오라클 보류

현재 추가된 p11 "두 image가 cell 안에 있어야 한다" test는 위 기각된 clamp 가설을 전제하므로 확정
회귀 게이트로 사용하지 않는다. PDF asset 대응과 origin 의미를 확인한 뒤, 실제 정답지의 page/owner를
고정하는 oracle로 교체하거나 삭제한다.

실제로 전체 integration 실행은 이 provisional test에서 자연스럽게 실패했다. 실패 bbox는
`x=123.32, y=332.70, w=561.68, h=91.53`, cell은 `y=351.9..843.9`였고, 이는 위 진단 표와
일치한다. 이 실패를 renderer 회귀로 보고 baseline 또는 production layout을 억지로 바꾸지 않는다.
먼저 test oracle을 PDF의 asset 순서·page owner에 맞게 재정의한다. 이 실행은 여기서 Cargo가
exit 101로 종료되어 이후 integration binaries와 overflow-cell gate까지는 도달하지 못했다.

## PDF asset 대조

한컴 2022 기준 PDF p11을 144dpi로 rasterize하여 source asset과 직접 대조했다. 이 page의
row 2 시작은 PDF 좌표로 약 `y=352px`(96dpi 환산)이며, 현재 partial cell top `351.9px`와
일치한다. 따라서 이 문제는 PDF page를 잘못 맞춘 것이 아니라 **같은 physical row fragment
안의 object origin** 문제다.

| source 순서 | export-doclang asset | native frame | PDF p11에서의 역할 |
| --- | --- | --- | --- |
| p0/control 0 | `block-0.jpg` (`914×79`) | `42126×6865 HU` | product 소개 한 줄 image |
| p0/control 1 | `block-1.jpg` (`969×479`) | `32500×18109 HU` | 제품 3개 image |
| p6/control 7 | `block-2.jpg` (`951×639`) | `32193×17730 HU` | 제품 상자 image |
| p19/control 10 | `block-3.jpg` (`897×55`) | `44584×5141 HU` | 다음 설명 image |
| p31/control 11 | `block-4.jpg` (`962×351`) | `36921×13426 HU` | 다음 page의 JOUZ image |

PDF p11은 첫 세 asset을 모두 row 2 안에 순서대로 보인다. 현재 구현은 p0/control 0을 20px
위로, p0/control 1을 276px 위로 올리고 p6/control 7을 다음 page로 미룬다. 특히 같은 p0의
두 control이 서로 다른 기대 anchor를 요구하므로 cell clip만 고치는 수정도, 한 paragraph에
단일 origin을 더하는 수정도 정답일 수 없다.

다음 조사 단위는 control이 놓인 **문단 내 character/line anchor**다. parser가 두 floating
control의 source anchor를 보존하는지, partial layout이 그 anchor를 line 0으로 평탄화하는지를
확인한다. 보존하지 않았다면 회귀는 renderer의 보정이 아니라 source model 보존부터 고쳐야 한다.

### 2차 진단 — source control 위치는 보존되어 있으나 partial path가 사용하지 않음

확장 진단의 p11 결과는 다음과 같다.

| control | source text position | cell vertical-align | effective align | slice inner height | measured content height | resolved full row height |
| --- | ---: | --- | --- | ---: | ---: | ---: |
| p0/control 0 | 0 | Center | Top | 490.2px | 741.2px | 1087.7px |
| p0/control 1 | 1 | Center | Top | 490.2px | 741.2px | 1087.7px |

이는 parser가 두 control을 모두 position 0으로 잃어버린 경우가 아니다. source에서 빈 p0 안에
control 0과 1의 순서는 각각 `0`, `1`로 보존된다. 그러나 `layout_partial_table_cells`의 floating
picture loop는 두 control 모두에 같은 `para_y_before_compose=352.8`을 넘기며 character position을
전혀 사용하지 않는다. 그 결과 p0/control 1은 첫 image 뒤의 flow anchor를 받지 못하고 source의
`-276.3px` offset만 적용되어 `y=76.5`로 올라간다.

또한 단순히 원래 cell의 Center를 복원하는 것도 정답이 아니다. full-row 중심 보정은
`(1087.7 - 741.2) / 2 = 173.3px`에 불과하여 p0/control 1의 현 위치를 PDF의 제품 3개 image
위치까지 옮기지 못한다. 따라서 다음 수정 후보는 cell-level valign 복원이 아니라, **partial
fragment에서 non-inline Square control의 source character 순서와 preceding picture flow를 어떤
규칙으로 anchor에 반영해야 하는지**를 분리하는 것이다.

## 원인 후보를 코드 경로와 대조

`cell_units` / 높이 측정은 Square·Tight·Through non-inline picture의 frame height를
`cell_non_inline_control_flow_height`로 셀 flow에 포함한다. 반면 partial renderer의 picture loop는
그림을 배치한 뒤 `non_inline_control_flow_height`만 더한다. 이 함수는 TopAndBottom 이외에는
항상 `0px`를 반환한다.

따라서 p0의 순차 control은 pagination에서는 서로 다른 physical unit으로 소비되지만, rendering에서는
둘 다 같은 paragraph anchor에서 계산된다. p0/control 1의 source position `1`도 이 경로에서
사용되지 않는다. 이는 위 PDF/diagnostic과 일치하는 측정-방출 계약 불일치다.

### 제한된 수정 가설

다음 조건을 모두 만족하는 picture만 대상으로 한다.

1. `PartialTable`의 `cut_units`가 있고, 현재 slice가 그 non-inline control unit을 실제로 소유한다.
2. native RowBreak physical fragment다.
3. `flow_with_text=true`, `VertRelTo::Para`, wrap이 `Square|Tight|Through`다.

이 조건에서는 physical unit의 flow anchor를 사용하고, 해당 slice에서 이미 unit으로 소비한
negative vertical offset을 다시 picture position에 적용하지 않는다. 각 picture 뒤에는
`cell_non_inline_control_flow_height`만큼 advance한다. 이 규칙이면 p0/control 0은 row 2 top에서,
p0/control 1은 첫 picture height 뒤에서 시작하며, source unit 소비와 paint owner가 일치한다.

이것은 일반 floating picture의 offset을 삭제하는 변경이 아니다. non-RowBreak, TopAndBottom,
비-flow picture 및 control unit을 소유하지 않은 continuation은 기존 경로를 유지한다.

### 통과 기준

- p11에서 asset 0→1→2가 PDF와 같은 순서로 row 2 physical owner 안에 나온다.
- p12의 JOUZ asset은 p11로 앞당겨지지 않는다.
- 기존 p8 Square gate와 `issue_2430_cell_rewrap_threshold`가 통과한다.
- PDF page count 감소만으로 성공 판정하지 않는다.

### p6 ownership 확인이 선행되어야 하는 이유

같은 p11 slice에서 p6은 paragraph line 자체는 visible(`para_y=467.4px`)이지만
`visible_non_inline_controls=false`여서 product-box image가 전혀 emit되지 않는다. p11 PDF에는
그 asset이 있으므로 p0의 두 control만 고쳐서는 충분하지 않다. 다음 진단은 `cell_units`의
`para_idx`/unit range를 출력해 p6 control unit이 `[0,29)` 밖으로 분류된 이유를 확정한다. 이 분류가
잘못이면 먼저 pagination owner를 고치고, 분류가 맞으면 partial render의 control visibility contract를
고친다.

진단 결과, p6 control unit의 첫 index는 `29`이고 p11 cut은 half-open `[0,29)`다. 따라서 p6을
그리지 않은 renderer visibility 판정은 현재 pagination 계약에는 맞는다. 반대로 p0은 unit
`0..20`을 소비하지만 renderer에서 0px만 advance한다. 즉 p11에 product-box가 없는 직접 원인은
visibility가 아니라 **p0 Square picture의 320px unit 소비를 physical row height/fragment end에
반영하지 못한 pagination geometry**다. p0와 p6 사이의 empty paragraph units까지 포함하면 PDF p11의
필요 높이는 현재 slice `490.2px`보다 크다.

그러므로 다음 구현은 p6의 visibility를 강제로 열지 않는다. 먼저 RowBreak fragment height가
source unit 소비와 같은 기준으로 계산되도록 고치고, 그 결과 p11 cut이 PDF에 맞는 owner 범위를
갖는지 확인한다. 임시 unit 진단 코드는 이 결론을 기록한 뒤 제거한다.

### 3차 조사 계획 — paginator 결정값을 renderer 관측과 분리

여기까지의 증거만으로 `table_partial`의 그림 anchor를 변경하면, 이미 pagination이 확정한
`[0, 29)` owner 범위와 충돌한다. 따라서 다음 계측은 renderer의 좌표가 아니라
`typeset_block_table`의 RowBreak scan 값으로 범위를 한정한다. p11의 table paragraph(`pi=98`)에
대해 각 fragment의 `cursor_row`, 기존 `start_cut`, `avail_for_rows`, scan 결과의 `consumed`,
`end_row`, `split_end_cut`, `split_end_limit`를 기록한다.

판정은 다음 둘 중 하나다.

1. scan 자체가 29번 control unit 앞에서 예산을 소진했다면, p0 Square controls·empty spacer가
   차지한 physical height가 PDF p11의 실제 row fragment와 왜 다른지 **paginator 측정**을 고친다.
2. scan은 p6을 포함해야 할 충분한 height/cut을 산출했는데 후속 단계가 `[0,29)`로 바꿨다면,
   그 후속 owner 변환만 고친다.

이 계측은 환경 변수로만 켜는 기존 `RHWP_TABLE_DRIFT` 진단을 사용하며 동작 변경을 만들지 않는다.
수치가 문서에 기록되기 전에는 renderer 동작 코드를 수정하지 않는다.

### 3차 진단 결과 — cut 측정과 실제 row footprint가 915px 갈라짐

`RHWP_TABLE_DRIFT=1`로 같은 native HWP를 page 11까지 조판한 결과는 다음과 같다.

| row | `cut_row_h` (pagination) | `MeasuredTable` (renderer footprint) | 차이 |
| ---: | ---: | ---: | ---: |
| 0 | 447.6px | 257.8px | +189.8px |
| 1 | 18.4px | 18.4px | 0.0px |
| 2 (문제 cell) | 1814.7px | 1087.7px | +727.0px |
| 3 | 19.4px | 19.4px | 0.0px |
| 4 | 734.9px | 736.7px | -1.8px |
| 5 | 18.4px | 18.4px | 0.0px |
| **합계** | **3053.5px** | **2138.5px** | **+915.0px** |

p11 첫 fragment에서 paginator는 `avail_for_rows=969.4px` 안에 `end_row=3`,
`consumed=958.1px`, `split_end_limit=490.2px`, `end_cut=[29]`를 확정했다. 이 값은
row 0을 447.6px로 소비한 뒤 row 2에 490.2px만 남긴 결과다. renderer가 실제로 paint하는
row 0 footprint는 257.8px이므로, 이 단계만 renderer와 같은 물리 기준으로 맞춰도 row 2의
가능한 slice는 약 189.8px 커진다. PDF p11에 p6 product-box asset까지 필요한 추가 높이와
동일 차수다.

이는 두 별개 문제를 분리한다.

* **owner 경계:** paginator의 `cut_row_h`가 physical `MeasuredTable`보다 row 0·2에서 크게
  잡혀 p6 unit 29를 p12 owner로 넘긴다.
* **paint anchor:** owner인 p0 Square units 0..20도 partial renderer가 0px advance하여 서로
  같은 anchor에서 paint한다.

따라서 수정 순서는 (1) native HWP5 saved-rewind RowBreak에서 `cut_row_h`를 whole-row fit과
partial-cut에 각각 어떤 physical contract로 써야 하는지 확정하고 owner를 바로잡은 뒤,
(2) 실제로 owner가 된 Square controls의 순차 flow anchor를 renderer에 반영하는 것이다.
두 사항을 한 조건문으로 묶거나 p6 visibility를 강제하면 PDF와 반대인 page owner가 생긴다.

다음 분석은 `native_hwp5_rewinding_rowbreak_uses_painted_row_footprint`가 이 입력에서
`cut_row_h`의 큰 값을 선택한 근거와, #3820의 기존 fixtures에서 그 큰 값을 필요로 한 정확한
조건을 비교한다. 그 비교 전에는 측정값을 낮추는 코드를 적용하지 않는다.

### #3820 보정과의 분리

코드 경로를 다시 대조한 결과, 이 입력의 큰 `cut_row_h`는 #3820 보정이 새로 선택한 값이
아니다. #3820의 native-rewind 조건은 whole-row fit에만 `max(cut_row_h,
MeasuredTable.row_height)`를 쓰며, 59043 p11에서는 모든 문제 행에서 이미 `cut_row_h`가
`MeasuredTable`보다 크다(행 0: 447.6 > 257.8, 행 2: 1814.7 > 1087.7). 따라서 `max`의
결과는 보정 전후 모두 `cut_row_h`다.

즉 #3820을 끄거나 기존 3820 fixture의 physical-footprint 경계를 되돌리는 것은 p11을 고치지
않고 p94/p106 회귀만 유발한다. 이 Stage의 수정 범위는 **Square flow control을 포함한
`advance_row_cut`/`row_cut_content_height`와 partial picture paint 사이의 측정-방출 불일치**로
한정한다.

## 구현 설계 — 숫자 불변 control-owner 메타데이터

`CellUnit`의 현재 16px Square fragment는 높이와 paragraph index만 보존한다. 이 때문에 p0처럼
둘 이상의 non-inline control이 있는 문단에서는 partial renderer가 "p0의 unit 하나가 보인다"는
사실만 알고 두 그림 모두를 emit한다. 먼저 pagination의 높이·unit 개수·cut index는 변경하지
않고 다음 메타데이터만 추가한다.

1. 각 generic Square/Tight/Through fragment unit에, 그 16px 구간과 겹치는 source control index의
   inclusive range를 기록한다. 한 fragment가 두 control의 경계를 가로지르면 둘 다 기록한다.
2. paragraph 내 Square control의 세로 flow interval은 기존
   `cell_non_inline_control_flow_height`의 누적값으로 계산한다. TopAndBottom atomic unit과 inline
   control은 이 범위에 넣지 않는다.
3. partial/full fragment renderer는 기존의 paragraph-level
   `cell_cut_contains_non_inline_control_units` 대신 `(paragraph, control)`의 **첫 source
   unit**을 owner로 확인한다. 따라서 control의 뒷 generic unit만 가진 continuation은 같은
   picture를 다시 paint하지 않으며, p11 `[0,29)`과 p12 `[29,82)`는 각자 시작 control만
   paint할 수 있다.

이 설계는 16px chunk를 control별로 재분할하지 않는다. 재분할은 누적 높이가 같아도 cut index와
half-open 경계를 바꿔 #2007/#2430/#3820의 pagination 결과를 숨은 방식으로 바꿀 수 있다. source
control range만 덧붙이면 첫 단계는 pagination 수치 불변이며, owner 판정의 근거만 검증 가능해진다.

control별 owner가 render tree에서 확인된 다음에만, 같은 owner 범위 안의 Square picture anchor를
기존 `cell_non_inline_control_flow_height` 누적 flow와 일치시키는 2차 수정으로 간다. 이 순서를
지켜야 한 control이 두 page에서 중복 paint되거나, p6을 cut 밖에서 강제로 보이는 오류를 막는다.

## 시각 재확인

같은 실행에서 만든 current SVG p11과 Hancom 2022 PDF p11 raster를 다시 직접 확인했다.
current p11은 video와 제목 뒤의 row 2가 거의 빈 frame으로 끝난다. 정답 PDF p11은 같은 frame 안에
제품 3개 사진과 제품 상자 사진, 그 뒤 설명까지 연속해서 보인다. 따라서 이 Stage의 기준은
이미지 bbox containment 같은 내부 구조 조건이 아니라, **PDF의 control 순서와 page owner를 함께
만족하는 것**이다. 현재 `regulatory_59043_page11_square_pictures_stay_in_row2_fragment`은
두 image의 cell containment만 가정한 provisional gate이므로, control identity가 보존된 뒤 PDF가
보이는 control sequence를 명시하는 gate로 교체해야 한다.

### control-unit 범위 확인

일회성 `RHWP_DIAG_59043_CONTROL_UNITS` 결과에서 p11 cut `[0,29)`과의 관계는 다음과 같이
확정됐다.

| source control | generic unit range (inclusive) | p11 owner |
| --- | --- | --- |
| p0/control 0 | 1–6 | 포함 |
| p0/control 1 | 6–20 | 포함 (unit 6은 경계 겹침) |
| p6/control 0 | 29–41 | 제외, p12 시작 |
| p19/control 0 | 68–70 | p12 |
| p31/control 0 | 93–103 | p13 |

따라서 p11에서 p0의 두 control은 이미 **같은 fragment의 정당한 owner**다. 현재 두 번째
control이 보이지 않는 직접 원인은 source negative vertical offset을 같은 paragraph anchor에
그대로 적용하고, 첫 Square control 뒤에 0px만 advance하는 renderer다. 다음 수정은 p11의
owner인 Square controls에 한해 `cell_non_inline_control_flow_height`만큼 순차 전진하고, 그
physical anchor에는 saved negative offset을 다시 적용하지 않는다. p6은 여전히 p12 owner로
남겨 pagination-owner 분석에서 별도로 다룬다. 위 환경변수 진단은 이 결과를 문서화한 뒤 제거한다.

## 구현·검증 결과 — p0 owner의 paint contract만 수정

위 owner 메타데이터를 이용해, 현재 `cut_units`가 실제로 소유한 `flow_with_text + Square`
picture에만 다음 두 동작을 적용했다.

1. physical fragment의 page-local `picture_anchor_y`를 picture anchor로 사용한다. source의
   음수 `vertical_offset`은 이전 source ladder를 기준으로 저장되어 있으므로 이 fragment에 다시
   적용하지 않는다.
2. picture를 emit한 뒤 기존 pagination이 사용한
   `cell_non_inline_control_flow_height`만큼 paragraph anchor를 전진한다. 따라서 같은 p0 안의
   다음 control은 첫 control과 같은 y에서 paint되지 않는다.

CellUnit의 수·높이·순서·cut index는 바꾸지 않았다. `p0/control 0`과 `p0/control 1`만 p11
cut `[0,29)`에서 정당하게 paint되고, `p6/control 0`은 여전히 cut 밖이므로 p12에 남는다.
일회성 `RHWP_DIAG_CELLPIC`도 관측값을 남긴 뒤 제거했다.

### 결과 확인 (2026-08-07)

- 최신 `release-test` binary로 `issue_2430_cell_rewrap_threshold_no_oversplit` 통과
  (`1 passed, 0 failed`).
- 최신 #1921 binary로 `regulatory_59043_page11_square_pictures_stay_in_row2_fragment` 통과
  (`1 passed, 0 failed`). 이는 p0 두 image가 p11 row 2 cell 안에 있음을 확인한다.
- 같은 binary로 p8의
  `regulatory_59043_page8_square_picture_stays_in_its_table_cell`도 통과
  (`1 passed, 0 failed`).
- p8도 Hancom 2022 PDF p8와 96dpi raster로 직접 대조했다. 2×3 사진 표의 여섯 image와
  caption은 같은 cell에 유지되어, 이번 partial Square anchor 변경으로 해당 기존 형상이
  cell 밖으로 다시 빠지지 않았음을 확인했다. 이미 physical containment gate가 있으므로
  중복 p8 gate는 추가하지 않는다.
- 최신 `rhwp export-svg --page 10` raster를 Hancom 2022 PDF p11과 직접 대조했다. 이전의
  거의 빈 row 2와 달리 video/title 뒤에 제품 3개 image가 cell 안에 복원됐다.

그러나 PDF p11에는 이어서 제품 상자 image와 설명도 같은 page에 존재한다. 현재 rhwp에서는 이
asset이 p12 상단에 단독으로 남는다. 따라서 p11 **anchor 결함은 해소**됐지만, p6 owner가
`[29,41]`로 시작해 p11 half-open cut `[0,29)` 밖으로 밀린 **RowBreak owner/height 결함은
잔존**한다. p11 gate의 두 image containment는 이번 수정의 최소 회귀 오라클로 유지하되,
PDF-visible asset sequence(0, 1, 2)를 검증하는 최종 gate로는 아직 충분하지 않다.

다음 Stage는 이 남은 owner 문제만 다룬다. #3820의 `max(cut_row_h, measured)` 보정은 이 입력의
값을 바꾸지 않는다는 위 측정 결과를 재확인하며, Stage 42의 별도 overflow-cell 작업과 범위를
섞지 않는다.
