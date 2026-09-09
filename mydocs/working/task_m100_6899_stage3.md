# #6899 Stage 3 — 회귀 검증·비용 상한 점검·PR 준비

- Issue: #6899. 일자: 2026-09-08.
- 상태: 승인된 Stage 3 수행 완료. 결과 및 remote push·PR 생성 승인 대기.
- 구현 commit: `be90b31dd`. 최신 devel `e7e978589` 통합 commit: `adfa14b8a`.
- 기준: [구현계획](../plans/task_m100_6899_impl.md), [Stage 2](task_m100_6899_stage2.md).

## 1. 최신 기준 및 검증 중 보완

원격 devel의 PR #6881 병합으로 12커밋이 추가됐다. 제품/검토 문서 변경이며 이번 CI 변경 파일과
겹치지 않았다. 작업 브랜치에 충돌 없이 병합했고 아래 검증은 통합 후 실행했다.

첫 Python `test_*workflow.py` 실행 166건은 통과했지만 이 패턴은 `test_workflow_contract_wiring.py`
등 중간에 workflow가 들어가는 이름을 포함하지 않는다. 배선 검사를 별도 실행하자 3개 assertion이
새 reporter 테스트의 CI 미연결과 정적 발견 목록 누락을 검출했다. 이는 이번 구현의 누락이었다.

- `ci.yml`의 기존 CI impact 검사 단계에 reporter 테스트 실행 한 줄을 연결했다.
- 배선 테스트의 기대 파일 목록에 reporter를 추가했다. 발견·lint 내부 배선 단언은 유지했다.
- 전체 패턴을 `test_*workflow*.py`로 넓혀 재검증했다.
- 기존 policy 테스트에 success/failure/cancelled/timed_out 입력별 run/job/step 진단 필드 추가
  전후 `auditPolicyRuns` 결과 전체 동일성을 추가 검증했다.

## 2. 최종 로컬 검증

| 명령/검사 | 결과 |
| --- | --- |
| `node --test scripts/tests/ci-impact-classifier.test.cjs scripts/tests/ci-impact-policy.test.cjs scripts/tests/ci-impact-report.test.cjs` | 112/112 PASS |
| `node --test scripts/tests/verify-trusted-postmerge-ci-reuse.test.mjs scripts/tests/verify-trusted-postmerge-ci-reuse-squash.test.mjs scripts/tests/verify-trusted-postmerge-review-bridge.test.mjs` | 83/83 PASS |
| `python3 -m unittest discover -s scripts/tests -p 'test_*workflow*.py'` | 227/227 PASS |
| `node --check scripts/ci-impact-report.cjs` | PASS |
| `output/6899/tools/actionlint .github/workflows/ci-impact-policy.yml .github/workflows/ci.yml` | PASS, 진단 0건 |
| `git diff --check` | PASS |

actionlint v1.7.12 공식 릴리즈 Linux amd64 배포물을 내려받고 공식 checksum 목록과 SHA-256을
대조했다. 별도 ShellCheck 실행 파일은 없어 외부 ShellCheck 검사는 수행하지 않았다.
로그는 로컬 `output/6899/stage3-{node,reuse,workflow,actionlint}.log`에 있으며 PR에 stage하지 않는다.
Rust/Skia/WASM/Studio 빌드 및 시각 검증은 제품 source 변경이 없는 이번 PR 범위에서 생략했다.
upstream에서 이미 통합한 제품 변경을 이 PR의 신규 구현으로 계산하지 않는다.

## 3. 비용·보안·보호 조건

- 성공/pending/skip은 추가 진단 API 0회. 원본 사례의 모의 전체 보고는 5회,
  실패 worker 6개 모의 보고는 15회였다. 25번째 요청 거부와 전체 deadline 중단을 검증했다.
- 전체 24요청, 전체 진단 45초/요청당 5초, step 1분, 로그 1 MiB/job·전체 6 MiB,
  metadata 2 MiB/응답, summary 16 KiB를 유지한다. 소수 실제 시간 관측은 Stage 2에 기록했다.
  이는 CI 전체 성능 개선률이나 모든 실패에서의 상세 추출 보장을 의미하지 않는다.
- 인증은 GitHub API에만 전달하고 허용 HTTPS storage 다운로드에는 전달하지 않는다.
  비허용/2차 redirect, 큰 응답·무한 stream, 토큰·Markdown 입력, API 오류·timeout을 검사했다.
- helper는 trusted-base에서만 읽는다. PR 코드·artifact 실행, 로그 원문 복제, comment/status 쓰기 없음.
- reporter는 기존 status 게시 뒤 best-effort로 실행한다. helper 누락·예외는 고정 fallback으로 표시한다.
  runner 종료/전체 job 취소 시 summary 자체가 남지 않을 수 있다.
- `ci-impact-policy.cjs`와 classifier 구현은 upstream 대비 변경 없음. verdict·status context/description,
  required check·fast-pass·concurrency·권한은 유지한다.
- live devel protection 조회: required context `Build & Test`, app ID 15368, strict=false.
  조회 시점 값이며 protection 변경은 수행하지 않았다.

## 4. 제출 및 운영 경계

self PR 경로로 `collaborator_self_merge`, `intake_and_review`, `local_validation`을 적용한다.
문서 포함 diff가 1,000줄을 넘으므로 `rework_and_exceptions` 대형 PR 규칙도 확인했다.
이번 gate는 로컬 검증과 제출 준비이며 self-review·최신 CI·병합 판단을 생략하지 않는다.
번호 예측, PR review 파일/오늘할일 작성, remote push·PR 생성·comment·main 배포는 하지 않았다.

PR 본문 초안은 `output/6899/pr-body.md`. 이슈 자동 종료 표현 없이 `Refs #6899`를 사용한다.
devel 병합 → 별도 승인된 main 운영 적용 → live summary/원본 링크/판정 보존 확인까지 #6899는 유지한다.
다음 승인은 remote push와 devel 대상 PR 생성이다.
