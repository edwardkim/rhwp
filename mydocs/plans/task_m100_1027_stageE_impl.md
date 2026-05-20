# 구현계획서 — #1027 Stage E~F: per-item advance 완전 공유 (페이지네이터 ↔ 렌더러)

- 이슈: #1027 / 브랜치 `local/task1027`
- 선행: Stage A/B/C(커밋), Stage D 스냅(작업트리 보존·커밋 보류), Stage D 조사보고서
  `working/task_m100_1027_stageD.md`
- 설계서: `tech/shared_layout_measurement_engine.md`

## 1. 목표

페이지네이터(`typeset.rs`)의 **per-item 세로 advance** 를 렌더러(`layout.rs
layout_column_item`)의 실제 advance 와 **항목별로 동일**하게 만든다. Stage C HeightCursor
(inter-item vpos 스냅) 와 결합하면 두 엔진의 y-진행이 1:1 정합 → 페이지 분할이 렌더러가
그릴 위치와 일치 → 노트 8쪽 + overflow 무회귀.

## 2. 진단 근거 (Stage D 조사 확정)

- **plain 문단**: 페이지네이터 `format_paragraph.total_height` == 렌더러 advance (pi=346~348
  diff=0). **이미 정합** → 변경 불필요.
- **불일치는 표·Shape advance 에 국한**:
  - treat_as_char 인라인 표: 페이지네이터 `effective_height(65.2)` vs 렌더러 호스트
    LINE_SEG `fmt.total_height(82.1)` → 16.9px 과소.
  - Shape: TAC 표 보정 시 para142 Shape 가 40.6px overflow → Shape advance 도 불일치.
- 단일 항목만 고치면 불일치가 다른 항목으로 이동(whack-a-mole) → **표·Shape 동시 정합** 필요.

## 3. 원칙 — 렌더러 advance 를 단일 권위로

렌더러의 per-item 세로 advance(`y_out - y_in`, `RHWP_DEBUG_TAC_CURSOR` 로 관측 가능)를
**측정 권위**로 삼는다. 페이지네이터의 항목 높이가 이 advance 와 같아지도록 정합한다.
검증: 항목별 `페이지네이터 advance == 렌더러 dy` parity 하니스(여러 샘플).

가능한 곳은 렌더러 advance 계산식을 height-only 순수 함수로 추출(Stage A/B/C 방식)하여
양쪽이 호출. 추출이 어려운 곳(그리기 강결합)은 공식 정합 + parity 테스트로 가드.

## 4. 단계 (E1~E3, F)

### Stage E1 — 표 advance 정합
- treat_as_char 인라인 표(`typeset_block_table`/`typeset_tac_table`): 호스트 LINE_SEG
  기반 advance(렌더러 `fmt.total_height` 대응)로 정합. effective_height 직접 누적 제거.
- 블록(분할) 표: 렌더러 표 host advance(effective + outer_margin + host spacing)와 정합,
  분할 시 row-cut 높이 일치(#993/#1022 측정 공유 활용).
- parity: 표 항목 `페이지네이터 advance == 렌더러 dy` 단위/통합 테스트.
- 게이트: 노트 8쪽 유지, AI 184p overflow ≤ 13(±), svg_snapshot 무회귀.
- → `working/..._stageE1.md` + 커밋

### Stage E2 — Shape advance 정합
- TAC Shape(글상자) / 비-TAC Shape host advance 를 렌더러(`layout_shape_item`,
  shape_bottom+spacing)와 정합. para142 류 Shape overflow 해소.
- parity: Shape 항목 advance 대조.
- 게이트: E1 + Shape overflow 0, 노트 유지.
- → `working/..._stageE2.md` + 커밋

### Stage E3 — partial table / 다단 확장
- 분할 표 잔여 행, 다단(col_count>1) zone/단 경계의 advance·vpos 스냅 정합.
- Stage C 스냅의 다단 가드 해제(현재 col_count==1 한정).
- → `working/..._stageE3.md` + 커밋

### Stage F — 골든 재판정 + 최종 보고
- 병합본 골든 부채(svg_snapshot 267/617/677) 한컴 2022 PDF 재판정·복구.
- 비공개 184p sweep + 광범위 회귀(다른 표/박스/다단 문서) 페이지 수·overflow.
- → `report/task_m100_1027_report.md`

## 5. 검증 게이트 (전 단계 공통)
- 노트(pi=127) **8쪽** 유지(한컴 PDF 정합).
- AI 184p `LAYOUT_OVERFLOW` **≤ 13**(baseline) — 신규 회귀 0 목표.
- svg_snapshot 공개 5건 무회귀(3 debt 는 Stage F 재판정).
- lib 1316 + Stage C parity 유지. clippy 0.
- 항목별 parity 하니스: 페이지네이터 advance == 렌더러 dy.

## 6. 리스크·완화
- **최고 위험**: 표·Shape 측정 변경이 전 문서 페이지네이션에 영향(whack-a-mole 재발 위험).
- 완화: **표·Shape 를 한 단계 내에서 함께** 정합(부분 정합 금지), parity 하니스로 항목별
  검증, 각 단계 게이트 미달 시 롤백. plain 문단은 불변(이미 정합).
- 골든 광범위 재판정 불가피 → Stage F 한컴 PDF 대조.

## 7. 비범위
- 렌더러 백워드 클램프(8px) 자체 변경(Stage D 실험상 단독 효과 제한적·부작용 큼) — 보류.
- #1025 page-larger 단일 셀 분할. WASM 재빌드.
