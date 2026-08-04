# PR #3082 검토 기록 — CI 필수 gate 실패 시 후속 테스트 중단

## 메타

| 항목 | 내용 |
|---|---|
| PR | [#3082](https://github.com/edwardkim/rhwp/pull/3082) |
| 작성자 | `jangster77` |
| base / head | `devel` / `task/3064-ci-fail-fast` |
| 관련 이슈 | [#3064](https://github.com/edwardkim/rhwp/issues/3064) |
| 문서 작성 시점 참고값 | full-CI head `b12f0aea4`, mergeable `MERGEABLE`, merge state `CLEAN` |

## 변경 검토

- PR #3054에서 frontend gate 실패 뒤 다른 고비용 worker가 계속 실행된 원인을, 각 worker가
  `preflight`만 의존하는 sibling 구조와 shard의 `fail-fast: false`로 확인했다.
- 필수 gate를 `preflight → lint → frontend → native-skia → archive → shard` 순서로 연결했다.
- frontend 비대상 job의 정상 `skipped`는 native-skia 이후 체인을 허용하고, frontend 실패·취소 및 lint
  실패는 후속 worker 전체에 전파한다.
- 기본 테스트 matrix `[1,2,3,4,5,6,7,8]`은 유지하고 `fail-fast: true`만 적용했다. 정상 경로는
  8-way 병렬이고, 한 shard 실패 시 나머지 실행 중·대기 shard가 취소된다.
- 최종 `Build & Test` 집계는 항상 실행해 worker 실패 상태를 required check에 표시한다.
- 추가 요청에 따라 PR #3077의 `scripts/pr_triage.sh`를 `pr_review_workflow.md` 2.0절에서 찾고 사용할 수
  있도록 실행 명령, 조회 상한, 누락 방지와 사람의 최종 판정 책임을 문서화했다.

## 렌더 영향 판정

CI workflow와 운영 문서만 변경했다. renderer/layout/typeset, 샘플, golden, 기준 PDF를 변경하지 않아
visual sweep 대상이 아니다.

## 검증

### 로컬 정적 검증

| 항목 | 결과 |
|---|---|
| YAML parse | PASS |
| `actionlint .github/workflows/ci.yml` | PASS |
| 필수 job `needs` 계약 | PASS |
| shard matrix 1~8 보존 | PASS |
| `test-shard.strategy.fail-fast == true` | PASS |
| `bash -n scripts/pr_triage.sh` | PASS |
| `git diff --check` | PASS |

제품 코드와 테스트 명령은 바꾸지 않았다. CI workflow 변경은 최신 GitHub Actions 결과로 전체 회귀를
판정한다.

### GitHub Actions 실측

full-CI head `b12f0aea4` 기준:

- CI run [29932241496](https://github.com/edwardkim/rhwp/actions/runs/29932241496): **success**
  - preflight, lint, frontend, native-skia, archive가 순서대로 success
  - archive 완료 직후 shard 1/8~8/8이 모두 동시에 `in_progress`가 되어 8-way 병렬성 확인
  - 8개 shard 모두 success, 최종 `Build & Test` success
- CodeQL run [29932240860](https://github.com/edwardkim/rhwp/actions/runs/29932240860): **success**
- Render Diff run [29932240548](https://github.com/edwardkim/rhwp/actions/runs/29932240548): **success**

이전 head `389fe1988`의 CI run 29930709636은 update branch 뒤 force-cancel했고 최종 상태가
`completed/cancelled`임을 확인했다.

## 판단

요구한 실패 전파와 shard 병렬 보존이 workflow 계약 및 원격 성공 경로에서 확인됐다. 실제 실패 시
GitHub Actions matrix 취소는 `strategy.fail-fast: true`의 표준 동작을 사용한다. 최신 PR head의 정형
fast-pass 결과까지 통과하면 merge 가능으로 판단한다.
