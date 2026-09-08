# #6899 Stage 2 — 실패 진단 reporter 구현 및 집중 검증

- 일자: 2026-09-08
- 상태: 구현·집중 검증 완료, 결과 및 Stage 3 진행 승인 대기.
- 기준 통합: upstream/devel `54f4a0237e...`를 `6d07ca5e4`로 작업 브랜치에 병합. 충돌 없음.
- 범위: Controller 배선, 신규 진단 helper, 관련 테스트·운영 안내. 제품 source·baseline 변경 없음.

## 1. 구현 결과

- `scripts/ci-impact-report.cjs`: 기존 정책 판정과 독립된 읽기 전용 reporter.
- collector가 이미 받은 run ID/attempt, job ID/attempt 및 step 번호를 보존하도록 부가 필드만 추가했다.
- 기존 status 게시 이후 `Explain CI failure evidence`를 실행한다. best-effort, 1분 step timeout,
  취소 guard를 적용했으며 status나 policy output을 새로 쓰지 않는다.
- status 게시 성공 표식과 stale 표식으로 의도적인 failure 게시와 실제 publish API 오류를 구분한다.
- resolve/checkout/helper 실패에도 workflow의 고정 fallback summary가 동작한다.
- 선행 실패, 내부 단계 오류, 증적 미확인, 오래된 이벤트, 성공/대기, 정책 차단을 구분한다.
- 실패 시 live PR 및 run identity를 재확인하고 attempt 전용 job 목록을 조회한다.
  worker와 Build & Test 집계를 구분하며 다중 실패를 제한된 목록으로 표시한다.
- 원문 전체 로그 대신 허용된 테스트명·panic/컴파일 오류·검출 건수·exit code를 추출한다.
  Markdown 이스케이프, credential 패턴 차단, 다운로드 host 제한 및 인증 헤더 분리를 적용했다.
- 추가 요청 24회, 전체 45초/요청당 5초, 로그 1 MiB/job·6 jobs, metadata 2 MiB/응답,
  summary 16 KiB 상한을 구현했다. 정상/대기/skip은 추가 진단 API 0회다.

## 2. 집중 검증

| 실행 | 결과 |
| --- | --- |
| `node --test scripts/tests/ci-impact-report.test.cjs scripts/tests/ci-impact-policy.test.cjs` | 67/67 PASS (reporter 30 + 기존 policy 37) |
| `python3 -m unittest discover -s scripts/tests -p 'test_ci_impact_policy_workflow.py'` | 15/15 PASS |
| `node --check scripts/ci-impact-report.cjs` | PASS |
| PyYAML로 Controller YAML 구문 파싱 | PASS — actionlint를 대신하는 전체 검증은 아님 |
| `git diff --check` | PASS |
| 기존 policy/classifier source diff | 변경 없음 |

첫 집중 실행에서는 테스트 2개가 이스케이프된 Markdown 원문과 표시 문자열을 동일하게 가정해 실패했다.
출력 안전성은 유지하고 기대값을 이스케이프 계약에 맞춘 뒤 재검증했다. 테스트명 `::`도 표시상 원문을
보존하는 entity로 처리했다. 실패를 제외하거나 핵심 오류 추출 조건을 완화하지 않았다.

추가로 악성 redirect/다단계 redirect, 토큰 전달 금지, 과대·무한 stream 중단,
API 403/404/429/500, 네트워크 timeout/deadline/request cap, attempt/head mismatch,
job pagination 제한, 다중 workflow 실패, UTF-8 summary 상한, 실제 YAML fallback 실행을 검증했다.

## 3. 실제 GitHub read-only 검증

1. GitHub job log API의 실제 redirect host를 `productionresultssa1.blob.core.windows.net`으로 확인했다.
   signed URL 및 토큰 값은 출력/기록하지 않았다.
2. reporter transport와 extractor로 과거 job `102045013116` 로그를 직접 읽었다.
   68,446 bytes, 부분 수집 아님, 추가 요청 2회(로그 URL 조회 + 다운로드), 약 1,796 ms.
3. 실제 추출 결과: `text_overlap_baseline::text_overlaps_do_not_grow_partition_13`,
   글자 겹침 보호 검사 실패, 신규 검출 1건(baseline 없음/회귀 미확정), exit code 100.
   문서명·원문 내용은 재게시하지 않았다.
4. 전체 reporter를 live PR identity 검사와 실행했을 때는 API 1회/약 658 ms 후 오래된 이벤트로 중단했다.
   현재 PR #6898 head는 `2e0ab46ab591ce8dfb165e64312ced0f6f549116`로 바뀌었고,
   원본 실패 head `5f2ea958...`와 다르기 때문이다. 이를 현재 PR 실패 보고 성공으로 주장하지 않는다.

원격 Actions의 새 workflow 실행, status·comment 게시, push는 수행하지 않았다.
위 시간은 소수 read-only 관측값이며 전체 CI 성능 영향의 최종 측정이 아니다.

## 4. 보호 조건과 남은 gate

- 기존 `auditPolicyRuns` verdict, status context/description, fast-pass·concurrency·permissions 유지.
- reporter는 PR 코드·artifact를 실행하거나 baseline을 변경하지 않는다.
- 기존 target/캐시 정리 및 다른 PR의 제품 수정은 이번 작업에 포함하지 않는다.
- Stage 3에서 관련 classifier/fast-pass/CI/CodeQL/Render Diff 계약 확대 검증,
  actionlint 준비, 변경 최종 점검·비용 상한 검증·PR 준비를 진행한다.
- devel 반영과 main 운영 배포를 구분한다. 아직 workflow는 원격에 적용되지 않았다.
