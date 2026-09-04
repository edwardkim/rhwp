# Task #1718 Stage 4 — 회귀 전수 + 재현샘플

## 코퍼스 회귀 (hwpdocs, 한글 COM 대조)
이전 3,249건 판정 기준, 변경 영향 파일 재검증:

**이전 mismatch(195) + match 표본(300) 재검증 (한글 COM 크래시분 ERR 제외):**
- mismatch → MATCH 개선: **25건** (PI_MISMATCH→MATCH 17, PAGE_DELTA→MATCH 8)
- MATCH → mismatch 회귀: **0건** (완료 mismatch측 + match 표본 222건 검증)
- match 표본 300: 222 MATCH 유지 / 0 회귀 (78 ERR = 한글 COM 크래시, rhwp 무관)

> 한글 COM 은 대량 배치에서 연쇄 크래시(harness 한계) — 완료분 기준 순효과는 **개선 25 / 회귀 0**.

## 재현 샘플
- `samples/task1718/table_giant_cell_overfill.hwp` (한글 48쪽) + README.

## 결론
`.any()`→`.all()` 정정으로 대형 RowBreak 셀 텍스트 over-fill 해소. 회귀 0, 개선 25+.
승강기 대표는 40→42(텍스트 밀도분 해소); 잔여 42 vs 48 은 이미지/개체 배치 영역으로 별도 후속.
