# PR #2187 사전 처리 판단 보고서

## 판단

PR #2187은 collaborator-mediated Route A로 원 PR을 유지하고, 최신 head CI 통과와 작업지시자 승인 후
merge하는 것을 권고한다. 현재 검토에서 blocking finding은 없다.

## 근거

- 기존 `@rhwp/editor` 공개 method와 runtime dependency 0 계약을 유지한다.
- MessageChannel v1은 exact Studio origin, parent source, version/capability/session/port를 검증한다.
- binary는 caller 입력의 복사본을 transfer해 원본 소유권을 보존한다.
- malformed/version mismatch를 timeout으로 숨기지 않고 구조화 오류로 실패시킨다.
- timeout, pending promise, listener, timer, port와 iframe cleanup을 회귀 테스트로 고정했다.
- 제한된 legacy 경로도 iframe source와 exact response origin을 검증한다.

## 최신 baseline 결합

원격 head `7ce03c01`과 최신 `devel@3c1cba96`은 `rhwp-studio/src/main.ts`에서 충돌했다. contributor
commit을 rewrite하지 않고 merge commit `82815ffd`로 결합했다.

충돌 해소는 최신 `devel`의 `getRendererDiagnostics`를 새 embed handler/router로 이전해 renderer readiness
계약과 MessageChannel 구조를 모두 보존한다. 관련 router unit, renderer contract와 실제 legacy browser E2E도
함께 보강했다.

## 검증 요약

- fresh dev WASM: PASS
- editor 15/15, Studio 230/230
- binding/editor 3/3, shared/extension SW 88/88, dist 3/3
- Studio·Chrome·Firefox build, VS Code compile: PASS
- 실제 Chrome public SDK/legacy embed 9개 계약: PASS
- 최신 `devel` 대비 metrics: reported functions +33, CC 총합 +73
- 상위 20 합, CC>25 개수·합, CC>100 개수, max CC: delta 0

## contributor credit

원본 contributor commit `023041f5`, `c7cbcf5f`, `9713a84f`, `7ce03c01`은 author와 history를 그대로
보존한다. collaborator 변경은 최신 `devel` merge/conflict resolution과 review 문서 commit으로 분리한다.
별도 integration PR이나 squash는 사용하지 않는다.

## merge 전 조건

- [ ] `82815ffd`와 review 운영 문서를 contributor head에 반영
- [ ] collaborator 리뷰 댓글로 #2183 완료와 최신 baseline 결합 결과 공유
- [ ] 최신 head `Frontend package gates` PASS
- [ ] 최신 head GitHub Actions required surface PASS
- [ ] `mergeable` / `mergeStateStatus` 재확인
- [ ] collaborator review 제출
- [ ] 작업지시자 merge 승인

## merge 후 확인

- PR merge commit과 contributor credit 확인
- #2186 자동 close 여부 확인; 열려 있으면 작업지시자 승인 후 근거 댓글과 함께 close
- #2022 다음 작업을 #2125 Phase A로 전환
- 원 PR merge와 issue close 결과는 GitHub metadata를 원천 기록으로 사용하고 별도 사후 문서 PR은 만들지 않음

Refs #2186, #2187, #2022
