# #8850 시각 검증 증거

- 검증 head: `4e41d2f710` (native-skia release sha256 `4fa54ca4…`, fresh WASM `rhwp_bg.wasm` sha256 `4bb662db…`), base upstream/devel `b768bee1c4`
- 명령: `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96` (TSV), `--silhouette-only` 를 뺀 같은 명령(review·overlay PNG). WASM 은 `--wasm-pkg pkg`.
- 판정값: `silhouette_raw_match_percent`. 글꼴: 문서 face 굴림체 공급.

| 입력 | 기준 PDF | 쪽수 rhwp/한컴 | Native | fresh WASM |
|---|---|---|---|---|
| `samples/issue8850/36474334-two-tac-tables-reduced.hwpx` (`12107ac5…`) | `pdf/issue8850/36474334-two-tac-tables-reduced-2020.pdf` (Hancom PDF 1.3.0.550, engine 2020, `ccfc976b…`) | 1 / 1 | p1 99.11 (base 22.41) | p1 99.11 |

- 축소본: 공개 결재문서 36474334 에서 두 TAC 표를 든 `pi=3` 만 남기고 구역 정의를 그 문단으로 옮겼으며, 칸 글자는 같은 수의 공백으로 바꿨다(글자를 지우면 한컴이 손상 문서로 거부).
- 확대 판독: 둘째 표는 첫 표 바로 아래에서 시작하고 본문 안에 있다. 남은 차이는 둘째 표 전체가 한컴보다 1~2px 위.
- `native/`·`wasm/`: 전쪽 TSV 원본, `png/`: 1쪽 review·overlay
