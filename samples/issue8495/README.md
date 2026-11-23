# samples/issue8495 — 자리차지 표와 같은 문단의 문단 기준 그림 원점

[#8495](https://github.com/edwardkim/rhwp/issues/8495)의 정식 회귀 fixture다.

구례군 공개 자치법규 별표(「[별표 2] 의회 의원 배지 제식」, HWP, 1쪽) 원본이다.

| 항목 | 값 |
| --- | --- |
| host 문단 | 0.4 「 금색」 — 문단 앞 간격 0, 첫 저장 줄 vpos 9564HU |
| 앞 문단 | 문단 뒤 간격 568HU(7.57px) |
| 개체 | 문단 기준 자리차지 표 3×2(세로 1622HU), 문단 기준 그림 둘(위아래 1946HU, 어울림 1442HU), 종이 기준 묶음 개체 |
| 한/글 Print | 그림 윗변 266.59px·259.88px(96dpi) |

- 회귀 계약: 전체 1쪽. 두 그림의 윗변 = host 문단 첫 저장 줄의 윗변 − 문단 앞 간격 + 그림의
  세로 오프셋. 수정 전에는 앞 문단의 문단 뒤 간격을 더하기 전 흐름 위치가 기준이어서 그림과
  치수선이 7.57px 위에 그려졌다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진: **2020** — `info --json` 의 `lastSavedWith.product` 가 `null`(2024 저장본이
  아니므로 engine 2020).

기준 PDF: [`pdf/issue8495/council-badge-para-float-sibling-origin-2020.pdf`](../../pdf/issue8495/council-badge-para-float-sibling-origin-2020.pdf)
— 50,851 bytes, SHA-256 `df2eb60a480a6e609915b951a3337a73ebd241d9b9d0bc12cc8a26f7bb8ab54b`,
Creator `Hwp 2022 12.0.0.4547`(engine 2020 profile), 1쪽.
