# 원본 전체 GIF 자르기 검증

`jongbu_law.hwpx` 3쪽의 GIF는 원본 `imgDim`이 비어 있고 저장된 선택 끝점이 실제 그림
범위를 조금 넘는다. 독립 한컴 2018 Print는 전체 그림을 보존한다. 수정 전 SVG는 GIF에
별도의 자르기 창을 적용해 오른쪽을 잘랐고, 수정 후에는 전체 GIF를 표시한다.

- 입력: [`jongbu_law.hwpx`](../legal_hwpx_cellsz_host_line/input/jongbu_law.hwpx),
  SHA-256 `63c5669a5fbf99bed259bb6040b3a1ec17b218e7115b867c32bd545dbb623cc1`.
- 기준: [`jongbu_law_hancom2018_print.pdf`](../legal_hwpx_cellsz_host_line/input/jongbu_law_hancom2018_print.pdf),
  SHA-256 `1ce35eb7e2d6f93e41834215d6cf7653e3a09ea2fc544856b3df3a89e6d24511`.
- 2026-11-01, 96 DPI, 색상 경계 보정 전 2px 실루엣. 전체 6쪽 중 최저 Native
  98.86%, fresh WASM 98.87%; 핵심 3쪽은 양쪽 100.00%.
- 전쪽 TSV: [Native](native.tsv), [fresh WASM](wasm.tsv).
- 3쪽: [Native 비교](native-p003-review.png)·[overlay](native-p003-overlay.png),
  [WASM 비교](wasm-p003-review.png)·[overlay](wasm-p003-overlay.png).

시각 측정에는 시스템 설치 글꼴 전체를 사용했다. 회귀 검사는 시스템 글꼴에 의존하지
않고 원본 GIF가 SVG의 중첩 자르기 창 없이 표시되는 관계를 검사한다. 기존 수정 전
고정 실행 파일의 3쪽 SVG에서는 GIF에 별도 `svg` viewport가 있어 이 조건이 실패했고,
현재 후보 SVG에서는 페이지 바깥 `svg`만 조상으로 남는다. 페이지의 다른 그림은 별도
자르기를 유지할 수 있으므로 이 검사에 포함하지 않는다.
