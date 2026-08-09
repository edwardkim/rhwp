---
kind: analysis
status: in_progress
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-07
---

# Task #3820 Stage 43 — #1921 p11 RowBreak owner geometry

## 이 Stage로 이월한 사실

Stage 41은 p11 첫 partial fragment의 paint-anchor 결함을 분리·수정했다. p0의 두 Square
picture는 PDF와 같은 row 2 cell 안으로 복원됐고 `issue_2430` 및 #1921 p8/p11 focused gate도
통과했다. 그러나 Hancom 2022 PDF p11의 세 번째 asset(제품 상자)과 그 설명은 rhwp p12에
남아 있다. 즉 남은 결함은 picture paint가 아니라 **page owner 결정**이다.

| source control | source unit range | current owner | Hancom PDF owner |
| --- | ---: | --- | --- |
| p0/control 0 | 1–6 | p11 | p11 |
| p0/control 1 | 6–20 | p11 | p11 |
| p6/control 0 | 29–41 | p12 | p11 |
| p19/control 0 | 68–70 | p12 | p12 이후 |

현재 p11 cut은 half-open `[0,29)`이다. 따라서 p6 control을 renderer visibility에서 강제로
그리면 같은 asset을 p11/p12 양쪽에서 emit하거나 다음 fragment owner를 깨뜨린다. 이 Stage는
그 우회책을 사용하지 않는다.

## 이미 기각한 원인

- #3820의 `max(cut_row_h, MeasuredTable.row_height)`를 제거하는 방법: 이 입력은 이미
  `cut_row_h`가 measured height보다 크므로 선택 값이 달라지지 않으며, #3820 p94/p106 회귀만
  만든다.
- p6의 visibility만 p11에서 강제로 여는 방법: half-open unit owner와 충돌하고 p12 중복 paint
  위험이 있다.
- p0의 saved negative offset을 복원하는 방법: Stage 41에서 이미 PDF와 반대인 cell 위 paint를
  재현한다.

## 1차 scan 기록 — p6 첫 source unit 직전의 4.7px 경계

`RHWP_TABLE_DRIFT=1 RHWP_DIAG_SCAN=1 RHWP_CUT_DBG=1`으로 최신 source를 p11까지
조판해 `pi=98`만 추출했다. 이 계측은 동작 변경 없이 현재 cut을 설명한다.

| 항목 | 값 |
| --- | ---: |
| p11 row-area 가용 높이 | 969.4px |
| row 0 + row 1의 페이지 소비 | 447.6px + 18.4px |
| row 2의 cut 예산(패딩 제외) | 501.5px |
| p11 row 2 실제 소비 | 490.2px (`end_cut=[29]`) |
| p6 첫 generic control unit | index 29, 16.0px |
| index 29까지 포함했을 때 논리 소비 | 506.2px |
| 현 예산 초과 | 약 4.7px |

따라서 p6가 p12로 넘어간 직접 계기는 16px chunk 하나가 **4.7px** 모자라기 때문이다. 하지만
그것만 보고 전역 RowBreak tolerance를 넓히면 안 된다. HWPX에는 이미 64px tolerance가 있고
native HWP5는 2px인 이유가 #3820/overflow fixture의 physical bottom 보호이기 때문이다.
이 입력이 Hancom PDF에서 4.7px를 수용하는 source-local 조건을 찾아야 한다.

또 하나의 필수 조건이 확인됐다. p6의 control range는 `29–41`이라 p11이 index 29를
소유하면 p12의 `[30, …)`도 같은 range와 겹친다. 현재 Stage 41의 overlap 기반 owner helper를
그대로 사용하면 p6를 p11과 p12 양쪽에서 emit할 수 있다. owner semantics는
"range와 겹치는 모든 fragment"가 아니라 **control range의 첫 unit을 포함한 fragment가
picture를 emit한다**여야 한다. p0 control 0/1은 모두 p11에 첫 unit이 있어 기존 Stage 41
결과를 보존한다.

## 정답지 재대조 — 4.7px 허용만으로는 완료가 아님

이 Stage의 비교 기준은 `samples/issue1921/59043_regulatory_analysis.hwp`와 같은 문서를
Hancom 2022로 출력한 PDF다. PDF p11과 p12를 각각 rasterize해 다시 확인했다.

| 페이지 | Hancom PDF의 경계 | 현재 rhwp의 경계 | 판정 |
| --- | --- | --- | --- |
| p11 | p0의 소개·제품 3개 image, p6의 제품 상자, 이어지는 설명까지 같은 row 2 fragment | p0의 두 image만 p11; 제품 상자·설명은 p12 | 미해결 |
| p12 | JOUZ header/다음 image에서 시작 | 제품 상자·설명으로 시작하고 JOUZ는 더 뒤로 밀림 | 미해결 |

그러므로 `index 29`의 16px만 p11에 억지로 넣는 전역/문서별 tolerance는 정답이 아니다. p6
control의 첫 generic chunk는 16px이지만, 그 control은 실제 image frame을 paint하고 뒤의
설명·다음 control의 page owner까지 연쇄시킨다. 우연히 4.7px를 넘겨 p6만 p11에 보이게 해도
p12의 시작이 PDF와 같다는 보장은 없다.

이 재대조로 다음을 확정한다.

1. Stage 41의 paint-anchor 수정과 control-entry owner 규칙은 **필요조건**이다. 같은 control이
   continuation page에서 재emit되는 것을 막고 p0 두 image의 physical anchor를 복원한다.
2. 그러나 실제 owner를 p11/p12 경계에 맞추는 것은 `advance_row_cut`가 사용하는 source-unit
   geometry와 renderer가 paint하는 physical geometry의 불일치 문제다.
3. 다음 수정은 row 0/2의 과대 cut footprint가 어떤 paragraph·spacer·control unit으로 구성되는지
   일반 계측으로 확인한 뒤에만 한다. `min(cut, measured)`, global tolerance 증가, p6 visibility
   강제는 모두 금지한다.

## 구현 전 계측 계약

기존 `RHWP_CUT_DBG`는 unit 높이·vpos range만 보여 source owner를 판별하기에 부족하다. 다음
계측은 동작을 바꾸지 않는 환경변수 출력으로 한정한다.

- row/cell별 `CellUnit`의 **index**, `para_idx`, height, spacer/hard-break 상태,
  `non_inline_control_range`를 함께 기록한다.
- `pi=98`이라는 입력별 조건은 넣지 않는다. 어떤 HWP5 RowBreak table에서도 같은 provenance를
  확인할 수 있는 일반 진단이어야 한다.
- 이 출력으로 p0, p6, p19, p31의 첫 generic unit과 그 앞 text/spacer의 누적 높이를 표로
  남긴 뒤, 실제 geometry 보정의 eligibility를 정의한다.

그 계측 전에는 stage 41의 `CellUnit` metadata가 page owner를 고친다는 사실과 geometry를
고친다는 사실을 혼동하지 않는다. metadata는 paint owner 판정용이며, 단위 높이·수·cut
half-open 경계는 아직 그대로다.

## 2차 scan — `pi=98`의 과대 footprint를 만든 빈 문단 사다리

계측을 source control과 함께 다시 실행한 결과, p11의 row 2 `CellUnit`은 control 자신만이
아니라 **내용·control이 모두 비어 있는 문단**에도 각각 19.1px을 소비하고 있었다. 다음 표의
index는 debug 출력의 원래 half-open unit index다.

| source paragraph | unit 범위 | 현 높이 | 앞·뒤 meaningful 문단 | 해석 |
| --- | ---: | ---: | --- | --- |
| p0 | 0–20 | 337.4px | 시작 / p6(Square control) | text + Square controls 2개 |
| p1–p5 | 21–26 | 114.6px | p0(Square) / p6(Square) | 연속 빈 spacer 5문단 |
| p6 | 27–41 | 245.3px | p0(Square) / p19(Square) | text + Square control |
| p7–p18 | 42–65 | 458.4px | p6(Square) / p19(Square) | 연속 빈 spacer 12문단 |
| p19 | 66–70 | 77.4px | p6(Square) / p31(Square) | text + Square control |
| p20–p30 | 71–91 | 401.1px | p19(Square) / p31(Square) | 연속 빈 spacer 11문단 |
| p31 | 92–103 | 179.0px | p19(Square) / 다음 content | text + Square control |

`p1–p5`, `p7–p18`, `p20–p30`는 모두 `text.trim().is_empty()` 이고 `controls.is_empty()`이며,
각 run의 양쪽 meaningful paragraph에는 Square/Tight/Through 비인라인 flow control이 있다.
이 run의 974.1px은 `MeasuredTable`의 2,138.5px과 row-cut 3,053.5px의 915px 차이와 같은
방향의 과대치다. 특히 현재 rhwp p12에서 p6 제품 상자와 p19 설명 사이에 보이는 큰 공백은
`p7–p18` 458.4px과 일치한다. Hancom PDF p11에는 그 공백이 없고, 제품 상자와 설명이
연속되어 있다.

### 수정 가설 H1 — native HWP5 RowBreak float-ladder spacer

저장 HWP5의 이 패턴에서 빈 문단 사다리는 실제 줄바꿈 공간이 아니라, 문단 기준 비인라인
개체를 저장하는 legacy anchor이다. 행 cut geometry에 그것을 일반 em 줄로 더하면 다음
control owner가 한 페이지 늦어진다. 다음처럼 **모두** 만족할 때만 unit 높이와 visibility를
0으로 만든다.

1. native HWP5 profile의 비-TAC `RowBreak` table이다. HWPX·CellBreak·인라인 table에는
   적용하지 않는다.
2. 빈 문단 run의 길이가 2 이상이다. 저자가 의도한 단일 빈 줄은 보존한다.
3. run의 양쪽 가장 가까운 meaningful paragraph가 Square/Tight/Through 비인라인 control을
   각각 하나 이상 가진다.
4. 셀에는 visible content가 있다. 순수 빈 셀은 이 규칙의 대상이 아니다.

이는 `pi=98`이나 파일명에 의존하지 않는 source-구조 predicate다. H1이 맞다면 row 0의
과대 소비도 함께 줄어 row 2의 p6/p19 entry가 p11에 남고, p12는 p31(JOUZ)부터 시작한다.
반대로 p11/p12 PDF 경계가 맞지 않거나 focused gate가 깨지면 이 변경을 유지하지 않고 이
Stage에 반증을 기록한다. 넓은 tolerance·파일별 보정은 여전히 금지한다.

## H1 실행 결과 — spacer는 원인이지만 충분조건이 아님

H1 적용 후 `issue_2430_cell_rewrap_threshold`는 1/1 통과했다. #1921의 기존 pin은 두 값이
바뀌어 실패했다.

| 관측 | 적용 전 | H1 적용 후 | Hancom PDF | 판정 |
| --- | ---: | ---: | ---: | --- |
| 문서 page count | 39 | 38 | 37 | 개선됐으나 완료 아님 |
| p11 row 2 image node 수 | 2 | 3 | p0 2개 + p6/p19 content가 완전히 같은 fragment | 부분 개선 |
| p12의 선행 내용 | p6 제품 상자·설명 | p19 설명 두 줄이 남음 | JOUZ header부터 | 미해결 |

실제 raster 대조에서 H1 후 p11에는 video, 제품 3개, 제품 상자의 **상단**이 들어온다. 그러나
제품 상자는 cell 하단 clip에 잘리고, 그 설명 두 줄은 p12의 맨 위에 남는다. Hancom PDF p11은
제품 상자 전체와 설명을 모두 가지며, PDF p12는 JOUZ header부터 시작한다. 따라서 H1은
가짜 458px 공백을 제거했지만, physical row fragment가 source control을 중간에서 잘라
renderer가 전체 control을 emit하는 문제를 드러냈다.

동일 scan에서 p11 row 2 cut은 `[0, 36)`으로 변했다. p6 control의 generic unit range는
`[29, 42)`이므로 p6가 **중간까지만** cut에 포함된다. Stage 41의 entry-owner 규칙은 첫 unit
29가 포함된 p11에서 p6 전체 image를 한 번만 emit한다. 하지만 cut geometry는 36에서 끊겨
cell clip이 image 하단을 자른다. renderer owner와 row-cut geometry의 atomicity가 다르다.

### 다음 가설 H2 — RowBreak cut은 source control의 entry가 아니라 complete range를 소비해야 함

H1으로 남은 유효 row 2 budget 501.5px에서 p6 전체 range까지의 누적은 583px이다. p11에 p6를
entry 단위만으로 emit하면 불완전 image가 생긴다. 따라서 다음 계측은 각 row의 full
`row_cut_content_height`가 `MeasuredTable`보다 왜 큰지와, p6/p19 range의 **끝**까지 소비할
공간이 어느 앞 row의 과대 cut에 묶였는지를 확인해야 한다.

바로 "control entry를 보이면 full range를 허용"하는 변경은 금지한다. 실제 cell fragment
height를 넘겨 page/footnote와 겹치거나 p12 소유를 빼앗을 수 있다. 먼저 row 0 및 row 2의
cell별 unit 합·measured height·control range를 하나의 일반 diagnostic으로 기록하고, Hancom
PDF p11의 full p6/p19와 p12 JOUZ 경계를 만족하는 physical capacity 규칙을 도출한다.

## H2 계측 결과 — p11의 capacity를 가로막은 것은 row 0의 병렬 Square 합산

`RHWP_CUT_GEOMETRY=1` 진단으로 `pi=98`의 full-row와 p11 partial-row footprint를 cell별
source paragraph로 분해했다.

| row | 현재 row cut | MeasuredTable | source | 판정 |
| ---: | ---: | ---: | --- | --- |
| 0 | 447.6px | 257.8px | cell p0 = 445.8px | 189.8px 과대 |
| 1 | 18.4px | 18.4px | title | 정합 |
| 2 | 841.0px | 1,087.7px | p0 337.4 + p6 245.3 + p19 77.4 + p31 179.0 | H1 후에는 under, p11 cut은 p6 중간에서 끝남 |

row 0 cell p0는 텍스트 없는 한 문단에 Square picture 두 개를 저장한다. dump의 물리 값은
각각 `vOffset=+251HU, height=14,377HU`와 `vOffset=0, height=19,055HU`다. 두 picture는
수평으로 나란한 하나의 visual band이며 수직 범위의 합은 33,432HU가 아니라 union
`[0, 19,055]`HU다. 이는 254.1px이고 pad를 더한 `MeasuredTable=257.8px`과 일치한다.

반면 현재 `cell_units_uncached`의 Square/Tight/Through generic fragment는 같은 문단의
각 control height를 단순 합산해 445.8px을 만든다. 그 때문에 p11 row 2 예산은
`969.4 - 447.6 - 18.4 - pad = 501.5px`로 줄고 p6 range `[29,42)` 중 `[29,36)`만 들어간다.
row 0을 물리 union 257.8px로 맞추면 row 2 예산은 약 691.3px이 된다. H1 후 필요한
`p0 337.4 + p6 245.3 + p19 77.4 = 660.1px`이 모두 들어가고 p31은 다음 page에서 시작한다.
이 수치는 Hancom PDF p11/p12 경계와 일관된다.

### H3 구현 범위 — 병렬 empty-host Square control의 vertical union

다음 변경은 모든 Square control의 합산을 max로 바꾸지 않는다. p0 row 2처럼 negative
stored offset으로 서로 다른 수직 band를 만드는 control은 envelope/순서 의미가 있어 넓은
변경이 회귀를 만든다. 다음 조건을 모두 만족한 **한 paragraph**만 union으로 바꾼다.

1. native HWP5의 non-TAC `RowBreak` table이며, host paragraph text가 비어 있다.
2. Square/Tight/Through 비인라인 picture/shape가 2개 이상이고 모두 `VertRelTo::Para`다.
3. 모두 nonnegative saved `vertical_offset`을 가지며, 각 `(offset, offset + flow-height)`
   interval이 공통으로 겹친다. 즉 실제로 병렬 배치된 하나의 vertical band다.
4. 기존 control별 fragment provenance는 남겨 page owner를 잃지 않는다. aggregate
   `other_h`만 union height로 줄이고, range tagging은 같은 compressed unit span에 두
   source control을 계속 표시한다.

H3의 반증 기준은 명확하다. p11 raster에서 product box/설명 전체가 cell 안에 들어가지 않거나,
p12가 JOUZ header로 시작하지 않거나, `issue_2430`/p8가 깨지면 변경을 되돌리고 다음 Stage에서
원인을 다시 분리한다.

## H3 실행 결과 — page count는 정합, 다음 control의 prefix가 남음

H3 후 `issue_2430`은 다시 1/1 통과했고, #1921 page count는 **37**로 Hancom PDF와 일치했다.
PDF p11의 video·제품 3개·제품 상자·설명도 거의 같은 순서와 위치로 복원됐다. 그러나 direct
raster 대조에서 p11 footer 바로 위에 다음 JOUZ control의 얇은 상단이 보이고, p12는 JOUZ
header를 잃은 채 다음 title/image부터 시작한다. RenderTree p11 row 2 image 수가 5인 것도 이
잔여 prefix를 수치로 확인한다.

이는 H3이 row 0 capacity를 복원한 뒤 row 2가 p31 control의 **첫 generic unit**까지 소비했기
때문이다. p31은 p11에 들어갈 공간이 없지만 현재 row cut은 control range 중간까지 끊을 수 있고,
entry-owner 규칙은 그 첫 unit만으로 p11에 full control emit 권한을 준다. p12는 continuation이므로
같은 control을 다시 emit하지 않아 header가 사라진다.

### H4 — generic Square control은 entry page에서 range 전체가 맞을 때만 시작

`CellUnit.non_inline_control_range`는 이미 control별 first/last source unit을 보존한다. native
HWP5 비-TAC RowBreak의 `advance_row_cut`에서 다음 조건이면 range 시작 전에서 cut을 멈춘다.

1. 현재 unit이 해당 control range의 **첫** unit이다.
2. 이 fragment 안에서 아직 콘텐츠가 하나 이상 소비되어 있어 다음 page로 넘길 수 있다.
3. 그 control의 마지막 generic unit까지 더한 높이가 현재 row budget을 초과한다.

이미 이전 page가 control을 시작한 continuation은 `first unit`이 아니므로 그대로 이어 간다.
control 자체가 page보다 큰 경우도 start unit 진행 보장 때문에 기존 fragment path를 유지한다.
이렇게 하면 p6/p19가 충분히 들어갈 때만 p11 owner가 되고, 남은 p31은 통째로 p12 owner가 된다.

## H4 실행 결과 — p11/p12 RowBreak owner는 복원, p12의 독립 TopAndBottom 배치는 잔존

H4 뒤에는 변경 직후 우선 게이트인
`cargo test --profile release-test --test issue_2430_cell_rewrap_threshold`의 실제 test binary를
직접 실행해 **1/1 통과**를 확인했다. #1921의 기존 test는 p8 containment는 통과했지만,
그 test가 이전 구현을 `39쪽`, p11 row 2 `2 images`로 고정하므로 각각 실제 `37쪽`, `4 images`에서
실패한다. 이 실패는 본 Stage의 결론이 아니며, PDF 대조가 완료되기 전에 숫자만 새 값으로 바꾸지
않는다.

직접 SVG→raster와 Hancom PDF p11/p12를 대조한 결과는 다음과 같다.

| page | Hancom PDF | H4 rhwp | 판정 |
| --- | --- | --- | --- |
| p11 | video, product 3개, product box, 설명 1개 — 총 4 image | 같은 4 image가 row 2 cell 안에 있고 JOUZ prefix 없음 | **owner 복원** |
| p12 상단 | JOUZ banner와 `블로그 담배제품 후기·추천` title | 같은 banner/title 복원 | **owner 복원** |
| p12 row 4 | Instagram image 2개가 위·아래로 cell 안에 쌓임 | 첫 image만 cell 안, 둘째는 `x=675.7..1242.6`으로 cell 우측 밖에 paint됨 | **미해결** |

RenderTree는 p12 `pi=98,row=4,col=0` cell을 `x=102.0..714.3`, `y=261.2..998.0`으로
기록한다. 첫 TopAndBottom image(`ci=0`)는 `x=108.8..675.7,y=263.1..628.6`으로 containment를
만족하지만, 둘째(`ci=1`)는 `x=675.7..1242.6,y=263.1..626.2`로 같은 y에서 오른쪽으로 밀린다.
Hancom PDF는 둘째를 첫 image 아래의 약 `y=630`부터 그린다. 즉 RowBreak source owner의 문제와
별개로, **같은 빈 paragraph 안의 연속 TopAndBottom control을 LINE_SEG flow slot에 배정하지
않는 paint 문제**가 남았다.

### H5 가설 — partial RowBreak path가 empty TAC picture의 LINE_SEG slot을 버린다

실제 p12는 일반 `table_layout.rs`가 아니라 continued RowBreak table의
`table_partial.rs` paint path를 탄다. source dump에서 p12 row 4는 빈 host paragraph 하나이며,
Picture control 두 개가 모두 `treat_as_char=true`, `flow_with_text=true`, `TopAndBottom`,
`VertRelTo::Para`, offset 0임을 확인했다. 각각의 저장 `LINE_SEG`는
`vpos=0,height=27,409HU`와 `vpos=27,741,height=27,233HU`로 두 개의 세로 slot을 명시한다.

`layout_picture_full`의 실제 호출 provenance도 둘 다 `parent_para_index=98`, cell 4,
control 0/1을 기록했다. `paragraph_layout.rs`의 `run_tacs` 진단은 출력되지 않아 이 경로가 아님을
확인했다. 원인은 `table_partial.rs`의 TAC fallback이다. 빈 run 문단에서는
`will_render_inline=false`가 되고, fallback이 `inline_x`만 누적하면서 두 Picture를 모두
`para_y_before_compose`에 배치한다. 그래서 둘째가 `x=675.7..1242.6,y=263.1`로 셀 밖으로 나간다.
반대로 non-partial `table_layout.rs`는 이미 빈 문단의 TAC 순번을 `LINE_SEG` 순번에 1:1 매핑해
`inline_x`를 reset하고 `tac_img_y`를 다음 `vertical_pos`로 옮긴다. partial path만 그 계약을
복제하지 못한 구현 불일치다.

다음 변경은 일반 TopAndBottom 및 text-bearing paragraph에 확대하지 않는다. 다음 조건을 모두
만족하면 control 순번을 line-seg 순번으로 매핑해 해당 slot의 y를 anchor로 쓴다.

1. continued RowBreak table fragment의 **TAC Picture fallback**이며, 해당 문단의 composed run이
   모두 비어 있다.
2. 저장 `LINE_SEG`가 둘 이상이고, 동일 문단의 TAC를 control source 순서로 배정할 수 있다.
3. 기존 normal-table path와 동일하게 첫 control은 첫 slot, 다음 control은 다음 slot의 x/y를 사용한다.
   텍스트가 있는 문단과 한 줄에 여러 TAC가 들어가는 기존 char-position 경로는 변경하지 않는다.

그 외에는 기존 anchor를 그대로 보존한다. 적용 전 반증 기준은 p12 두 image의 complete cell
containment 및 PDF 같은 vertical order이며, p11 4 image/p8 2 image/`issue_2430`이 모두 유지돼야
한다. `table_layout.rs`의 same-contract path와 결과가 다르면 이 변경은 되돌리고 다음 Stage에서
source LINE_SEG 해석을 분리한다.

## H5 실행 결과 — p12의 두 TAC Picture가 PDF와 같은 세로 slot을 사용한다

H5는 partial RowBreak fallback의 빈-run TAC Picture에만 normal-table의 slot 매핑을 이식했다.
`cargo test --profile release-test --test issue_2430_cell_rewrap_threshold`는 변경 직후 새
release-test binary에서 1/1 통과했고, 기존 #1921 p8/p11 gate도 3/3 통과했다.

새 binary의 직접 SVG→raster 대조(`p11`, `p12`)에서는 다음을 확인했다.

| page | 변경 전 | H5 후 | Hancom PDF 대조 |
| --- | --- | --- | --- |
| p11 | H4에서 이미 video·제품 3개·제품 상자·설명 4 image owner를 복원 | 동일하게 유지 | 정합 |
| p12 row 4 | control 1이 `x=675.7..1242.6,y=263.1`로 우측 셀선을 침범 | 첫 image 아래의 둘째 LINE_SEG slot으로 이동, 두 image 모두 cell 안 | 정합 |

이는 저장 `vpos=27,741HU`의 두 번째 slot을 실제 y anchor로 사용한 결과이며, source의
`horizontal_offset`을 임의로 제거하거나 모든 TAC를 세로로 쌓는 일반화가 아니다. 다음 회귀 게이트는
p12 row 4의 image가 정확히 두 개인지, 각각 cell containment를 만족하는지, 둘째 y가 첫째보다 큰지를
고정한다. 따라서 같은 결함이 x 누적 또는 첫 slot 재사용으로 돌아오면 page count만 같아도 실패한다.

## 확인할 가설과 순서

1. p11 RowBreak scan의 `avail_for_rows`, `consumed`, `split_end_limit`, `end_cut`와 PDF page
   좌표를 다시 나란히 기록한다.
2. row 0/row 2의 `cut_row_h`가 source unit의 실제 physical height보다 왜 큰지,
   `advance_row_cut`와 `row_cut_content_height` 경로를 분리한다.
3. generic unit의 source owner 계측 결과를 기록하고, row 0/2의 `cut_row_h` 과대를 만든
   paragraph·spacer·control 경로를 특정한다.
4. geometry 변경 전에 control owner를 overlap이 아니라 **range entry unit**으로 좁힌다.
   이 규칙은 적용했고 `issue_2430` 및 #1921 p8/p11 focused gate로 컴파일·회귀를 먼저
   확인했다. p11에 p6 entry unit을 포함시키는 geometry 변경 뒤에는 p12 `[30, …)`에서
   p6가 재emit되지 않는 PDF-anchored gate를 추가한다.
5. 그 뒤 PDF p11의 asset sequence 0→1→2와 p12의 다음 asset 경계를 검증하는 회귀 gate를
   작성한다. 현재의 p11 2-image containment gate만으로는 완료 처리하지 않는다.

## 금지와 완료 기준

- Stage 42의 overflow-cell 수정과 이 Stage의 p11 owner 수정은 하나의 원인으로 추정하지 않는다.
- 페이지 수 39→38 또는 37 감소만으로 성공 처리하지 않는다.
- `issue_2430_cell_rewrap_threshold`를 먼저, 그 뒤 #1921 p8/p11 ownership gate를 확인한다.
- 완료는 Hancom PDF p11의 세 asset 및 p12 이후 경계가 직접 대조될 때만 선언한다.
