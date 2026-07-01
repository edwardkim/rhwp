# Task #1716 재현 샘플

## table_scattered_header_rowbreak.hwpx
- 원본: 국토교통부 행정규칙 `[별표 2] 건설공사 품질시험기준(제8조제1항 관련)`
  (건설공사 품질관리 업무지침). hwpdocs 표본 3000개 PI↔페이지 검증 최악 아웃라이어.
- 특징: pi=12 표(183행×7열, 쪽나눔 RowBreak)의 셀 4265개 중 364개에 `header="1"` 이
  상단 제목행뿐 아니라 **본문 행 전반에 흩어져** 있음.

## 버그(수정 전)
반복 제목행 overhead 를 cursor 아래 모든 `is_header` 행으로 합산 → cursor 전진 시 overhead
누적 → 가용 높이 0 → **페이지당 1행 폭주**. rhwp **173쪽** vs 한글 **52쪽** (+121).

## 수정 후
`Table::leading_header_rows()`(상단 연속 제목행 블록만)로 페이지네이터·렌더러 통일.
rhwp **53쪽**(한글 52, +1). 페이지당 46/46/44/7행 정상 배치.
(잔여 +1쪽은 별개 B 유형 미세 행높이 표류.)

## 재현
```
rhwp dump-pages "samples/task1716/table_scattered_header_rowbreak.hwpx" | grep -c "=== 페이지"
# 수정 전 173 → 수정 후 53
```
