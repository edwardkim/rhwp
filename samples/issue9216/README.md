# samples/issue9216 — 칸 첫 문단 바로 뒤의 저장 재시작

[#9216](https://github.com/edwardkim/rhwp/issues/9216)의 정식 회귀 fixture다.

공개 입법예고 첨부(교육부, 「규제영향분석서」, 국민참여입법센터 공개, hwp5, 10쪽) 원본이다.

| 항목 | 값 |
| --- | --- |
| 대상 | 구역 0 `pi=37`, RowBreak 1×1 표 안의 3×3 중첩 표, 3행 가운데 칸 |
| p[0] (3줄) | 저장 vpos 0, 1624, 3248 |
| p[1] (빈 줄) | 저장 vpos **0**, 높이 600, 줄 간격 152 |
| p[2] (3줄) | 저장 vpos 752, 2376, 4000 |
| 대조군 | 같은 표 2행 오른쪽 칸: p[1] vpos 4872 로 이어짐(쪽 경계 아님) |

- 회귀 계약: 전체 10쪽. p[0] 글은 4쪽, p[2] 글은 5쪽에 있고, 5쪽의 다음 제목 문단은 이어진 표 조각 아래에 있다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진: **2020** — `lastSavedWith.product` 가 `hancom-office-2010` 이라 저장소 정책의 2022 이하 버킷이다.

기준 PDF: [`pdf/issue9216/regulatory-impact-nested-row-restart-2020.pdf`](../../pdf/issue9216/regulatory-impact-nested-row-restart-2020.pdf)
— 345,030 bytes, SHA-256 `cea4db2e6c969edd99cc437d69a91cd49804e17530f2b36694465ce73ed28ee3`,
Creator `Hwp 2022 12.0.0.4547`(engine 2020 profile), 10쪽.
