---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-09
---

# Task #3820 Stage 96 — #2279 r27 nested fragment PDF oracle 재판정

## 인계와 발견

Stage 95의 HWPX incremental edit/cache 보정 뒤 전체 명령

```text
CARGO_TARGET_DIR=target/pr-review CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --tests
```

을 실행했다. 라이브러리 test 3,377개와 `#2214`, `#2185`, `#2020`, `#1921` 등 선행 binary는
통과했지만, `tests/issue_2279_layout_oracles.rs`에서 다음 실패가 나왔다.

```text
issue_2279_nested_cell_units_split_r27_not_r26
p29에 r27 continuation 부재 — 분할 구조 변화
LAYOUT_OVERFLOW_CELL: section=0 pi=22 line=0 y=1137.3 page_bottom=1122.5 overflow=14.8px
```

stdout을 `tee`로 저장한 실행은 pipeline의 마지막 exit를 반환하므로, Cargo의 실제 실패 출력과
test summary를 근거로 이 전체 회귀는 **아직 통과가 아니다**. 후속 전체 실행은 `set -o pipefail`로
실제 Cargo exit code를 보존한다.

## 영향 분리

Stage 95의 production 변경은 `text_editing`의 **편집 뒤** LineSeg suffix reflow와 focused
render-tree cache tail patch다. 반면 #2279 test는 `DocumentCore::from_bytes()` 직후 편집 없이
`build_page_render_tree(27/28)`만 호출한다. Stage 95 commit에는 `typeset`, table fragment,
height measurement, source fixture, PDF oracle 변경이 없다.

따라서 이 실패를 Stage 95가 만든 결과로 가정하거나 test baseline을 즉시 바꾸면 안 된다. 다음
순서로 PDF 기준을 재판정한다.

1. `pdf/issue1921/86712_regulatory_analysis-2024.pdf`의 28--29쪽과 현재 HWP SVG/render tree를
   나란히 렌더한다.
2. r26 전용 숫자 `2891017`과 r27 heading `편익 수혜자`의 실제 page owner를 PDF text와 시각으로
   함께 확인한다.
3. render tree에서 동일 nested table source의 fragment, clip, text bbox를 덤프해 p29 continuation
   소실이 layout regression인지 oracle/test stale인지 결정한다.
4. production 결함이면 원인을 최소 범위로 수정하고, baseline stale이면 PDF 근거와 함께 #2279
   assertion만 정확한 owner 계약으로 교정한다. 어느 경우에도 page-count만으로 판정하지 않는다.

## PDF 직접 대조 — 결함 판정

2026-08-09에 `pdf/issue1921/86712_regulatory_analysis-2024.pdf`의 28--29쪽을 150dpi PNG로
렌더하고, 같은 HWP의 rhwp print-SVG 28--29쪽과 직접 비교했다.

- PDF와 rhwp p28은 모두 r27의 `편익 수혜자` 근거설명 블록까지는 가진다. 그러나 PDF는 그
  뒤의 3×12 내부표(문단 20)를 **다음 쪽의 맨 위**에서 시작한다. 현재 rhwp는 이 표를 p28
  y=968.1에서 시작해 p21을 y=1110.7, p22를 y=1137.3에 배치한다. 본문 bottom 1122.5를
  14.8px 넘기므로 표의 하단이 p28에 잘린다.
- PDF p29에는 그 3×12 표, 연도별 기본형건축비 표, r27의 다음 5×2 표와 `근거설명` 본문 첫
  줄들이 순서대로 있다.
- 현재 rhwp p29는 3×12 표를 온전히 다시 시작하지 못하고, 연도별 표와 다음 표의 4개 heading
  행만 남긴다. `pi=182`의 마지막 r4 행(`근거설명`)은 통째로 p30으로 이월되어 p29에는 PDF에
  있는 본문이 없다.

따라서 `편익 수혜자`라는 기존 p29 sentinel은 heading이 p28에만 있으므로 교체해야 한다. test가
보고한 **r27 continuation 소실 자체는 PDF로 확정된 production defect**이며, p28의 nested-table
overflow와 p29의 r4 전체 이월은 같은 page-fragment 판단에서 생긴 연쇄 결함이다.

## sourceㆍrender-tree 추적

증적은 `mydocs/pr/assets/task_m100_3820_stage96_issue2279_r27_continuation/`에 보관한다.
`visual_sweep.py --pages 28-29`는 SVGㆍPDFㆍoverlayㆍrender tree를 모두 냈지만 자동 flag는 0개였다.
따라서 이 경우에는 단순 ink 면적 비교보다 source owner와 page fragment 순서를 검사하는 oracle이
필요하다.

- `pi=172`은 28×4 RowBreak 외부표다. p28에는 `rows=27..28`, 그 안의 r27 우측 cell은 1×1
  내부표를 포함한다. 그 내부표의 문단 20이 문제의 3×12 표다.
- 현재 p28의 `pi=172` fragment는 이 3×12 표를 시작하고 `p22`를 body bottom 밖으로 그린다.
  PDF contract는 이 fragment cut을 문단 20 **직전**으로 되돌리는 것이다. 자체가 한 페이지보다
  큰 표를 일반적으로 금지하는 규칙은 아니다.
- `pi=182`는 뒤따르는 5×2 RowBreak 표다. p29 render tree는 `rows=0..4`만, p30은 `rows=4..5`
  만 가진다. r4/c1은 1×1 내부표와 12문단을 가지며 첫 문구는 `정비사업 후 조기 입주`다.
- 따라서 첫 수정 후 p29를 다시 조판해 PDF와 같은 r4 첫 fragment가 생기는지 확인한다. 여전히
  r4 전체를 넘기면, nested cell의 fragmentable unit 계산을 별도로 고친다. 두 문제를 한 추측성
  변경으로 묶지 않는다.

## 구현 전 계약과 순서

1. `table_layout`의 nested-table unit과 RowBreak cut 후보를 추적해 p28에서 문단 20을 시작시킨
   결정을 찾는다.
2. 해당 표가 남은 높이에 온전히 맞지 않을 때 cut을 그 표 앞에 두도록 최소 보정한다. p28 overflow를
   제거하고 p29에 `주거용 건물` 3×12 표가 나타나는지 확인한다.
3. 새 p29 geometry에서 `pi=182` r4의 첫 `정비사업 후 조기 입주` fragment가 PDF처럼 p29에
   생기는지 검사한다. 실패할 때만 r4 내부 1×1 표의 unit 분할을 고친다.
4. `issue_2279`는 p28에 3×12 표가 없고 p29에 위 두 sentinel이 있는지, 그리고 p28 body overflow가
   없는지를 검사하도록 교체한다. 기존 r26 비재등장과 p65 계약은 유지한다.

## paginator 진단 — 수정 전 수치

`RHWP_DIAG_SCAN=1`로 현재 #2279 single test를 실행했다. 문제 fragment에서 paginator가 보고한
값은 다음과 같다.

```text
pi=172 cur_h=688.3 declared=837.8
CUT_TRY r=27 budget=266.0 padding=3.0 consumed_h=255.2 end_cut=[1,16]
CUT_TRY r=27 budget=968.3 padding=3.0 consumed_h=867.7 end_cut=[1,65]
pi=182 cur_h=519.8 declared=382.8
CUT_TRY r=4 budget=344.6 padding=3.0 consumed_h=774.4 end_cut=[1,1]
LAYOUT_OVERFLOW_CELL: pi=22 y=1137.3 page_bottom=1122.5 overflow=14.8px
```

즉 `pi=172`의 첫 조각은 p28 잔여 266px에 대해 cut 16을 선택하고, 다음 p29에서는 cut 65까지
진행한다. `pi=182` r4는 남은 344.6px보다 774.4px로 측정돼 whole-row 이월된다. 다음 추적은
`cell_units_uncached`가 문단 20의 3×12 표를 어떤 cell unit으로 만들고, `scan_block_table_split_rows`
가 그 unit을 p28에서 허용하는 정확한 조건을 확인하는 것이다. 아직 이 수치만으로 임계값을 조정하지
않는다.

## 회귀 도입 범위

`#2279`의 원래 oracle commit `0fd964974` 이후에는 table fragment/page-owner 변경이 다수 누적됐다.
가장 최근 후보는 `7c43d7d1f`(RowBreak 중첩표 owner 보정)와 그 이전의 #3820 nested fragment 계열이다.
현재 결함이 원래 #2279의 미검증 축인지, 후속 owner 보정이 되돌린 회귀인지 구분하기 위해 별도
detached worktree에서 `7c43d7d1f^`와 필요하면 그 이전 후보의 **동일 single test**를 실행한다.
이 비교는 현재 작업 트리ㆍtarget 산출물을 건드리지 않으며, 도입 commit을 확인한 뒤에만 그 diff의
최소 조건을 되돌리거나 새 guard를 설계한다.

첫 비교 결과 `7c43d7d1f^` (`26d7eed6d`)도 같은 `p29에 r27 continuation 부재`와
`pi=22` 14.8px overflow로 실패했다. 따라서 최근 short-child owner 보정 하나의 회귀로 단정할 수
없다. 다음 비교점은 #2279 oracle을 추가한 `0fd964974`다. 그 점에서도 실패하면, 이 test의 historical
green 기록과 현재 PDF contract가 서로 달랐는지까지 분석 대상으로 남긴다.

`0fd964974`의 동일 binary는 `1 passed; 0 failed`로 통과했다. 따라서 oracle 자체가 처음부터
stale였던 것이 아니라, `0fd964974..26d7eed6d` 사이의 실제 regression이다. 이 구간은 3,009개의
일반 commit을 포함하므로 source-touch commit만 추측하지 않고, 별도 worktree에서 이 single test를
기준으로 이분 탐색한다. 이 단계의 목적은 과거 변경을 통째로 revert하는 것이 아니라 **손상된
fragment 조건 하나**를 찾아 현재 PDF contract에 맞는 최소 보정 위치를 결정하는 것이다.

### 이분 탐색 결과와 코드 후보의 위치

동일한 `issue_2279_nested_cell_units_split_r27_not_r26` binary를 기준으로, good
`0fd964974`와 bad `26d7eed6d` 사이를 `git bisect run`으로 탐색했다. 첫 bad commit은 다음이다.

```text
4815c9b5f0efa75d5c85992232a08e7611e2d64a
fix: #3820 RowBreak rowspan 행 꼬리 보존
```

이 commit은 `typeset.rs`에 `start_row_height_override`/`end_row_height_override`를 추가하여,
앞 row-span이 덮는 짧은 RowBreak 행이 내용은 완전히 소비했지만 선언 높이의 빈 tail을 다음 fragment로
넘기도록 한다. 이는 #3820의 `76076 p35→p36`를 보호하는 별도 계약이다. 그 외에도 continuation
cursor의 full cut 처리와 partial-table 직렬화가 동시에 달라진다. `pi=172` r27은 중첩표를 가지므로
commit 안의 `prior-span + !row_has_nested` 단독 fast path에는 직접 들어가지 않는다. 그러므로 **#3820
수정을 통째로 revert하거나 `row_has_nested` guard만 뒤집는 것은 원인으로 증명되지 않았다.**

후보 1이었던 uncommitted `table_layout.rs` 실험은 위 사실을 반영해 다음만 시도했다.

- RowBreak cell fragment가 중첩표의 중간을 소비했고,
- 그 중첩표부터 남은 logical tail 전체가 fresh page body에는 들어가며,
- hard break를 넘지 않는 경우에만,

그 중첩표 첫 unit으로 cut을 되감는다. 이는 PDF p28의 `문단 20 직전` cut 계약을 직접 표현한 후보이며,
아직 테스트나 PDF 재대조로 정당화되지 않았다. `svg` font alias와 visual sweep 변경도 같은 미커밋
작업트리에 있으나 이 r27 fragment 원인과는 독립이다. 아래 순서의 focused 검증이 실패하면 이 후보를
정답으로 간주하지 않고, `4815c9b5f`의 continuation override가 fragment 경계에 미치는 영향을 별도로
최소화한다.

이 기록 뒤에는 **이 문서에 원인·후보·예상 contract를 먼저 추가한 변경만** 코드에 적용한다.

### 후보 1의 반증

문서화 뒤 현재 후보를 검증했다.

```text
CARGO_TARGET_DIR=target/pr-review CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test issue_2430_cell_rewrap_threshold -- --nocapture
# 2 passed; 0 failed

RHWP_DIAG_NESTED_REWIND=1 CARGO_TARGET_DIR=target/pr-review CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test issue_2279_layout_oracles \
  issue_2279_nested_cell_units_split_r27_not_r26 -- --nocapture
# FAILED: p28에 3×12 내부 표가 부분 진입
```

진단에서 p28의 문제 cell은 `start=0, j=16, units=70`인데, 모든 candidate unit이 같은
source paragraph `p=0`으로 평탄화되어 있었다. 따라서 `paragraph.controls`에 `Control::Table`이
있다는 사실만으로는 문단 20의 3×12 표와 그 앞의 다른 nested control을 구별할 수 없다. 후보 1의
`para_idx` 기반 `nested_start`는 fragment start(0)로 판정되어 진행 방지 guard에 걸렸고, overflow는
그대로 `14.8px`였다.

이 결과로 **문단 index만으로 되감는 후보 1은 기각**한다. 다음 후보는 `CellUnit`이 이미 보유한
`mixed_nested_starts_after_table`/nested fragment 경계와 source control 순서 중 실제 문단 20 시작을
식별할 수 있는 값을 먼저 확인한 뒤에 작성한다. 확인 전에는 helper의 조건을 넓히거나 높이 임계값을
조정하지 않는다.

후보 1은 기능상 효과가 없고 다른 RowBreak walk 세 경로에 불필요한 호출만 남기므로, 다음 관찰 전에
후속 후보로 교체했다. 이는 해결 변경을 되돌리는 것이 아니라 **기각된 미커밋 실험을 기준선과
진단에서 분리하는 정리**다.

### 후보 2 — marker 범위 확인 전 상태

현재 작업트리에는 후보 1을 대체하는 미검증 helper가 있다. 이 버전은 outer source `para_idx`가 아닌
`mixed_nested_starts_after_table` marker를 사용해, marker 바로 앞의 마지막 visible mixed atom으로
되감으려 한다. 이는 p20(3×12)의 **끝** 뒤에 p21의 source fragment가 시작할 때는 p20의 시작을 직접
가리키지 못한다. 더구나 현재 p28 cut은 `j=16`인데 앞선 profiler에서 marker는 unit 61에서 관찰되었다.
따라서 marker가 selected prefix 밖이면 helper는 어떤 되감기도 하지 못할 가능성이 높다.

이 후보의 목적은 실제 source paragraph ownership을 모델에 추가하지 않고 경계를 복원할 수 있는지
확인하는 것이다. 기대한 p20 owner start를 표현하지 못한다는 결과가 나오면 즉시 기각한다. 그 경우
다음 분석은 `NestedFlowFragment → CellUnit` 투영에서 child source paragraph index를 보존하는, 동작을
바꾸지 않는 provenance 필드 추가로 한정한다. 그 provenance를 통해서만 p20의 첫 atom과 fresh-page
tail height를 계산한 뒤 세 번째 행동 후보를 설계한다.

문서화 후 후보 2에 대해 다시 `issue_2430`을 먼저 실행해 `2 passed`를 확인했지만, 곧바로 실행한
#2279 focused gate는 동일하게 `p28에 3×12 내부 표가 부분 진입`으로 실패했다. 예상한 대로 marker가
current cut prefix에 없어서 p28의 cut을 바꾸지 못했다. **후보 2도 기각**하며, 이 helper를 유지한 채
조건을 넓히지 않는다. 다음 구현 전에 p20 ownership을 보존하는 provenance 필드가 전체
`NestedFlowFragment` 생성 경로에서 어떤 값으로 채워지는지 테스트 전용 진단으로 먼저 확인한다.

## 병행 중인 font/visual-sweep 변경의 근거와 경계

이 작업트리에 남아 있는 `svg.rs`, `svg/tests.rs`, `scripts/visual_sweep.py` 변경은 r27 page fragment
보정과 독립인 **증적 rasterization 정합** 변경이다. 로컬 font directory에서 실제 파일명을 재확인했다.

```text
/Users/tsjang/Library/Fonts/H2GTRM.TTF
/Users/tsjang/Library/Fonts/HMKMM.TTF
```

- HWPX legacy face `한양중고딕`은 설치 family/full name `HY중고딕`/`HYGothic-Medium` 및 실제
  파일 `H2GTRM.TTF`와 달라, `--font-style` SVG의 `local()` alias 없이는 PNG 검증 host에서 다른
  font 또는 두부로 rasterize될 수 있다.
- `휴먼명조`의 실제 후보는 `HMKMM.TTF`다. 기존 첫 후보 `HYMJRE.TTF`는 HY견명조 파일이므로,
  source font가 휴먼명조인 경우 portable SVG font discovery에 잘못된 대체를 줄 수 있다.
- visual sweep은 SVG 좌표를 바꾸지 않고 `export-svg --font-style`로 local font face만 명시한다.
  따라서 이 변경은 layout PDF oracle을 고치는 수단이 아니며, font identity가 같은 환경의 raster
  증적을 확보하는 수단이다.

이 범위는 `legacy_hanyang_faces_have_portable_local_aliases` unit test와 86712 p28--29 SVG의
`@font-face` 확인으로 검증한다. r27 code correction과 혼합해 성공으로 기록하지 않는다.

## 후보 3 전 분석 — child source provenance

`rhwp dump samples/86712_regulatory_analysis.hwp --section 0 --para 172`로 원본 구조를 다시
확인했다. `pi=172`의 r27 우측 셀은 outer p0 안의 1×1 table 하나이며, 그 child cell은 25문단이다.
그 안에서 p20이 문제의 3×12 표, p23이 뒤따르는 5×4 표다. 현재
`nested_table_mixed_fragment_heights`는 이 child의 unit을 `NestedFlowFragment`로 바꿀 때 height,
trailing, `starts_after_table`만 보존하고, child paragraph index를 버린다. 상위 outer cell은 그 모든
fragment를 자기 p0 unit으로 다시 기록한다.

따라서 후보 3의 첫 단계는 동작 변경이 아닌 provenance 투영이다.

1. `NestedFlowFragment`에 optional child `source_para_idx`를 추가한다.
2. canonical/legacy 두 fragment 생성 경로 모두에서 각 fragment의 source paragraph index를 채운다.
3. outer mixed `CellUnit`이 이 값을 그대로 보존하게 한다.
4. `RHWP_DIAG_NESTED_OWNER=1`에서 p28의 `start=0, j=16` 앞뒤 unit index, child para index,
   높이와 candidate p20 run을 출력한다.

이 단계는 선택된 unit, 높이, 페이지 수를 바꾸지 않는다. 진단에서 p20 run의 첫 index와 `tail_height`
가 확인되기 전에는 rewind 조건을 다시 넣지 않는다. 확인 뒤 행동 후보는 다음의 **좁은 조건**만 가질 수
있다: RowBreak outer cell, immediate 1×1 child의 `Control::Table` source para run, current fragment가
그 run 일부를 소비, 해당 run부터의 tail이 fresh body에 완전히 fit, 그리고 tail에 hard break 없음.
이 조건은 86712 이름/페이지 번호를 사용하지 않으며 #3820 rowspan-tail path와도 분리된다.

### provenance 관찰 결과 — paginator cut과 renderer owner가 발산

`RHWP_DIAG_NESTED_OWNER=1` 관찰으로 p20(3×12)은 child unit 59(height 143.7), p23(5×4)은
unit 65(height 233.5)임을 확인했다. p20 뒤 p21의 `starts_after_table` marker는 unit 61이다.
그러나 p28의 paginator cut은 child source p0--p5만 소비한 `end_cut=[1,16]`이다. 즉 paginator 자체는
p20을 p28에 소비한 적이 없다.

같은 현재 HWP에서 `export-text --page 27/28`와 Hancom PDF `pdftotext -f 28 -l 29 -layout`를 직접
대조했다.

- PDF p28은 `< 주거용건물 건설공사비지수 ... >` 제목 뒤에서 끝나며, 다음 physical page(p29)가
  3×12 표로 시작한다.
- 현재 rhwp p28은 p20 3×12 표 전체(`88.27` 등)를 이미 출력한다. p29에도 같은 표 전체가 다시
  나타난다.
- 따라서 `88.2`가 p28에 있다는 새 gate의 실패는 올바른 결함 검출이다. 반대로 p28에서는 아직
  p20의 unit을 소비하지 않았으므로 **height/cut rewind가 원인이 아니다.**

결론적으로 defect는 *paginator가 결정한 outer CellUnit cut을 partial renderer가 1×1 child의
source paragraph 범위로 투영하지 못해, child table control 전체를 앞 fragment에 paint하는 것*이다.
후속 p29의 owner/본문 결함도 이 중복 paint와 cursor 범위 불일치의 연쇄로 먼저 다뤄야 한다.

따라서 후보 3의 행동 단계는 rewind가 아니라 다음으로 변경한다.

1. 새 provenance를 이용해 outer partial cell의 `[start_cut,end_cut)`을 immediate 1×1 child의
   source paragraph 범위로 변환한다.
2. renderer가 child p20/p23 `Control::Table`을 해당 source range가 실제로 포함될 때만 paint하도록
   한다. 테이블 행 자체의 내부 cut은 기존 `NestedTableUnitCut` 경로를 유지한다.
3. p28에 p20 표가 없고 p29에 한 번만 존재하는 focused contract를 먼저 통과시킨 뒤, p29의
   `정비사업 후 조기 입주` 본문 owner를 PDF와 재대조한다.

이것은 current unit 높이·page breaker를 바꾸지 않으며, `#3820`의 rowspan tail 보존과 독립된
partial rendering 경계 보정이다.

### 후보 4 — 2단계 1×1 child의 재귀 partial cursor

관찰 결과에 따라 다음 행동 후보를 제한한다. `mixed_nested_split_from_cut`은 이미
`mixed_nested_recursive=true`인 stream에 대해 outer `[start_cut,end_cut)`와 같은 unit count를
child 1×1 table의 `NestedTableCut(start_cut,end_cut)`로 전달하고, `layout_partial_table_resolved`가
그 cursor 범위만 paint하는 경로를 갖고 있다. 현재 legacy fallback만 `recursive=false`여서 바로 이
경로를 사용하지 못한다.

후보 4는 다음 **모두**가 참인 immediate child에만 `mixed_nested_recursive`를 올린다.

- parent가 block `RowBreak` table이고 treat-as-char가 아니다.
- host paragraph가 정확히 하나의 1×1 child table을 가진다.
- child 자체도 1×1이며, 그 유일 셀에 한 단계 더 깊은 `Control::Table` source paragraph가 있다.

이때 69개의 mixed fragment를 동일한 child cursor stream으로 보존한다. p28의 outer cut 0..16은
child p0--p5만, p29 cut 16..65는 p20(3×12)을 포함한 실제 source range만 renderer에 전달한다.
첫 판정은 **p28에서 p20가 사라지고 p29에는 한 번만 나타나는가**다. p23 및 후속 `정비사업 후 조기
입주` owner는 이 첫 보정 뒤 PDF와 다시 비교해, pagination 범위의 별도 결함인지 확인한다. 이 후보는
row height, cut 계산, page count를 변경하지 않는다.

### 후보 4 구현 기록 — focused gate 전

위 제한 조건을 `cell_units_uncached`의 **빈 host paragraph + immediate 1×1 child** 분기에만
구현했다. 그 조건이 참일 때에만 child fragment의 `mixed_nested_recursive`를 올려 기존
`mixed_nested_split_from_cut` → `layout_partial_table_resolved` 경로가 same unit cursor를
전달하게 한다. 혼합 텍스트를 가진 host, 일반 1×1 child, child 안에 표가 없는 경우, inline table은
기존 scalar 경로를 유지한다.

이 기록은 결과가 아니다. 다음 순서는 사용자가 지정한
`issue_2430_cell_rewrap_threshold` focused gate, 이어서 #2279 p28--29 PDF 직접 비교다. 실패하면
이 후보를 확장하거나 height 계산을 바꾸지 않고, 실패 위치를 이 문서에 먼저 기록한다.

### PDF 직접 대조로 수정한 cursor 소유 분석

candidate 4를 되돌린 뒤, `RHWP_DIAG_CELLPARA=1`과 `export-text`로 outer r27 right cell의
실제 RowCut을 확인했다. physical page와 0-based CLI page를 혼동하지 않도록 아래는 모두
**physical p27--p29**로 표기한다.

| physical page | outer r27 child cut | RHWP scalar 출력 | Hancom PDF 출력 |
|---|---:|---|---|
| p27 | `0..16` | r27의 `편익 수혜자`부터 child 전체를 paint | r26의 끝; r27 child 없음 |
| p28 | `16..58` | 앞 문단 **및** p20 3×12 표(`88.27`)를 paint | 앞 문단만; 표는 없음 |
| p29 | `58..end` | p20 3×12 표를 다시 paint | p20 3×12 표부터 5×4 표와 후속 본문 |

증거 파일은 `mydocs/pr/assets/task_m100_3820_stage96_issue2279_r27_continuation/`
아래 `baseline_after_candidate4_revert/86712_regulatory_analysis_028.txt`, `_029.txt`와
`/tmp/stage96-diag-page27.err`--`page29.err`이다. PDF는
`pdftotext -f 27 -l 29 -layout pdf/issue1921/86712_regulatory_analysis-2024.pdf`로 대조했다.

따라서 기존의 “p28 paginator는 p20을 소비하지 않았으므로 renderer만 고치면 된다”는 표현은
불완전하다. p20에는 맞지만, p28에 필요한 p0의 계속 부분도 outer cut의 `16`만으로는 표현되지
않는다. `0..16`이 p27에서 실제로 화면에 보이지 않도록 하는 body/row placement 계약과,
`16..58`이 p28에 남길 child source를 분리해야 한다.

candidate 4가 p28의 `편익 수혜자`를 없앤 이유도 이 표와 일치한다. child table은 1행이고,
그 1행에 `start_cut=[16]`을 전달하면 child `cell_units`의 앞쪽 source를 정상적으로 생략한다.
그러나 그 숫자는 **outer continuation의 fragment ordinal**일 뿐 PDF p28에서 보일 child paragraph
시작점이 아니다. 따라서 69개 ordinal을 child 1×1 `RowCut`에 직결하는 방법은 사용할 수 없다.

다음 후보는 renderer에 별도 source-para window를 도입하기 전에, p27 `0..16` fragment가 PDF에서
왜 body 밖에 남아야 하는지와 outer row의 `y`/clip을 조사한다. p28/p29의 문단 source window는 그
조사 결과와 함께 계산한다. 이 단계에서는 코드 변경을 하지 않는다.

### 렌더 PNG 검증 — export-text 판정 정정

위 표의 `RHWP scalar 출력`은 `export-text` 기준이다. 이 명령은 RenderTree의 clip 밖 자식도
텍스트로 보존하므로 실제 화면 표시의 근거가 될 수 없다. `rhwp_svg_p27_p29/` SVG를 96dpi PNG로
래스터화하고 PDF p27--p29와 좌우 contact sheet로 직접 확인했다.

증적: `stage96-issue2279-r27-final/review/review_028.png`,
`stage96-issue2279-r27-final/review/review_029.png` (각 이미지: RHWP | PDF | overlay).

- RHWP p28에는 `88.27` table node가 export-text로는 남지만, cell clip 밖이어서 PNG에는 보이지
  않는다. PDF p28도 표 없이 끝나므로 **p28의 p20 visible-owner 결함은 현재 증명되지 않았다**.
- RHWP p29에는 3×12 표와 5×4 표가 실제로 보이며 PDF p29의 동일 owner와 대응한다.
- 실제 차이는 RHWP p27 하단에 r27 `근거설명`의 앞 내용이 보이는데 PDF p27에는 없다는 점이다.
  이는 outer cut `0..16`의 paint/clip 경계 결함이며, p28/p29에 69-unit child cursor를 직결하는
  별도 문제가 아니다.

따라서 candidate 4의 목적 자체를 철회한다. 다음 focused 판정은 `page_contains_paintable_text`
기준으로 전체 #2279 oracle을 재실행하고, p27의 r27 visible leakage를 RenderTree clip/bbox로
계량한다. `export-text`만으로 p20 중복이라고 결론 내리거나 그 결론을 위한 코드 보정을 추가하지
않는다.

### #2279 전체 focused gate 결과

`cargo test --profile release-test --test issue_2279_layout_oracles -- --nocapture` 결과는 4개 중
3개 통과, 1개 실패다. 실패 assertion은 p29의 `주민대표단 구성`(p23 5×4 표) raw render-tree
검색이다.

```text
p29에 3×12 표 뒤 5×4 내부 표 부재 — r27 block 순서 회귀
```

이는 p28 p20의 paintable 결함과 별개다. p29 SVG PNG에는 5×4 표의 격자와 헤더가 보이는 듯하므로,
다음 단계에서 CLI SVG와 `DocumentCore::build_page_render_tree`의 profile/clip 차이 및 해당 text node를
직접 대조한다. assertion을 약화하거나 코드를 수정하지 않는다.

### 후보 5 — 첫 inner-table 전 문단 묶음의 fresh-page defer

`render_tree_029.json`을 직접 확인한 결과 p23 5×4 표는 p29 render tree에 존재한다. 다만 첫 셀
header가 `"주민대표단 "`과 `"구성"` 두 `TextRun`으로 나뉘어 있어 단일-run substring assertion이
실패했다. 이 gate는 표 단위 text-sequence assertion으로 교체해야 하며, renderer 결함의 증거가
아니다.

반대로 p27 차이는 실제 PNG에 존재한다. typesetter 진단은 다음과 같다.

```text
# p27의 남은 본문 266px에서 outer r27을 새로 시작
CUT_TRY r=27 budget=266.0 ... end_cut=[1, 16]
# p28 fresh body에서 그 continuation을 계속
CUT_TRY r=27 budget=968.3 ... start_cut=[1,16] end_cut=[1,58]
```

right cell의 fragment 0..58은 p0--p19(첫 내부표 p20 직전)이며, p20 3×12 표는 unit 59다.
0..58의 전체 높이는 fresh body 안에 맞고, PDF도 바로 그 계약으로 p28에 p0--p19, p29에 p20을
배치한다. 현재 p27의 `0..16` 조각은 PDF에 없는데 RHWP가 scalar child를 그려 실제 잉크로 누출한다.

따라서 후보 5는 renderer child cursor가 아니라 **typesetter row-start 정책**만 바꾼다.

1. native HWP5 block `RowBreak` table의 **새 row**(`row_start_cut` 비어 있고, 현재 table fragment의
   첫 row가 아님)만 대상이다.
2. 그 row의 immediate 1×1 wrapper cell에 child 1×1이 있고, child의 첫 `Control::Table` source
   paragraph 전까지의 mixed fragment prefix가 존재해야 한다.
3. 현재 잔여 예산이 만든 partial end-cut이 그 prefix 안에서 끝나며, label cell을 포함한 그 prefix
   전체가 `current_body_area`의 fresh-page height에는 들어갈 때만 현재 row를 전혀 소비하지 않고
   다음 page에 defer한다.
4. 그 외 ordinary text cell, child 내부표가 없는 1×1, 이미 continuation인 row, fresh body에도
   들어가지 않는 prefix는 현재 `advance_row_cut` 계약을 그대로 쓴다.

이 조건은 source paragraph provenance (`mixed_nested_source_para_idx`)를 이용하지만, 69 unit cursor를
child row cut에 전달하지 않는다. 기대 결과는 p27에서 `편익 수혜자` paintable text가 없어지고,
p28은 `start_cut=[]`로 p0--p19를 한 fragment에 보이며, p29는 기존처럼 p20/p23을 보이는 것이다.

구현 전 gate는 (a) p27에 해당 text가 paintable하지 않음, (b) p28에는 paintable함, (c) p28 p20은
paintable하지 않음, (d) p29의 3×12/5×4 table이 tree에 존재함이다. 이 gate가 baseline에서 실패하는
것을 확인한 뒤에만 후보 5를 구현한다.

### 후보 4 첫 focused 결과 — 거부

`issue_2430_cell_rewrap_threshold`는 2/2 통과했다. 그러나
`issue_2279_nested_cell_units_split_r27_not_r26`는 다음 첫 assertion에서 실패했다.

```text
p28에 r27 콘텐츠 첫 유닛 부재 — 1×1 중첩 셀 유닛화 회귀 (rows=0..27 로 후퇴)
```

즉 69개 outer fragment를 child 1×1 table의 `NestedTableCut`에 그대로 넣는 가정은 성립하지
않는다. child의 직접 row cursor는 1행뿐인데 69개 문단/inner-table fragment cursor와 같은 축이
아니다. 이 후보는 p20 중복을 줄이더라도 p28의 앞쪽 r27 content까지 사라뜨리는 회귀를 만들므로
확장하지 않는다. 다음 분석은 candidate 4를 되돌린 기준에서, **child p0의 source paragraph
범위만** 실제 render loop에 전달할 별도 mapping이 가능한지 확인한다. row-height/cut 계산은 그대로
둔다.

## 완료 기준

- PDF와 rhwp의 p28--29 r26/r27 owner를 직접 비교한 증거가 남는다.
- #2279 회귀 gate는 실제 PDF contract를 검사한다.
- focused gate와 `set -o pipefail` 전체 release-test가 최종 `0 failed`/exit `0`이다.

## 후보 5 결과 — clip-aware owner oracle 및 SVG 글꼴 fallback

2026-08-09에 child source provenance를 보존한 RowBreak 보정과 함께, render tree의
**source 보존 범위**와 SVG의 **실제 paint 범위**를 분리해 oracle을 확정했다.

- `RenderNode`는 debugging/recomposition을 위해 `TableCell` clip 밖의 자식을 보존한다.
  따라서 일반 재귀 텍스트 검색만으로는 p28에서 cell clip으로 잘린 3×12 표를 "보인다"고
  오판한다. `issue_2279_layout_oracles.rs`는 `Body`/clip-enabled `TableCell`/`TextBox`의
  SVG clip 교집합을 적용하는 `page_contains_paintable_text`로 p27--p29 owner를 판정한다.
- p29의 5×4 표 헤더 `주민대표단 구성`은 TextRun 둘로 나뉘므로, 표 차원(5×4) 안의
  source-order text sequence를 검사한다. 단일 TextRun substring의 거짓 음성을 제거했으며,
  표 자체의 존재 검증은 유지한다.
- focused 결과는 `issue_2279_layout_oracles` **4 passed, 0 failed**다. p27 r27 조기
  paint 없음, p28의 r27 앞 문단은 paint, p28의 3×12 표는 clip 밖, p29의 3×12 및 5×4
  표는 paint/tree owner로 확인했다.

SVG 증적에서 글꼴 미설치/표기명 차이로 두부(□)가 생기는 별도 문제도 같은 stage에서
고쳤다. font binary를 SVG/저장소에 복제하지 않고 `export-svg --font-style`의 `@font-face
local()` 후보만 기록한다.

| 원 문서 face | local fallback 순서 | 실제 파일 탐색 우선순위 |
|---|---|---|
| `한양중고딕` | `한양중고딕` → `HY중고딕` → `HYGothic-Medium` | `H2GTRM.TTF` |
| `휴먼명조` | `휴먼명조` → `HumanMyeongJo` | `HMKMM.TTF` |

이 방식은 SVG 좌표와 PDF layout oracle을 바꾸지 않는다. 저작권 글꼴 data를 증적에 embed하지
않으므로, 설치된 검증 host에서는 local face로 rasterize되고 설치되지 않은 host에서는 안전한
시스템 fallback으로만 내려간다.

### 재현ㆍ증적

새 결과는 기존 before 증적을 덮어쓰지 않고
`mydocs/pr/assets/task_m100_3820_stage96_issue2279_r27_continuation/stage96-issue2279-r27-final/`
아래에 보관했다.

- `svg/86712_regulatory_analysis_065.svg`에 `휴먼명조` 및 `한양중고딕`의 `@font-face local()`
  alias가 기록됐다.
- `review/review_028.png`, `review/review_029.png`는 PDF 28--29쪽과 직접 대조한 PNG이며,
  p28에는 3×12 표가 paint되지 않고 p29에는 3×12/5×4 표가 이어진다.
- `manifest.json` 및 `analysis/metrics.json`의 final run은 SVG/render tree 65/65쪽, 요청
  PDF/raster 2/2쪽을 모두 완료하고 자동 구조 flag 0건을 보고한다. p28은 3×12 표가 없는
  문단 끝, p29는 3×12/5×4 표를 보이는 PDF owner와 일치한다. 평균 pixel match는 88.37020%,
  평균 ink match는 9.12531%지만 글꼴·rasterizer 차이에 민감하므로 이 수치만으로 수용을
  선언하지 않고 clip-aware oracle과 육안 review를 함께 사용한다.

커밋에는 위 두 review PNG, p65의 fallback SVG, manifest/metrics만 대표 증적으로 보존한다.
전체 65쪽 SVG/render-tree/PNG 복제본은 같은 입력·명령으로 재생성 가능하므로 저장소에 기계적으로
중복 보관하지 않는다. 원본 HWP와 기준 PDF는 이미 `samples/`와 `pdf/`의 추적 경로에 보존한다.

### focused 검증

```text
cargo fmt --check
# pass

CARGO_TARGET_DIR=target/pr-review CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test issue_2279_layout_oracles -- --nocapture
# 4 passed; 0 failed

CARGO_TARGET_DIR=target/pr-review CARGO_INCREMENTAL=0 \
  cargo test --profile release-test \
  renderer::svg::tests::legacy_hanyang_faces_have_portable_local_aliases --lib
# 1 passed; 0 failed

python3 -m unittest scripts/tests/test_visual_sweep.py
# Ran 37 tests ... OK
```

기존 전체 회귀에서 실제로 실패했던 baseline도 같은 target에서 재실행했다.

```text
CARGO_TARGET_DIR=target/pr-review CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test overflow_cell_baseline
# overflow_cell_lines_do_not_grow ... ok
# 1 passed; 0 failed; finished in 77.97s; exit 0
```

따라서 이전 로그의 `76076_regulatory_analysis` 신규 10줄, `86712_regulatory_analysis`
66→91줄, `issue3637` 19→23줄 증가는 현 source에서 재현되지 않는다.

### 후보 5 보정 — first inner-table atom의 safe cut

첫 fresh-page defer만 적용한 상태에서는 p28의 outer `end_cut=[1,59]`가 child p20의
3×12 표 첫 atom(unit 59)을 scalar paint에 이미 포함했다. p28 render tree에서 표가 y=968.1에
보이고 `88.2` text가 존재한 것으로 확인되어, `end_cut == first_table_atom`을 그대로
허용하는 것은 PDF contract가 아니었다.

`fresh_rowbreak_wrapper_safe_prefix_end_cut`은 native HWP5 `RowBreak`의 immediate 1×1 wrapper,
fresh row, source-provenance가 있는 첫 inner table에만 적용한다. 그 조건에서 table atom과
바로 앞 spacer를 함께 다음 page에 남기는 마지막 안전 cut을 사용한다. 일반 nested table,
이미 continuation인 row, treat-as-char table에는 적용하지 않는다.

현재 binary의 render-tree 직접 확인은 p28의 3×12 table 0개, p29의 3×12 table 1개와 뒤 5×4
table 1개다. final review PNG도 같은 owner를 보이며, #2279 oracle은 단일 thread로 3회 연속
`4 passed; 0 failed`였다. 이 보정 이후 overflow baseline도 `1 passed; 0 failed; exit 0`
(`85.64s`)로 다시 확인했다.

다음 stage에서는 이 focused gate를 보존한 채 overflow baseline과 영향 범위 focused suites를
통과시킨 뒤에만 전체 `cargo test --profile release-test --tests`를 한 번 실행한다. 이 문서의
focused 성공만으로 전체 회귀가 통과했다고 기록하지 않는다.

### 2026-08-10 정정 — p28 하단의 실제 3×12 표 paint와 safe end-cut

위의 초기 SVG/PNG 판정은 `end_cut=[1,58]`을 단순한 반열림 CellUnit 범위로 간주한 결론이었다.
후속으로 **현재 release-test CLI가 만든 SVG를 직접 조사**하자 이 결론은 유지되지 않았다.
`p28` SVG에는 y=968.1--1002.2 범위의 3×12 표 border와 `88.2` 텍스트 glyph가 실제로
기록돼 있었고, 한컴 PDF p28에는 그 표가 없다. 이는 render-tree clip-aware oracle의 실패가
거짓 양성이 아니라 실제 PDF mismatch임을 확인한다.

source provenance 진단은 child 1×1의 69 mixed unit 중 다음 경계를 보였다.

```text
unit 57: source p19, 일반 문단, h=20.0
unit 58: source=None, p19 뒤 spacing, h=2.7
unit 59: source p20, table=true, 3×12 표, h=143.7
unit 61: source p21, starts_after_table=true
unit 65: source p23, table=true, 5×4 표, h=233.5
```

원인은 outer `RowCut`의 end 값이 nested scalar renderer에 inclusive owner처럼 전달되는
projection 경계였다. 표 source 직전 spacing인 58을 end로 두어도 p20 표가 현재 page에
paint된다. 따라서 fresh p28의 안전 경계는 **57**이며, p19까지 보존하고 p19 spacing/p20 표를
다음 page로 넘긴다.

구현은 두 단계다.

1. p27의 잔여 266px에서 r27 prefix `[1,16]`가 생기고, 그 prefix 전체(926.2px)가 fresh body
   971.3px에 수용될 때 outer row를 다음 page로 defer한다.
2. fresh p28에서 first inner-table atom 59에 정확히 닿는 cut은 provenance 기반 safe cut 57로
   좁힌다. native HWP5·non-TAC·RowBreak·empty 1×1 wrapper·모든 sibling label 셀 소진이라는
   조건을 함께 만족할 때만 적용한다.

보정 뒤 focused 결과는 다음과 같다.

```text
issue_2430_cell_rewrap_threshold: 2 passed; 0 failed
issue_2279_nested_cell_units_split_r27_not_r26: 1 passed; 0 failed
```

이 결과는 p27 r27 조기 노출 없음, p28 3×12 표 조기 paint 없음, p29 3×12→5×4 순서 보존을
동시에 검사한다. 전체 4개 #2279 gate와 PDF raster p27--p29 대조, overflow baseline 및 전체
release-test는 다음 검증 단계에서 별도로 기록한다.

### 2026-08-10 PDF 재대조 — safe cut 적용 결과

safe cut 적용 뒤 현재 `release-test/rhwp`로 한컴 기준 PDF를 다시 비교했다. 기준은
`pdf/issue1921/86712_regulatory_analysis-2024.pdf`이고, `visual_sweep.py`의 1-based p27--p29
선택 실행을 사용했다. SVG export는 문서 전체 65쪽에서 수행하고 raster/overlay만 세 쪽으로
제한했다.

```text
key: stage96-issue2279-r27-safe-after
p27 proxy: 7.96782%
p28 proxy: 7.20295%
p29 proxy: 11.04767%
structural flagged pages: []
```

낮은 raster proxy는 이 오래된 HWP 표의 글꼴·줄폭·표 geometry 차이를 반영하므로 acceptance
값이 아니다. review PNG와 직접 SVG를 함께 확인한 구조 판정은 다음과 같다.

| 페이지 | 한컴 PDF 기준 | 수정 뒤 rhwp |
|---|---|---|
| p27 | r26 후행 표 fragment | r27의 `편익 수혜자`/3×12 표가 조기 paint되지 않음 |
| p28 | p0--p19와 title까지만, p20 3×12 표 없음 | p20 3×12 표 border·`88.2` glyph 없음 |
| p29 | p20 3×12 표 뒤 p23 5×4 표 | 동일한 3×12→5×4 순서로 시작 |

검토 근거는 `mydocs/pr/assets/task_m100_3820_stage96_issue2279_r27_continuation/`
`stage96-issue2279-r27-safe-after/review/review_027.png`부터 `review_029.png` 및 같은 경로의
`overlay/overlay_metrics.json`에 보존했다. 이 확인은 이 stage의 **r27/p28 boundary 결함**을
닫는 근거이며, 문서 전체의 다른 PDF fidelity 차이까지 해결했다는 주장은 아니다.

### 2026-08-10 전체 회귀 발견·범위 보정 — 59043 p35

첫 전체 `cargo test --profile release-test --tests`는 `issue_1921_59043_pagination_pin`에서
두 건 실패했다. 새 defer 규칙이 59043 마지막 행 r5에도 적용되어 PDF가 p35에 남기는 569.1px
도입부를 p36으로 이월한 것이 원인이었다. 이는 `prefix_height <= fresh_body_height`만으로는
서로 다른 RowBreak wrapper의 PDF 계약을 구별하지 못한다는 반례다.

| fixture | prefix / fresh body | PDF가 요구하는 동작 |
|---|---:|---|
| 86712 r27 | 926.2 / 971.3px | 거의 한 쪽인 prefix와 다음 표를 새 page에서 시작 |
| 59043 r5 | 569.1 / 971.3px | p35에 도입부를 남기고 p36에서 나머지 계속 |

따라서 `should_defer_fresh_rowbreak_wrapper_prefix`는 **새 본문의 80% 이상을 차지하면서
한 page에 수용되는 prefix**로 조건을 좁혔다. 이 값은 two-fixture 증거에 따른 경계 조건이며,
문서/테이블 식별자나 page number를 hard-code하지 않는다. 보정 후에는 다음을 다시 통과했다.

```text
issue_2430_cell_rewrap_threshold: 2 passed; 0 failed
issue_1921_59043_pagination_pin: 5 passed; 0 failed
issue_2279_layout_oracles: 4 passed; 0 failed
```

최종 SVG raster 증적은 `stage96-issue2279-r27-final/`로 다시 생성했으며,
특히 p28 review에서 3×12 표가 없고 p29에서 표가 시작하는 것을 재확인했다. 이 보정 뒤 전체
release-test는 별도로 재실행해 최종 exit code를 기록한다.

### 2026-08-10 반례 범위 축소 — mixed-tail rewind의 1×1 한정

80% 범위 보정 후 전체 gate는 `issue_2097_band_fill`의
`21217935_simsa_jipyo.hwp`를 8쪽 기준 대신 9쪽으로 만들었다. 이 fixture도 1×1 중첩
cell 안에 표 뒤 문단 tail을 가진다. 따라서 earlier candidate의 generic
`rewind_rowbreak_mixed_nested_table_tail_for_fresh_page`가 #2279의 해법이 아니라 기존
RowBreak 계약을 깨는 별도 동작인지 확인했다.

되감기만 환경 진단 스위치로 비활성화한 비교에서 #2097 gate는 즉시 8쪽으로 회복했고,
#2279 4개 oracle도 모두 통과했다. 원인은 helper 자체가 아니라, 다열 outer RowBreak 표에도
marker가 전파될 수 있는 범위였다. 따라서 실제 코드는 helper와 세 call site를 제거하지 않고,
**outer table이 1×1인 mixed nested-cell projection일 때만** 실행하도록 좁혔다. #2097의
17×4 표는 이 guard 밖이므로 기존 8쪽 pagination을 유지한다. 이 상태에서 다음 focused gate를
다시 실행했다.

```text
issue_2097_band_fill:               1 passed; 0 failed  (21217935 = 8p)
issue_1921_59043_pagination_pin:    5 passed; 0 failed  (59043 = 37p)
issue_2279_layout_oracles:          4 passed; 0 failed
```

결론적으로 남기는 변경은 (a) child source-paragraph provenance, (b) fresh body의 80% 이상을
차지하는 prefix만의 defer, (c) 첫 inner-table atom을 실제로 paint하지 않는 safe cut, (d) 1×1
mixed nested-cell projection으로 한정한 tail rewind다. 다열 일반 표의 완결 표/tail 되감기는
#2097 반례 때문에 포함하지 않는다. 이 stage는 다음 전체
`cargo test --profile release-test --tests`의 exit code가 0일 때만 완료로 전환한다.

### 2026-08-10 두 번째 전체 회귀 확인 — #2097 mixed-tail 범위 guard

범위 보정 뒤 전체 release-test는 `issue_2097_band_fill`의
`21217935_simsa_jipyo.hwp`에서 9쪽/한글 COM 기준 8쪽이라는 새 회귀를 잡았다. 이 fixture에는
prefix-defer 진단이 전혀 발동하지 않았으므로, 원인을 별도 mixed-tail rewind로 분리했다.

`RHWP_DISABLE_STAGE96_REWIND=1`로 layout 값만 비활성화한 대조에서는 #2097이 즉시 8쪽으로
통과했고, 같은 조건의 #2279 4개 oracle도 모두 통과했다. 따라서 p28 경계가 필요로 하는 것은
다열 표 전반의 rewind가 아니라, provenance 기반 fresh-prefix defer + inclusive owner safe cut과
1×1 nested projection에만 허용되는 rewind다. 임시 환경 스위치는 제거했고 helper에는
`row_count == 1 && col_count == 1` guard를 둔다.

제거 뒤 최종 source에서 재확인한 focused 결과:

```text
issue_2430_cell_rewrap_threshold: 2 passed; 0 failed
issue_2097_band_fill: 1 passed; 0 failed
issue_2279_layout_oracles: 4 passed; 0 failed
issue_1921_59043_pagination_pin: 5 passed; 0 failed
```

즉 86712 p28 표 조기 paint, 59043 p35 도입부, 21217935 8쪽 page-count를 같은 변경에서
동시에 보존한다. 최종 PDF sweep과 전체 release-test는 이 상태를 기준으로 다시 실행한다.

최종 source의 PDF sweep은
`stage96-issue2279-r27-final/`에 보존했다. p28 review PNG에서도 3×12 표가
없고, p29에서 첫 3×12 표가 시작하는 것을 다시 육안 확인했다. p27/p28/p29의 내용 픽셀
보조값은 각각 7.96782%, 7.20295%, 11.04767%이며, 이 값은 이 문서의 글꼴·표 geometry
차이 때문에 낮으므로 page-boundary 구조 판정의 대체값으로 사용하지 않는다.

### 2026-08-10 전체 회귀 baseline 분리 — #2308 short-child boundary

현재 source의 전체 release-test는 `issue_2308_render_normalized_derived_state` 5개 중
p81/p82 short RowBreak child owner contract 한 건에서 멈췄다. 한컴 2024 PDF 기준은 p81 첫 줄이
`… 등의 사고`까지 남고 p82가 `를 예방…`으로 이어지는 것이다.

이 실패가 Stage 96의 회귀인지 확인하려고, 변경 없는 `HEAD` archive를 별도 target에서 동일하게
실행했다. 결과는 같은 assertion 실패(나머지 4개 통과)였다. 또한 현 구현의 #2279 diagnostic에는
#2308 p81에서 Stage 96의 defer/safe-cut이 발동한 기록이 없다. 그러므로 #2308은 **현재 stage가
도입한 회귀가 아니라 기존 baseline 결함**으로 분리한다. 다만 PDF 계약과 충돌하는 실제 결함이므로
다음 stage에서 원본 PDF p81--p82 렌더·outer row 구조를 먼저 분석해 별도 보정한다. 이 stage의
전체 release-test 결과는 baseline failure 때문에 exit 0이 아니며, focused 성공을 전체 성공으로
기록하지 않는다.

### 2026-08-10 #2308 재현을 현 stage에서 계속 다루는 이유와 관측

사용자가 요구한 전체 회귀는 실패를 기존 baseline이라는 이유로 통과 처리할 수 없다. 따라서
Stage 96의 86712 보정과 원인을 분리한 뒤에도 #2308을 같은 작업 흐름에서 계속 조사한다.
독립 Oracle은 `samples/issue1891/76076_regulatory_analysis-2024.pdf` p81--p82이며, RHWP
현재 `export-text -p 80/81` 결과는 p81에 목표 child 첫 줄이 없고 p82가 다음처럼 통째로
소유함을 확인했다.

```text
p82: ○ 구내운반차 안전조치를 통해 근로자와 부딪히는 등의 사고
p82: 를 예방함으로써 산업재해 감소 …
```

이는 PDF의 p81 `… 등의 사고` / p82 `를 예방…` 분할과 다르므로 test가 과도한 것이 아니다.
`RHWP_DIAG_SHORT_CHILD=1`은 target parent가 native stored-pagination RowBreak, 5행,
마지막 host row, 1×1 non-TAC child(3 문단), parent 110.7px 대비 child flow 196.4px라서
short-child eligibility 자체는 참임을 보였다. 반면 `native_short_parent_child_row_is_fragmentable`
의 target-specific trace는 아직 출력되지 않았다. 즉 현재 failure는 source child 형태를 못 찾은
것이 아니라, 해당 table이 page-tail row-cut path에 도달하기 전에 통째로 다음 조각으로 이월되는
경로일 가능성이 크다.

다음 수정 전에는 `fit_measured_table_nested_tail_to_declared_height`의 applied/rejected 사유와
target parent의 pagination row cursor를 계측한다. page budget·font metric·전체 RowBreak tolerance를
추측으로 조정하지 않으며, p34 saved cell margin과 p81/p82 owner contract를 함께 유지한다.

### 2026-08-10 #2308 수치 근인 — short child의 선언 tail을 일반 stale-height로 오인

`fit_measured_table_nested_tail_to_declared_height` 계측에서 target parent의 measured row는
`[23.28, 23.28, 23.28, 23.28, 218.56]px`이고, table 선언 총높이 110.7px에서 앞 네 행을
제외한 마지막 row target은 17.6px이었다. 즉 reduction은 201.0px이다. 기존 helper는
`reduction <= 64px` 및 target tail이 기존 tail의 85% 이상일 때만 fit을 허용하므로 이
구조를 거절한다. 결과적으로 typeset은 p81의 남은 공간에서 마지막 child row를 분할하지
않고 p82로 통째로 보낸다.

PDF는 일반 표 높이 축소를 요구하지 않는다. 마지막 host가 text 없는 single table control이고
reset-only trailing paragraph를 가지며, child가 non-TAC 1×1·3문단 이하이고
`child.common.height > parent.common.height`인 native RowBreak short-child만 예외다.
이 조건은 동일 fixture의 p33/p34 counterexample(각각 child stored height가 parent 이하)을
제외한다. 따라서 다음 변경은 64px/85% 일반 가드를 폐기하지 않고, 이 exact structure에만
큰 마지막-tail fit을 허용한다. 그 뒤 p81에는 첫 line, p82에는 source continuation만
paint되는지를 #2308 assertion과 PDF direct 비교로 확인한다.

### 2026-08-10 PDF raster 직접 대조 — 선행 pi=831 small-tail fit 누락

PDF p81과 현재 SVG p81을 144dpi로 직접 렌더해 비교했다. current RHWP는 direct-benefit
table(pi=831)의 bottom이 PDF보다 약 50 CSS px 아래이며, 그 결과 indirect-benefit table
(pi=842)는 현 page에 62.3px만 남아 앞 3행에서 끝난다. PDF에는 pi=842의 앞 4행과
`○ … 등의 사고` 첫 줄까지 존재한다.

pi=831 계측은 measured rows가 `[23.28, 23.28, 23.28, 23.28, 23.28, 44.08, 524.37]px`,
선언 높이는 634.7px이다. 마지막 nested-tail만 50px 줄이면 PDF와 같은 다음 표 시작 위치가
나온다. 이 50px은 기존 small-drift 안전 범위(64px 이내, tail의 85% 이상)에 든다. 그러나
최근 helper가 `child paragraphs <= 3`을 **모든** fit의 전제조건으로 삼아, 13문단인 pi=831까지
제외했다. 3문단 조건은 p842처럼 큰 overflow를 예외적으로 fit할 때만 필요하다.

따라서 helper를 두 판정으로 분리한다.

1. text 없는 single-child host/reset-only tail인 1×1 child는 기존 64px/85% small-drift fit을
   문단 수와 무관하게 허용한다. 이는 pi=831의 50px PDF geometry를 회복한다.
2. 그 범위를 넘는 large fit은 3문단 이하이고 child stored height가 parent보다 큰
   short-child에만 허용한다. p842 first-line owner에만 적용되며 p33/p34의 반례를 배제한다.

이후 p81 SVG에서 pi=831 bottom과 pi=842의 4행/첫 줄을 PDF와 다시 대조하고, p34 saved margin과
focused/전체 release-test를 실행한다.

### 2026-08-10 #2308 two-tier fit 결과

두 판정을 분리해 적용했다. exact empty-host/single-child/reset-only 형태에는 문단 수와
무관하게 기존의 small-drift 한계(64px, 85%)를 유지하고, 이를 넘는 fit은 3문단 이하이며
child stored height가 parent보다 큰 short-child에만 열었다. source path나 문구를 식별자로
사용하지 않았으며, table/row/control 구조와 선언 높이만 사용한다.

수정 뒤 `issue_2308_render_normalized_derived_state`는 5/5 통과했다. PDF p81--p82를
144dpi raster로, RHWP 최종 source는 SVG로 다시 대조했다. RHWP p81에는 PDF와 같이
`○ 구내운반차 안전조치를 통해 근로자와 부딪히는 등의 사고`의 첫 줄이 남고, p82 export-text는
`를 예방함으로써 산업재해 감소…`로 시작한다. 즉 pi=831의 50px small-tail 측정 drift와
pi=842의 native short-child owner 경계가 함께 회복됐다.

같은 source에서 focused regression은 다음과 같다.

```text
issue_2430_cell_rewrap_threshold: 2 passed; 0 failed
issue_2308_render_normalized_derived_state: 5 passed; 0 failed
issue_2097_band_fill: 1 passed; 0 failed
issue_2279_layout_oracles: 4 passed; 0 failed
issue_1921_59043_pagination_pin: 5 passed; 0 failed
```

전체 `cargo test --profile release-test --tests`는 이 focused matrix와 PDF 대조가 끝난
최종 source에서 별도로 실행해 종료 코드와 test summary를 기록한다.

### 2026-08-10 전체 release-test 결과 — #4138 기준선 분리 필요

최종 source에서 `CARGO_TARGET_DIR=target/pr-review CARGO_INCREMENTAL=0 cargo test --profile
release-test --tests`를 실행했다. 최적화 빌드 713개가 완료되고 단위 3,391개와 다수 integration
fixture가 실행된 뒤, `issue_4138_split_cell_stale_linesegs`의 두 test가 191쪽을 산출해 195쪽
기대값 assertion에서 실패했다. 따라서 전체 명령은 **exit 101**이며 통과로 기록하지 않는다.

```text
split_cell_into_reflows_stale_segs_and_rebuilds_ladder: 191 pages, expected 195
split_cells_in_range_reflows_stale_segs: 191 pages, expected 195
```

이 fixture는 Stage 96의 86712/76076 nested-row 보정과 다른 편집/reflow 경로다. 그러나 focused
성공만으로 분리 결론을 내리지 않는다. 다음 단계는 변경 없는 `1ea5f0210` worktree에서 같은
`issue_4138` test를 실행해 기준선과 현재의 191/195 결과를 직접 비교하는 것이다.

기준 worktree 결과도 **동일한 exit 101, 191 pages**였다. 그러므로 Stage 96 보정이 #4138의
page count를 195에서 191로 바꾼 회귀는 아니다. 다만 195라는 oracle과 현재 `devel` 구현이
불일치하는 baseline 결함/기대값 결함 중 어느 쪽인지는 아직 미판정이다. 다음 조사에서는 test
fixture의 원본 페이지 수, edit 뒤 한컴 2020 재저장/렌더 기준이 191 또는 195 중 어디를 지지하는지
확인한 뒤에만 구현 또는 기대값을 수정한다.

### 2026-08-10 후속 완료 — #4138과 p65 fallback 증적

#4138의 한컴 2020 현재 입력 오라클 판정과 197쪽 복원은
`task_m100_3820_stage98_4138_split_cell_page_count_oracle.md`에서 완료했다. Stage 96에서
기록한 191/195는 원인 분리 과정의 중간값이며 최종 기대값이 아니다.

또한 `stage96-issue2279-r27-final/svg/86712_regulatory_analysis_065.svg`를 현재 HEAD로
재생성해 오래된 font-face 선언을 갱신했다. 기존 SVG는 `휴먼명조`의 local 후보를
`휴먼명조`, `HumanMyeongJo`로만 제한해, 이 Mac에서 bitmap EBDT 위주의 `HMKMM.TTF`를
Chrome이 선택하면 한글이 두부로 보일 수 있었다. 현재 선언은 다음 outline 우선순위를 쓴다.

```text
Batang → 바탕 → AppleMyungjo → Noto Serif CJK KR → 휴먼명조 → HumanMyeongJo
```

현재 Mac raster는 `/System/Library/Fonts/Supplemental/AppleMyungjo.ttf`를 선택했고 p65
제목 한글이 정상 출력됐다. `한양중고딕`도 설치된 `H2GTRM.TTF`의 실제 family
`HY중고딕`과 연결된다. 따라서 별도 문서별 font hack을 추가하지 않고, outline fallback을
앞세운 현재 공통 SVG 계약과 재생성 증적을 사용한다.
