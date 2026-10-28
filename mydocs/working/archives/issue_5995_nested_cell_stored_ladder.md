# #5995 중첩 표 셀 저장 사다리 무시 — RowBreak 1×1 선형 셀 vpos 스냅 복원

## 무엇을

`30269_붙임1)제도개선권고안.hwp`(HWP5, 22쪽) 문단 0.136 — 자리차지 1×1 RowBreak
외부 표(물리 10–11쪽, 인쇄 "- 6 -") 안에 4×2 내부표([별표 1])가 있는 문서에서,
외부 셀 27개 문단이 저장 lineseg 사다리를 무시하고 재흐름돼 한글 2020 오라클과
세로 배치가 어긋났다: 내부표 위 본문 압축(−5.2mm), 내부표 아래 없는 빈 띠(+11mm),
외부 표 총높이 −17.9mm.

## 왜 (원인)

1. **게이트 과협소**: RowBreak 1×1 선형 셀의 저장 vpos 스냅
   (`preserve_linear_single_cell_vpos`)이 측정(`table_layout.rs` 유닛 축)과
   렌더(`table_partial.rs` 문단 스냅) 양쪽에서 `table.common.vertical_offset == 0`
   을 요구했다. 이 문서의 외부 표는 저자가 남긴 **−98HU(−0.03mm)** 오프셋이 있어
   두 게이트가 모두 꺼졌고, 셀 전체가 재흐름됐다. 셀 내부 vpos 사다리는 셀 콘텐츠
   기준 좌표라 표 자신의 세로 오프셋과 무관하다.
2. **spacing_before 이중 가산**: 렌더 스냅과 표-문단 뒤 `next_vpos_y` 밀어내기가
   저장 vpos(이미 spacing_before 포함)로 스냅한 뒤 `layout_composed_paragraph` 가
   spacing_before 를 다시 더해 +1.8mm 씩 밀렸다. `table_layout.rs` 의 기존 앵커
   경로(`anchored_y - spacing_before`)와 같은 규약으로 통일했다. 표 호스트 문단은
   재가산 경로를 타지 않으므로 빼지 않는다.

## 어떻게 (변경)

- `src/renderer/layout/table_layout.rs` — 측정 유닛 축 게이트: `== 0` →
  `unsigned_abs() <= 141`(반 mm 미만은 잔여값).
- `src/renderer/layout/table_partial.rs` — 렌더 스냅 게이트 동일 완화 +
  스냅·`next_vpos_y` 에 spacing_before 이중 가산 제거(텍스트 문단 한정,
  `preserve_linear_single_cell_vpos` 형상 한정).

## 검증 실측 (한글 2020 오라클 PDF, 괘선 벡터 좌표)

| 항목 | 오라클 | 수정 전 | 수정 후 |
|---|---|---|---|
| 내부표 상단(10쪽) | 135.66mm | 126.44 (−9.2mm) | 135.11 (−0.55mm) |
| 내부표 아래 간격 | 7.6mm | 18.7 (+11.1) | 7.7 (+0.1) |
| 본문 줄 드리프트 | — | −5.2~+8.7mm 요동 | 전 구간 균일 +1.6~1.9mm |
| 외부 표 총높이(10+11쪽) | 371.9mm | 353.5 (−18.4) | 374.5 (+2.6) |
| 총 쪽수 | 22 | 22 | 22 |

잔여: 10쪽 조각 마지막 줄 1개("없는 경우에는…")가 rhwp 에서 11쪽으로 넘어간다
(조판 fit 예산 909.2px vs 저장 유닛 910.0px — 0.8px 부족). 렌더·조판이 서로
일치하는 별개의 fit-마진 축이라 후속으로 분리한다.

증적: `D:\hwpdocs_issues\5995_30269\` (원본·오라클 PDF·전후 비교 PNG·전체 덤프).
