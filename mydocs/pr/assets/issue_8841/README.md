# #8841 시각 검증 증거

- 검증 head: `c36e05f1cb` (native-skia release sha256 `cc9c96fd…`, fresh WASM `rhwp_bg.wasm` sha256 `785614be…`), base upstream/devel `b768bee1c4`
- 명령: `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96` (TSV), `--silhouette-only` 를 뺀 같은 명령(review·overlay PNG). WASM 은 `--wasm-pkg pkg`.
- 판정값: `silhouette_raw_match_percent`. 글꼴: 이 환경에 있는 문서 face(굴림체)만 공급, 함초롬바탕 없음.

| 입력 | 기준 PDF | 쪽수 rhwp/한컴 | Native | fresh WASM |
|---|---|---|---|---|
| `samples/issue8841/36414883-negative-tac-line-spacing.hwpx` (`11e15b49…`) | `pdf/issue8841/36414883-negative-tac-line-spacing-2020.pdf` (한컴 2022 12.0.0.4547 Print, engine 2020, `fea8d4d0…`) | 1 / 1 (base 2) | p1 99.23 | p1 99.18 |

- `native/`·`wasm/`: 전쪽 TSV 원본, `png/`: 1쪽 review·overlay
