---
kind: snapshot
status: completed
canonical: mydocs/plans/task_m100_6634.md
issue: 6634
last_verified: 2026-09-06
---

# #6634 Stage 5-B 실증 — exact-head 비게시 Actions와 정책 정정

## 1차 실행 기준

- 후보 branch: `task_m100_6634`
- exact remote SHA: `829b698cbd9626fd40a4f05a22a3780c3f9204ca`
- 포함한 devel: `51ad998e33ef7f5191b0e1b0b656dc44cef33a1c`
- 실행 승인: 메인테이너가 Release Binary와 Publish All Packages의 비게시 dispatch를 승인
- 실행 전 공개 상태: `@rhwp/core`, `@rhwp/editor`, VS Code Marketplace, Open VSX의 `0.8.6`이
  모두 `already-present`

두 workflow는 같은 exact SHA에서 2026-09-05 14:45 UTC에 시작했다. KST 기준 완료 시각은 날짜가
바뀐 2026-09-06이다.

## 1차 원격 실행 결과

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

## 최종 exact-head 재실증

정정 policy와 회귀 테스트를 commit하고 최신 `upstream/devel`을 병합한 뒤, 원격 작업 브랜치의 정확한
SHA에서 두 workflow를 다시 실행했다.

| 항목 | 값 |
| --- | --- |
| branch | `task_m100_6634` |
| exact remote SHA | `559edb06826e7b8bfa2348d5951d78cf18d066e9` |
| 포함한 devel | `ff1ce007b428547da74e0d6b7e9a196592c60ff6` |
| Release Binary | [34001610087](https://github.com/edwardkim/rhwp/actions/runs/34001610087), 26분 34초, success |
| Publish All Packages | [34001611474](https://github.com/edwardkim/rhwp/actions/runs/34001611474), 11분 23초, success |

Release Binary는 5개 native archive build가 모두 성공하고 `Attach to GitHub Release`가 skipped됐다.
이어진 reusable workflow에서는 source guard, WASM, VSIX와 aggregate가 성공하고 외부 publish job 4개가
skipped됐다. CLI archive 5개, `wasm-pkg`, `vscode-vsix`, `release-publish-evidence`의 artifact 8개가
생성됐다.

직접 Publish All Packages도 source guard, WASM, VSIX와 aggregate가 성공하고 외부 publish job 4개가
skipped됐다. `wasm-pkg`, `vscode-vsix`, `release-publish-evidence` artifact 3개가 생성됐다.

두 aggregate evidence는 모두 다음 계약을 만족했다.

- `mode=verify`, `githubSha=559edb06826e7b8bfa2348d5951d78cf18d066e9`
- source guard, WASM, VSIX gate `success`
- npm core/editor, VS Code Marketplace, Open VSX `jobResult=skipped`, `state=verify-only`
- `errors=[]`, `accepted=true`, `verdict=completed`

정정된 policy로 새 exact-head run을 수집한 결과 run 2개와 모든 job·artifact pagination이 완결됐고
waiver는 0건이었다. offline verifier는 두 run을 모두 `verify-only`로 수락했다.

| verifier 항목 | 값 |
| --- | --- |
| policy SHA-256 | `dbd0bcd8d2829fdf7ffebfab5245f2cf9d2fc022906a2cf556d58f0579bd7b24` |
| inventory SHA-256 | `c8ac74b8cfc885817c73b6e317943e0a7fcd8030cb7f2b15e50897eac3805df5` |
| verdict SHA-256 | `febb524fa0a9df0ab39700ebcf1a85115eda85fc4afe8d54b7a1a14c240d0385` |
| errors / verdict | 0건 / `ok=true` |

실행 후 `@rhwp/core`, `@rhwp/editor`, VS Code Marketplace, Open VSX는 모두 기존 `0.8.6`의
`already-present` 상태였다. `test` Git tag와 GitHub Release도 생성되지 않았다.

## 최종 판정과 다음 게이트

Stage 5 종료 게이트인 **전체 직접 호출 경로 성공, 외부 publish 차단, 공개 채널 무변경, 정정 policy의
exact-head 증적 수락**을 모두 충족했다. verifier를 완화하거나 waiver를 사용하지 않았다.

이 문서는 원격 실증 뒤 작성하는 증적 기록이므로, 이를 commit한 SHA 자체를 위 workflow가 검증했다고
주장하지 않는다. 다음 `devel -> main` promotion에서는 #6689 절차에 따라 그 시점의 exact `devel` SHA로
새 증적을 만들어야 한다.

다음은 Stage 6이다.

1. 최신 `upstream/devel`과 열린 workflow PR을 재확인하고 merge tree를 검증한다.
2. 최종 보고서에 원인·변경·검증·보안·비용·rollback·다음 release canary를 정리한다.
3. 보고서 결과 승인 뒤 commit, push와 PR 생성은 각각 별도 승인을 받는다.
