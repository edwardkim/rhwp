# #8778 시각 검증 증거

- 검증 head: `745deec173` (#8785 위, native-skia release sha256 `8cf17a81…`, fresh WASM `rhwp_bg.wasm` sha256 `bedd3c96…`)
- 명령: `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96` (TSV), 같은 명령에 `--pages` (review·overlay PNG). WASM 은 `--wasm-pkg pkg`.
- 판정값: `silhouette_raw_match_percent`(TSV 4열). 글꼴: 문서 face 만 둔 디렉터리를 `RHWP_FONT_PATH` 로 공급.

## 입력과 기준 PDF

| 키 | 입력 | 기준 PDF | 쪽수 rhwp/한컴 |
|---|---|---|---|
| reduced | `samples/issue8778/36495398-split-row-segments-reduced.hwpx` (sha256 `ed64160c…`) | `pdf/issue8778/36495398-split-row-segments-reduced-2020.pdf` (engine 2020, `c8ea97be…`) | 3 / 3 |
| float-prefix | `samples/issue8778/156739836-float-prefix-row-reduced.hwpx` (`180f2440…`) | `pdf/issue8778/156739836-float-prefix-row-reduced-2020.pdf` (engine 2020, `e08b2e5f…`) | 4 / 4 |
| original | 결재문서 `36495398_결재문서본문.hwpx` 원본(저장소 밖, sha256 `81e6abe2…`) | 한컴 engine 2020 Print PDF(`d293b489…`) | 5 / 5 (base 6) |

## 결과(최저)

| 키 | Native | fresh WASM | 90% 미만 |
|---|---|---|---|
| reduced | 99.95 | 99.98 | 없음 |
| float-prefix | 95.69 | 95.77 | 없음 |
| original | 81.87 | 81.27 | p4·p5 |

원본 p5 는 별표 글자(HY신명조, 이 환경에 없음) 모양만 다르고 줄 위치·길이는 같다. p4 는 같은 글꼴 차이에
더해 "□ 행정사항"(`pi=31`)부터 약 6.5px 아래인 기존 차이가 있다 — base 와 같은 위치이며 공급 글꼴에 따라
바뀌지 않는다(별도 추적: rhwp-issue-docs#1080).

- `native/`·`wasm/`: 전쪽 TSV 원본
- `png/`: reduced p2·p3, float-prefix p4, original p4·p5 의 review·overlay(Native·WASM)
