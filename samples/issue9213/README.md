# samples/issue9213 — 여러 문단 캡션의 음수 줄 간격

[#9213](https://github.com/edwardkim/rhwp/issues/9213)의 정식 회귀 fixture다.

공개 보도자료(고용노동부, 「2022년 상반기 직종별사업체노동력조사 결과」, 정책브리핑 공개, HWPX, 32쪽) 원본이다.

| 항목 | 값 |
| --- | --- |
| 대상 | 구역 3 `pi=158`, 글자처럼 취급 25×14 표, 위 캡션(간격 283HU) |
| 캡션 문단 1 | 저장 줄 vpos 0, 높이 1100, 줄 간격 −552 |
| 캡션 문단 2 | 저장 줄 vpos 548, 높이 900 |
| 호스트 줄 | vpos 5013, 높이 30730 = 표 28999 + 캡션 1448 + 간격 283 |
| 다음 표 `pi=159` | 호스트 줄 vpos 37043 (앞 줄 끝 35743 + 1300) |

- 회귀 계약: 전체 32쪽. 30쪽에서 캡션은 표 위에 있고, 표는 호스트 줄 안에서 끝나며, 다음 표와의 틈은 저장 간격 1300HU 다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진: **2020** — `lastSavedWith.product` 가 `hancom-office-2018` 이라 저장소 정책의 2022 이하 버킷이다.

기준 PDF: [`pdf/issue9213/labor-survey-2022h1-caption-ladder-2020.pdf`](../../pdf/issue9213/labor-survey-2022h1-caption-ladder-2020.pdf)
— 1,235,936 bytes, SHA-256 `e834e74b61c8236b9af521503658975a1985c33e10ee8d93f51b34d7ac61e8b0`,
Creator `Hwp 2022 12.0.0.4547`(engine 2020 profile), 32쪽.
