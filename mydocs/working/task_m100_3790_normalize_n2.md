# #3790 정상화 N2 — 구현·집중 검증 결과

- 작성: 2026-09-14 KST
- 상태: **N2 완료. N3 통합 검토·PR 준비 승인 대기.**
- 계획: [수행계획](../plans/task_m100_3790_normalize.md), [구현계획](../plans/task_m100_3790_normalize_impl.md)
- 조사: [N1 결과](task_m100_3790_normalize_n1.md)
- 기준 제품 코드: `922946438fd89346a022df82e76648f88472f0cc`
- 구현 직전 HEAD: `344c793d1` (계획 승인 기록)
- 구현·테스트·운영 문서 커밋: `822a1f76d`
- 브랜치: `task_m100_3790_normalize`

## 1. 결과와 목적 적합성

devel 대상 PR의 선택 실행과 advisory 감사를 유지하면서, main 대상 감사·정책 게시 및 trusted
reuse 조회를 차단했다. 기존 main·release 검사나 Workflow promotion preflight는 제거하지 않았다.
발행자가 v6인데 세 소비자가 v5만 요구하던 계약 불일치를 수정했다. 버전 검사 자체를 완화하거나
임의의 success를 게시하는 우회가 아니다.

| 경계 | 구현·검증 결과 |
| --- | --- |
| job 진입 | 명확한 단일 비devel 연결과 비PR 완료 이벤트 제외 |
| live PR 식별 | open devel, 저장소, head SHA·source branch·repository 일치 필수 |
| 명시된 연결 PR | 해당 번호를 조회하며 다른 같은-head PR로 대체하지 않음 |
| 빈 연결 | 기존 devel PR 검색 후 정확한 단일 후보를 재조회; fork 정상 경로 유지 |
| 게시 직전 | base/head SHA·branch·repository 재확인; retarget·close·base 전진 시 무게시 |
| trusted reuse | 실제 v6 producer 출력 수용; v5·미지원 버전·중복/누락 필드 거부 |
| 비대상 보고 | 내부 장애로 오인하지 않고 제외 사유만 남김; 실제 실패 보고는 유지 |

복수 PR 연결은 모두 main이어도 runner의 최소 신원 조회가 남을 수 있다. job 표현식에서 복수
메타데이터를 완전하다고 추정하지 않고 live 확인하는 보수적 경계다. 명시 연결 조회는 10개로
제한하고 초과/잘못된 번호를 미확정으로 기록한다. 미확정을 success로 취급하지 않는다.
**main workflow 기록 0건을 달성했다는 주장이 아니라, 확인된 비대상에 대한 감사·게시 차단이다.**

## 2. 변경 범위와 보호한 동작

- Controller YAML: job/resolve/publish/reporter 경계만 수정.
- CI·CodeQL·Render Diff: trusted reuse 소비 계약 수정. 기존 CI 검증 step에 새 Node 테스트 추가.
- 새 `scripts/tests/ci-impact-controller-contract.test.cjs`: 실제 YAML inline 코드를 추출해 모의
  GitHub API로 실행한다. 별도로 복제한 판정 함수를 검증하는 테스트가 아니다.
- 기존 Python workflow 계약 및 운영 매뉴얼 2개 현행화.
- producer, classifier, evidence collector, report helper, 제품 Rust/WASM 코드는 변경하지 않음.

CI·CodeQL·Render Diff의 YAML을 구현 직전과 구조 비교했다. trigger와 concurrency는 동일하며,
preflight 외 job은 기존 CI step의 테스트 명령 추가를 제외하면 동일하다. #6819의 취소 정책,
#7069 증거 수렴, #6899 상세 오류 보고, #7070 병합 후 중복 CI 제거를 보존했다.
새 heavy job, timeout 확대, 자동 재실행, 추가 권한, required check 변경은 없다.

## 3. 검증 증적

| 검사 | 결과 |
| --- | --- |
| 구현 전 새 실행형 테스트 19개 | 5 통과 / 14 실패 — 결함 검출 확인 |
| 최종 Node 관련 테스트 | **200/200 통과**, 약 0.918초 |
| 최종 Python workflow 테스트 | **120/120 통과**, 약 10.128초 |
| YAML BaseLoader 파싱 | 변경 workflow 4개 통과 (`on` 키 보존) |
| inline JavaScript syntax | 11개 통과 |
| 기존 CI·CodeQL·Render Diff 비변경 구조 | 3개 통과 |
| 운영 문서 상대 링크 / 메타데이터 | 변경한 2개 모두 통과 |
| `git diff --check`, staged diff check | 통과 |

실행 명령:

```bash
node --test scripts/tests/ci-impact-controller-contract.test.cjs \
  scripts/tests/ci-impact-classifier.test.cjs scripts/tests/ci-impact-policy.test.cjs \
  scripts/tests/ci-workflow-evidence.test.cjs scripts/tests/ci-impact-report.test.cjs \
  scripts/tests/collect-postmerge-duration-data.test.mjs
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  scripts.tests.test_ci_impact_policy_workflow scripts.tests.test_ci_impact_workflow \
  scripts.tests.test_codeql_workflow scripts.tests.test_render_diff_workflow \
  scripts.tests.test_review_only_fast_pass_workflows \
  scripts.tests.test_postmerge_duration_workflow \
  scripts.tests.test_trusted_postmerge_ci_reuse_workflow
python3 scripts/check_markdown_links.py mydocs/manual/github_operations.md \
  mydocs/manual/pr_review/review_only_fast_pass.md
```

로컬 로그는 `output/3790/n2-red.log`, `n2-focused.log`, `n2-node-final.log`,
`n2-python-final.log`에 보존했다. 이 시간은 로컬 실행 비용이며 GitHub runner 절감 실측이 아니다.
테스트 도중 reporter의 Node 내장 모듈 이름을 잘못 지정한 mock을 수정한 뒤 전체 묶음을 재실행했다.
최종 증적은 수정 후 결과다.

메타데이터 검사기는 `--help`를 지원하지 않아 전체 610개 문서 검사로 실행되었으며, 기존 문서
4개에서 필수 필드 누락 16건이 검출되었다. 해당 파일은 구현 직전 HEAD와 바이트 동일함을 확인했다.
변경한 두 운영 문서는 같은 검사기의 `validate_file`로 따로 검사하여 통과했다. 기존 오류는
이번 CI 정상화 범위 밖이므로 고치지 않았다.

## 4. 미검증과 다음 단계

- actionlint는 PATH 및 기존 로컬 설치 위치에서 발견하지 못해 미실행. YAML 파싱·JS syntax와
  모의 실행 검증을 actionlint 또는 GitHub 실제 실행의 대체 성공으로 주장하지 않는다.
- N2는 로컬 코드 검증이다. GitHub webhook·실제 runner·상태 게시·절감 시간은 아직 미검증.
- push, PR 생성, 댓글, 원격 CI dispatch, main 직접 수정, merge는 수행하지 않았다.
- N3에서 정확한 head의 통합 검토와 PR 초안·검증 근거를 준비한다. 이후 승인된 push/PR 절차를 따른다.
- default branch 등록형 Controller YAML은 정상 main 승격 후 활성화된다. devel 병합만으로 운영
  정상화를 완료했다고 보고하지 않는다. main 등록 후 devel 정상 감사와 main 무감사·무게시의 실제
  관측까지 남아 있다.
