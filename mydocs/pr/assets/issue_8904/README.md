# #8904 시각 증거

- 입력: `samples/issue8904/36376635-synthetic-tail-saved-reset.hwpx`(공개 결재문서 36376635 원본, 한컴오피스 2020 저장)
- 기준: `pdf/issue8904/36376635-synthetic-tail-saved-reset-2020.pdf`(한컴 engine 2020 Print, 3쪽)
- 측정: `scripts/visual_sweep.py --silhouette-only --dpi 96`, 판정값 `silhouette_raw_match_percent`, 문서 face 만 모은 글꼴 폴더 공급
- 코드 head: 1ce9842468(base devel 411d4a91fd), 수정 전: base devel 411d4a91fd

| 경로 | 쪽수(rhwp/PDF) | 1 | 2 | 3 |
|---|---|---|---|---|
| 수정 전 Native | 3 / 3 | 99.92 | 97.71 | 62.30 |
| head Native | 3 / 3 | 99.92 | 99.69 | 100.00 |
| head fresh WASM | 3 / 3 | 100.00 | 99.78 | 100.00 |

`native/`, `wasm/`, `native-base/` 는 TSV 원본이고 `png/` 는 2·3쪽 대표 review·overlay 이미지다.
