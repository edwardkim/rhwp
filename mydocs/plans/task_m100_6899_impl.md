# #6899 구현계획 R2 — 실패 증적 중심 보고

## R2 재설계 (2026-09-08, 메인테이너의 재설계·구현 지시)

아래 R2가 이후에 남겨 둔 최초 계획보다 우선한다. 기존 후보 `38bb7bb87`은 Git 이력에 보존한다.
새로운 자식 이슈, branch reset, baseline 변경은 하지 않는다.

### 잘못 잡았던 경계

- workflow conclusion만 따라갔으므로 CodeQL Analyze 성공 뒤 GHAS CodeQL check 실패가 누락됐다.
- 테스트 panic 중심 추출은 설치 단계 네트워크 오류를 exit code로 축약했다.
- 첫 workflow 로그 수집이 뒤 workflow/check의 제한된 진단 예산을 먼저 소모할 수 있었다.
- 테스트 개수와 안전성은 검증했지만 실제 실패 보고의 정보 충족 여부를 완료 gate로 삼지 못했다.
- devel 구현과 main 운영 배포를 구분했으나, 미적용 상태를 최종 목표의 미완료로 충분히 강조하지 못했다.

### R2 데이터 흐름과 불변식

1. 기존 policy status 게시 → 독립 진단. policy 판정과 진단 관측 결과를 별도 필드로 표시한다.
2. live PR identity 검증 → workflow/run/attempt 및 실패 job/step **목록을 먼저** 수집한다.
3. 동일 head의 GitHub Advanced Security CodeQL check를 workflow 성공 여부와 독립적으로 읽는다.
   provider/name/head/PR 연결을 확인한다. GHAS check를 Actions attempt에 귀속한다고 추정하지 않는다.
4. check title·annotation의 경로/행/규칙 제목/검출 설명을 길이 제한·escape 후 제공한다.
   API 응답의 arbitrary URL/raw_details는 출력하지 않는다. 현재 head의 검사 결과임을 표시한다.
5. 모든 목록 수집 후 제한된 worker 로그를 보강한다. 집계 실패와 직접 실패를 구분하고,
   설치/테스트/분석의 실패 위치를 명시한다. 네트워크 오류는 확인된 오류 패턴을 근거로만 분류한다.
6. 요약 계약: **어디서 / 어떤 증적 때문에 / 무엇이 미확인인지 / 다음 조치 / 원본 링크**.
   추출한 관측을 근본 원인·제품 회귀로 자동 승격하지 않는다.
7. `checks: read`만 추가한다. 기존 write 권한 확대·verdict/required check 변경은 금지한다.
   Checks annotations API에 필요한 최소 read 권한이며 원격 배포는 별도 승인한다.
8. 24요청/45초/요청당 5초, 6개 상세 job, 1 MiB/job, 2 MiB metadata, 16 KiB summary 유지.
   audit에서는 CodeQL workflow 증적 유무·성공 여부와 독립적으로 check를 조회하므로 **항상 API 0회 주장을 폐기**한다.
   초기 publish·비대상·stale는 로그를 조회하지 않는다. 제한에 걸리면 누락을 명시한다.

### 운영 및 검증 gate

- 이번 PR의 실제 Archive B 설치 실패와 GHAS 경고를 같은 reporter로 read-only 재생해
  로컬 Markdown 보고서를 만들고 위 다섯 정보가 모두 나오는지 직접 확인한다.
- 오프라인 테스트는 두 실제 실패의 정규화 fixture, CodeQL workflow 성공/check 실패 단독,
  API 권한 실패, stale·다른 head/provider/PR, 목록·시간·크기 상한을 검사한다.
- CodeQL이 지적한 테스트는 sanitizer가 아니라 출력 assertion이다. 태그명 하나가 아닌
  `<`, `>` 전체 부재 및 대소문자 변형의 동일 escape를 검사하도록 고친다. alert dismiss는 하지 않는다.
- 기존 policy/classifier source는 그대로 두고 기존 계약 전체를 재실행한다.
- GitHub check_run trigger는 head가 Actions와 연결된 경우 재귀 방지로 실행되지 않을 수 있어
  즉시 해결책으로 추가하지 않는다. 기존 workflow_run 시점의 snapshot임을 명시한다.
  이후 늦게 바뀐 GHAS check는 다음 Controller 이벤트/승인된 재실행에서 확인할 수 있으며
  이 한계를 감추고 실시간 전수 보고라고 주장하지 않는다.
- main 적용 이전에는 운영 완료가 아니다. 로컬 재생 → 승인된 PR push/CI → 승인된 병합 →
  별도 main 적용 승인 → 실제 Controller summary 검증까지 #6899를 유지한다.
- 외부 근거: [Checks API](https://docs.github.com/en/rest/checks/runs?apiVersion=2022-11-28),
  [이벤트 실행 제약](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows).

---

## 최초 계획 (R2 이전 기록, 아래 API 0회·권한 불변 항목은 R2로 대체)

- 상태: 2026-09-08 승인 범위 구현 및 Stage 3 로컬 검증 완료. remote push·PR 생성 승인 대기.
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

Stage 3 검증 보완: 신규 테스트의 CI 미배선을 발견해 `ci.yml`의 기존 lint 단계에 reporter 테스트 실행을
추가하고 `test_workflow_contract_wiring.py` 목록을 현행화했다. `ci-impact-policy.test.cjs`에는
collector 부가 필드 전후 verdict 동일성 검사를 추가했다. 정책 구현 자체는 변경하지 않는다.

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
