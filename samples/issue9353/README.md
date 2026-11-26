# samples/issue9353 — 음수 줄 간격 두 줄 칸의 행 나눔

[#9353](https://github.com/edwardkim/rhwp/issues/9353)의 정식 회귀 fixture다.

10k 공개 코퍼스의 의왕시 조례 별표 5 「변속차로의 최소길이」(HWP, 3쪽) 원본이다.
44×12 RowBreak 표 하나가 세 쪽에 걸친다.

| 항목 | 값 |
| --- | --- |
| 셀[156] r=37,c=1 | `21대 이상 / 50대 이하` 두 줄 — ls[0] vpos 0, 줄 높이 1200HU, 줄간격 −240HU, ls[1] vpos 960HU |
| 저장 전진 | 0 + 1200 − 240 = 960 — 둘째 줄은 첫 줄이 저장한 전진에 정확히 놓인다 |
| 같은 꼴 | 셀[129] r=32,c=1 `11대 이상 / 30대 이하` |
| 수정 전 | 960 < 0 + 1200 을 쪽 되감김(hard break)으로 읽어 행 37 을 2·3쪽에 나눠 그림(3쪽 머리에 `50대 이하` 조각) |

- 회귀 계약: 전체 3쪽. 모든 쪽에서 둘째 줄 `50대 이하` 는 첫 줄과 같은 쪽에 있다. 행 37 은 2쪽,
  3쪽은 `51대 이상` 행으로 시작한다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진: **2024** — `lastSavedWith.product` 가 `hancom-office-2024`.

기준 PDF: [`pdf/issue9353/uiwang_lane_min_length-2024.pdf`](../../pdf/issue9353/uiwang_lane_min_length-2024.pdf)
— 81,913 bytes, SHA-256 `84249e6ae57e4c95074afd48ad48963536cbd3dd17f5d33489f4484a91dc35e5`,
Hwp.exe 13.0.0.564 `Hancom PDF` Print(`hancom2024_pdf_driver_one_up`), 3쪽.
