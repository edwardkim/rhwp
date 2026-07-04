# PR #1442 리뷰 처리 계획

## 목적

Task #493 PR #1442를 Collaborator 절차에 맞춰 검토하고, 리뷰 문서와 오늘할일을 archive 경로 기준으로 PR head에 반영한 뒤 GitHub Actions 재확인 후 merge한다.

## 처리 단계

1. PR 메타데이터 확인
   - PR #1442 URL, base/head, draft 여부, merge state 확인 완료.
   - base는 `edwardkim/rhwp:devel`, head는 `edwardkim/rhwp:task_m100_493`이다.
   - head가 원본 저장소 브랜치이므로 문서 커밋은 `upstream/task_m100_493`에 직접 push한다.

2. 변경 범위 검토
   - HWPX 셀 보호/필드 이름/양식 모드 편집 가능 속성 보존이 중심이다.
   - Studio 보호 셀 hover/click/input 차단 UX와 보호 셀 선택 상태의 `셀 속성...` 진입을 포함한다.
   - 표 외곽 클릭 선택과 표 개체 선택 상태의 `표 속성...` 진입을 포함한다.
   - 표/셀 속성 대화상자의 탭별 크기 흔들림을 대화상자 전용 레이아웃으로 보정했다.
   - 관련 이슈는 `Closes #493`으로 PR 본문에 명시돼 있다.

3. 로컬 검증
   - `cargo test --test issue_493_cell_attrs`: 통과
   - `cargo test --test issue_493_hwpx_cell_field_name`: 통과
   - `cargo test --test issue_258_clickhere_form_mode`: 통과
   - `cargo test set_cell_field_text_updates_text_metadata --lib`: 통과
   - `npm --prefix rhwp-studio run build`: 통과
   - `cargo build --release`: 통과
   - `cargo test --release --lib`: 통과 (`1842 passed; 0 failed; 6 ignored`)
   - `cargo test --profile release-test --tests`: 통과
   - `cargo fmt --check`: 통과
   - `cargo clippy --all-targets -- -D warnings`: 통과
   - `wasm-pack build --target web --out-dir pkg`: 통과

4. 시각 검증
   - 보호 셀 hover guard, 보호 셀 클릭 선택, 보호 셀 입력 차단, 보호 셀 `셀 속성...` 진입, 표 개체 `표 속성...` 진입을 확인했다.
   - `표/셀 속성` 대화상자의 탭 전환 크기 안정성은 작업지시자 시각 검증 완료 상태다.

5. 리뷰 문서/오늘할일 커밋
   - `mydocs/pr/archives/pr_1442_review.md`
   - `mydocs/pr/archives/pr_1442_review_impl.md`
   - `mydocs/orders/20260619.md`
   - 문서 전용 변경이므로 `git diff --check`와 변경 파일 범위 확인으로 검증한다.

6. 원격 push
   - 작업지시자가 remote push를 지시했으므로 문서 커밋을 `upstream/task_m100_493`에 push한다.
   - push 전 원격 PR head를 fetch하고 로컬 브랜치를 fast-forward해 원격의 `devel` merge commit을 보존했다.
   - push 후 PR diff에 archive 리뷰 문서와 오늘할일이 포함됐는지 확인한다.

7. GitHub Actions 재확인
   - 문서 커밋 push 후 required checks가 재실행되면 완료를 기다린다.
   - 모든 required checks가 통과하면 merge 절차로 넘어간다.

8. 후속 처리
   - `Closes #493` 자동 close 여부를 확인한다.
   - auto-close가 실패하면 workflow 기준에 따라 이슈 close 코멘트 초안을 준비한다.
   - merge 후 `upstream/devel`을 fetch하고 로컬 기준 브랜치를 동기화한다.
   - 임시 원격 브랜치 `task_m100_493` 삭제 여부를 확인한다.

## 주의 사항

- 리뷰 문서는 active `mydocs/pr/` 경로를 거치지 않고 archive 경로에 바로 작성한다.
- 문서 커밋 push 후 CI 통과 여부만 추가하려고 새 문서 커밋을 다시 push하지 않는다.
- GitHub PR/issue 코멘트는 초안을 작업지시자에게 보여주고 승인받은 뒤 등록한다.
