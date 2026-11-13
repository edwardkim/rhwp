# sacru2red 2건 — 보정 전 독립 검증

2026-11-13. 기준 devel `180a4ec97205b67c4dc4f90416063c01ecfdf396`, 누적 검토 source
`ea627d6017`. 제품 producer 입력 3363개 mode/blob/path는 Native 실제 source
`60f5659d5bbd814c6653d961fb85be2703a285d9`와 같으며 목록 SHA-256은
`56e348b066268c059f8af6390d3282f88ecc535f84b0ab9a8457e8af476bddfc`다.

| 문서 | 전체 쪽수 | Native 최저 raw | fresh WASM 최저 raw |
| --- | --- | --- | --- |
| footer | 13 | 96.17% | 96.11% |
| e-cellbreak | 2 | 94.62% | 94.90% |
| f-rowbreak | 2 | 93.46% | 93.79% |

## 입력·기준·출력

- footer 입력·기준은 기존 `samples/issue6465/press_release_footer_logos.hwpx`와
  `pdf/pr6485-visual/pr6485-issue6465-press-release-footer-logos-2020.pdf`를 재사용한다.
  기준 SHA-256 `d5a3db9320675907e92a3407d75ce0726fbe605e5bf19fe9bdc46befd07d1487`.
- e·f 입력은 원 PR #8975의 `samples/issue7620/` HWP를 그대로 썼다.
  [동일 입력·Desktop Print 해시](../../sacru2red_20261113_spacing_finding.md)를 따른다.
- 새 기준은 `pdf/issue7620/*-2020.pdf`다. 원 기여의 PDFium 웹 PDF는 참고로 보존하고
  데스크톱 Print 판정에서는 제외한다. MCP의 실제 제품은 Hancom 2020 `11.0.0.9136`,
  `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`이다.
- 전쪽 TSV는 `tsv/{native,wasm}-{footer,cellbreak,rowbreak}.tsv`다.
  `*-three-way.png`는 같은 배율의 Print·Native·fresh WASM 실제 출력이다.
  2쪽 표 뒤 본문의 8px(Native)·7px(WASM) 차이는 수치 통과와 별개로 보류했다.

## 실제 실행

Native Skia 직접 PNG와 Chrome `156.0.8078.47`의 fresh WASM Canvas PNG를
`scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`으로 측정했다.
PDF는 pypdfium2의 동일 DPI 래스터다. 렌더러 경로를 SVG로 대체하지 않았다.

- Native CLI SHA-256 `dac82fc87ddfb5f75fd2062c1d893333eca1968691999587cdefd7a2125b39e2`.
- fresh WASM SHA-256 `547e45d169df08ee7c07caf85e36684a1f4e33a0c069584e4b320015a4d2d858`.
- 빌드 명령은 `RHWP_WASM_CDYLIB_ONLY=1 CARGO_TARGET_DIR=/home/edward/mygithub/rhwp/target/pr-review CARGO_BUILD_JOBS=8 scripts/wasm-pack-locked.sh --target web --out-dir <루트 output의 검토 package>`이며 exit 0, 16분 58초다. release lto=false와 wasm-opt를 완료했다.
- `VISUAL_SWEEP_CHROME`을 위 Chrome의 절대 경로로 지정했다. 최초 브라우저 탐색 실패는
  경로 지정으로 해소했으며 성공한 전체 실행의 결과만 표에 썼다.
- 기본 웹폰트 공급을 사용했다. e·f는 HCRBatang/HCRDotum 원장의 실제 프로그램을 공급했다.
  글꼴 프로그램은 이 증적·배포물에 포함하지 않았다.
- 최소 글꼴 Rust 회귀와 Print 글꼴 공급 시각 검증은 분리한다. 초기 Native Clippy(183.727초),
  WASM Clippy(130.410초)는 통과했다. 사용자의 간격 보정 승인에 따라 workspace build를 중지했고
  최종 필수 검사는 보정 뒤 실행한다. 이 초기 결과를 최종 head의 검사 완료로 쓰지 않는다.

원시 로그·JSON·중복 래스터는 루트 `output/pr-review/sacru2red-20261113`에만 보관한다.
