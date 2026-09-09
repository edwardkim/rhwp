# PR #6626 검토 - OOXML 차트 기본 레이아웃과 서식

- 원 PR head: `1a68d9dc12d8e141083d66f7f3a9723a6465e972`
- 통합 기준: `upstream/devel` `51043f5f8d0453b9bc929233de443fa60cb3df4b`
- 통합 후보: `b1091864d`, `a8b89b34c`를 포함한 `9088bd705cafd004d703fcf4fa1a40002e9e3bee`
- reviewer: `jangster77` 요청 완료

## 판정: 승인

`#6624`의 한컴 기본 제목·축·격자·범례·plot area 규칙을 XML 선언과 기본값으로 반영한다.

## 검증 및 증적

- column, line, pie, stock, scatter 5종을 canonical 한컴 2020 PDF와 직접 비교했다. page diff는 각각 `1.84%`, `1.60%`, `1.08%`, `1.32%`, `1.68%`다.
- [column](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6626_chart_column_p001.png), [line](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6626_chart_line_p001.png), [pie](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6626_chart_pie_p001.png), [stock](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6626_chart_stock_p001.png), [scatter](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6626_chart_scatter_p001.png)

원 PR은 직접 merge하지 않고 승인된 통합 PR에서만 수용한다.
