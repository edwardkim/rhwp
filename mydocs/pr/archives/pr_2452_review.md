# PR #2452 리뷰 - 중첩 표 셀 선택 삭제의 최내곽 경로 적용

## 메타데이터

| 항목 | 값 |
|---|---|
| 원 PR | [#2452](https://github.com/edwardkim/rhwp/pull/2452) |
| 작성자 | `kevin9327` |
| base | `devel` |
| 리뷰 경로 | collaborator-mediated 외부 PR, G4 체리픽 통합 |
| 적용 커밋 | `7a3342ad708746eab5be5734c7952d9320f96ce5` |
| 적용 순서 | G4 1/2, G3 merge 뒤 최신 `upstream/devel` 위 체리픽, 충돌 없음 |
| 작성 시점 참고값 | 원 PR CI 통과. 통합 PR의 최신 head CI와 mergeable 상태를 merge 전 다시 확인한다. |

## 변경 검토

중첩 표를 hit-test하면 flat `controlIndex`/`cellIndex`는 `cellPath[0]`의 바깥 셀을 가리킨다.
기존 `DeleteSelectionCommand`는 그 flat 좌표로 삭제 범위와 undo 텍스트를 처리해, 실제 선택한
최내곽 셀 대신 바깥 셀을 지울 수 있었다.

PR은 삭제, 삭제 전 텍스트 수집, 문단 길이 확인을 `cellPath[last]` 기반 ByPath API로 통일한다.
Rust의 `delete_range_in_cell_by_path`와 WASM export를 추가하고 mutation registry에도 등록해
문서 변경 추적 규약을 지킨다.

통합 검토에서 실제 2단 중첩 표 구조를 구성해 안쪽 `INNER`의 중간 두 글자만 삭제했을 때 `IER`가
되고, 같은 외곽 셀의 `OUTER`가 그대로 남는 회귀 테스트를 추가했다. 원 PR의 정적 source guard만으로
대상 축이 실제로 분리되는지 보장하기 어려운 공백을 메인터너 보강으로 메웠다.

## 검증

| 게이트 | 결과 |
|---|---|
| `delete_range_in_nested_cell_by_path_preserves_outer_cell` | PASS |
| nested-cell Studio source guard + mutation routing 9건 | PASS |
| `npx tsc --noEmit` | PASS |
| `wasm-pack build --target web --out-dir pkg` | PASS |
| `npm run build` | PASS |
| `cargo fmt --check` | PASS |
| `git diff --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |

## 시각 검증 판단

변경 범위는 이미 조판된 페이지의 renderer/layout 출력이 아니라 중첩 표 셀 편집 명령의 대상
해석과 WASM bridge다. 실제 문서 구조 회귀와 Studio production build로 명령 경로를 검증했으며,
PDF visual sweep 또는 MCP 기준 PDF의 비교 대상은 아니다. 원 PR 본문에 적힌 실제 브라우저 왕복은
merge 뒤에도 대표 중첩 표 문서로 수동 확인한다.

## 최종 의견

G4 통합 PR에 원 커밋을 보존해 수용한다. #2453의 같은 경로 보정 및 undo 회귀와 함께 merge하며,
merge 전 통합 PR 최신 head의 GitHub Actions와 mergeable 상태를 재확인한다.
