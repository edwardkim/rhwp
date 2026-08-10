---
kind: investigation
status: in_progress
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 118 — 각주·미주 구분선 고정 길이 해석

## 범위와 시작 상태

- 브랜치: `task/3820-production-fidelity`
- 시작 commit: `26e2c7460`
- source: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- reference: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- Stage 115에서 p120 표 outer origin은 별도 해결했다. p120·p121 각주 구분선은
  source `separatorLength=-1`인데도 rhwp가 본문 폭의 1/3로 그려 한컴 PDF보다 길다.
- Stage 117은 동시 issue2007 exact-font 작업에 사용됐으므로 이 독립 보정은
  Stage 118로 진행한다.

## 정답 계약과 dual oracle

OWPML `NoteShapeType.noteLine.length`의 공식 스키마 설명은 다음과 같다.

- `0`: 구분선 없음
- `-1`: 5cm
- `-2`: 2cm
- `-3`: 단 크기의 1/3
- `-4`: 단 크기 전체
- 그 외 양수: HWPUNIT 단위 사용자 지정 길이

FootNoteShapeType과 EndNoteShapeType이 같은 base를 상속한다. HWP5
`HWPTAG_FOOTNOTE_SHAPE`도 signed i16 슬롯으로 읽고 쓰므로 negative sentinel은
parser/serializer에서 이미 보존된다.

정책연구 p120/p121의 기존 rhwp 선은 96dpi에서 `201.5733px=151.18pt`다.
스키마의 `-1` 정답은 `96 × 5 / 2.54 = 188.9764px=141.732pt`다. 한컴 PDF는
raster 추정이 아니라 vector path 자체가 p120 `M 71 764 L 214 764`, p121
`M 71 753 L 214 753`으로 폭 `143pt`다. 따라서 스키마 정답과 한컴 PDF 사이에도
약 `1.27pt`의 방출 잔차가 있다. 이번 단계는 source semantic인 정확한 5cm를
자동 회귀로 고정하고, PDF 확대 판독에서는 기존 `+8.05pt` 과장을 약 `-1.40pt`
잔차로 축소했는지를 함께 본다. 이를 pixel-identical 완료로 표현하지 않는다.

## 원인

실제 p120/p121 caller는
`layout/picture_footnote.rs::layout_footnote_area`다. 이 경로는

- 양수를 `separator_length / 50000 × footnote width` 비율로 잘못 해석하고,
- 모든 `<=0`을 FootnoteArea 폭의 1/3로 처리했다.

별도 `layout_endnote_separator_item` 경로는 양수 HWPUNIT은 맞지만 모든 `<=0`을
단 너비의 1/3로 처리했다. 그래서 실제 p120/p121의 `-1`이 잘못됐고, 고정 길이
`-2`와 양수 HWPUNIT도 동일한 수평 해석 결함이 있었다.

감사 중 다음 두 계약은 이 변경과 분리해야 함을 확인했다.

1. `0=구분선 없음`은 선 길이뿐 아니라 separator 위·아래 여백과 선 높이 예약을
   함께 바꾸는 pagination 계약이다. 이번 수평 paint 보정에서는 resolver가 폭 0을
   반환하게 하되, 0폭 `LineNode` 생성과 기존 수직 예약 제거까지 완료했다고
   주장하지 않는다.
2. FootnoteArea는 현재 다단 각주도 body 전체 폭으로 합친다. 따라서 footnote의
   상대 sentinel `-3/-4`를 올바르게 그리려면 실제 소유 단의 x·width와 placement를
   먼저 모델링해야 한다. 이 단계에서 `-4`를 body 전체 폭으로 넓히면 새 회귀가 된다.

## 채택한 최소 수정

- 공통 `note_separator_length_px(raw, available_width, dpi)`는 `0/-1/-2/-3/-4`와
  양수 HWPUNIT의 공식 수평 의미를 해석하고 결과를 사용 가능한 폭으로 clamp한다.
- endnote caller는 실제 `col_area.width`를 갖고 있으므로 공통 resolver를 그대로 쓴다.
- footnote caller는 구조와 무관한 `-1/-2/positive` 및 `0`만 공통 resolver로 고친다.
  실제 단 폭을 모르는 상대 `-3/-4`는 이번 단계에서 기존 1/3 동작을 보존한다.
- parser, serializer, typeset, page owner, separator의 수직 y·예약 높이는 바꾸지 않는다.

이 범위는 p120/p121의 실물 결함을 고치면서 다단 FootnoteArea 구조 변경을 섞지 않는
가장 작은 변경이다.

## 회귀 계획

- 순수 resolver: 폭 600px, 96dpi에서 `0=0`, `-1=188.976`, `-2=75.591`,
  `-3=200`, `-4=600`, `7200HU=96`, oversized positive=600을 고정한다.
- endnote caller: `-2/-4` direct Line 폭을 각각 2cm/단 전체로 고정한다.
- footnote caller: 양수 `7200HU`가 비율이 아니라 정확히 96px인지 direct Line으로
  고정한다.
- 정책연구 p120/p121: FootnoteArea direct-child separator 폭을 둘 다
  `188.976±0.05px`로 단언하고 시작 x, 215쪽, body/footnote owner와 비겹침을
  보존한다.
- 기존 #1139 p9 50mm 녹색 endnote 선, #1050 HWPX `-1` parse,
  HWPX serializer `issue1984_footnote_shape_reflects_ir`를 재실행한다.
- 코드 수정 뒤 첫 Cargo gate는 사용자 지시대로
  `tests/issue_2430_cell_rewrap_threshold.rs`다.

## 1차 구현·검증

첫 candidate에서 p120/p121 `-1`과 endnote sentinel을 바로잡았고 다음이 통과했다.

- 코드 수정 후 첫 gate `issue_2430_cell_rewrap_threshold`: 2/2
- 범위를 좁힌 최종 resolver + footnote/endnote direct caller: 4/4
- 정책연구 p120 exact 실물: 1/1
- `issue_1050_footnote_serialize`: 7/7
- HWPX serializer `issue1984_footnote_shape_reflects_ir`: 1/1
- #1139 p9 50mm 녹색 endnote 선: 1/1
- `issue_3738_rowbreak_table_footnote_fragment` 전체: 33/33
- `cargo fmt --check`, `git diff --check`,
  `cargo clippy --profile release-test --all-targets -- -D warnings`: 통과

candidate sweep은 전체 SVG/render-tree 215/215, 요청 p120~p121 2/2,
automatic flag 0이었다. 두 페이지 separator는 모두 `x=94.5px`,
`width=189.0px`로 정확한 5cm와 일치했고 y는 p120 `1019.0px`, p121
`1003.3px`로 유지됐다. 다만 첫 candidate가 `0`과 footnote `-3/-4`까지 한 번에
일반화해 새 회귀 위험이 있었으므로 감사 결과에 따라 위 최소 범위로 좁혔다.

candidate output:
`output/task-3820-stage118-policy-note-separator-candidate/`

## 잔여 이슈

- `separatorLength=0`에서 수직 separator 예약까지 제거하는 pagination 계약
- 다단 footnote의 `EachColumn/RightColumn` placement와 상대 길이 `-3/-4`
- 한컴 PDF의 143pt vector와 스키마 5cm(141.732pt) 사이 약 1.27pt 방출 잔차
- PDF와 다른 separator 선 굵기·농도

위 항목은 p120/p121 고정 길이 결함과 별도 stage에서 분석한다.

## 최종 판정

범위를 좁힌 source에서 focused gate를 다시 실행하고, commit 뒤 동일 binary로
p120/p121 144dpi sweep을 재생성한 후 완료 판정을 기록한다.
