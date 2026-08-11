# Task #4514 Stage 1 — overlay 표 필러 흐름 복원·클램프 해제

Issue: #4514. 계획: [`task_m100_4514_plan.md`](../plans/task_m100_4514_plan.md).
브랜치: `fix/4514-overlay-filler-flow` (fix/4515-table-overlap-diag 적층 — #4515 진단·
픽스처·테스트를 검증에 사용).

## 구현 (2축, +31/-2줄)

1. **typeset.rs — #1955 흡수 arming 을 흐름 소비 앵커로 한정.**
   `overlay_shape_shortcut_para` 필드 추가. #703 Shape 단축(흐름 0)으로 배치된 overlay
   표 anchor 는 후행 빈 문단 흡수를 arming 하지 않는다 — 흡수의 전제("표가 fragment 로
   플로우를 이미 소비")가 성립하지 않고, 그 빈 문단들이 유일한 흐름 공간이다(저장
   사다리 실측: 필러 각자 줄 높이 전진 합 ≈ 표 높이, 102 끝 9768 → 118 시작 45288 이
   필러 103~117 로 정확히 연속). oversized fragment 경로(#1955 원 사례)는 종전 흡수 유지.
2. **table_layout.rs — Para-기준 상향 클램프 해제 (다행 RowBreak overlay 한정).**
   쪽 하단을 넘는 앵커의 표를 body_bottom 으로 끌어올리던 클램프(880→491.4px, 555.5px
   겹침의 직접 원인)를 `overlay_multirow_rowbreak` 에서 해제 — 앵커 위치를 보존하고
   하단 bleed 는 쪽에서 잘리게 둔다. 1×1 장식 래퍼(#1271 워터마크류)는 종전 클램프 유지.

시도 후 폐기한 대안: 쪽을 넘는 overlay 표를 본문 표 경로로 보내 행 분할(crossing
escape). 표가 흐름을 소비하면서 흡수 해제가 페이지 전진과 얽혀 이중 계상이 생겼고
(+7쪽, 54쪽), crossing 판정이 앵커 위치 의존이라 연쇄 오차에 취약해 되돌렸다. 순수
사다리 흐름 + 클램프 해제가 저장 계약과 정합한다.

## 실측 (sample1-repro.hwp, 47→48쪽)

| 축 | 수정 전 (devel HEAD) | 수정 후 |
|---|---|---|
| 표 겹침 페이지 | 6쪽 (8·12·13·22·25·29, 최대 555.5px) | **0쪽** (잔여 최대 1.7px = 임계 이하 접합) |
| LAYOUT_TABLE_OVERLAP | 8건 | **0건** |
| ECR-001~005 구간 | 2쪽 (8쪽에 4개 겹침) | **3쪽 — 한컴 구조와 동일** (N: 001+002시작 / N+1: 002계속+003+004시작 / N+2: 005) |
| 총 페이지 | 47 (한컴 46) | 48 (한컴 46) |
| LAYOUT_OVERFLOW 기존 2건 | 16·42쪽 | 위치 이동만 (17·43쪽) |

시각 증적 (headless Chrome SVG 캡처, scratchpad shot-*.png / before-4514.pdf·after-4514.pdf):
- 8쪽: 수정 전 표 4개 중첩 판독 불가 → 수정 후 ECR-001 전체 + ECR-002 시작 (한컴 동일)
- 9쪽: ECR-002 계속(자리차지 RowBreak 분할) + ECR-003 전체 + ECR-004 시작 (한컴 동일)
- 10쪽: ECR-005 정상. 상단에 ECR-004 잔여분 **공백** — 아래 '남은 한계'

## 남은 한계 (후속 분리)

- **overlay 표 쪽 분할 페인트 부재**: 쪽 하단을 넘는 글앞/글뒤 표의 잔여 행이 다음
  쪽에 다시 그려지지 않는다(흐름 공간은 확보되어 공백으로 남음). 한컴은 잔여 행을
  다음 쪽 상단에 분할 페인트한다. Shape z-layer 경로에 fragment 기제가 필요 — 별도
  이슈로 등록 예정.
- 총 페이지 48 vs 한컴 46: 위 분할 페인트 부재와 host 줄 계상 잔차. 분할 구현 시 재측정.

## 검증

- 신규 `tests/issue_4514_overlay_table_flow.rs`: 전 페이지 최상위 표 무겹침(2px) +
  ECR 구간 3연속쪽 + 총 48쪽 고정 — 통과.
- `issue_4515_table_overlap_diag`(자기일관) · `issue_703` · `issue_775` ·
  `issue_1271_hwpx_behind_text_table` — 통과.
- release-test 전체·Native Skia 3종: 커밋 후 실행 (아래 기록 갱신 예정).
