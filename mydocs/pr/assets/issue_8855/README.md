# #8855 시각 검증 증거

- 검증 head: `1b9316ccd3` (native-skia release sha256 `203d1bea…`, fresh WASM `rhwp_bg.wasm` sha256 `5264e6c2…`), base upstream/devel `e00f9b7e26` (native sha256 `0110e9b5…`)
- 명령: `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96` (TSV), `--silhouette-only` 를 뺀 같은 명령(review·overlay PNG). WASM 은 `--wasm-pkg pkg`.
- 판정값: `silhouette_raw_match_percent`. 글꼴: 문서 face 중 굴림·굴림체·한컴바탕·HY견고딕 공급(합계 64MiB 이내), HY신명조·HY중고딕은 이 환경에 없음. `native_base/` 는 맑은 고딕이 더 든 글꼴 폴더로 쟀다.

| 입력 | 기준 PDF | 쪽수 rhwp/한컴 | Native | fresh WASM |
|---|---|---|---|---|
| `samples/issue8855/36388005-host-below-header-table.hwpx` (`225eba45…`) | `pdf/issue8855/36388005-host-below-header-table-2020.pdf` (Hancom PDF 1.3.0.550, engine 2020, `dd5abd9f…`) | 2 / 2 (base 1) | p1 100.00, p2 100.00 (base p1 73.70) | p1 100.00, p2 100.00 |

- 확대 판독: 1쪽 그림 아래끝과 뒤 글줄, 머리 표와 첫 본문 사이, 2쪽 쪽 아래 고정 표를 한컴 PDF 와 같은 영역에서 대조했다. 2쪽 회색 띠에 1~2px 차이가 남는다.
- `native/`·`wasm/`·`native_base/`: 전쪽 TSV 원본, `png/`: 쪽별 review·overlay
