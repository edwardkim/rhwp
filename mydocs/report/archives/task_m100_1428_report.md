# 완료 보고서 — Task M100-1428

- 이슈: https://github.com/edwardkim/rhwp/issues/1428
- 제목: 개체 속성 비율 유지 옵션 및 모달 외부 클릭 닫힘 정합화
- 브랜치: `local/task_m100_1428`
- 작성일: 2026-06-18
- 작업 모드: 기여자 모드. 오늘할일 문서는 생성하지 않음.

## 1. 결과 요약

rhwp-studio의 개체 속성 대화상자와 누름틀 편집 동작을 한컴오피스 기준에 가깝게 보정했다.

개체 속성 기본 탭에는 `비율 유지` 설정을 추가했다. 이 설정은 문서 포맷 속성이 아니라
사용자 UI 설정이며, `rhwp-settings`에 저장된다. 설정이 꺼져 있으면 너비와 높이를 독립적으로
입력할 수 있고, 켜져 있으면 기존 rhwp-studio처럼 한쪽 값 변경 시 다른 축을 원본 비율로 보정한다.

모달 대화상자는 외부 overlay 클릭만으로 닫히지 않게 정리했다. 확인, 취소, 닫기 버튼, Escape처럼
명시적인 조작만 닫힘 경로로 남겼다.

누름틀 편집 쪽에서는 고치기 완료 후 hidden textarea 포커스와 캐럿을 복구하고, 빈 guide 클릭,
누름틀 경계 클릭, 인접 누름틀 guide hit-test, 누름틀 복사/붙여넣기 후 입력 위치를 보정했다.

## 2. 변경 사항

| 파일 | 내용 |
|---|---|
| `rhwp-studio/src/ui/picture-props-dialog.ts` | 기본 탭 `비율 유지` 체크박스 추가, ON/OFF에 따른 너비/높이 상호 보정 분기 |
| `rhwp-studio/src/core/user-settings.ts` | `dialog.picturePropsKeepRatio` 사용자 설정 추가 |
| `rhwp-studio/src/ui/dialog.ts` 및 각 모달 UI | overlay 클릭 닫힘 경로 제거 또는 명시 닫힘 경로로 정리 |
| `rhwp-studio/src/ui/field-edit-dialog.ts` | 누름틀 고치기 대화상자 종료 후 포커스/캐럿 복구 콜백 추가 |
| `rhwp-studio/src/engine/input-handler*.ts` | 누름틀 마우스 진입/이탈, 빈 guide 클릭, 붙여넣기 후 field 바깥 상태 보정 |
| `src/document_core/queries/cursor_rect.rs` | ClickHere guide hit-test가 같은 문단의 실제 guide 위치와 field range를 맞추도록 보정 |
| `src/document_core/commands/clipboard.rs` | 내부 붙여넣기 결과에 field 포함 여부를 전달 |
| `tests/issue_258_clickhere_form_mode.rs` | 인접 누름틀 guide hit-test와 붙여넣기 후 입력 위치 회귀 테스트 추가 |
| `rhwp-studio/tests/user-settings.test.ts` | 개체 속성 비율 유지 사용자 설정 저장 테스트 추가 |
| `mydocs/plans/task_m100_1428*.md` | 수행/구현 계획 기록 |
| `mydocs/working/task_m100_1428_stage*.md` | stage 1-5 구현 및 검증 기록 |

## 3. 구현 세부

`비율 유지`는 HWP/HWPX 저장 속성으로 모델링하지 않았다. 기존 `sizeFixed`와 별개의 UI 편의 설정으로
취급했고, 기본값은 기존 동작과 호환되도록 ON으로 두었다.

모달 닫힘 정책은 팝업성 메뉴가 아니라 대화상자 UI에만 적용했다. 대화상자 외부 클릭은 무시하고,
명시적인 닫기 조작만 유지했다.

누름틀 경계 처리는 키보드 방향키에서 쓰던 field boundary 모델을 마우스 클릭과 붙여넣기 후 상태에도
맞춰 확장했다. 빈 guide 클릭은 실제 guide start를 우선하고, 값이 있는 누름틀의 오른쪽 바깥 클릭은
field end 밖으로 이동한 상태로 표시한다.

## 4. 검증

최종 PR 준비 검증:

```bash
git diff --check upstream/devel..HEAD
cargo fmt --check
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm test
cargo build --release
cargo test --release --lib
cargo test --profile release-test --tests
cargo clippy --all-targets -- -D warnings
wasm-pack build --target web --out-dir pkg
cd rhwp-studio && npm run build
```

결과:

- `rhwp-studio` 테스트: 75 passed
- `cargo test --release --lib`: 1830 passed, 6 ignored
- `cargo test --profile release-test --tests`: 통과
- `cargo clippy --all-targets -- -D warnings`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
  - `pkg/rhwp_bg.wasm`: 5.5M
  - `pkg/rhwp.js`: 274K
- `rhwp-studio` production build: 통과
  - Vite chunk size warning은 표시됐지만 build는 성공했다.

## 5. PR 준비 상태

- 최신 `upstream/devel` 기준으로 branch ahead 5 상태를 확인했다.
- `upstream/task_m100_1428` 원격 브랜치와 기존 PR은 아직 없다.
- PR 준비 문서로 `mydocs/report/task_m100_1428_report.md`와
  `mydocs/report/task_m100_1428_pr_body.md`를 추가했다.
- PR 생성 시 원격 브랜치는 `task_m100_1428`로 push하고, base repository는 `edwardkim/rhwp`, base branch는 `devel`로 한다.

```bash
git push upstream HEAD:task_m100_1428
gh pr create --repo edwardkim/rhwp --base devel --head task_m100_1428 --title "task 1428: 개체 속성 비율 유지와 누름틀 편집 정합화" --body-file mydocs/report/task_m100_1428_pr_body.md
```

## 6. 후속

PR 생성은 작업지시자 승인 후 진행한다.
