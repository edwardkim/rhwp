# #8888 시각 증거

- 입력: `samples/issue8888/36384898-unstored-next-float-bottom-margin.hwpx`(공개 결재문서 36384898 원본, 한컴오피스 2020 저장)
- 기준: `pdf/issue8888/36384898-unstored-next-float-bottom-margin-2020.pdf`(한컴 engine 2020 Print, 2쪽)
- 측정: `scripts/visual_sweep.py --silhouette-only --dpi 96`, 판정값 `silhouette_raw_match_percent`, 문서 face 굴림·맑은 고딕 공급
- 코드 head: c9548826a1, 수정 전: devel b5064fae1e 코드(base devel 3170fddc48 은 문서만 다름)

| 경로 | 쪽수(rhwp/PDF) | p1 | p2 |
|---|---|---|---|
| 수정 전 Native | 2 / 2 | 32.34 | 100.00 |
| head Native | 2 / 2 | 99.73 | 100.00 |
| head fresh WASM | 2 / 2 | 100.00 | 100.00 |

`native/`, `wasm/`, `native-base/` 는 TSV 원본이고 `png/` 는 대표 review·overlay 이미지다.
