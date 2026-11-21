# samples/issue9291 — 저장 사다리가 남긴 쪽 첫 빈 줄

[#9291](https://github.com/edwardkim/rhwp/issues/9291)의 정식 회귀 fixture다.

부산광역시 금정구 공개 과업지시서(「금정구청사 중장기 기본구상 용역」, HWP, 14쪽) 원본이다.

| 항목 | 값 |
| --- | --- |
| 구역 정의 | 빈 줄 감추기(SectionDef bit 19) 켬 — flags `0x00080000` |
| 2쪽 첫 문단 | pi=21 빈 문단 — vpos 0, 줄 높이 1800HU, 줄간격 1080HU, 문단 뒤 간격 0 |
| 다음 문단 | pi=22 「목 차」 — 첫 저장 줄 vpos 2880HU = 0 + 1800 + 1080 + 문단 앞 간격 0 |
| 수정 전 | 빈 줄을 지워 2쪽 본문 전체가 38.4px 위에 그려짐(3쪽도 같은 축) |

- 회귀 계약: 전체 14쪽. 2쪽 「목 차」 줄 윗변 − 본문 윗변 = 저장 vpos 2880HU. 다음 문단이 빈 줄과
  같은 vpos 에 저장된 반례(메모리 변형)에서는 빈 줄을 감춰 「목 차」 줄이 본문 윗변에 놓인다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진: **2020** — `info --json` 의 `lastSavedWith.product` 가 `hancom-office-2018`
  (2024 저장본이 아니므로 engine 2020).

기준 PDF: [`pdf/issue9291/geumjeong-task-order-page-start-empty-line-2020.pdf`](../../pdf/issue9291/geumjeong-task-order-page-start-empty-line-2020.pdf)
— 199,891 bytes, SHA-256 `d3d6e42a5c712865a3e4276cd2010aa53a55e5a58d684329293484bb34d20a06`,
Creator `Hwp 2022 12.0.0.4547`(engine 2020 profile), 14쪽.
