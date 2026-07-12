# PR #2061 리뷰 — VSCode 뷰어 2쪽 보기 모드

- PR: #2061 `[vscode] 뷰어 2쪽 보기(양면 스프레드) 모드 (#2051)`
- URL: https://github.com/edwardkim/rhwp/pull/2061
- 기준 브랜치: `devel`
- head branch: `pr-task2051`
- 작성자: @planet6897
- 관련 이슈: #2051
- 문서 작성 시점 참고값: 원 PR head `4672711837c6b186bd9b145942e7444c821dbacf`, merge state `BEHIND`
- 처리 경로: 여러 PR 체리픽 누적 검토. 통합 PR #2066 에 기능 커밋을 포함하고, 본 review 문서를 같은 PR head 에 포함한다.
- 최종 merge 조건: 통합 PR #2066 최신 head 기준 GitHub Actions 통과 + 작업지시자 승인.

## 결론

merge 후보로 본다. VSCode HWP/HWPX 뷰어 하단 상태바에 `1쪽/2쪽` 토글을 추가하고, 2쪽 모드에서는 페이지를
두 장씩 같은 행에 좌우 배치한다. 기존 `pageInfos[i].element` 계약을 유지해 가상 스크롤, 현재 페이지 추적,
쪽 이동, 줌 로직이 동일하게 동작하도록 구성되어 있다.

원 PR 은 #2060 사이드바 변경 위에서 분기되었고 `BEHIND` 상태이므로, #2049/#2060 과 함께 `upstream/devel`
기준 체리픽 통합 PR #2066 으로 처리한다.

## 변경 범위

- `rhwp-vscode/src/hwp-editor-provider.ts`
- `rhwp-vscode/src/webview/viewer.ts`
- `mydocs/plans/task_m100_2051.md`
- `mydocs/report/task_m100_2051_report.md`

## 체리픽 기록

| 항목 | 값 |
|------|----|
| 원 기능 커밋 | `4672711837c6b186bd9b145942e7444c821dbacf` |
| 통합 PR 커밋 | `42910f616` |
| 체리픽 순서 | 7 / 7 |
| 충돌 | 없음 |
| 선행 PR 의존 | #2060 사이드바 레이아웃 변경 필요 |

## 검증

| 검증 | 결과 |
|------|------|
| 원 PR GitHub Actions | CI/CodeQL/Render Diff 계열 통과. 원 PR은 `BEHIND` |
| `cargo fmt --check` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib` | 통과, 2150 passed / 7 ignored |
| `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` | 통과 |
| `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` | 통과 |
| `wasm-pack build --target web --out-dir pkg` | 통과 |
| `cd rhwp-vscode && npm ci` | 통과. 기존 의존성 기준 high severity audit 1건 보고 |
| `cd rhwp-vscode && npm run compile` | 통과 |
| `cd rhwp-vscode && npx --yes @vscode/vsce package` | 통과 |
| VSCode 실기 검증 | `samples/KTX.hwp` 에서 `1쪽` 버튼을 눌러 `2쪽` 상태와 1쪽/2쪽 좌우 배치 확인 |

증적:

- `mydocs/pr/assets/pr_2061_vscode_ktx_double.png`

## visual 판단

이 PR 은 문서 렌더링 결과를 바꾸지 않고 viewer 배치 모드를 추가한다. 따라서 한컴 기준 PDF visual sweep 보다는
VSCode custom editor 에서 2쪽 모드 버튼 상태와 좌우 배치를 직접 확인하는 것을 검증 기준으로 삼았다.

## merge 후 처리

- #2066 merge 후 #2051 close 여부를 확인한다.
- 원 PR #2061 에 통합 PR #2066 으로 처리됐음을 남기고 close 한다.
