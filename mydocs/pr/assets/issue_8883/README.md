# #8883 시각 증거

- 입력: `samples/issue8883/36421178-long-word-first-char.hwpx`(공개 결재문서 36421178 원본)
- 기준: `pdf/issue8883/36421178-long-word-first-char-2020.pdf`(한컴 engine 2020 Print, 2쪽)
- 측정: `scripts/visual_sweep.py --silhouette-only --dpi 96`, 판정값 `silhouette_raw_match_percent`, 문서 face 굴림 공급
- 코드 head: 5526bd027d, 수정 전: devel b5064fae1e

| 경로 | 쪽수(rhwp/PDF) | p1 | p2 |
|---|---|---|---|
| 수정 전 Native | 2 / 2 | 64.30 | 100.00 |
| head Native | 2 / 2 | 100.00 | 100.00 |
| head fresh WASM | 2 / 2 | 100.00 | 100.00 |

`native/`, `wasm/`, `native-base/` 는 TSV 원본이고 `png/` 는 대표 review·overlay 이미지다.
