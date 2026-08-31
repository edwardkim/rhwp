# PR #6528 검토 - inline TAC table stored line advance

- 검토일: 2026-08-31
- 작성자: `planet6897`
- base: `devel` (`upstream/devel@99419b6b2`에서 통합 시작)
- 원 PR head: `7eed781260257c1f54cd79b9d72bb57d1ceab0b5`
- 통합 commit: `a9727942c`
- 상태: 승인 (통합 검증본 기준)

## 범위

- inline TAC table의 저장된 line break와 top offset을 레이아웃에 반영한다.
- `issue6181/156562368_inline_tac_table_line_advance.hwpx` fixture와 회귀 테스트를 추가한다.

## 검토 결과

- 저장 line break를 `(visible_char_index, line_seg_index)`로 해석하고, line top offset을 적용하는 변경이 회귀 테스트로 고정됐다.
- 목표 회귀 테스트 `issue_6181_inline_tac_table_line_advance`는 `release-test`에서 종료 코드 `0`으로 통과했다.
- Hancom 2020 기준 PDF와 p5 직접 비교를 완료했다. 자동 위험 신호는 `0`건이며, inline table 이후 텍스트 흐름과 표 내부 행 배치가 기준 구조를 유지했다.
- 시각 증적: [p5 review 패널](assets/pr_6528_issue6181_p5_review.png)
- 기준 PDF: `pdf/pr_6528_issue6181_p5_2020.pdf`, SHA-256 `e22b77cad3dffe4a22b3a3c95b6bbcffeb860985dcb3deb523a615cb1f48353a`
- visual sweep: pixel match `87.88947%`, ink match `33.73365%`; 글꼴 rasterizer 차이는 있으나 flagged page 없음.

## 공통 검증

- Rust format, native/WASM/workspace/all-target Clippy, workspace build 통과
- 전체 `nextest` 종료 코드 `0`

## 병합 조건

- 원격 병합 또는 통합 PR 게시 직전에 원 PR head와 CI green 상태를 다시 확인한다.

