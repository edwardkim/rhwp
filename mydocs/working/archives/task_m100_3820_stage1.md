---
kind: investigation
status: active
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-08-04
---

# Task #3820 Stage 1 — current `devel` 재현과 source-contract 분리

## 대상과 기준선

- 이슈: [#3820](https://github.com/edwardkim/rhwp/issues/3820),
  [#3821](https://github.com/edwardkim/rhwp/issues/3821)
- 기준 commit: `ec1b21096820112c99fdc2ba74a782377ae6f172` (`upstream/devel`와 동일)
- HWP: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
  (`50094a3db2b2003b293c5cbf43014d001aa97929acb488cef0cb7ea0e16b3113`)
- 한컴 2020 기준 PDF:
  `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
  (`7879ffee6313575132187c44c0090cd2e62c32c12c29b7eabd989181acf27b3a`)

이 문서는 **코드 변경 전에** current `devel`과 기준 PDF를 대조해 결함의 존속과
수정 범위를 확정한 분석 기록이다. 본 Stage에서 전역 `215 ↔ 219` page-map을 숫자 하나로
보정하지 않는다. 각 결함은 source owner·fragment·footnote contract를 먼저 분리한다.

## 재현 실행과 관찰

다음 selected visual sweep은 새 검토 전용 target의 `ec1b2109` binary로 실행했다.

```bash
python3 scripts/visual_sweep.py \
  --key issue3820-3821-current \
  --hwp 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --pages 94,106-108,156 --dpi 144 \
  --rhwp-bin target/task-3820-3821-fidelity/release-test/rhwp \
  --out output/task-3820-3821-fidelity/current-sweep
```

run manifest는 `run_state=complete`이며 219 SVG/render-tree page와 지정한 5장의
PDF/raster page를 모두 만들었다. `analysis/metrics.json`은 human p107의
`column_text_flow_collapse`, human p156의 `square_wrap_text_overlap`과
`column_line_band_drift`를 flag했다. detector flag는 PDF 결함 판정의 대체물이 아니며,
아래처럼 같은 page의 직접 PNG/PDF 대조로 확정했다.

| 이슈 | human page | current `devel` 대조 | 판정 |
| --- | --- | --- | --- |
| #3821 | 156 | `pi=1692/ci=1` Square 그림 64의 왼쪽에 끝나야 할 `pi=1697` 본문 9줄이 그림의 가로 영역을 가로질러 전폭으로 paint된다. PDF는 같은 본문을 그림 왼쪽 narrow band에 둔다. | **존속, P0** |
| #3820 | 94 | 표 28의 `불특정기증 (Unspecified Donation)` 행이 rhwp p94에 과보존된다. PDF는 `지정 기증`·`간접기증`에서 p94를 끝내고 마지막 행을 p95 continuation으로 둔다. | **존속, P0** |
| #3820 | 106 | 표 29의 첫 fragment row boundary가 PDF와 다르다. rhwp가 PDF p106에서 넘겨야 할 표 내용을 더 보존하여 다음 쪽 owner를 바꾼다. | **존속, P0** |
| #3820 | 107–108 | p106 표 owner 차이와 독립적으로 p107 body tail을 rhwp가 과보존한다. PDF는 body tail을 p108 상단으로 이어서 그림 52와 함께 배치하지만 rhwp p108은 그림 52부터 시작한다. | **존속, P0** |

## #3821 원인과 수정 계약

현재 `DeferredSquarePictureControl`은 next physical page에 그림을 이월하면서
`wrap_target_para_index` 하나에만 `WrapAnchorRef`를 넣는다. 실물 source의 page-tail
그림 64는 다음과 같이 여러 문단에 걸쳐 저장된 narrow line band를 쓴다.

| source para | stored line segment | 의미 |
| --- | --- | --- |
| 1693 | full-width tail 뒤 `vpos=0, cs=0, sw=25139` reset | next page band 시작 |
| 1694–1696 | `vpos=4000,4600,5200`, 같은 `cs/sw` | 빈 guide 문단; band 연결 유지 |
| 1697 | `vpos=5800..21800`, 같은 `cs/sw` | 실제로 paint되는 본문 9줄 |
| 1698 | `vpos=23800`, 같은 `cs/sw` | 그림 실제 bottom 밖의 후속 일반 흐름 |

그림의 `vertical_offset + height`는 `518 + 22713 = 23231 HWPUNIT`다. 따라서 첫 reset
문단부터 `vpos < 23231`인 연속 문단만 deferred wrap target으로 등록해야 한다. 이 방식은
빈 guide 문단을 누락하지 않으면서 p1698 이후 일반 본문으로 wrap anchor가 새는 것을 막는다.

수용 기준:

1. p1693–p1697만 target set에 포함하고 p1698은 제외한다.
2. p156의 visible `pi=1697` TextLine band가 그림 64의 left edge를 넘지 않는다.
3. previously resolved p127 Square-wrap 결함은 재발하지 않는다.
4. post-fix selected sweep에서 p156의 `square_wrap_text_overlap`과
   `column_line_band_drift`가 사라지고 review PNG가 PDF의 narrow band와 일치한다.

## #3820의 분리된 후속 분석 계약

p94 및 p106–108은 RowBreak table fragment와 body/footnote owner 문제다. #3821의
Square-picture band 변경으로 해결됐다고 간주하지 않는다.

### p94 표 28 — whole-fit과 RowBreak paint 측정 공간의 불일치

표 28은 `pi=1000/ci=0`, `RowBreak`, 4×3이다. current whole-fit gate는
`current_height=613.3px`, `table_total=322.6px`, `available=956.2px`만 보므로 표 전체를
p94에 둔다. 그러나 같은 입력의 `MeasuredTable` row footprint 합은 `401.4px`다. 즉
whole-fit은 선언/축소 높이를 사용하고, RowBreak fragment renderer는 더 큰 실측 행 높이를
사용하는 두 측정 공간이 공존한다. PDF는 마지막 source row `Unspecified Donation`을 p95
continuation으로 보내며, 이 `401.4px` 물리 footprint와 일치한다.

host의 `Footnote` control은 caption marker source로 존재하지만, p94 표 owner 결정을
만드는 원인이라고 추정하지 않는다. 이번 결함의 직접 증거는 footnote reservation이 아니라
`322.6px` whole-fit과 `401.4px` RowBreak measurement의 `78.8px` 차이다. 따라서 전역
footnote margin을 바꾸거나 host footnote queue를 새로 만들지 않는다.

수정은 다음의 좁은 source contract에서만 whole-fit과 row scan 모두 paint footprint를
사용한다.

- native HWP5, 비-글자처럼취급, `TopAndBottom` + `Para`, `RowBreak` 표;
- 모든 셀이 ordinary row(`row_span=1`)이고 table cell footnote가 없음;
- 현재 흐름이 page 하단 절반에 있고 다음 source 문단의 stored `vpos`가 table anchor보다
  위로 rewind된다;
- `MeasuredTable` 합이 cut footprint보다 명백히 크다.

이 범위에서는 `max(cut_row_h, measured_row_height)`를 first whole-fit/fragment full-row
판정에 사용한다. 부분 행을 실제로 자르는 cut 계산은 기존 `cut_row_h`를 유지한다.

### p106 표 29 — pagination cut 측정 공간과 paint 측정 공간의 불일치

표 29는 `pi=1136/ci=0`, `RowBreak`, 8×3이다. current p106 fragment의 실제 render tree는
row 0–3과 row 4의 일부를 paint하여 bottom `1078.1px`까지 내려가지만, body bottom은
`1039.3px`다. `LAYOUT_OVERFLOW`도 `38.8px`을 보고한다. 반면 PDF는 row 0–2에서 p106을
끝내고 p107에서 나머지 row를 이어 간다.

원인은 fragment scanner의 `cut_row_h`와 renderer의 `MeasuredTable::row_heights`가 서로
다르기 때문이다. 동일 source의 값은 아래와 같다.

| row | scanner `cut_row_h` px | rendered row px |
| --- | ---: | ---: |
| 0 | 79.2 | 90.9 |
| 1 | 121.9 | 133.6 |
| 2 | 79.2 | 90.9 |
| 3 | 36.5 | 48.3 |
| 4 | 79.2 | 90.9 |
| 5 | 164.5 | 176.3 |
| 6 | 57.9 | 69.6 |
| 7 | 36.5 | 48.3 |

총합도 scanner `655.0px` 대 rendered `748.8px`로 `93.8px` 차이가 난다. scanner는
`page_avail=367.6px` 안에 row 0–3 및 row 4 일부가 들어간다고 판단하지만, paint 단계는
더 큰 실제 row height를 사용해 footer/body 경계를 넘긴다. 즉 footer 높이를 전역으로
추정해 보정할 문제가 아니라, **first RowBreak fragment의 whole-row fit 판단이 paint와
동일한 measured row footprint를 사용하지 않는 결함**이다.

수정 후보는 row가 아직 자르기 전(`start_cut` 없음)인 native HWP5 ordinary RowBreak의
first-fragment scan에서 `max(cut_row_h, measured_row_height)`를 fit footprint로 쓰는
것이다. partial-row cut은 여전히 `cut_row_h` 기반으로 계산해야 하므로, 이 후보는
full-row preflight에만 적용한다. 먼저 p106 focused regression과 existing RowBreak unit
tests로 과도한 이월이 없는지를 확인한다. 이 검증 없이 global `footnote_safety_margin`이나
`pagination_tolerance_px`를 바꾸는 수정은 금지한다.

### p107–108 body tail

source에서 p1137은 표 29 anchor `vpos=44000`보다 위인 `vpos=36232`로 rewind하며, p1145는
`vpos=6000`으로 다시 page-top reset된다. 따라서 p107–108 불일치는 그림 52를 절대좌표로
옮길 문제가 아니다. p106 표 fragment를 PDF row owner로 바로잡은 후, p107의 body tail 및
p108의 `pi=1147` 그림 52가 저장된 reset 순서대로 재배치되는지를 별도 확인한다. 표 수정 후
남으면 이 항목은 별도의 text/figure owner Stage로 이월한다.

1. p94 표 28은 source `(pi, ci)`·row index·first fragment의 available height와 PDF row owner를
   비교해 마지막 행을 p95로 보내는 정확한 fit contract를 만든다.
2. p106 표 29는 footer/page-number safe boundary와 row split을 source measured height에서
   확인한다. p107에 남는 fragment가 PDF owner와 같은지를 별도 회귀로 고정한다.
3. p107–108은 표 29 결과를 고정한 뒤 body tail, footnote 1, 그림 52의 independent physical owner를
   비교한다. 그림 anchor만 옮기는 broad fix는 금지한다.
4. 각 수정 뒤 selected PDF sweep을 다시 실행하고, 아직 남은 항목은 다음 Stage 분석 문서로
   이월한다. 해결로 표시하는 것은 해당 page의 direct PDF review와 focused regression을 모두
   통과한 경우뿐이다.

## 검증 계획

코드 적용 뒤 다음 순서로 실행한다.

```text
cargo test --profile release-test issue_3821_square_picture_wrap_band_is_bounded_and_contiguous
cargo test --profile release-test issue_3821_page_tail_square_picture_wrap_reaches_visible_text_after_guides
python3 scripts/visual_sweep.py ... --pages 94,106-108,127,156 ...
git diff --check
```

테스트 명령은 검토 전용 `CARGO_TARGET_DIR=target/task-3820-3821-fidelity`와
`CARGO_INCREMENTAL=0`에서 순차 실행한다. 전체 integration suite는 이 focused 결과와
수정 범위를 확인한 뒤 별도로 판단한다.
