# 저장본 한컴 Print의 대조와 미달 근거

제품 source `cfc5be8fe`, 한컴 2024 13.0.0.3901, Print method 0, 96 DPI다.
`native/`, `wasm/`의 TSV는 작업 원본을 바이트 그대로 보존했다.

| 자료 | Native raw % | fresh WASM raw % |
| --- | ---: | ---: |
| scan-jbig2-200 | 99.89751 | 99.89889 |
| scan-jpx-200 | 99.89671 | 99.89810 |
| jbig2-symbol_global | 99.67991 | 100 |
| jpx-palette | 99.53271 | 100 |
| jpx-indexed | 99.53271 | 100 |
| jpx-alpha | **27.39104** | **27.39104** |
| jpx-premult | **52.99065** | **52.99065** |
| jbig2-mask | 100 | 100 |
| jpx-mask | 100 | 100 |

alpha 두 쪽은 Print 대조가 미달이다. 이 표는 원본 PDF 기준의 17쪽 100% 결과와 구분한다.
한컴 재저장 HWP의 PNG에도 원래 RGBA 값이 보존됐고, 그 HWP의 Print도 HWPX Print와
같은 1비트 mask와 인쇄 픽셀을 만들었다. 각각 `alpha-hancom-resave-proof.json`,
`alpha-print-proof.json`에 값과 hash를 보존했다. 8비트 alpha를 원래 4×2 해상도의 1비트 mask로
바꾸는 한컴 Print 경로의 제한이다. 기준을 바꾸거나 RHWP alpha를 손실시키지 않았다.

아래는 한컴으로 다시 저장한 HWP의 Print다. green의 alpha 64는 사라지고 blue alpha 128은
일부 행만 불투명해진다. 원래 PDF/RHWP의 연속 alpha 합성과 직접 대조했다.

![한컴 재저장 HWP의 alpha Print](jpx-alpha-hancom-hwp-print.png)
![한컴 재저장 HWP의 premult Print](jpx-premult-hancom-hwp-print.png)

[원본 Print·한컴 재저장 대조군](../../../../../pdf/issue8183/hancom-print/README.md),
[원본 PDF 기준 alpha](../wasm/jpx-alpha_review.png),
[3단계 판정](../../../../working/task_m100_8183_stage3.md)을 참고한다.
