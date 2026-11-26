# samples/issue9290 — 표로 여는 쪽의 저장 쪽 원점

[#9290](https://github.com/edwardkim/rhwp/issues/9290)의 정식 회귀 fixture다.

고용노동부 공개 보도자료(「2025 우리 노동부 타운홀 미팅 개최」, HWPX, 3쪽) 원본이다.

| 항목 | 값 |
| --- | --- |
| 3쪽 첫 문단 | pi=25 — 글 없는 host, 문단 앞 간격 1500HU, 첫 저장 줄 vpos 1500HU |
| 개체 | 문단 기준 자리차지 표 1×3(「붙임」), 세로 오프셋 0 |
| 다음 문단 | pi=26 첫 저장 줄 vpos 5400HU = 1500 + 줄 높이 1500 + 줄간격 900 + 문단 앞 간격 1500 |
| 수정 전 | 표 아래 본문 전체가 20px(1500HU) 위에 그려짐 |

- 회귀 계약: 전체 3쪽. 3쪽 표 윗변 − 본문 윗변 = 표의 바깥 위 여백, 표 아래 본문 줄
  (pi=26..=33)의 윗변 − 본문 윗변 = 각 줄의 저장 vpos.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진: **2020** — `info --json` 의 `lastSavedWith.product` 가 `hancom-office-2022`
  (2024 저장본이 아니므로 engine 2020).

기준 PDF: [`pdf/issue9290/labor-townhall-table-opened-page-2020.pdf`](../../pdf/issue9290/labor-townhall-table-opened-page-2020.pdf)
— 216,786 bytes, SHA-256 `20f1811357f632b27e12949e420fbda9f72453790129cc202c7080a6b073ae39`,
Creator `Hwp 2022 12.0.0.4547`(engine 2020 profile), 3쪽.
