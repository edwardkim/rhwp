# #6815 3단계: reusable workflow 연결과 제출 전 검증

Issue: [#6815](https://github.com/edwardkim/rhwp/issues/6815)

## 구현 계획

- 2단계 `e0e4faaa2`의 verifier를 reusable workflow의 신뢰 base checkout에서만 import한다.
- collector는 current-base bridge 한 개를 지나되 파일/commit 목록의 불완전한 경계는 Full로 닫는다.
- Full candidate에 실제 검사 merge artifact가 있을 때 필요한 commit/tree 객체만 fetch한다.
  checkout은 계속 신뢰 base이며 객체 fetch 후 candidate 코드를 실행하지 않는다.
- 독립 Git tree 증거를 evaluator에 전달하고 기존 duration artifact 재사용·출력·required check를 유지한다.
- CI에 신규 테스트를 연결하고 실제 workflow script를 실행하는 통합 mock으로 collector부터 output까지 검증한다.
- 운영 문서에 지원 범위와 최초 배포/후속 실측의 차이를 명시한다.

## 검증 결과

2026-09-06 KST, 기준 `upstream/devel`은 `6a193a648dba3df6d5c4cffa0182bc02f3e011ff`다.
최종 fetch에서도 기준선 변경이 없어 rebase는 필요하지 않았다.

### 구현 중 추가 확인

- 실제 associated-PR API는 `changed_files`를 제공하지 않았다. PR 상세 API로 파일 수를 읽고,
  요약/상세 응답의 PR·head·merge·repository가 일치하는지 확인하도록 보완했다.
- `git show --format=%P`는 shallow 경계에서 부모를 숨긴다. runner의 `--depth=1` fetch 뒤에도
  실제 부모를 읽도록 `git cat-file commit`의 원본 header를 검사하고 replacement object를 비활성화했다.
- 임시 Git 저장소의 실제 shallow fetch, 실제 workflow JavaScript 실행, API 파일 목록 잘림,
  fetch 실패, source 충돌 보정, 만료 artifact, 실패한 최종 head를 회귀 테스트로 고정했다.

### 통과한 로컬 검증

```bash
node --test scripts/tests/ci-impact-classifier.test.cjs \
  scripts/tests/ci-impact-policy.test.cjs \
  scripts/tests/verify-trusted-postmerge-ci-reuse.test.mjs \
  scripts/tests/verify-trusted-postmerge-ci-reuse-squash.test.mjs \
  scripts/tests/verify-trusted-postmerge-review-bridge.test.mjs
python3 -m unittest discover -s scripts/tests -p 'test_*workflow.py'
actionlint .github/workflows/trusted-postmerge-ci-reuse.yml .github/workflows/ci.yml
git diff --check
```

- Node 164개 통과(신규 bridge 계약 49개 포함).
- Python workflow discovery 162개 통과. 별도 선택 계약 134개도 통과했으며 일부 중복이다.
- actionlint 오류 0건. 제품 Rust/renderer 변경이 없어 Cargo·WASM·시각 검증은 실행하지 않았다.
- 변경 문서 4개의 내부 링크와 변경 매뉴얼 metadata를 검사해 통과했다.
- live `devel` protection의 required context는 `Build & Test`였다. 이름·발행 job·권한을 바꾸지 않았다.
- 전체 문서 metadata 검사에서는 기준선과 동일한 기존 4개 문서의 필수 필드 누락 16건이 나왔다.
  `benchmark_vs_alternatives.md`, issue-4964 README, issue-5511 README·CLI inventory가 해당하며
  모두 이번 diff 밖이다. 관련 없는 metadata 변경은 포함하지 않았다.

### 실제 #6813 데이터의 읽기 전용 재실행

GitHub의 PR/commit/run/job/artifact 응답과 실제 Git 객체를 사용해 현재 workflow script를 실행했다.
기존 run을 재실행하거나 GitHub 상태를 수정한 결과가 아니다.

| Workflow | 기존 post-merge | 수정 후 판정 | 재사용 source run | timing 재사용 |
| --- | --- | --- | --- | --- |
| CI | `candidate-full-lane-evidence-unavailable` | `reuse=true` | [34029158620](https://github.com/edwardkim/rhwp/actions/runs/34029158620) | `true`, B/C/D |
| CodeQL | `candidate-full-lane-evidence-unavailable` | `reuse=true` | [34029158604](https://github.com/edwardkim/rhwp/actions/runs/34029158604) | `false`, 대상 아님 |

두 판정의 reason은 `current-base-review-bridge-green-pr-workflow-reused`이며 warning은 없었다.
테스트된 merge tree는 `d205ad5c53988e3533b91c65fa81ab1930141b23`, 최종 squash tree는
`6f613d232cb96bcee2ace0d6785eadd20816ff77`다. 두 tree의 차이는 허용된 `mydocs/**`뿐이었다.
따라서 과거 실패 계보를 새 판정으로 통과시키되, 검증하지 않은 소스 변경은 통과시키지 않는 것을 확인했다.

## 제출 및 남은 검증

- 기본 경로: collaborator self-merge. 보조 경로: intake, local validation, review-only fast-pass.
  `pr_review_workflow.md`, 선택표와 해당 자식 문서, `github_operations.md`의 O3 절차를 적용했다.
- 작업지시자가 준비 완료 뒤 remote push·PR 생성을 승인했다. PR review와 오늘할일은 **옵션 2**로
  분리하고 이번 코드 PR에 trailing commit/push하지 않는다. 이 단계별 구현 보고서는 코드와 함께 커밋한다.
- 이번 PR은 enforcement 변경이라 Full CI 대상이며, 최초 post-merge도 기존 base verifier를 사용하므로
  Full일 수 있다. PR CI·후속 runner에서 실제 heavy skip을 관찰한 결과로 과장하지 않는다.
- #6815는 `Ref`로 연결하고 열린 상태를 유지한다. 실제 이후 코드 PR의 current-base 문서 merge에서
  CI/CodeQL skip, 기존 timing artifact 재사용, required check 정상 유지까지 확인한 뒤 완료를 판단한다.
- rollback은 이번 변경 commit들의 revert PR로 수행한다. source/workflow 변경이 잘못 skip되거나
  required aggregate가 사라지면 병합을 중단하고 기존 fail-closed 동작으로 복귀한다.
- 원본 log, 임시 API JSON, 임시 재실행 script는 커밋하지 않는다. 원격 PR 본문에는 검증 결과만 적는다.
