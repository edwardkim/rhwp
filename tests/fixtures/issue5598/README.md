# 좁은 셀 독립 Print 대조군

`tests/cases/issue_5598_cell_hanging_indent_clamp.rs`의 Rust builder에서 만든 두 합성 HWPX다.
원래 합성 입력의 0×0 용지와 미지정 글꼴은 한컴에서 정상 인쇄할 수 없어, A4와 바탕 9pt를
명시했다. 실제 원문 `자치수형자 선정 절차`의 사본으로 취급하지 않는다.
테스트는 builder의 직렬화 결과가 아래 입력과 바이트 단위로 같은지도 확인한다.

| 내어쓰기 | HWPX SHA-256 | 독립 한컴 Print PDF |
|---|---|---|
| −7832 HU | `47950a486a132d7819c5ed011b04f81a77cffdaa61ce67969c926e0bd0a693b8` | [Print](../../../pdf/issue5598-rust-defined-controls-20261015/narrow-cell-indent--7832-defined-rust-2020.pdf) |
| 0 HU | `039bc60fe5667542cb457a61ed005cbbb22911e45a6a0475dcd7aca2ae7bf117` | [Print](../../../pdf/issue5598-rust-defined-controls-20261015/narrow-cell-indent-0-defined-rust-2020.pdf) |

한컴2020 11.0.0.9136, PrintMethod=0, 원본 용지 유지로 각각 1쪽을 직접 인쇄했다.
PDF SHA-256은 각각 `39c1559e62dc98e34d6530498bf838939bf290e5ad4b72241be18ac26cb5257a`,
`b2a1f82b7d171589cd3690703b9229886ee5c4a1b79ae04e13254ce38f5960a9`다.

96dpi 전쪽 Native/fresh WASM 실루엣 일치율은 두 입력 모두 100%였다. 가시 내용과 셀 포함을
6배 확대 대조했다. 독립 Print의 마지막 ‘회’ 원점 84.9145813pt를 CSS px로 환산한 ±2px 검사도
이 페이지별 98% 조건을 근거로 적용했다. 내용 순서·누락·중복과 소유 셀 포함 검사를 함께 유지한다.
말미 공백이 글자 배분에 포함되던 이전 renderer에서는 이 원점 검사가 약10.7px 차이로 실패했다.

검증 원본 TSV와 명령·빌드 해시는 ignored
`output/pr-review/planet6897-followup-20261011/`의
`additional-remaining-boundary-fresh-wasm-8f43-visual-results.json`,
`additional-native-narrow-final-candidate-visual-results.json`,
`issue5598-independent-visible-pin-before-after.json`,
`boundary-relational-focused.log`에 기록했다. 전역 최종 PR 검증은 별도로 수행한다.
