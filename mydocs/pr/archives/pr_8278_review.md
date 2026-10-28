# PR #8278 리뷰 — 문서 전용 병합의 nextest 실행 시간 갱신 생략

## 최종 판정

**승인 — 최종 code head의 로컬 계약 검사와 전체 GitHub CI 성공을 확인하고 일반 merge했다.**
검토 head는 `9cd6411c6e7a071345c2d6c6a16ff76a6a1f65ad`, merge SHA는
`ecc240a569ef764963f56a5d948f1714ae6cc56f`다. 사용자가 지시한 옵션 2에 따라
이 검토와 오늘할일을 별도 문서 전용 PR로 반영한다.

## 문제와 변경

- [PR #8278](https://github.com/edwardkim/rhwp/pull/8278), 작성자 `jangster77`, base `devel`,
  관련 [이슈 #8277](https://github.com/edwardkim/rhwp/issues/8277).
- [기존 run](https://github.com/edwardkim/rhwp/actions/runs/40300506279/job/121496914491)은
  문서 전용 PR #8274 병합 후 전체 checkout에 약 2분 48초를 쓰고,
  `no-verified-pr-duration-measurements`로 갱신을 보류했다.
- duration workflow의 push 경로를 PR CI의 trigger 생략 경로와
  `AGENTS.md`, `CLAUDE.md`, `llms.txt`, 일반 `mydocs/**`에 맞췄다.
  루트 Markdown 전체를 무조건 제외하지 않는다.
- `mydocs/tech/canvaskit-parity-implementation.md`와 `mydocs/tech/text-ir-v2.md`는
  PR CI와 같이 렌더링 계약 문서로 다시 포함한다. 코드 혼합·알 수 없는 경로와 수동 실행은 유지한다.
- 기존 계약 테스트에서 PR CI 경로 목록 불일치와 문서·혼합·코드·렌더링 계약 문서의 run/no-run을 검사한다.

## 검증 결과

비교 base는 `01ced3d42724285c8f9765a453393ab6af326617`이며, rebase는 하지 않았다.

| 검사 | 명령·결과 |
| --- | --- |
| Python workflow 계약 | `venv\Scripts\python.exe -m unittest scripts/tests/test_postmerge_duration_workflow.py scripts/tests/test_ci_impact_workflow.py scripts/tests/test_codeql_workflow.py scripts/tests/test_workflow_contract_wiring.py scripts/tests/test_review_only_fast_pass_workflows.py` — 91건 통과 |
| Node 영향 분류·정책 | `node --test scripts/tests/ci-impact-classifier.test.cjs scripts/tests/ci-impact-policy.test.cjs scripts/tests/ci-impact-controller-contract.test.cjs scripts/tests/ci-impact-report.test.cjs scripts/tests/ci-workflow-evidence.test.cjs` — 187건 통과 |
| YAML·공백·링크 | actionlint 1.7.12, base/head `git diff --check`, 운영 매뉴얼 내부 링크 통과 |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/40303008152) | Lint, Native Skia, Frontend package, A~E builder·worker, Build & Test 성공 |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/40303008155) | JavaScript/TypeScript·Python·Rust·Actions 분석 및 최종 CodeQL 성공 |
| [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/40303007872)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/40303008143) | 성공 |
| [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/40308082715) | 최종 head의 status success |
| Merge gate | 동일 head, 실패·대기 없음, MERGEABLE/CLEAN 재조회 후 `--match-head-commit` 일반 merge |

## 공통 검토와 적용 후 관찰

- 조판 원칙·Visual Sweep: 비해당. 메타데이터 workflow의 event 라우팅과 계약 테스트·운영 설명만 변경하며,
  제품 파싱·조판·렌더링 코드와 기존 fixture를 변경하지 않았다.
- 검증 입력 커밋 확인: HWP/HWPX/PDF를 검증 입력으로 사용하거나 추가하지 않았다.
- 권한·실행 경계: 기존 job 명령과 권한, PR required check `Build & Test`를 유지했다.
  merge 뒤 제품 검증 CI를 수동 실행하지 않았다.
- 코드 PR 병합 후 [duration metadata run](https://github.com/edwardkim/rhwp/actions/runs/40308158691)은
  성공했다. 실제 `refreshed_targets=46`과 metrics branch의 `222ae0a6 → 9e8d1846` 발행을 확인했다.
  [문서 전용 PR #8280](https://github.com/edwardkim/rhwp/pull/8280)은 PR 전체 review-only fast-pass와
  merge 뒤 duration no-run의 실제 대조군이다.
- 문서 PR merge 전 최신 head의 preflight·Build & Test·mergeability를 확인한다. merge 뒤에는
  정확한 merge SHA의 workflow 목록을 재조회해 duration run 부재를 확인한다.
  아직 실행하지 않은 문서 PR의 결과를 통과로 기록하지 않는다.

## 후속 comment와 정리

문서가 devel에 반영되고 문서 PR의 duration no-run까지 확인한 뒤 원 이슈·PR에
최종 SHA·검토 링크·로컬/CI 결과·실제 skip 관찰을 한국어 존댓말로 남긴다.
문서 PR 자체에 대한 후속 기록 PR이나 반복 오늘할일은 만들지 않는다.
이번 작업의 두 local/remote branch와 `output/issue8277`을 정리한다.
기존 `output/pr-review`의 #8111·글꼴 cache와 공유 `target/pr-review`는 다른 작업의 자료이므로 보존한다.
