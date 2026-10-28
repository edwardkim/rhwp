---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 99 — 정책연구 p74 각주 예약과 본문 owner

## 목적

Stage 98의 현재 HEAD 전수 원장에서 실제 PDF owner 차이로 확정된 정책연구 문서
p74→p75를 보정한다. 전체 215쪽 수를 맞춘 사실이나 자동 후보 0을 완료 조건으로 사용하지
않고, 본문·각주 경계의 줄 단위 소유와 직접 PDF raster를 함께 맞춘다.

## 시작 상태

- 시작 commit: `47734b3ac`
- source:
  `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- oracle:
  `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- 현재/기준 페이지 수: 215/215
- Stage 98 review:
  `output/task-3820-stage98-policy-candidates-review/stage98-policy-candidates/review/`
- 작업트리의 Stage 96 문서 수정은 별도 작업이므로 편집·stage·되돌리기 하지 않는다.

## 확정 현상

PDF p74는 para 839의 첫 줄 `…장기이식 환자 및 기증`에서 끝나고, p75가
`자에 대한 정보를 관리하기 위한 자동화시스템을 유지…`로 시작한다. rhwp는 이 두 번째
줄까지 p74에 배치하고 p75를 세 번째 줄부터 시작한다.

현재 render tree에서 para 839는 다음과 같다.

- line 0: p74 `y=856.5`, `h=13.3`
- line 1: p74 `y=883.2`, `h=13.3`
- FootnoteArea 시작: p74 `y=882.8`

따라서 line 1은 각주 영역을 약 13.7px 침범한다. source `LINE_SEG`도 해당 경계가
`[ts=0,vpos=58000]`, `[ts=61,vpos=0]`, `[ts=113,vpos=2000]`이므로 올바른 분할은
p74 line 0만 유지하고 p75가 line 1부터 소유하는 것이다.

## 원인 가설과 수정 범위

`native_hwp5_existing_footnote_reset_overlap_break_line`은 현재 문단에 control이 있으면
후보에서 제외한다. 또한 본문 조판 시점에는 이미 등록된 note 99의 높이만 알고, 같은
para 839의 note 100은 뒤에서 등록되어 FootnoteArea가 사후 위로 확장된다.

전역 `respect-vpos-reset`은 이 문서를 269쪽으로 과분할하므로 사용하지 않는다. 다음 exact
구조에만 projected footnote top을 계산한다.

1. 페이지에 기존 body footnote가 있다.
2. 현재 문단의 control은 footnote뿐이다.
3. 해당 footnote marker가 candidate reset 이전 줄에 있다.
4. 기존 각주 exact 높이와 현재 각주 exact 높이, between margin을 합친 projected top이
   candidate 줄과 실제 충돌한다.
5. 기존 source/flow 2px 및 collision 0.5px 안전 조건은 유지한다.

후속 `multi_note_routed`가 note 100을 marker page에 귀속시키는 기존 계약은 재사용한다.
표/그림 para 840의 이동은 이 한 줄 조기 배치의 downstream이므로 별도 float 보정을 먼저
넣지 않는다.

## 회귀 조건

- 총 215쪽 유지
- p74: `기증`까지 존재하고 `자동화시스템`은 없음
- p75: `자에 대한 정보를 관리하기 위한 자동화시스템`으로 시작
- p74 FootnoteArea에 note 99와 100이 모두 존재
- p74 para 839 bottom이 footnote separator/top을 넘지 않음
- p75에 para 839 continuation이 존재
- p74/p75 새 144dpi PDF 직접 비교와 text/layout ledger 통과

## 결과

현재 문단 control이 모두 footnote인 경우에만 marker 위치를 조사한다. reset 직전 줄에
marker가 있을 때 candidate 이전의 새 각주 content를 composer로 재측정하고, 기존 exact
FootnoteArea에 note 사이 margin과 함께 더해 projected top을 계산했다. 다른 inline
control, marker가 다른 줄에 있는 reset, HWPX/HWP3 profile에는 적용하지 않는다.

검증 결과:

- 신규 p74/p75 회귀: 1/1 성공
- `issue_3738_rowbreak_table_footnote_fragment` 전체: 27/27 성공
- 페이지 수: 215/215 유지
- 새 직접 비교: `output/task-3820-stage99-policy-p74-p75/`
- text owner/page-boundary 후보: 0건
- p74 diff 13.06%, p75 diff 12.16%

직접 비교에서 p74는 PDF와 같이 `…장기이식 환자 및 기증`에서 끝나고 note 99/100을
보유한다. p75도 PDF와 같이 `자에 대한 정보를 관리하기 위한 자동화시스템…`으로 시작한다.
따라서 이 stage의 본문/각주 owner 결함은 해소됐다. 남은 pixel 차이는 이 두 쪽의 owner
결함과 분리하여 다음 전수 후보에서 다룬다.

## 다음 stage

Stage 98 정책 원장의 다음 실제 후보인 p90→p91 표 fragment owner를 PDF와 직접 판정한다.
동시에 자동 검출기가 놓친 표 셀 우측선 침범과 partial glyph clip 후보기를 별도 stage로
추가해, 215쪽을 사람이 하나씩 지적하지 않아도 다음 결함을 우선순위화한다.
