# #4080 롤백 Stage 1 - cache 고아 ref 정리 이전 상태 복구

- **기준 브랜치**: `upstream/devel` (`d634e608be45d2fd072364a21952a8409d01d9ea`)
- **대상**: PR #4082 merge commit `d634e608be45d2fd072364a21952a8409d01d9ea`
- **작업 브랜치**: `task/revert-4082-cache-orphan`
- **작업 시각**: 2026-08-07 KST

## 1. 수행 계획

작업지시자가 지정한 merge commit 이후 CI 동작 문제가 확인돼, `git revert -m 1`로
PR #4082를 첫 부모 기준으로 되돌린다. 이 방식은 병합 이전 `devel` 상태를 복원하면서
이미 병합된 history를 rewrite하지 않는다.

되돌릴 범위는 하나의 merge commit에 포함된 다음 변경 전체다.

- 고아 ref cache 삭제와 cache 한도 경보를 추가한
  `.github/workflows/cache-generation-sweep.yml`
- CI preflight에 workflow 계약 test를 추가한 `.github/workflows/ci.yml`
- 이에 종속된 cache sweep workflow test 및 #4080 구현·계획·검토 문서

## 2. 사전 분석

2026-08-07의 PR #4094 CI 실패 로그는 cache restore/save 이전의 runner `Set up job`에서
`actions` metadata 조회가 timeout 및 `Service Unavailable`로 실패한 것을 보였다.
동시에 GitHub Status는 Actions major outage를 보고했다. 즉 이 장애의 직접 원인을
cache 용량으로 단정하지 않는다.

다만 저장소 Actions cache는 53개, 10,241,001,878 bytes로 한도 근처이며, 닫힌 PR ref의
cache가 남아 있다. 이 관찰은 별도 운영 판단 대상으로 남기고, 이번 작업은 작업지시자가
지정한 #4082 전체 롤백만 수행한다.

## 3. 검증 계획

1. `git revert -m 1 --no-commit d634e608...` 결과가 #4082의 first-parent diff를 정확히 반전하는지 확인한다.
2. `actionlint .github/workflows/cache-generation-sweep.yml .github/workflows/ci.yml`을 실행한다.
3. `git diff --check`와 workflow YAML 파싱을 실행한다.
4. 변경된 workflow의 계약 test가 제거돼 현재 tree에 남아 있지 않은지 확인한다.

전체 GitHub Actions 검증은 PR 생성·push 승인 뒤 최신 head에서 별도로 확인한다.

## 4. 수행 결과

`git revert -m 1 --no-commit d634e608be45d2fd072364a21952a8409d01d9ea`를 적용했다.
결과는 #4082 merge의 first parent tree와 다음 범위에서 정확히 일치했다.

- cache sweep workflow의 고아 ref 삭제, 한도 경보와 실패 임계치가 제거됐다.
- CI preflight의 cache sweep·workflow wiring 계약 test 배선이 제거됐다.
- #4082가 추가한 구현·계획·검토 문서와 계약 test가 함께 제거됐다.
- 이번 롤백 Stage 문서는 별도 작업 기록으로 유지한다.

## 5. 검증 결과

다음 검증을 순차 실행해 모두 통과했다.

| 검증 | 결과 |
| --- | --- |
| `git diff --cached --exit-code d634...^1 -- <#4082 변경 파일>` | 통과. #4082 first-parent tree 복원 확인 |
| `actionlint .github/workflows/cache-generation-sweep.yml .github/workflows/ci.yml` | 통과 |
| Python `yaml.safe_load`로 두 workflow 파싱 | 통과 |
| `rg`로 제거된 계약 test 참조 확인 | 참조 없음 |
| `git diff --cached --check` | 통과 |

## 6. 원인 판정과 후속 조건

PR #4094의 2026-08-07 CI 실패는 workflow cache restore/save 이전 runner `Set up job`에서
action metadata 조회가 timeout 및 `Service Unavailable`로 실패한 외부 장애였다. 같은 시각
GitHub Status는 Actions major outage를 보고했다. 따라서 이 실패를 #4082 cache sweep의
직접 결함으로 단정하지 않는다.

그러나 #4082는 고아 ref의 자동 전량 삭제, 총량 임계 실패, 새 CI 계약 test를 한 merge에
함께 도입했다. cache가 53개·10,241,001,878 bytes로 한도 근처인 상황에서 해당 운영 변경을
유지할 근거가 충분하지 않다는 작업지시자 판단에 따라, 이번 revert로 이전 세대 정리 정책으로
되돌린다. 원격 PR CI가 최신 head에서 통과하기 전에는 GitHub issue를 close하지 않는다.
