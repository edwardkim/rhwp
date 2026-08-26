# PR #5685 통합 검토 기록

- 대상 PR: [#5685](https://github.com/edwardkim/rhwp/pull/5685)
- 작성자: `jangster77` (maintainer integration)
- 통합 원 PR: [#5675](https://github.com/edwardkim/rhwp/pull/5675),
  [#5676](https://github.com/edwardkim/rhwp/pull/5676),
  [#5684](https://github.com/edwardkim/rhwp/pull/5684)
- code candidate: `960d7439bfa34b2b1eccbe7b7b2dafad1af21621`

## 누적 범위와 검토 결론

- #5675는 `d20647822438a2f869e08c7f15c214051d9bf31d` →
  `c67c27da24beb5681acca87521ae16cdb1d33e99` →
  `c63711e6f1d707fe8bd422680cfaa1295f1fe091` 순으로 적용했다. 도형의 회전·대칭만 command
  역연산으로 전환하고, 저장 바이트가 달라질 수 있는 그림·OLE는 snapshot을 유지한다. 두 경로는
  `executeOperation`을 거쳐 양식 모드 차단을 우회하지 않는다.
- #5676의 `f22b676c4587a2733672535b3a9de6727d2f83c6`은 선택 범위 실재성 검사를
  `CursorState.selectRange` 소유 계약으로 이동한다. 실제 호출은 undo 복원 한 곳뿐이어서 셀
  선택 경로를 새로 제한하지 않는다.
- #5684의 `db1fe3a3e006102f18efe8616c5b956f9f39ff0a`은 CI WASM stub에 `exportHwp`와
  `exportHwpx`를 생성 `pkg/rhwp.d.ts`와 같은 `Uint8Array` 반환형으로 명시한다.
- 체리픽 충돌과 차단 결함은 없었고, contributor 원 branch와 history는 변경하지 않았다.

## 로컬 검증

- `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`, Studio TypeScript 검사를 통과했다.
- `npm --prefix rhwp-studio test`: `1,019 passed, 1 skipped`.
- 로컬 headless Chrome에서 `e2e:undo-object-selection`의 그림·표 undo/redo 선택 해제 계약을 통과했다.
- 이전 VM Chrome CDP 주소의 도달 불가는 로컬 headless 재검증으로 환경 문제로 분리했다.

## GitHub CI

- code candidate의 Build & Test, Lint, Frontend package gate, Native Skia, Canvas visual diff,
  CodeQL, archive·regular/slow shard, Proptest, Adapter inter-diff가 통과했다.
- 최종 병합 조건은 이 trailing head의 required CI 성공과 작업지시자 승인이다.

## 결론

- 통합 PR #5685는 최신 trailing head CI 성공 뒤 병합 권고다.
