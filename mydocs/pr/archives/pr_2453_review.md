# PR #2453 리뷰 - 중첩 표 셀 서식 적용과 undo의 최내곽 경로 적용

## 메타데이터

| 항목 | 값 |
|---|---|
| 원 PR | [#2453](https://github.com/edwardkim/rhwp/pull/2453) |
| 작성자 | `kevin9327` |
| base | `devel` |
| 리뷰 경로 | collaborator-mediated 외부 PR, G4 체리픽 통합 |
| 적용 커밋 | `3a428b3841eb5825e8618e02bd89ee9270e8e9d3` |
| 적용 순서 | G4 2/2, #2452 뒤 최신 `upstream/devel` 위 체리픽, comment-only 충돌을 통합 헬퍼 설명으로 해소 |
| 작성 시점 참고값 | 원 PR CI 통과. 통합 PR의 최신 head CI와 mergeable 상태를 merge 전 다시 확인한다. |

## 변경 검토

`ApplyCharFormatCommand`도 flat 셀 좌표를 사용해 실제 안쪽 선택 대신 바깥 셀에 서식을 적용하고
undo할 위험이 있었다. PR은 apply, 현재 char shape 조회, 문단 길이, undo 복원을 모두 ByPath API로
전환하고 WASM bridge 및 mutation registry를 함께 갱신한다.

공통 `cellPathJsonForPara` 헬퍼는 #2452와 중복될 수 있다는 작성자 메모가 있었으나, 체리픽 순서에서
하나의 구현만 유지되어 기능 중복이나 실행 충돌은 없다.

통합 검토에서는 2단 중첩 표의 안쪽 `INNER` 전체에 bold를 적용한 뒤, undo가 쓰는
`set_char_shape_id_in_cell_by_path`로 기본 shape ID를 복원했다. 적용 전후 모두 외곽 `OUTER`의
shape ID가 바뀌지 않음을 확인해 execute와 undo 양쪽의 대상 축을 검증했다.

## 검증

| 게이트 | 결과 |
|---|---|
| `apply_char_format_in_nested_cell_by_path_preserves_outer_cell` | PASS |
| nested-cell Studio source guard + mutation routing 9건 | PASS |
| `npx tsc --noEmit` | PASS |
| `wasm-pack build --target web --out-dir pkg` | PASS |
| `npm run build` | PASS |
| `cargo fmt --check` | PASS |
| `git diff --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |

## 시각 검증 판단

서식이 화면에 나타나는 결과 자체는 사용자-visible이지만, 이 PR은 char shape의 renderer/layout
해석을 바꾸지 않고 편집 명령이 어느 중첩 셀에 변경을 기록하는지만 교정한다. 실제 중첩 구조에서
apply/undo 대상을 검증했고 Studio production build도 통과했다. 따라서 PDF visual sweep 또는 MCP
기준 PDF 비교 대상은 아니며, 원 PR 본문의 브라우저 왕복은 merge 뒤 대표 문서로 수동 확인한다.

## 최종 의견

G4 통합 PR에 원 커밋을 보존해 수용한다. #2452의 삭제 경로 보정과 함께 merge하며, merge 전 통합 PR
최신 head의 GitHub Actions와 mergeable 상태를 재확인한다.
