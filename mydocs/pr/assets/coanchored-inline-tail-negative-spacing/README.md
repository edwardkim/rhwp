# 인라인 중첩 표 뒤의 음수 줄 간격

[합성 입력과 출처](../../../../tests/fixtures/coanchored_inline_tail/README.md)

macOS 한컴오피스 한글 Viewer 12.31.9에서 전체 문서를 A4로 인쇄했다. 이 PDF는 로컬 대조용이다. RHWP의 공식 제출용 기준본은 Windows 한컴 2020의 Hancom PDF 인쇄로 확보한다.

| 출력 | 전체 쪽수 | 1쪽 raw 실루엣 일치율 | 2쪽 raw 실루엣 일치율 |
| --- | ---: | ---: | ---: |
| Mac 한컴 Viewer 인쇄 | 2 | | |
| 수정 전 Native Skia | 3 | | |
| 수정 전 fresh WASM Canvas | 3 | | |
| 수정 후 Native Skia | 2 | 98.83% | 100.00% |
| 수정 후 fresh WASM Canvas | 2 | 98.82% | 100.00% |

두 RHWP 출력은 Print 프로필, compat 2022, 96 DPI로 측정했다. 기준 PDF는 PDFium으로 래스터화했다. 수정 후 1쪽에서 제목 표, 본문 표, 중첩 표, 다음 글줄이 보인다. 2쪽에는 다음 표가 남는다. 384 DPI 확대에서 중첩 표와 다음 글줄 사이의 간격은 Mac 인쇄본이 약 6 CSS px 더 크다.

- Native [전쪽 TSV](mac-viewer/native/silhouette.tsv) · [1쪽 대조](mac-viewer/native/review-page-1.png) · [1쪽 오버레이](mac-viewer/native/overlay-page-1.png)
- WASM [전쪽 TSV](mac-viewer/wasm/silhouette.tsv) · [1쪽 대조](mac-viewer/wasm/review-page-1.png) · [1쪽 오버레이](mac-viewer/wasm/overlay-page-1.png)
