---
kind: snapshot
status: active
canonical: mydocs/plans/task_m100_6634.md
issue: 6634
last_verified: 2026-09-06
---

# #6634 Stage 5-B 1차 실증 — exact-head 비게시 Actions와 정책 정정

## 실행 기준

- 후보 branch: `task_m100_6634`
- exact remote SHA: `829b698cbd9626fd40a4f05a22a3780c3f9204ca`
- 포함한 devel: `51ad998e33ef7f5191b0e1b0b656dc44cef33a1c`
- 실행 승인: 메인테이너가 Release Binary와 Publish All Packages의 비게시 dispatch를 승인
- 실행 전 공개 상태: `@rhwp/core`, `@rhwp/editor`, VS Code Marketplace, Open VSX의 `0.8.6`이
  모두 `already-present`

두 workflow는 같은 exact SHA에서 2026-09-05 14:45 UTC에 시작했다. KST 기준 완료 시각은 날짜가
바뀐 2026-09-06이다.

## 원격 실행 결과

| workflow | run | 소요 | 판정 |
| --- | --- | ---: | --- |
| Release Binary | [33972764228](https://github.com/edwardkim/rhwp/actions/runs/33972764228) | 29분 4초 | success |
| Publish All Packages | [33972765438](https://github.com/edwardkim/rhwp/actions/runs/33972765438) | 12분 4초 | success |

두 run 모두 `workflow_dispatch`, branch `task_m100_6634`, head SHA
`829b698cbd9626fd40a4f05a22a3780c3f9204ca`로 일치했다.

Release Binary는 다음 경계를 실제로 통과했다.

- Linux x86_64/AArch64, macOS x86_64/AArch64, Windows x86_64 5개 build와 archive upload 성공
- `Attach to GitHub Release` skipped
- `Publish packages after binary release / Validate release source`, `Build WASM`, `Build VSIX once`,
  `Publish channel aggregate` 성공
- 중첩 npm core/editor, VS Code Marketplace, Open VSX publish job 4개 skipped
- CLI archive 5개와 `wasm-pkg`, `vscode-vsix`, `release-publish-evidence`의 8개 artifact 생성

직접 Publish All Packages는 source guard, WASM, VSIX, aggregate가 성공했고 외부 publish job 4개가
skipped됐다. `wasm-pkg`, `vscode-vsix`, `release-publish-evidence` artifact 3개를 생성했다.

두 evidence JSON은 모두 다음 값을 가졌다.

- `mode=verify`
- `githubSha=829b698cbd9626fd40a4f05a22a3780c3f9204ca`
- gates 3개 `success`
- 네 channel `jobResult=skipped`, `state=verify-only`
- `errors=[]`, `accepted=true`, `verdict=completed`

실행 뒤 네 공개 채널은 실행 전과 똑같이 `0.8.6 already-present`였다. `test` Git tag와 GitHub Release도
생성되지 않았다. 따라서 비게시 보호 경계는 실제 GitHub 환경에서 지켜졌다.

## promotion policy RED와 원인

workflow 자체는 성공했지만 최초 offline verifier는 Release Binary run만 거부했다. 누락으로 보고된 것은
중첩 성공 job 4개와 skipped job 4개였다.

Stage 4 policy는 reusable workflow의 REST job 이름 접두사를 caller job ID인 `publish-packages`로
추정했다. 실제 GitHub API는 YAML ID가 아니라 caller의 표시명 `Publish packages after binary release`를
접두사로 사용했다.

| 구분 | 값 |
| --- | --- |
| 잘못 추정한 이름 | `publish-packages / Build WASM` 등 |
| 실제 API 이름 | `Publish packages after binary release / Build WASM` 등 |
| 영향 | 배포 workflow 성공에는 없음; #6689 promotion evidence만 fail-closed 거부 |

policy의 8개 중첩 job 이름을 실제 API 값으로 고치고, 정확한 성공·skip 이름 배열을 회귀 테스트로
고정했다. focused promotion test 33건과 workflow 전체 161건이 통과했다.

정정 policy로 두 기존 run을 read-only collector가 다시 수집한 결과 pagination 완결, waiver 0건이었다.
offline verifier는 두 run을 모두 `verify-only`로 수락했고 오류 0건, `ok=true`를 반환했다.

collector 실행 뒤 결과 표시용 `jq`가 runs JSON을 객체로 잘못 가정해 한 번 실패했다. collector가 만든
원본은 정상 배열이었고 훼손되지 않았다. 배열 스키마로 다시 표시하고 같은 원본에 offline verifier를
실행했으며 위 성공 판정을 얻었다.

## 현재 판정과 다음 게이트

1차 원격 실증은 workflow 실행계약과 정정 방향을 입증했다. 그러나 정정한 policy와 회귀 테스트는 아직
로컬 변경이며, 이를 commit하면 후보 SHA가 달라진다. #6689의 exact-head 불변식 때문에 이전 녹색 run을
새 최종 후보의 promotion 증적으로 재사용하지 않는다.

따라서 Stage 5-B의 현재 상태는 **정책 결함 정정 완료, 최종 exact-head 재실증 대기**다. 다음 순서는
다음과 같다.

1. policy 정정·회귀·이 기록을 로컬 commit
2. 별도 승인 뒤 새 task head를 push
3. 별도 승인 뒤 두 verify-only workflow를 새 exact head에서 다시 실행
4. collector `ok=true`, 외부 채널 무변경을 재확인한 뒤 Stage 5-B 종료

verifier를 완화하거나 waiver를 사용하지 않는다.
