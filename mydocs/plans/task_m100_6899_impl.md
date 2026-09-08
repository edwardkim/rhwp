# #6899 구현계획 — 판정과 독립된 실패 진단 보고

- 상태: 2026-09-08 메인테이너 구현계획 승인. Stage 2 구현 착수.
- 근거: [수행계획](task_m100_6899.md), [Stage 1 조사](../working/task_m100_6899_stage1.md).
- 핵심 결정: `auditPolicyRuns`의 판정 및 reason 계약은 유지하고, 별도 reporter가 증적을 연결한다.

## 1. 변경 파일

| 경로 | 역할 |
| --- | --- |
| `.github/workflows/ci-impact-policy.yml` | 진단용 metadata 보존, trusted helper sparse checkout, publish 뒤 진단 step와 고정 fallback |
| `scripts/ci-impact-report.cjs` (신규) | 데이터 정규화·유형 분류·안전한 summary, 제한된 실패 로그 조회/추출 |
| `scripts/tests/ci-impact-report.test.cjs` (신규) | 주입형 API/clock/stream을 사용한 오프라인 경계 테스트 |
| `scripts/tests/test_ci_impact_policy_workflow.py` | helper 신뢰 경계·보고 위치·권한·시간 상한·취소 guard 검증 |
| `mydocs/manual/github_operations.md` | 실패 summary 해석과 원본 증적 이동 절차 |

필요하면 작은 정규화 JSON fixture를 scripts/tests/fixtures 아래 추가한다.
`ci-impact-policy.cjs`의 verdict 함수를 보고 기능 때문에 변경하지 않는다.
Rust source/test, 샘플, baseline, CI worker 실행 범위, 권한 및 concurrency는 변경하지 않는다.

## 2. 배선과 데이터 계약

1. 기존 collector의 동일 API 결과에서 run ID/attempt, job ID/attempt, step number를 부가 필드로 보존한다.
   policy JSON의 기존 필드·결과는 그대로 둔다. 보고 파일과 policy 출력은 분리한다.
2. 기존 최소 summary와 status 게시를 먼저 완료한다. 게시 step에는 outcome 식별용 ID를 부여한다.
3. `always() && !cancelled()`로 실행되는 마지막 best-effort 진단 단계를 추가한다.
   continue-on-error 및 1분 step timeout으로 reporter가 verdict를 바꾸지 않게 한다.
4. helper 존재·resolve 결과·각 단계 outcome을 확인한다. helper는 오직 trusted-base에서 로드한다.
   helper가 없거나 로드 실패하면 고정 문자열과 단계 outcome으로 fallback summary를 남긴다.
5. 진단 모델은 `upstream-failure`, `controller-error`, `evidence-unavailable`, `pending`, `success`,
   `stale/skipped`를 구분한다. 단계 outcome failure와 policy가 의도적으로 게시한 failure를 구분한다.
6. PR/head/base, workflow/run/attempt, 실패 job/step, 오류 종류와 원본 링크를 출력한다.
   Build & Test 같은 집계 job도 표시하되 worker 실패와 구분한다. 단일 근본 원인을 자동 단정하지 않는다.

## 3. 원본 증적 조회와 오류 추출

- 성공·pending·정상 skip 경로는 기존 수집 자료와 단계 outcome만 사용하며 추가 진단 API 0회.
- 실패 경로만 run identity를 재확인하고 선택 attempt에 귀속된 jobs를 조회한다.
  `latest` 자료의 attempt가 맞지 않으면 재사용하지 않는다. run/job/head/base 연결 불명확 시 추출하지 않는다.
- workflow 3개, workflow당 job 목록 최대 2페이지(100/page), 상세 실패 job 전체 최대 6개.
- 이번 사례의 annotation은 exit code뿐이므로 추가 checks 권한 없이 `actions: read`의 job 로그를 사용한다.
- 필요한 job 로그만 조회하고 스트리밍 읽기 상한을 적용한다. 전체 run ZIP이나 PR artifact는 받지 않는다.
- GitHub API가 주는 로그 다운로드 redirect는 GitHub가 사용하는 허용 HTTPS 저장소 주소만 수용한다.
  구현 시 실제 응답 host 계약을 확인·고정하며 임의 URL, credentials, private IP, 비HTTPS, 재귀 redirect는 거부한다.
  저장소 토큰을 다운로드 host로 전달하지 않는다. signed URL·원문 오류 response는 summary에 남기지 않는다.
- nextest FAIL/test name, panic/assertion 유형, compiler error code, timeout/exit code,
  baseline의 신규 검출 건수 등 허용 패턴을 구조화한다. 임의 stdout/문서 본문/stack 전체는 복제하지 않는다.
  샘플 식별자·문자열은 허용된 공개 저장소 상대 경로임을 입증할 수 없으면 요약에서 생략한다.
- 추출 못한 경우 `오류 상세 미확인: <분류된 사유>`와 원본 job 링크를 제공한다.
  일부 로그만 읽은 경우 부분 수집임을 표시하고, 마지막 오류까지 읽었다고 주장하지 않는다.

## 4. 비용·보안 상한

| 항목 | 상한 |
| --- | --- |
| 추가 진단 요청 | 전체 24회, redirect 다운로드 포함, 자동 retry 0 |
| 진단 실행 | 전체 45초, 요청당 최대 5초, step timeout 1분 |
| 로그 읽기 | job당 최대 1 MiB, 전체 6 MiB, 상한 도달 시 stream 중단 |
| 메타데이터 읽기 | 응답당 최대 2 MiB, job 목록 초과 시 일부 누락 표시 |
| 출력 | summary 전체 16 KiB, 단일 표시 필드 240자, 상세 오류 최대 6건 |

네트워크 대기·파싱·출력 모두 전체 deadline 안에 포함한다.
ANSI/control/Actions command 표식 제거, Markdown/HTML escape, token/credential 형태 차단을 적용한다.
허용 패턴 이외 문자열은 그대로 공개하지 않는다. 런타임 shell 보간·eval·PR 파일 require는 금지한다.
링크는 검증된 repository/run/job 숫자 ID로 구성한다.

한도에 도달하면 `상세 수집 제한`으로 보고하고 기존 판정은 유지한다.
정상·실패 호출량과 시간은 모의 API 테스트에서 상한을 검증하고 실제 사례 read-only 조회에서 관측값을 기록한다.

## 5. 검증 계획

1. Stage 2 전 최신 devel 통합. 변경 후 기존 policy 37건과 workflow 13건을 유지·확장한다.
2. 신규 helper 테스트: 원본 사례, upstream/자체오류 분리, 성공/pending/cancel/timeout,
   여러 실패/집계 구분, stale/attempt mismatch, 조회 403/404/rate-limit/timeout,
   로그 부재/부분 수집/잘못된 redirect/큰 입력/민감정보/Markdown injection/UTF-8 경계.
3. 기존 verdict와 status 출력은 입력별 동일성 확인. reporter 예외·timeout으로 판정이 바뀌지 않음 확인.
4. 기존 classifier/policy·fast-pass·CI/CodeQL/Render Diff workflow 계약을 영향 범위에 따라 실행한다.
5. actionlint를 준비해 YAML 검증. 제품 코드가 없으므로 Cargo/Skia/WASM 전체 빌드는 실행하지 않는다.
6. main/devel helper 혼합 버전(새 YAML + helper 없음)에서 안전한 fallback 확인.
   실행 취소·runner 종료 시 보고 불가 한계를 문서화한다.

## 6. 완료와 적용

Stage 2 구현/집중검증 → Stage 3 회귀·비용·PR 준비 → 별도 승인 후 push/PR/self-review/merge 순서다.
새 helper를 실행하는 main YAML 배선이 배포되기 전에는 운영 개선 완료라고 보고하지 않는다.
main 승격은 기존 #6819 차이까지 포함할 수 있으므로 이번 변경만 적용됐다고 추정하지 않고 별도 승인받는다.
실제 승인된 운영 적용 후 원본 run/job 링크, 테스트명·핵심 오류, verdict 보존을 확인해야 이슈를 닫는다.
