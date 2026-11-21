# samples/issue9272 — 나뉜 표 끝 조각 바로 아래의 빈 줄

[#9272](https://github.com/edwardkim/rhwp/issues/9272)의 정식 회귀 fixture다.

산업통상자원부 입법예고 공개 문서(「불공정무역행위조사 및 산업피해구제에 관한 법률 시행령
일부개정령(안)」, HWPX, 한컴오피스 2020 저장, 7쪽) 원본이다.

| 항목 | 값 |
| --- | --- |
| 나뉜 표 | 구역 0 문단 31 — 15×2 RowBreak 신구조문 대비표, 6쪽에 행 12..15 끝 조각(584.2px 에서 끝) |
| 빈 문단 | 문단 32~42 — 저장 `vpos` 43813HU(= 584.2px)부터 2560HU 간격, 줄 높이 1600HU |
| 되감긴 표 | 문단 43 — 용지 기준 자리차지 표(3×2 「의안 소관 부서명」), host 저장 `vpos=2560` |
| 본문 높이 | 70012HU — 문단 41(66853+1600)까지 6쪽, 문단 42(69413+1600)는 7쪽 맨 위 |

- 회귀 계약: 전체 7쪽. 6쪽에 빈 문단 32~41 이 대비표 끝 조각 아래에서 줄 상자를 하나씩
  차지하고 문단 43 의 표는 없다. 7쪽에 문단 43 의 표가 한 번 그려지고 빈 문단 42 의 줄이
  문단 44 의 줄보다 위에 있다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진: **2020** — `version.xml` 의 `appVersion="11, 0, 0, 2129"`(한컴오피스 2020).

기준 PDF: [`pdf/issue9272/trade-remedy-decree-table-tail-empty-lines-2020.pdf`](../../pdf/issue9272/trade-remedy-decree-table-tail-empty-lines-2020.pdf)
— 175,063 bytes, SHA-256 `8d5576d2a6ffc8f3c3499c2f5981f43f26134a6d07111544fff347e5d7f61b85`,
Creator `Hwp 2022 12.0.0.4547`(engine 2020 profile), 7쪽.
