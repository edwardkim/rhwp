# 최신 devel 반영 후 검증

검증 중이다. 이전 기준의 결과는 상위 README에 보존하며 이 실행과 혼용하지 않는다.

- 기준: `cb9c45cf774a03eddb5d56fdb1fd28919f10a969`.
- 후보 제품 소스: `d189352a1b3b0e09718022f7da47daaa5205fab9`.
- 세 PR의 원본 5개: compat 2022, 각 1쪽, 기존 한컴 웹기안기 Print PDF.
- 기능 보존 대조군: 편람 HWP/HWPX, compat 2024, 각 383쪽, 기존 한컴 2024 Print PDF.
- 모든 비교: 96dpi, PDFium `pdfium_smooth_rgb_exact_dpi_v2`, 색상 경계 보정 전 2px raw 실루엣.
- 글꼴: 원 PR 사례는 한컴바탕·굴림 원본, 편람은 KoPub 6종·HCR Batang·Wingdings 원본과 한컴바탕 subset.
  한컴바탕 subset은 원본 HWPX의 Unicode 집합에 해당하는 986개 문자의 윤곽·advance를 원본과 대조했다.
  private 글꼴은 Git에 추가하지 않는다.
- 한 번의 후보 빌드에서 공유 캐시가 기준 바이너리를 재사용한 것을 SHA 대조로 발견했다.
  그 결과는 무효로 분리하고 후보 root crate의 실제 재컴파일과 새 해시·원 사례 출력을 확인한다.
  무효 결과를 후보 통과 근거로 사용하지 않는다.

최종 전쪽 TSV, 직접 판독과 빌드·검사 결과는 실행 완료 뒤 아래에 기록한다.

## 원 PR 5개 사례의 전쪽 측정

각 원본은 1쪽이고 Native/fresh WASM 전체 쪽수는 독립 PDF와 일치한다.

| 원본 | 기준 Native | 후보 Native | 기준 WASM | 후보 WASM |
| --- | ---: | ---: | ---: | ---: |
| empty_para_tac_picture | 89.09% | 99.72% | 89.51% | 99.63% |
| textbox_valign_no_lineseg | 85.19% | 99.55% | 85.70% | 99.42% |
| form_checkbox_width_1100 | 88.77% | 96.21% | 88.15% | 95.98% |
| form_checkbox_width_3000 | 92.10% | 96.27% | 91.89% | 96.05% |
| form_checkbox_width_9921 | 96.76% | 96.62% | 96.55% | 96.40% |

기준·후보의 전쪽 TSV는 `base/{native,wasm}/<원본>/silhouette.tsv`,
`candidate/{native,wasm}/<원본>/silhouette.tsv`에 바이트 그대로 보존했다.
