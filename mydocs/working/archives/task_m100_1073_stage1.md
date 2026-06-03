# Stage 1 보고서 — Task #1073: 조사 + 타당성(go/no-go) 평가

- 브랜치: `local/task1073` (소스 무변경)

## 근본 원인 확정

분할 컷 모델이 **중첩 표를 단일 atomic 유닛으로 취급** → 페이지 경계에서 쪼갤 수 없음.

- `cell_units`(`table_layout.rs:3519~3571`): 셀 문단에 `Control::Table`(중첩 표)이 있으면
  (`has_table_in_para`) **CellUnit 1개**(height = 중첩 표 전체 높이, vis 0..line_count) push.
- `advance_row_cut`(`table_layout.rs:3652`): 셀 유닛을 순회하며 `avail_height` 까지 누적 후
  컷. 시작 유닛(j==start)은 항상 소비 → 중첩 표 atom 1개는 통째 소비, `fully_consumed=true`.
- `typeset_block_table` 분할 walk(`typeset.rs:3135~3158`): `is_row_splittable` + advance_row_cut
  `fully_consumed` → 페이지 시작 행이면 **강제 통째 배치(오버플로 감수)**(3137~3140).

→ kps-ai pi=674: 외부 3×6 래퍼표 셀[0](rs=1,cs=6)이 **29행 중첩 표**(~1676px) 보유. 이 행이
페이지(918px) 시작에서 atom 통째 배치 → **758px overflow**(y=1805).

## 한컴 정답지 (확정)

`pdf/kps-ai-2022.pdf` p62~63: 표를 **행 단위로 페이지 분할**(p62 제목+1.기본정보+2.운영계획+
3.민간소프트웨어 / p63 시장침해+4.필요성+5.종합의견+푸터). = 중첩 표 행 경계 분할.

## 기존 자산 (부분)

- 렌더러 `table_partial.rs:1067~1132`: 중첩 표가 셀 가용 공간 초과 시 `calc_nested_split_rows`
  로 **행 범위 필터(부분 렌더)** 지원. 단 **단일 셀 `available_h` 기반**이며, 페이지네이션
  컷이 아니라 렌더 시점 inner_area 로 계산. 페이지네이션이 중첩 표를 atom 으로 두므로
  inner_area = 중첩 표 전체 높이 → `nested_h <= available_h` → 분할 미발동 → 통째 렌더.
- 즉 **부분 렌더 primitive 는 있으나, 페이지네이션→렌더 다중 페이지 흐름이 미연결**.

## 부가 관찰
- pi=674 표 `vert=문단(4294967155≈-141)` 비정상값 — 어울림(Square) wrap. overflow 의 직접
  원인은 atom 미분할이며 vert offset 은 2차(배치 y). Stage 2 에서 분리 확인.

## go/no-go 평가

**중첩 표 페이지 분할 = 중규모~대규모 신규 기능.** 필요한 변경:
1. `cell_units`: 중첩 표 셀을 **per-중첩행 유닛**으로 분해(현재 atom 1개) — 컷 모델 핵심 변경.
2. `advance_row_cut`/`row_cut_content_height`: 유닛 기반이라 대체로 수용하나 vis 의미(현재 줄
   범위) 와 중첩행 범위 충돌 → 의미 확장 필요.
3. 부분 렌더: typeset 컷 → `NestedTableSplit`(중첩행 범위) 매핑을 페이지 chunk 별로 배선.
- **리스크**: `cell_units`/`advance_row_cut`/`row_cut_content_height` 는 **모든 표 분할이 공유**
  하는 단일 측정·컷 공간 → 광범위 표 문서 회귀 위험. 부분 렌더 매핑도 신규.

## PoC 결과 (C안 — 회귀폭 측정 시도)

`cell_units`(table_layout.rs) 의 중첩 표 atom 을 **per-중첩행 유닛으로 분해**하는 PoC
(`POC_1073` env gate) 적용 → kps-ai pi=674 overflow **758px 불변**.

원인: 분할이 cell_units 한 곳이 아니라 **독립 4 레이어에서 atom 취급**되어, 단일점 변경으로는
경로가 활성화조차 안 됨:
1. `is_row_splittable`(height_measurer.rs:1499): 셀 `line_heights.len() > 1` 일 때만 true.
   중첩 표 셀 = line_heights 1개(atom) → **false → advance_row_cut 경로 진입 차단**(typeset.rs:3135).
2. `MeasuredCell.line_heights`: 중첩 표를 단일 높이로 측정 → splittable 신호 없음.
3. `cell_units`: atom 1개(본 PoC 대상).
4. 부분 렌더(table_partial.rs): 컷→`NestedTableSplit` 매핑은 단일 셀 available_h 기반.

→ **회귀폭을 단일점 PoC 로 측정 불가** — 측정하려면 1~4 를 동시에 바꿔야 하며, 이는 사실상
전면 구현(A)의 대부분. PoC 자체가 "값싼 격리 변경 불가능 + 다층 좌표 변경 필수"를 입증.

## 갱신 권고 (작업지시자 결정 필요)
- **(A) 전면 구현 (별도 대형 타스크)**: is_row_splittable + MeasuredCell 측정 + cell_units +
  row_cut_content_height + 부분 렌더 매핑을 함께. 정합 최상이나 공유 컷 모델 광범위 회귀 검증 부담.
- **(B) 보류 + 한계 문서화 (권고)**: 본 결함을 known limitation 으로 기록(troubleshootings),
  C군(대용량 누적 드리프트)/D군(Shape) 우선 — 비용 대비 효과 우위.

→ PoC 결과상 값싼 부분 수정 경로가 없으므로 (A)는 독립 대형 타스크로 분리, 현 #1073 은 (B)
권고. 방향 승인 요청.
