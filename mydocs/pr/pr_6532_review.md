# PR #6532 검토 - Studio legacy installed substitute face

- 검토일: 2026-08-31
- 작성자: `planet6897`
- base: `devel` (`upstream/devel@887b4ce15`로 rebase)
- 원 PR head: `ef0d03fc74ac34bae709f5e24c61c51f4b412fcf`
- 통합 commit: `f535a4636`
- 상태: 승인 (통합 검증본 기준)

## 범위

- Studio의 installed substitute face 탐색이 문서의 `altType` 선언에 막히지 않도록 exact lookup 뒤 type-agnostic lookup을 추가한다.
- legacy 한양 face 4종의 `altType` 0/1/2와 installed face 부재 경로를 계약 테스트로 고정한다.

## 검토 결과

- exact `altType` lookup을 먼저 유지해 기존 우선순위를 보존하고, 실패한 경우에만 type-agnostic lookup으로 확장했다.
- `npx tsx --test tests/issue-6263-legacy-face-alttype.test.ts` 결과: `2 pass`, `0 fail`.
- 변경은 Studio substitution 계약에 한정되며, Rust renderer paint output을 바꾸지 않는다.

## 공통 검증

- Rust format, native/WASM/workspace/all-target Clippy, workspace build 통과
- 전체 `nextest` 종료 코드 `0`

## 병합 조건

- 원격 병합 또는 통합 PR 게시 직전에 원 PR head와 CI green 상태를 다시 확인한다.

## Merge 후 contributor PR comment 계획

- 대상: [#6532](https://github.com/edwardkim/rhwp/pull/6532)와 관련 issue #6263.
- 선행 조건: 통합 PR의 merge SHA가 `upstream/devel`에 포함될 것.
- 내용: 통합 PR·merge SHA, Studio contract `2 pass / 0 fail`, 전체 nextest, substitution 범위가 renderer paint output을 바꾸지 않는다는 검토 결론을 남긴다.
- issue가 OPEN이면 merge 반영과 검증 증적을 comment로 남긴 뒤 close 여부를 확인한다.
