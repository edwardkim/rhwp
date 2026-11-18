# #8834 시각 검증 증거

- 검증 head: `8f531162f2` (native-skia release sha256 `ea94067e…`, fresh WASM `rhwp_bg.wasm` sha256 `ad4f12d4…`)
- base: upstream/devel `024d610633`
- 명령: `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96` (TSV), 같은 명령에서 `--silhouette-only` 를 뺀 실행(review·overlay PNG). WASM 은 `--wasm-pkg pkg`.
- 판정값: `silhouette_raw_match_percent`(TSV 4열). 글꼴: 이 환경에 있는 문서 face(굴림체·HY견고딕·맑은 고딕)만 둔 디렉터리를 `RHWP_FONT_PATH` 로 공급. 함초롬돋움·함초롬바탕은 이 환경에 없다.

## 입력과 기준 PDF

| 입력 | 기준 PDF | 쪽수 rhwp/한컴 |
|---|---|---|
| 공개 결재문서 `36387608_결재문서본문_고객지원시스템 운영 성과측정 관련 성과지표 달성도 측정결과 보고.hwpx` (저장소 밖, sha256 `e2a0f181…`, 한컴오피스 2020 저장) | 로컬 한컴 2022(12.0.0.4547) Print PDF(engine 2020, `84402c22…`) | 1 / 1 (base 2) |

## 결과

| | Native | fresh WASM |
|---|---|---|
| p1 | 84.80 | 85.14 |
| base p1 | 63.98 (2쪽, 마지막 문단·고정 틀이 2쪽) | — |

90% 미만은 본문 대부분(함초롬돋움 212자·함초롬바탕 178자)이 이 환경에 없는 글꼴로 대체된 차이다. 줄·표·
쪽 아래 고정 틀의 세로 위치는 한컴과 맞고(overlay), 표의 가로 위치가 공백 네 칸(함초롬바탕) 폭만큼 다르다.

- `native/`·`wasm/`: 전쪽 TSV 원본(`36387608_base_p1_silhouette.tsv` 는 base 1쪽)
- `png/`: 1쪽 review·overlay(Native·WASM)
