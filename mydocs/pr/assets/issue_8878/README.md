# #8878 시각 증거

- 입력: `samples/issue8878/36441289-tac-host-below-float.hwpx`(공개 결재문서 36441289 원본)
- 기준: `pdf/issue8878/36441289-tac-host-below-float-2020.pdf`(한컴 engine 2020 Print, 2쪽)
- 측정: `scripts/visual_sweep.py --silhouette-only --dpi 96`, 판정값 `silhouette_raw_match_percent`
- 코드 head: 52da22a0af, 수정 전: 1fd8576c91(#8861 을 devel b5064fae1e 위에 올린 상태)

| 경로 | 쪽수(rhwp/PDF) | p1 | p2 |
|---|---|---|---|
| 수정 전 Native | 1 / 2 | 29.33 | 없음 |
| head Native | 2 / 2 | 99.97 | 100.00 |
| head fresh WASM | 2 / 2 | 100.00 | 100.00 |

`native/`, `wasm/`, `native-base/` 는 TSV 원본이고 `png/` 는 대표 review·overlay 이미지다.
