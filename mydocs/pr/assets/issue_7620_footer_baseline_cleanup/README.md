# #7620 후속 — footer 문서 body_overflow baseline 정리 증적

## 입력과 기준

| 입력 | 기준 PDF | 쪽수 |
|---|---|---|
| `samples/issue6465/press_release_footer_logos.hwpx` | `pdf/pr6485-visual/pr6485-issue6465-press-release-footer-logos-2020.pdf` | 13(rhwp 13) |

기준 PDF는 #6481/#6485에서 한컴 2020 MCP Print로 만든 기존 파일을 재사용했다
(SHA-256 `d5a3db9320675907e92a3407d75ce0726fbe605e5bf19fe9bdc46befd07d1487`).

## 측정 조건

- 2026-11-11, `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`(대표 PNG는 `--silhouette-only` 없이).
- 지표: `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣). 기준 PDF는
  pypdfium2 5.13.0(`pdfium_smooth_rgb_exact_dpi_v2`)으로 래스터화했다. 색상 경계 대조 픽셀은 13쪽 1개, 나머지 0이다.
- source: `upstream/devel` `99b9d7ec5ca36884ea8fd981ac815c1edc34c653`(이 PR은 조판 코드를 바꾸지 않는다).
  Native `rhwp.exe`(native-skia, release-test) SHA-256 `816e77d518c2387866e021644d372d78b0336e3166708ef97659e3f274eb720a`,
  fresh WASM `rhwp_bg.wasm` `a47aa15fbf3ee44a8fd9c3050c31ee65a18ddb05450dd37183452801dc7e9405`.
- #8945 병합 뒤 devel `4ab860651` 위로 리베이스한 code head `8ee9084b3`(트리 동일한 `7144a2fe3`에서 빌드)로 다시 쟀다.
  Native `f483aed3a7ee95ec9f0029edf4265fc2dff6a8503038d69fdf87f2f8eca74148`, fresh WASM
  `529443d1b4e902c7d5b49d0d869aa4d903f69231f1f1136ca33dd97be4ab52c9`. TSV 2개와 11쪽 PNG 4장이 위 측정과 바이트 단위로 같다.
- 글꼴: visual sweep 기본 웹폰트 캐시 환경(`output/pr-review/native-webfont-cache`).

## 결과

| 경로 | 11쪽 | 전 13쪽 최저(쪽) | 90% 미만 |
|---|---|---|---|
| Native | 98.19 | 96.14(1) | 없음 |
| fresh WASM | 98.32 | 94.94(1) | 없음 |

11쪽 표는 render tree에서 y=126.7px, h=916.5px(선언 68736HU = 916.48px)로 끝이 1043.2px이며
본문 바닥(75.6 + 971.4 = 1047.0px) 안이다. #7644 검토 시점(4c7092397)에는 같은 표가 924.8px로 그려져
바닥을 4.5px 넘었다. review PNG에서 표 하단 외곽선과 쪽 번호 위치가 PDF와 같다.

TSV 원본은 `tsv/<native|wasm>_press_release_footer_logos_silhouette.tsv`다. producer가 쓴 원본은 CRLF
(Native `cb9f542f…`, WASM `592f86f3…`, 각 364바이트)이고, 저장소 공백 검사에 맞춰 줄 끝만 LF로 바꿨다(값 동일).
