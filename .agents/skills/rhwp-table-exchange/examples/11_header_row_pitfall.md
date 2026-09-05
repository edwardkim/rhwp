# 11 — 헤더 행 함정 (첫 줄 = 0행)

권위: 레시피 02, [pitfalls.md](../references/pitfalls.md) §2.

## 잘못된 편집

```csv
서버 이관,홍길동,1차 완료
DB 백업,김철수,진행중
문서 정리,박영희,대기
```

표는 4행이다. → [07](07_row_count_mismatch.md).

행 수를 맞추려고 빈 줄을 하나 더하면, 0행 `제목/담당자/세부 내용` 이
`서버 이관/홍길동/1차 완료` 로 덮인다.

## 올바른 편집

```csv
제목,담당자,세부 내용
서버 이관,홍길동,1차 완료
DB 백업,김철수,진행중
문서 정리,박영희,대기
```

헤더를 바꿀 때만 0행을 고친다. 그때 `changed` 에 `row:0` 이 보인다.

픽스처: [../fixtures/csv/table0_header_dropped.csv](../fixtures/csv/table0_header_dropped.csv),
[../fixtures/matrices/header_row.json](../fixtures/matrices/header_row.json).
