# #8894 시각 증거

- 입력: `samples/issue8894/36340478-unstored-tac-column-top-spacing.hwpx`(공개 결재문서 36340478 원본, 한컴오피스 2020 저장)
- 기준: `pdf/issue8894/36340478-unstored-tac-column-top-spacing-2020.pdf`(한컴 engine 2020 Print, 7쪽)
- 측정: `scripts/visual_sweep.py --silhouette-only --dpi 96`, 판정값 `silhouette_raw_match_percent`, 문서 face 굴림·HY헤드라인M·HY견고딕·HY견명조·맑은 고딕 공급
- 코드 head: d0dc3bd2e0(base devel 5f4e95c187), 수정 전: devel 5f4e95c187(측정 바이너리 b5064fae1e 코드와 7쪽 SVG 동일, rebase 전 head aa0d2e9ff8 와 rebase 후 head 의 SVG 도 동일)

| 경로 | 쪽수(rhwp/PDF) | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|---|---|---|
| 수정 전 Native | 7 / 7 | 99.99 | 100.00 | 99.89 | 99.85 | 99.81 | 50.83 | 27.18 |
| head Native | 7 / 7 | 99.99 | 100.00 | 99.89 | 99.85 | 99.81 | 97.01 | 99.78 |
| head fresh WASM | 7 / 7 | 99.99 | 100.00 | 99.98 | 99.96 | 99.97 | 97.03 | 99.99 |

`native/`, `wasm/`, `native-base/` 는 TSV 원본이고 `png/` 는 6·7쪽 대표 review·overlay 이미지다.
