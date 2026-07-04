# PR #1376 리뷰 처리 기록

## PR 정보

| 항목 | 값 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1376 |
| 작성자 | `mrshinds` (`wpresign`) |
| 작성자 상태 | first-time contributor |
| base | `devel` |
| head | `mrshinds:fix/tac-host-line-spacing` |
| 최신 head OID | `20191ed42b8f23f48609351247ab9a6b308464b5` |
| maintainer can modify | true |
| 상태 | 로컬 보정 완료, push 승인 대기 |

## Stage 1 - 최신 PR 상태 확인 - 완료

- contributor가 후속 커밋으로 초기 CI 실패 원인이던 `issue_521` 회귀를 보정했다.
- 2026-06-15에 PR head가 `20191ed4`로 최신화되어 `upstream/devel` 최신 문서 반영분을 포함했다.
- PR diff 기준 contributor가 생성한 리뷰 문서/오늘할일 문서는 없다.
- GitHub Actions는 contributor 후속 커밋 기준 통과 상태였으나, 한컴 PDF reference 시각 비교에서 `CELL` 세로 위치 차이가 남았다.
- `pdf/tac-host-spacing.pdf`를 한컴 PDF reference로 추가해 rhwp 렌더와 비교했다.

## Stage 1.5 - 기여자 의도 정리 - 완료

기여자 PR의 의도:

- 비가시 컨트롤 뒤 TAC 표에서 `control_index`와 `lineSegArray` 인덱스가 서로 달라지는 케이스를 재현한다.
- 해당 케이스에서 host line spacing이 누락되어 다음 문단이 위로 당겨지는 문제를 보정한다.
- `samples/tac-host-spacing.hwpx`와 `NEXT PARAGRAPH` baseline 회귀 테스트로 방어한다.

메인터너가 추가로 생성/반영할 문서:

- `mydocs/pr/archives/pr_1376_review.md`
- `mydocs/pr/archives/pr_1376_review_impl.md`
- `mydocs/orders/20260615.md`

## Stage 2 - 시각 차이 원인 분석 - 완료

문제는 표와 다음 문단 간격이 아니라 셀 내부 텍스트 위치였다.

- 한컴 PDF reference: `CELL`이 셀 중앙 쪽에 배치됨
- PR 렌더: `CELL`이 셀 하단 쪽에 배치됨

원인:

- `tac-host-spacing.hwpx` 셀은 `vertAlign="CENTER"`이다.
- 기본 표 경로에서 Center 정렬용 `text_y_start`를 계산한다.
- 이후 `layout_composed_paragraph()`가 셀 내부 문단을 column top으로 판단해 `LINE_SEG.vertical_pos=800HU`를 다시 더한다.
- 결과적으로 Center 정렬 y가 이중 보정되어 `CELL` baseline이 하단으로 내려간다.

## Stage 3 - 로컬 보정 - 완료

보정 내용:

- `src/renderer/layout/table_layout.rs`
  - Center/Bottom 셀에서는 `layout_composed_paragraph()`의 column-top vpos fallback을 억제한다.
- `src/renderer/layout/table_partial.rs`
  - 분할 표 경로에도 같은 규칙을 적용한다.
- `src/renderer/layout/table_cell_content.rs`
  - 보조 셀 콘텐츠 경로에 콘텐츠 높이 기반 Center/Bottom 정렬 계산을 추가하고 vpos fallback을 억제한다.
- `src/renderer/layout/integration_tests.rs`
  - `CELL` baseline이 한컴 PDF reference 기준 `~122.1px` 범위에 들어오는지 검증한다.
- `tests/golden_svg/issue-617/exam-kor-page5.svg`
  - 의도된 Center 셀 세로 위치 변경을 golden에 반영한다.

## Stage 4 - 시각 검증 - 완료

생성 명령:

```bash
target/release/rhwp export-svg samples/tac-host-spacing.hwpx -o output/pr1376_recheck_after/pr -p 0
pdftoppm -r 96 -png -singlefile pdf/tac-host-spacing.pdf output/pr1376_recheck_after/hancom/tac-host-spacing_hancom_96dpi
rsvg-convert -f png -o output/pr1376_recheck_after/pr/tac-host-spacing.png output/pr1376_recheck_after/pr/tac-host-spacing.svg
```

측정 결과:

```text
C: x=77.46666666666667, y=122.86666666666666
N: x=75.58666666666667, y=161.2666666666667
```

판단:

- `CELL`은 한컴 PDF reference처럼 셀 중앙 쪽으로 이동했다.
- `NEXT PARAGRAPH` 위치는 기존 TAC host line spacing 기대값을 유지한다.
- PR 코멘트 첨부용 비교 이미지는 `mydocs/pr/assets/pr_1376_hancom_pdf_vs_rhwp_after.png`로 저장했다.

## Stage 5 - 로컬 검증 - 완료

| 명령 | 결과 |
|---|---|
| `cargo build --release` | 통과 |
| `cargo test --release --lib test_tac_host_line_spacing_with_preceding_invisible_controls -- --nocapture` | 통과 |
| `cargo test --release --lib` | 통과, 1820 passed / 6 ignored |
| `cargo test --profile release-test --tests` | 통과 |
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| `wasm-pack build --target web --out-dir pkg` | 통과 |

## Stage 6 - push 전 승인 대기

push는 작업지시자 승인 전까지 실행하지 않는다.

승인 요청 시 포함할 변경:

- 세로 정렬 보정 코드
- 회귀 테스트 보강
- golden SVG 갱신
- 한컴 PDF reference `pdf/tac-host-spacing.pdf`
- 시각 검증 코멘트 첨부 이미지 `mydocs/pr/assets/pr_1376_hancom_pdf_vs_rhwp_after.png`
- PR 리뷰 문서와 오늘할일 갱신

승인 후 PR #1376 head 브랜치에 push하고 GitHub Actions 완료를 대기한다.
