# PR #2060 리뷰 — VSCode 뷰어 좌측 네비게이션 사이드바

- PR: #2060 `[vscode] 뷰어 좌측 네비게이션 사이드바 — 썸네일 + 목차 + 북마크 + 쪽번호 이동 (#2050)`
- URL: https://github.com/edwardkim/rhwp/pull/2060
- 기준 브랜치: `devel`
- head branch: `pr-task2050`
- 작성자: @planet6897
- 관련 이슈: #2050
- 문서 작성 시점 참고값: 원 PR head `a882f3dbff0c61c40b7919828cf6b43f0c7469fd`, merge state `BEHIND`
- 처리 경로: 여러 PR 체리픽 누적 검토. 통합 PR #2066 에 기능 커밋을 포함하고, 본 review 문서를 같은 PR head 에 포함한다.
- 최종 merge 조건: 통합 PR #2066 최신 head 기준 GitHub Actions 통과 + 작업지시자 승인.

## 결론

merge 후보로 본다. VSCode HWP/HWPX 뷰어에 페이지 썸네일, 목차, 북마크, 쪽번호 이동, 사이드바 접기/펼치기를
추가한다. Rust 쪽 변경은 기존 structure query 를 WASM 읽기 전용 API 로 노출하는 범위이고, 실제 VSCode
격리 프로필에서 KTX 샘플의 썸네일/목차/북마크 패널 전환까지 확인했다.

원 PR 은 `BEHIND` 상태이고 #2061 이 이 사이드바 변경 위에서 분기되어 있어, #2049/#2061 과 함께
`upstream/devel` 기준 체리픽 통합 PR #2066 으로 처리한다.

## 변경 범위

- `src/document_core/queries/structure.rs`
- `src/wasm_api.rs`
- `rhwp-vscode/src/hwp-editor-provider.ts`
- `rhwp-vscode/src/webview/viewer.ts`
- `mydocs/plans/task_m100_2050.md`
- `mydocs/plans/task_m100_2050_impl.md`
- `mydocs/working/task_m100_2050_stage1.md`
- `mydocs/working/task_m100_2050_stage3.md`
- `mydocs/working/task_m100_2050_stage4.md`
- `mydocs/report/task_m100_2050_report.md`

## 체리픽 기록

| 항목 | 값 |
|------|----|
| 원 기능 커밋 | `5c5c927c5d03`, `749c743e0466`, `0578443540eb`, `7747d2ff4b0`, `a882f3dbff0` |
| 통합 PR 커밋 | `bd0e4f90a`, `c6f2dc694`, `b9c3f97b6`, `d0d58b50a`, `c553f4700` |
| 체리픽 순서 | 2 / 7 - 6 / 7 |
| 충돌 | 없음 |
| 제외 커밋 | `7598995465e8` 는 #2049 async init 과 중복되어 제외 |
| 선행 PR 의존 | #2049 의 async init 수정을 같은 통합 PR 에 선행 적용. #2061 이 후행 의존 |

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
| VSCode 실기 검증 | `samples/KTX.hwp` 27쪽 문서에서 썸네일 목록, 목차 항목, 북마크 빈 상태 패널 확인 |

증적:

- `mydocs/pr/assets/pr_2049_vscode_ktx_render.png`
- `mydocs/pr/assets/pr_2060_vscode_ktx_outline.png`
- `mydocs/pr/assets/pr_2060_vscode_ktx_bookmark.png`

## visual 판단

이 PR 은 rhwp 렌더링 fidelity 자체가 아니라 VSCode viewer UI 를 확장한다. 기준 PDF visual sweep 대상은 아니며,
실제 VSCode custom editor webview 에서 사이드바 UI 와 문서 렌더링이 함께 동작하는지를 화면으로 확인했다.

## merge 후 처리

- #2066 merge 후 #2050 close 여부를 확인한다.
- 원 PR #2060 에 통합 PR #2066 으로 처리됐음을 남기고 close 한다.
