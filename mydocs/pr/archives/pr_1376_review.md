# PR #1376 검토 - TAC 표 host line spacing 및 셀 세로 정렬 보정

- PR: https://github.com/edwardkim/rhwp/pull/1376
- 제목: `fix(layout): apply TAC table host-line spacing when control index differs from line-seg index`
- 작성자: `mrshinds` (`wpresign`)
- 작성자 상태: first-time contributor
- base: `devel`
- head: `mrshinds:fix/tac-host-line-spacing`
- 최신 head OID: `20191ed42b8f23f48609351247ab9a6b308464b5`
- maintainer can modify: true
- 처리 상태: 로컬 보정 완료, push 승인 대기

## 1. 현재 판단

2026-06-15 재검토 기준으로 contributor의 후속 커밋은 기존 CI 실패였던
`test_521_tac_table_outer_margin_bottom_p2` 문제를 해소했다.

다만 한컴 PDF 기준 시각 비교에서 `tac-host-spacing.hwpx`의 셀 내부 텍스트 `CELL`이
rhwp에서는 셀 하단 쪽에 렌더링되어 merge 불가 상태였다. 원인은 Center 정렬 셀에서
`text_y_start`를 계산한 뒤 `layout_composed_paragraph()`의 column-top
`LINE_SEG.vertical_pos` fallback이 다시 적용되어 첫 줄 y가 이중 보정되는 점이었다.

로컬 보정 후 `CELL` baseline은 `133.53px`에서 `122.87px`로 이동했고,
한컴 PDF reference 기준 중심 배치와 맞는다. `NEXT PARAGRAPH` baseline도
`161.27px`로 기존 TAC host line spacing 기대값을 유지한다.

PR head가 2026-06-15에 `20191ed4`로 최신화되었으나, PR diff에 리뷰 문서나
오늘할일 문서는 포함되어 있지 않다. 따라서 메인터너 보정 커밋에 PR 리뷰 문서와
오늘할일 갱신을 함께 포함한다.

## 2. 기여자 의도와 변경 범위

### 2.1 기여자 의도

기여자의 핵심 의도는 TAC 표가 비가시 컨트롤 뒤에 있을 때 `control_index`와
`lineSegArray` 인덱스가 어긋나 host line spacing 적용이 누락되는 문제를 해결하는 것이다.

기여자 PR diff 기준 변경 파일:

| 파일 | 기여자 의도 |
|---|---|
| `samples/tac-host-spacing.hwpx` | 비가시 컨트롤 뒤 TAC 표 host line spacing 재현 fixture 추가 |
| `src/renderer/layout.rs` | TAC 표 host line lookup이 control index 불일치로 실패할 때 올바른 line segment를 찾도록 보정 |
| `src/renderer/layout/integration_tests.rs` | `NEXT PARAGRAPH` baseline이 host line spacing을 반영하는지 회귀 테스트 추가 |
| `tests/issue_598_footnote_marker_nav.rs` | 최신 devel 반영 과정에서 footnote marker nav 테스트 정리 |

기여자가 PR 처리 문서를 생성하지 않았으므로, 이 문서와 처리 기록은 메인터너가 작성해
PR diff에 포함한다.

### 2.2 메인터너 추가 보정

| 파일 | 내용 |
|---|---|
| `src/renderer/layout/table_layout.rs` | Center/Bottom 셀에서 문단 레이아웃 column-top vpos fallback을 억제 |
| `src/renderer/layout/table_partial.rs` | 분할 표 경로도 같은 세로 정렬 규칙 적용 |
| `src/renderer/layout/table_cell_content.rs` | 보조 셀 콘텐츠 경로에 콘텐츠 높이 기반 Center/Bottom y 계산과 vpos fallback 억제 적용 |
| `src/renderer/layout/integration_tests.rs` | `tac-host-spacing.hwpx` 회귀 테스트에 `CELL` baseline 검증 추가 |
| `tests/golden_svg/issue-617/exam-kor-page5.svg` | 의도된 Center 셀 세로 위치 변경 반영 |
| `pdf/tac-host-spacing.pdf` | 한컴 PDF reference 추가 |

## 3. 한컴 PDF reference 시각 비교

기준 파일:

- `pdf/tac-host-spacing.pdf`

로컬 생성 산출물:

- 한컴 PDF PNG: `output/pr1376_recheck_after/hancom/tac-host-spacing_hancom_96dpi.png`
- rhwp SVG: `output/pr1376_recheck_after/pr/tac-host-spacing.svg`
- rhwp PNG: `output/pr1376_recheck_after/pr/tac-host-spacing.png`
- 비교 이미지: `output/pr1376_recheck_after/evidence/pr1376_after_hancom_vs_rhwp.png`
- PR 첨부용 비교 이미지: `mydocs/pr/assets/pr_1376_hancom_pdf_vs_rhwp_after.png`

측정값:

| 항목 | 보정 전 | 보정 후 | 판단 |
|---|---:|---:|---|
| `CELL` baseline | `133.53px` | `122.87px` | 한컴 PDF 기준 중심 배치로 개선 |
| `NEXT PARAGRAPH` baseline | `161.27px` | `161.27px` | TAC host line spacing 유지 |

## 4. 로컬 검증

| 명령 | 결과 |
|---|---|
| `cargo build --release` | 통과 |
| `cargo test --release --lib test_tac_host_line_spacing_with_preceding_invisible_controls -- --nocapture` | 통과 |
| `cargo test --release --lib` | 통과, 1820 passed / 6 ignored |
| `cargo test --profile release-test --tests` | 통과 |
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| `wasm-pack build --target web --out-dir pkg` | 통과 |

## 5. 남은 절차

push 전 작업지시자 승인 필요.

승인 후 다음 변경을 PR #1376 head에 push한다.

- 세로 정렬 보정 코드
- 회귀 테스트 보강
- golden SVG 갱신
- `pdf/tac-host-spacing.pdf`
- PR 리뷰 문서와 오늘할일 갱신
- 시각 검증 코멘트 첨부 이미지

push 후 GitHub Actions 완료를 확인하고, 통과 시 merge 가능 여부를 최종 판단한다.
