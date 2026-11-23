# samples/issue9291 — 저장 사다리가 남긴 쪽 첫 빈 줄

[#9291](https://github.com/edwardkim/rhwp/issues/9291)의 정식 회귀 fixture다.

통계청 공개 보도자료(「2012년 6월 소비자물가동향」, HWP, 18쪽) 원본이다.

| 항목 | 값 |
| --- | --- |
| 구역 정의 | 빈 줄 감추기(SectionDef bit 19) 켬 |
| 12쪽 첫 문단 | pi=164 빈 문단 — vpos 0, 줄 높이 1400HU, 줄간격 −280HU, 문단 뒤 간격 0 |
| 다음 문단 | pi=165 「2. 소비자물가 추이 (2-2)」 — 첫 저장 줄 vpos 1120HU = 0 + 1400 − 280 + 앞 간격 0 |
| 18쪽 첫 문단 | pi=210 빈 문단 — vpos 0, 줄 높이 1800HU, 줄간격 1080HU |
| 다음 문단 | pi=211 「2012년 소비자물가 동향 공표일정」 — 첫 저장 줄 vpos 2880HU |
| 수정 전 | 빈 줄을 지워 12쪽 본문이 14.93px, 18쪽 본문이 38.40px 위에 그려짐 |

- 회귀 계약: 전체 18쪽. 12쪽·18쪽 제목 줄 윗변 − 본문 윗변 = 다음 문단 저장 vpos. 다음 문단이 빈 줄과
  같은 vpos 에 저장된 반례(메모리 변형)에서는 빈 줄을 감춰 제목 줄이 본문 윗변에 놓인다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진은 한컴 2024다. 원 PR 인쇄본의 Creator가 한컴 2022여서 같은 원본을 MCP로 다시 인쇄했다.

기준 PDF는 [consumer-price-trend-page-start-empty-line-2024.pdf](../../pdf/issue9291/consumer-price-trend-page-start-empty-line-2024.pdf)다.
Creator `Hwp 2024 13.0.0.3901`, Producer `Hancom PDF 1.3.0.550`, 18쪽, 649,391바이트.
SHA-256 `dfb56730a6038b3634e1246d6a2a8eb21805a90a33e5f64e377c41447f6cd46c`.
