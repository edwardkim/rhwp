# 법령 문서의 그림 문단 분할 중간 검증

2026-10-30에 원본 `mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law.hwpx`와 독립 한컴 Print PDF `mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law_hancom2018_print.pdf`를 96 DPI로 비교했다. 입력 SHA-256은 `63c5669a5fbf99bed259bb6040b3a1ec17b218e7115b867c32bd545dbb623cc1`, PDF SHA-256은 `1ce35eb7e2d6f93e41834215d6cf7653e3a09ea2fc544856b3df3a89e6d24511`이다. PDFium으로 같은 페이지를 래스터화하고 색상 경계 보정 전 2px 내용 실루엣 일치율을 산출했다.

이 측정은 리베이스 전 HEAD `8a32730a8ddea1bd73390ff119dae753df7fad79`에 아직 커밋하지 않은 그림 분할 보정을 더한 중간 상태다. 보정한 `inline_flow.rs` SHA-256은 `3ae107356fb6530982b487b9bbce88d3153492a40a18a1058c1f0c177331030e`이며 Native 실행 파일 SHA-256은 `0e0c5c2377da8ac2aa1c6bd7e4d11ca3e6c92dcf5582ac0b62609ba8690bc535`다. 현재 head의 최종 재측정으로 취급하지 않는다.

Native와 fresh WASM은 각각 6쪽이며 전체 최저 raw는 97.47%와 97.46%다. 핵심 2쪽은 98.86%와 98.87%, 3쪽은 99.80%와 99.81%다. [Native 전체 TSV](native.tsv), [WASM 전체 TSV](wasm.tsv), [Native 2쪽 review](native-review-002.png) 및 [overlay](native-overlay-002.png), [WASM 3쪽 review](wasm-review-003.png) 및 [overlay](wasm-overlay-003.png)를 함께 확인한다. 6쪽 부칙의 간격 차이는 이 수치와 별도로 확인 중이다.
