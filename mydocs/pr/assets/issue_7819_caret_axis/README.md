# #7819 글자겹침 캐럿 축의 시각 증거

[입력 설명과 회귀 계약](../../../../tests/fixtures/char_overlap_caret_axis/README.md),
[입력·PDF SHA와 파일별 TSV](source_evidence.tsv)를 따른다. 22개 입력·22개 PDF·44개 TSV를
바이트 그대로 보존했다. Native142 / 표준 release fresh WASM048, 96dpi Print 프로필의
전쪽 raw는 모두 100%다. 모든 입력은 각자 1쪽으로 일치한다.

대표 자료는 다음과 같다.

| 범위 | review | standalone overlay |
| --- | --- | --- |
| Native 비 BMP 문자·각주 | [review](native_nonbmp_p001_review.png) | [overlay](native_nonbmp_p001_overlay.png) |
| fresh WASM 비 BMP 문자·각주 | [review](wasm_nonbmp_p001_review.png) | [overlay](wasm_nonbmp_p001_overlay.png) |
| fresh WASM 끝 주석·오른쪽 정렬 | [review](wasm_terminal_right_p001_review.png) | [overlay](wasm_terminal_right_p001_overlay.png) |
| fresh WASM 한 payload 대조군 | [review](wasm_single_payload_p001_review.png) | [overlay](wasm_single_payload_p001_overlay.png) |
| HWP 본문 경계 5배 확대 | [review](wasm_nonbmp_hwp_detail_review.png) | [overlay](wasm_nonbmp_hwp_detail_overlay.png) |
| HWPX 본문 경계 5배 확대 | [review](wasm_nonbmp_hwpx_detail_review.png) | [overlay](wasm_nonbmp_hwpx_detail_overlay.png) |

확대 그림은 96dpi 원본의 `(108,148)–(208,176)` 영역을 같은 배율로 표시한 판독용이다.
TSV는 확대·재표본화하지 않은 전체 원본 페이지에서 계산했다. 확대 overlay는 Visual Sweep의
원본 픽셀 색 규칙을 사용한다. HWPX의 큰 내부 글자는 독립 HWPX Print에도 있으며 이를 숨기거나
HWP의 작은 내부 글자와 같은 그림이라고 보고하지 않는다.

엔진 source SHA는 `8c32167d3d87f41beb5c6b045615f4f4e7df112d`다. 이후 frontend의 글꼴
소비 보정과 테스트 변경은 이 Rust WebCanvas·Native 엔진의 출력과 구분한다. 최종 PR source의
필수 재검증을 이 자료만으로 대신하지 않는다.
