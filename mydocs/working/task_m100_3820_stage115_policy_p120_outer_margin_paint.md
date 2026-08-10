---
kind: investigation
status: in_progress
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 115 — 정책연구 p120 빈 host 표의 outer-margin paint origin

## 범위와 시작 상태

- 브랜치: `task/3820-production-fidelity`
- 시작 commit: `dba5cd586`
- source: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- reference: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- Stage 110에서 p120→p121 body/footnote owner는 stale 원장으로 폐기했으나,
  p120 `pi=1283/ci=0` 표의 paint origin이 한컴 PDF보다 약 1mm 좌·상단에
  있음을 별도 실결함으로 확정했다.
- Stage 111~114는 issue2007 전수 재감사와 정책연구 잔여 원장 감사에 이미
  사용되었으므로 이 보정은 Stage 115에서 진행한다.

## 정답 형상

`pi=1283`은 빈 host 문단의 단일 `6×1` 표다.

- non-TAC, `TopAndBottom`, `RowBreak`
- `VertRelTo::Para`, `VertAlign::Top`, vertical offset 0
- `HorzRelTo::Column`, `HorzAlign::Left`, horizontal offset 0
- declared size `41954×23790 HWPUNIT`
- outer left/right/top/bottom 모두 `283 HWPUNIT`
- host stored vpos 0, 다음 실제 문단 stored vpos `24356`
- `24356 = 23790 + 283 + 283`

현재 표 bbox는 96dpi 기준 약 `(94.49, 83.16, 559.39, 317.20)px`이고,
한컴 PDF에 맞는 bbox는 약 `(98.27, 86.93, 559.39, 317.20)px`다. source의
outer-left/top 283HU를 paint origin에 더하면 PDF 외곽과 0.3pt 이내로 일치한다.

반면 다음 본문 `pi=1286`의 `7. 스페인` baseline은 현재 `355.28pt`, PDF
`355pt`로 이미 일치한다. 표의 paint subtree만 이동하고 flow cursor, 다음 본문,
page owner와 전체 215쪽은 움직이면 안 된다.

## 원인

파서는 CTRL_HEADER의 outer margin을 정상 복원하고 typeset도 빈 TopAndBottom
float의 outer top/bottom을 예약한다. 그러나 whole-table layout의 depth-0
Column/Left x 및 Para/Top y 공식은 outer-left/top을 paint origin에 적용하지 않는다.

과거 모든 depth-0 표에 outer margin을 적용한 변경은 실물 회귀 때문에
`65013dbc4`에서 되돌려졌다. #2097 다중 열 empty-host 표와 #2439 양수-offset
visible-host 표는 서로 다른 저장 좌표 계약이므로 broad 복원 대상이 아니다.

## 채택할 최소 계약

다음 조건을 모두 만족할 때만 source가 저장한 outer-box 사다리로 판정한다.

1. native HWP5 stored layout
2. 현재 활성 구역이 단일 단이고, 빈 host 문단의 top-level control이 표 하나뿐임
3. non-TAC, Para-relative TopAndBottom RowBreak
4. vertical Top/Inside, horizontal Column Left, x/y offset 0
5. 2행 이상·단일 표 열이고 각 행이 `col=0`, `row_span=1`, `col_span=1`인
   정확한 row ladder이며 declared size와 네 방향 outer margin이 양수·동일
6. host와 다음의 빈 무-control 문단에 synthetic가 아닌 실제 LINE_SEG가 있음
7. `next_vpos - host_vpos == declared_height + outer_top + outer_bottom`
   (저장 반올림 허용 1HU)
8. caption이 없고 측정 표 높이가 declared height와 `±0.5px` 이내로 일치함

저장 vpos 사다리는 세로 outer box를 직접 증명하므로, 수평 inset까지 확장하는 범위는
p120처럼 네 방향 margin이 모두 같은 경우로 한 번 더 제한한다. 이 계약은 fixture의
paragraph ID를 사용하지 않는다. `layout_table`에는 전용 paint
inset 여부만 전달하며, 표 bbox와 모든 자식에 left/top을 더한다. 실제 painted bottom은
그대로 보고하되 main caller의 flow advance에서는 top inset을 빼 현재 흐름을 보존한다.

## 회귀 계획

### 양성 실물

- p120 `pi=1283/ci=0` table bbox x/y는 현재 Column/body origin에서 각각
  `283HU` 이동한다.
- width/height는 `41954×23790HU`를 유지한다.
- p120 `pi=1286` baseline과 p120/p121 body/footnote owner는 불변이다.
- 전체 page count는 215다.

### 음성·관련 실물

- `issue_2430_cell_rewrap_threshold`를 코드 수정 직후 첫 Cargo gate로 실행한다.
- p120 exact 회귀와 `issue_3738_rowbreak_table_footnote_fragment` 전체를 실행한다.
- #2097 empty-host 다중 열 표, #2439 visible-host stack, #1772 outer-margin sync를
  실행해 broad margin 회귀가 없음을 확인한다.
- predicate 단위 회귀는 exact ladder 양성 및 짧은 ladder, 양수 offset, 복수 control,
  HWPX/TAC/Square/non-Para/1×1/synthetic line 음성을 고정한다.

## 구현

- `native_empty_host_physical_outer_box_paint_inset`이 위 저장 사다리와 구조를
  fixture ID 없이 판정한다.
- layout caller는 `zone_layout.column_areas.len()`을 `ColumnItemCtx`로 전달해
  실제로 채워진 `column_contents` 수가 아니라 활성 구역의 권위 단 수를 사용한다.
- `layout_table`의 main whole-table 호출만 paint inset을 전달하며, 다른 12개 호출은
  기존 경로를 유지한다.
- 표 bbox와 셀·테두리·자식은 outer-left/top만큼 이동한다. 반환된 visual bottom은
  실제 ink 위치를 유지하고, caller의 flow advance에만 top inset을 빼 후속 본문
  좌표를 보존한다.
- 단위 회귀는 활성 2단 구역, 측정 높이 불일치, 비대칭 margin, 잘못된 row ladder,
  복수 표·다음 object host·HWPX/TAC/Square/positive offset을 모두 거부한다.

## 검증 결과

안정된 최신 source에서 다음 결과를 확인했다.

- `issue_2430_cell_rewrap_threshold`: 2/2
- outer-box predicate/layout gate 단위 회귀: 3/3
- p120 exact 실물 회귀: 1/1
- `issue_3738_rowbreak_table_footnote_fragment` 전체: 33/33
- #2097 관련 6개 integration binary: 전부 통과
- #2439: 4/4, #1772: 2/2
- #1285: 2/2, #1692: 11/11, #501: 1/1, #898: 1/1
- KTX/exam_kor/복학원서 SVG snapshot: 각 1/1
- `cargo fmt --all`, `git diff --check`,
  `cargo clippy --profile release-test --all-targets -- -D warnings`: 통과
- commit 이후 동일-binary 최종 p120~p121 sweep은 아래 최종 판정 전에 완료한다.

dirty candidate sweep은 전체 SVG/render-tree 215/215, 요청 120~121 2/2,
automatic flag 0을 기록했다. p120 표 bbox는
`(98.3, 86.9, 559.4, 317.2)px`로 이동했고 다음 본문 `pi=1286`은
`(94.5, 461.2, …)px`로 유지됐다. p120 pixel match는 선행 92.21622%에서
92.79628%로 상승했고 p121은 90.59494%로 불변이다. 이 candidate는 최종 provenance로
사용하지 않으며, 코드 commit 뒤 동일 binary로 다시 산출한다.

각주 구분선 `separatorLength=-1` 오해는 이 표 paint 보정과 독립된 결함이다.
Stage 116에서 footnote/endnote 공통 sentinel resolver로 별도 처리한다.
