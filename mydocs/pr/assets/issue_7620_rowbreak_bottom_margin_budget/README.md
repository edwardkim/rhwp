# #7620 후속 — RowBreak 자리 차지 표의 첫 조각 바닥 예산 시각 증적

## 입력과 기준

| 입력 | 기준 PDF | 쪽수 |
|---|---|---|
| `samples/issue7620/b-rowbreak-bottom-margin-overflow.hwp` | `pdf/issue7620/b-rowbreak-bottom-margin-overflow-hancomdocs.pdf` | 2 |
| `samples/issue7620/c-rowbreak-table-moves-next-page.hwp` | `pdf/issue7620/c-rowbreak-table-moves-next-page-hancomdocs.pdf` | 2 |
| `samples/issue7620/d-rowbreak-splits-at-row-boundary.hwp` | `pdf/issue7620/d-rowbreak-splits-at-row-boundary-hancomdocs.pdf` | 2 |

입력은 `samples/issue7620/generate-budget.mjs`로 만든 생성 대조군이며, 기준 PDF는 같은 입력을
한컴독스 웹 한글에서 인쇄(Print)한 PDF다(cairo 1.15.12). 입력·PDF의 SHA-256은
`samples/issue7620/MANIFEST.json`에 있다.

## 측정 조건

- 2026-11-11(리베이스 후 재측정), `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`.
- 지표: `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣). 기준 PDF는
  pypdfium2 5.13.0(`pdfium_smooth_rgb_exact_dpi_v2`)으로 래스터화했다. 색상 경계 대조 픽셀은 모든 쪽 0이다.
- 수정 전(base): `upstream/devel` `e118de8abadd2dffb2798036b9ffd381b96ed7d5`.
  Native `rhwp.exe`(native-skia, release-test) SHA-256 `9c45265d5d8dfc7fc8f56183ef7e9a7d9baa76167d3177aeac611f6353ad3aa6`,
  fresh WASM `rhwp_bg.wasm` `2fe1e933018469ee85db62b81e2df8f7332faca03f40949f9b6006d7ede9300d`.
- 수정 후(fix): 같은 base 위의 이 브랜치(`a24bf8dd6`, `src/` 기준 최종 head와 같음).
  Native `e1d32820a5e32c25a511a9ac7c074c2a171d191cb7c3963b36c4ada489e28621`,
  fresh WASM `812b1e94ccf40f087f9016f3aaca4a86a069502396a41d5ef975b568ed98e9a8`.
- 리베이스 전 base `b5064fae1`에서 잰 값과 TSV·대표 PNG가 바이트 단위로 같았다.
- 글꼴: visual sweep 기본 웹폰트 캐시 환경(`output/pr-review/native-webfont-cache`).

## 결과(전 쪽)

| 입력 | 경로 | 수정 전 p1 / p2 | 수정 후 p1 / p2 |
|---|---|---|---|
| b | Native | 89.88 / 59.65 | 100.00 / 100.00 |
| b | fresh WASM | 90.21 / 60.49 | 100.00 / 100.00 |
| c | Native | 100.00 / 100.00 | 100.00 / 100.00 |
| c | fresh WASM | 100.00 / 100.00 | 100.00 / 100.00 |
| d | Native | 100.00 / 100.00 | 100.00 / 100.00 |
| d | fresh WASM | 100.00 / 100.00 | 100.00 / 100.00 |

TSV 원본은 `tsv/<native|wasm>-<base|fix>/<입력명>_silhouette.tsv`다.

## 대표 PNG

`b-rowbreak-bottom-margin-overflow-p00N-<native|wasm>-<review|overlay>.png`는 수정 후,
`...-devel-native-...`는 수정 전 Native다. 수정 전에는 표가 나뉘어 1행이 1쪽 바닥(바깥 아래 여백이
본문 밖으로 넘침)에, 2행과 표 뒤 문단이 2쪽에 놓였다. 수정 후에는 한컴 PDF와 같이 표 전체(2행)가 2쪽 위(본문 윗변 + 바깥 위 여백)에서 시작하고,
표 문단 뒤 글줄은 1쪽에 남는다.
