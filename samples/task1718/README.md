# Task #1718 재현 샘플

## table_giant_cell_overfill.hwp
`[별표 27] 경사형 휠체어리프트 안전기준(승강기안전부품 안전기준 및 승강기 안전기준)` 원본.

- 출처: 행정규칙 코퍼스(hwpdocs), 한글 2022 PDF 기준 **48쪽**.
- 구조: 전체가 단일 5행×1열 RowBreak 표, 마지막 거대 셀에 본문 654문단 + 그림 14 + 중첩표.

## 증상 / 검증
```
rhwp dump-pages samples/task1718/table_giant_cell_overfill.hwp | grep -c global_idx
```
- 수정 전: **40쪽** (한글 48 대비 −8, under-pagination)
- 수정 후: **42쪽** (over-fill grace 정정으로 텍스트 페이지 밀도 한글 정합)

원인: `advance_row_cut`/`advance_row_block_cut` 의 `visible_tail_before_spacer` grace 가
`.any(spacer)` 라 거대 셀 연속 텍스트 중간에서도 avail+120px 오버플로를 수용 → 페이지당
+1~5줄 over-fill. `.all(spacer)` 로 정정(진짜 꼬리줄만 grace).

> 남은 42 vs 48 격차는 이미지/개체 배치 영역(별도 후속). 본 수정은 텍스트 밀도 over-fill 해소분.
