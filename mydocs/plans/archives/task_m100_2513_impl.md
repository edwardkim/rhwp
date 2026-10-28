# M100 #2513 구현계획서 - embed loadFile 기본 대화상자 교착

## 1. 구현 원칙

- RED 테스트로 SDK 생략/명시 값 전달 계약을 먼저 고정한다.
- `suppressDialogs` 기본 계산만 바꾸고 transport/runtime 구조를 재설계하지 않는다.
- public 문서의 기본 예제와 option 표를 실제 동작에 맞춘다.
- 각 단계 완료 후 보고서와 검증 결과를 기록하고 승인 없이 다음 단계로 넘어가지 않는다.

## 2. 단계별 계획

### Stage 1 - SDK 계약 RED/GREEN

1. `npm/editor` 테스트에 옵션 생략은 true, 명시 false는 false인 RED 사례를 추가한다.
2. `RhwpEditor.loadFile()`의 params 계산을 최소 수정한다.
3. targeted Node tests를 실행한다.

완료 조건: 생략/명시 false/명시 true 세 경로가 모두 기대 params를 전송한다.

### Stage 2 - 실제 iframe 회귀와 문서

1. embed E2E에서 `대체 글꼴로 보기` polling/click을 제거한다.
2. zero-option `loadFile()`이 fresh iframe에서 제한 시간 안에 완료되는지 확인한다.
3. `index.d.ts`와 README의 기본값·예제를 수정한다.

완료 조건: 사용자 입력 없는 load/export E2E와 npm package gates가 통과한다.

### Stage 3 - 전체 검증과 PR

1. 수정 파일 범위 포맷과 diff를 점검한다.
2. 저장소 필수 gate를 실행한다.
3. 단계별 보고서·최종 보고서·오늘할일을 갱신하고 커밋한다.
4. fork의 `fix/issue-2513` 브랜치로 push한다.
5. `edwardkim/rhwp:devel` 대상 PR을 생성한다.

완료 조건: 필수 gate green, clean worktree, 이슈 연결 PR URL 확보.

## 3. 검증 명령

```bash
node --test npm/editor/tests/*.test.mjs npm/editor/tests/**/*.test.mjs
cd rhwp-studio && node e2e/embed-transport.test.mjs
cargo fmt --all -- --check
cargo test --profile release-test --tests
cargo clippy -- -D warnings
git diff --check
```

## 4. 롤백

- SDK의 생략 기본값 계산과 관련 테스트/문서 커밋만 revert한다.
- Studio runtime과 protocol schema는 건드리지 않으므로 별도 data migration은 없다.

## 5. 승인

- 상태: 승인 완료
- 기록: 2026-07-20 작업지시자 `승인`
- Stage 1부터 시작한다.
- 추가 기록: 2026-07-20 작업지시자 `해결책도 pull request로 구현해서 review 요청까지 . stage2 그후 진행`
- Stage 2와 Stage 3 전체 gate, 커밋·push·PR·리뷰 요청까지 계속 진행한다.
