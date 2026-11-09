# #8864 시각 증거

- 입력: `samples/issue8864/36338125-bottom-fixed-table-margin.hwpx`(공개 결재문서 36338125 원본)
- 기준: `pdf/issue8864/36338125-bottom-fixed-table-margin-2020.pdf`(한컴 engine 2020 Print, 2쪽)
- 측정: `scripts/visual_sweep.py --silhouette-only --dpi 96`, 판정값 `silhouette_raw_match_percent`
- 코드 head: 7875b6e673, base: b5064fae1e

| 경로 | 쪽수(rhwp/PDF) | p1 | p2 |
|---|---|---|---|
| base Native | 1 / 2 | 97.93 | 없음 |
| head Native | 2 / 2 | 98.93 | 100.00 |
| head fresh WASM | 2 / 2 | 99.34 | 100.00 |

`native/`, `wasm/`, `native-base/` 는 TSV 원본이고 `png/` 는 대표 review·overlay 이미지다.
