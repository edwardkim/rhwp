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

- 2026-11-10, `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`.
- 지표: `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣). 기준 PDF는
  pypdfium2(`pdfium_smooth_rgb_exact_dpi_v2`)로 래스터화했다. 색상 경계 대조 픽셀은 모든 쪽 0이다.
- 수정 전(base): `upstream/devel` `b5064fae1eefff92d60464c51844a38debec09f9`.
  Native `rhwp.exe`(native-skia) SHA-256 `deed3274586b5790c4a50c2269e7ac29f989793b7c895334ee9d445cd1f80246`,
  fresh WASM `rhwp_bg.wasm` `8b24d99db3db38845869dc09d480a234c5447b7f338fa300ccafc0ee4fc4e1d4`.
- 수정 후(fix): 같은 base에 이 브랜치의 `prepare.rs`·`deferred_float.rs` 변경을 적용한 빌드.
  Native `cf58e868b9be838a8f3298c683d7a23626b9baa482f7c2bc01387eb3d91b3d38`,
  fresh WASM `02350da86bd17af1b7e9e2a0acbc4ba523ae4928bbc56e74129372d39dfff028`.
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
