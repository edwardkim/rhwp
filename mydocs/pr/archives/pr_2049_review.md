# PR #2049 리뷰 — macOS VSCode webview WASM async init

- PR: #2049 `[vscode] macOS 웹뷰 WASM 로드 실패 수정 — initSync → async init (#2048)`
- URL: https://github.com/edwardkim/rhwp/pull/2049
- 기준 브랜치: `devel`
- head branch: `pr-task2048`
- 작성자: @planet6897
- 관련 이슈: #2048
- 문서 작성 시점 참고값: 원 PR head `dbe97cce626643d2d17a916c73fa33eba5523daa`, merge state `BEHIND`
- 처리 경로: 여러 PR 체리픽 누적 검토. 통합 PR #2066 에 기능 커밋을 포함하고, 본 review 문서를 같은 PR head 에 포함한다.
- 최종 merge 조건: 통합 PR #2066 최신 head 기준 GitHub Actions 통과 + 작업지시자 승인.

## 결론

merge 후보로 본다. macOS VSCode webview 에서 `initSync` 경로가 큰 WASM 버퍼를 메인 스레드에서 동기 컴파일해
차단되던 문제를 wasm-bindgen 기본 async `init` 경로로 바꾼다. 변경 범위가 `viewer.ts` 초기화 경로에 좁게
한정되어 있고, 실제 VSCode 격리 프로필에서 HWP Viewer webview 로 `samples/KTX.hwp` 렌더링까지 확인했다.

원 PR 은 `BEHIND` 상태라 #2060, #2061 과 함께 `upstream/devel` 기준 체리픽 통합 PR #2066 으로 처리한다.

## 변경 범위

- `rhwp-vscode/src/webview/viewer.ts`

## 체리픽 기록

| 항목 | 값 |
|------|----|
| 원 기능 커밋 | `1910b118935a4fd96f432ed2883f6fd2ec252fda` |
| 통합 PR 커밋 | `2487fe3eb` |
| 체리픽 순서 | 1 / 7 |
| 충돌 | 없음 |
| 제외 커밋 | `dbe97cce6266` 는 `devel` merge commit 이므로 제외 |
| 선행 PR 의존 | 없음. #2060/#2061 과 같은 통합 PR 에 포함 |

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
| `cd rhwp-vscode && npx --yes @vscode/vsce package` | 통과, `rhwp-vscode-0.7.17.vsix` 생성 |
| VSCode 격리 설치 | `edwardkim.rhwp-vscode@0.7.17` 설치 확인 |
| 실제 문서 열기 | `samples/KTX.hwp` 를 `rhwp.hwpViewer` association 으로 열어 webview 활성화와 문서 렌더링 확인 |

증적:

- `mydocs/pr/assets/pr_2049_vscode_ktx_render.png`

## visual 판단

이 PR 은 렌더링 알고리즘 변경이 아니라 webview 의 WASM 초기화 실패를 해소하는 수정이다. 따라서 한컴 기준 PDF
pixel visual sweep 대신 VSCode 실기 구동에서 HWP Viewer webview 가 생성되고 KTX 샘플 1쪽이 렌더링되는지 확인했다.

## merge 후 처리

- #2066 merge 후 #2048 close 여부를 확인한다.
- 원 PR #2049 에 통합 PR #2066 으로 처리됐음을 남기고 close 한다.
