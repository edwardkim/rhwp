---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 100 — 정책연구 p90 표 27 caption owner

## 목적

Stage 98 전수 후보의 다음 실제 차이인 정책연구 p90→p91 표 27 분할을 기준 PDF와 맞춘다.
병렬로 fidelity 자동 검출기에 셀 우측선 침범과 partial glyph clip 후보를 추가해 이후 전수
판정의 수동 의존도를 낮춘다.

## 시작 상태

- 시작 commit: `809e0e612`
- source/oracle: Stage 99와 동일한 정책연구 HWP ↔ 한컴 2020 PDF
- 현재/기준 페이지 수: 215/215
- 기존 증적:
  `output/task-3820-stage98-policy-candidates-review/stage98-policy-candidates/review/review_090.png`
  및 `review_091.png`
- Stage 96 문서와 #4138 Stage 98 산출물은 별도 작업이므로 stage/revert하지 않는다.

## 직접 판정

본문과 표 fragment의 쪽 소유 자체는 대체로 같다. 실제 차이는 표 caption이다.

- PDF p90: `표 27. EU 미성년자 생존 장기기증 허용 국가 규정`이 첫 fragment **위**에 있음
- PDF p91: `기타` continuation row 뒤에 바로 본문이 이어지고 caption이 없음
- rhwp p90: 첫 fragment 위 caption이 없음
- rhwp p91: continuation row **아래**에 caption이 늦게 붙음

현재 render tree에서 표는 `pi=962`, `ci=0`, 7×6이다. p90 fragment는
`y=715.0,h=278.5`, p91 continuation은 `y=83.2,h=83.5`이고 caption TextLine도 p91의
같은 `pi=962`, `y=166.1`에 있다. 즉 caption을 전체 표가 끝난 뒤 그리는 현재 fragment
계약이 Bottom caption처럼 동작하지만, 한컴 기준은 이 표의 caption을 첫 fragment 앞에서
그린다.

## 조사 원칙

1. source caption direction/position과 표 fragment cursor를 확인한다.
2. 특정 pi·문구·쪽 번호가 아니라 caption direction과 first/continuation fragment 구조로
   수정한다.
3. 실제 Bottom caption 반례는 유지한다. 저장 caption metadata가 Bottom인데 PDF가 first
   fragment 위에 두는 native HWP5 예외라면 그 구조 신호를 별도로 요구한다.
4. p90/p91 owner, 215쪽, 기존 caption·RowBreak 회귀를 함께 고정한다.

## 원인 확정

이 표에는 `Table::caption`이 없다. HWP5 source의 `pi=962` 자체가 다음 구조다.

- host text: `표 27. EU 미성년자 생존 장기기증 허용 국가 규정`
- 저장 line: `vpos=46000`, `line_height=1000HU`
- table: `(pi=962, ci=0)`, non-TAC `TopAndBottom`, `vert=Para`,
  `vertical_offset=1390HU`, `RowBreak`, 7×6
- 같은 문단의 table control 수: 1

즉 1000HU 높이의 host 줄은 1390HU 양수 offset 안에 온전히 들어가며, 한컴은 이 offset
lane에 host 줄을 먼저 그린 뒤 첫 표 fragment를 놓는다. 그러나 partial-table layout의
기존 규칙은 visible-host RowBreak 문단을 일괄적으로 terminal fragment 뒤로 미뤘다. 그 결과
p90에는 표만, p91에는 마지막 `기타` 행 뒤 host text가 나타났다.

수정은 native HWP5에서 다음 저장 계약을 모두 만족할 때만 기존
`pre_emit_visible_rowbreak_host_text`를 재사용한다.

1. visible non-whitespace host text
2. non-TAC `TopAndBottom` + `vert=Para` + `RowBreak`
3. 같은 host의 table control이 정확히 1개
4. 양수 `vertical_offset`
5. synthetic이 아닌 전체 저장 host span이 `vertical_offset` 이하

기존 #2015 보정이 pre-emitted host 높이를 offset에서 감액하므로 표 anchor를 이중으로
내리지 않는다. 다만 p90은 기존 각주 141의 exact boundary에서 마지막 fitting row를
회수하는 경로다. host 복원 뒤에도 p106 empty-host용 4px paint guard를 다시 빼면 마지막
row가 약 1px 차이로 밀렸다. 따라서 같은 exact visible-host offset-lane 구조에는 그
empty-host guard를 적용하지 않았다. 실제 render-tree에서 표 하단이 footnote separator를
넘지 않는 회귀 조건은 계속 유지한다.

이 조건은 진짜 post-table host 반례인 #1686(`line_height=1200HU`, `v_off=607HU`)을
배제하고, table control이 여러 개인 #2439 co-anchor stack도 배제한다.

## 결과

- 신규 p90/p91 caption/fragment 회귀: 성공
- `issue_3738_rowbreak_table_footnote_fragment`: 27/27 성공
- 인접 반례: #1549 2/2, #1686 4/4, #1755 1/1, #2439 4/4 성공
- 총 페이지 수: 215/215 유지
- 직접 PDF 비교:
  `output/task-3820-stage100-policy-p90-p91-physical/`
- p90 diff 16.61%, p91 diff 18.55%

직접 비교에서 p90은 PDF처럼 caption 뒤에 rows 0–5(relationship row 포함)를 보유하고,
p91은 `기타` continuation row만 보유한다. p91에 caption 재출력도 없다. 남은 pixel 차이는
표 글꼴·열/행 metric의 정합 문제이며 이 stage의 caption owner 결함과 분리한다.

## 다음 stage

`fidelity_compare.py`에 실제 가시 TextRun이 owning Cell 경계를 2px 이상 침범하는 후보와,
SVG clip이 glyph 상단/하단을 부분 절단하는 후보를 추가한다. 이 검출기를 76076 p34와
Stage 96 p65 증적에 적용한 뒤, 다음 실제 PDF 차이를 별도 stage에서 보정한다.
