# PR #6814 리뷰 보정 계획

- 단계: Hyper-Waterfall 후속 보정 — 계획 승인 후 구현·검증·게시 순서.
- 승인: 2026-09-07 사용자 “권장 순서대로 진행해주고, 보정 코멘트도 게시해줘.”
- 대상: [리뷰 5126268574](https://github.com/edwardkim/rhwp/pull/6814#pullrequestreview-5126268574).
- 기준 head: `bff48959d77528f6ee02c2744bd1779726440b77`.
- merge·이슈 종료는 이번 승인에 포함하지 않는다.

## 구현 순서와 계약

1. 리뷰 2·3: 문서 미열림과 구버전 binding 오류를 구분하고 UI에서 안내한다.
   적용 실패 시 모든 문단의 rollback을 시도하고 원래 오류와 복원 오류를 함께 보존한다.
   복원이 불완전하면 command를 Undo에 보존하여 재시도할 수 있게 한다.
   Undo 실패도 해당 엔트리를 유지하며, 부분 Redo 실패는 Undo 쪽으로 옮긴다.
   다른 command의 기존 실패 처리 계약은 바꾸지 않는다.
2. 리뷰 4: 첫 ref가 0보다 뒤에서 시작해도 기존 조회와 동일하게 첫 ID가 앞 구간을
   상속하도록 적용·조회·복원을 일치시킨다. 선택 밖 의미가 변하지 않아야 한다.
3. 리뷰 6: Node WASM 산출물 없이도 실제 command/history 실패 경로를 검사하는
   필수 unit test를 추가한다. 선택적인 실제 WASM 검증과 구분한다.
4. 리뷰 1·5·7~12: 범위 판정 공유, 명시적인 lookup 오류, 검증 오류 구체화,
   중복 직렬화·경로 계산·경계 ref 정리를 관련 수정 범위 안에서 진행한다.
5. 실제 결과로 self-review와 작업 기록을 보완하고 보정 코멘트를 게시한다.

## 검증과 롤백

- 새 회귀는 문서 없음/낡은 binding, 다중 문단 apply 및 rollback 복합 실패,
  Undo 재시도, 부분 Redo 실패, 첫 ref prefix와 선택 밖 서식 보존을 포함한다.
- Studio 필수 unit test와 빌드, fresh WASM 테스트 및 변경 범위의 Rust 회귀를 실행한다.
- Rust push 전 필수 lint 3종·workspace build·fmt·파생 suite 검사를 순차 수행한다.
  현재 디스크 여유가 부족하므로 실행 불가 시 이를 성공으로 기록하거나 push하지 않는다.
- 기존 브라우저 스크린샷은 기존 구현의 증적으로 유지하며 새 오류 경로 검증으로 간주하지 않는다.
- 보정 문제가 발견되면 이번 보정 커밋만 역패치한다. 기존 구현·사용자 파일·다른 worktree는
  되돌리거나 삭제하지 않는다.

## 정정할 리뷰 해석

- HashMap 누락 panic의 현재 도달 경로는 확인되지 않았다. 조용한 원본 ID fallback 대신
  불변식 위반을 명시적으로 드러내는 방향으로 보강한다.
- 복원 중 예외가 나면 기존 코드는 원래 적용 오류가 아니라 복원 오류를 던진다.
- 기존 CI 성공에는 실제 WASM Studio 테스트의 skip이 포함됐다. 로컬 실제 WASM 성공과
  CI에서 항상 실행되는 회귀의 범위를 구분한다.

## 구현 및 현재 검증 결과 (2026-09-07)

- 계획 commit: `e7380e47a`. 구현 candidate: `2f6f46376`.
- 1~12번 관련 보정 구현: fallible ID lookup, 겹침 iterator 공유, 첫 ref 상속,
  tail 경계 원본 보존, 오류 구체화, UI 안내, 실패 복구 history 보존, 직렬화·경로 캐시.
- 불완전 rollback은 원래 오류와 모든 복원 오류를 AggregateError에 남긴다.
  복구 command 위의 후속 편집은 기존 순서대로 Undo한 뒤 복구를 다시 시도할 수 있다.
  이 계약은 opt-in이며 다른 command의 기존 실패 시 제거 동작은 유지한다.
- 새 필수 테스트는 실제 command/history/bridge/InputHandler를 읽되 WASM import를 stub으로
  대체한다. 본문·중첩 셀 복합 실패, Undo 재시도, 부분 Redo, 빈 선택, 기존 Redo 보존,
  미열림·구버전 binding 안내, 미관련 오류 재전파를 검사하며 pkg/pkg-node가 필요 없다.
- `npm test`: **1428 passed, 0 failed, 0 skipped**. 최초 실행에서는 새 오류 cause 계약에
  맞지 않은 기존 assertion과 CommonJS 테스트의 alias import가 실패했으며 보정 후 재실행했다.
- `npm run build`: TypeScript + Vite 통과. 기존 pkg의 WASM을 사용한 frontend 빌드이며
  새 Rust로 만든 WASM 빌드라는 주장은 아니다.
- `node --test rhwp-studio/tests/mixed-char-format-recovery.test.ts`: 추가 edge case 후 재통과.
- `cargo fmt --all` / `--check`, 별도 integration source rustfmt, `git diff --check`: 통과.
- `cargo check --locked --offline --lib --target-dir target/pr-review`: 통과(19.27초).
  다른 review worktree의 읽기 전용 cache를 APFS clone으로 별도 target에 복사했으며 원본은
  변경하지 않았다. 이 검사는 필수 lint 묶음이나 Rust 행위 회귀를 대체하지 않는다.

### 남은 게이트와 중단 조건

- 저장 공간이 작업 중 8.6GiB에서 **6.4GiB**로 감소했다. 저장소 추적 파일만 약 3.34GiB다.
  별도 review checkout과 workspace/native/WASM 빌드가 필요한 전체 검증을 현재 여유 공간으로
  강행하지 않는다. 다른 작업 파일을 삭제하지 않고 공간 확보 또는 외장 경로를 요청했다.
- Rust focused 17건, review suite prepare/check, 필수 Clippy 3종·workspace build,
  release-test 전체·Native Skia·fresh WASM 검증은 미완료다. 이전 후보의 통과로 대체하지 않는다.
- source push 및 새 CI는 아직 수행하지 않았다. merge·이슈 종료도 수행하지 않았다.
- 리뷰 13: 현 source에는 `orders/20260907.md`가 없으나 확인한 upstream/devel에는 #6818 기록과
  source에 없는 내부 링크가 있다. 해당 문서를 복사·덮어쓰거나 add/add 충돌을 만들지 않았다.
  현재 단계는 구현 후 재검증 중이므로 8.2.1의 최종 준비 시점에 대장 통합 경로를 확정한다.
  현행 CLAUDE.md는 문서 포인터이며 리뷰가 인용한 #10/#13 본문과 동일하지 않다.

### 사용자 승인 코멘트 게시

- [보정 코멘트](https://github.com/edwardkim/rhwp/pull/6814#issuecomment-5561537199)를 게시했다.
  검증한 사실, 리뷰 해석 정정, 아직 로컬인 구현과 남은 검증을 구분했다.
- UTF-8 Markdown 파일을 `gh pr comment --body-file`로 게시하고 API 본문을 원문과 `diff`로
  비교했다. gh 출력의 추가 끝 개행 1개를 정규화한 뒤 일치함을 확인했다.
  한글 치환·선두 BOM 문제는 없다.
- 보정 문서 3개의 상대 링크 검사와 `git diff --check`가 통과했다.
