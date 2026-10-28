# PR #6903 — CI Controller 실패 증적 보고 self-review

## 검토 대상과 판정

- 일자: 2026-09-08. 관련 이슈: #6899.
- PR: https://github.com/edwardkim/rhwp/pull/6903
- 최종 self-review 판정: **승인**. 검토 범위는 devel에 제출한 보고 개선 구현이다.
- 검토 head: `bbcdf40ca7bb14340eb6ba128c8732b17c13a64f`.
- 통합 base: `9e4f504fe45b994fba5d9d8d067408a40005c228`.
- 조회 시점: OPEN, MERGEABLE / CLEAN. 이 문서의 후속 commit은 별도 head이므로 병합 전 상태를 재조회한다.
- 이 판정은 메인테이너의 병합 승인, main 배포 승인 또는 #6899 종료가 아니다.

## 접수 경로

- author / assignee: edwardkim / edwardkim. milestone v1.0.0, labels ci / enhancement.
- base / head branch: devel / task_m100_6899. self PR이므로 reviewer를 지정하지 않는다.
- base route: `collaborator_self_merge`.
- modifiers: `intake_and_review`, `local_validation`, `rework_and_exceptions`, `review_only_fast_pass`.
- `pr_review_workflow.md`, `pr_review/README.md` 및 위 자식 문서를 적용했다.
- 검토 시점 17 files, +1687 / -1. 1,000줄 초과 변경에 대한 대형 PR 절차를 적용하며 admin 우회하지 않는다.

## 재작업 원인과 검토 결과

이전 head `38bb7bb87`에서 Archive B는 Rust 설치 중 연결 재설정으로 실패했지만 reporter는
exit 1만 보여 주었다. CodeQL Analyze workflow 성공과 별개인 GHAS 보안 check 실패도 누락했다.
이는 main 미적용만의 문제가 아니라 reporter의 설계 결함이었다.
[R2 검증 기록](../../working/task_m100_6899_rework_stage1.md)에 실제 실패 입력 before/after가 있다.

R2 전체 diff와 원격 검증을 검토했으며 추가 병합 차단 결함은 발견하지 않았다.

- 실행 실패 목록과 독립 GHAS check를 로그보다 먼저 수집한다. 설치 실패·관측 오류·미실행 테스트,
  보안 규칙·경로·행·원본 링크를 제공하며 미확인 영역은 확정 원인처럼 쓰지 않는다.
- policy/classifier는 변경하지 않았다. 기존 verdict·status·트리거·재평가·concurrency를 유지한다.
  선행 CI 실패를 반복해서 전달하는 정상 Controller 동작을 억제하거나 캐시하는 변경은 없다.
- trusted-base helper만 실행한다. PR 코드/artifact는 실행하지 않는다. `checks: read`만 추가하며
  write 권한을 확대하지 않는다.
- PR/head/base, run/attempt/job 식별을 검증한다. GHAS는 별도 check의 provider/head/PR 연계를
  확인하며 Actions attempt에 속하는 것처럼 취급하지 않는다.
- GET 조회만 사용하고 API 토큰을 로그 storage로 전달하지 않는다. redirect 호스트·요청 횟수·시간·
  메타데이터/로그/요약 크기를 제한한다. 조회 실패는 증적 제한으로 보고하며 verdict를 바꾸지 않는다.
- 원문 로그 대신 허용된 진단 형태를 추출하고 외부 문자열을 escape·필터링한다. 테스트의
  `Bad HTML filtering regexp` 지적은 assertion을 보완했으며 alert dismiss로 해소하지 않았다.

## 검증 증적

아래 원격 검증은 모두 검토 head `bbcdf40ca7` 기준이다.

| 검증 | 결과와 근거 |
| --- | --- |
| CI Full | [34231805806](https://github.com/edwardkim/rhwp/actions/runs/34231805806) 성공. Build & Test, Lint, Native Skia, Archive A–D, Frontend package 성공 |
| 새 reporter 테스트 배선 | Lint job `102079671400` 로그에서 reporter 테스트 명령 및 실제 실패 형태·독립 GHAS 사례의 PASS 확인 |
| CodeQL Analyze | [34231805778](https://github.com/edwardkim/rhwp/actions/runs/34231805778) JS/TS·Python·Rust 모두 성공 |
| 독립 GHAS CodeQL | [102081029082](https://github.com/edwardkim/rhwp/runs/102081029082) 성공, 변경 코드 신규 경고 없음, annotation 0건. 저장소 전체 기존 경고 0건이라는 뜻은 아님 |
| CI Impact Policy | [34233404937](https://github.com/edwardkim/rhwp/actions/runs/34233404937) 성공. 기존 운영 Controller의 판정이며 R2 운영 적용 증거는 아님 |
| Adapter / Proptest | [Adapter](https://github.com/edwardkim/rhwp/actions/runs/34231805949), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34231805951) 성공 |

WASM Build·Frontend unit gates·Workflow promotion preflight·duration refresh는 SKIPPED다.
이를 실행 성공으로 집계하지 않는다.

- 최신 base 통합 후 로컬 Node classifier/policy/reporter 122건, 재사용 계약 120건,
  Python workflow 230건, 합계 472건 통과. 변경 YAML 2개 actionlint v1.7.12 통과.
- node 구문·문서 링크·diff 공백 검사 통과. 외부 ShellCheck는 미설치로 별도 미실행.
- 제품 source·샘플·baseline·Rust test 변경이 없어 로컬 Cargo/WASM/시각 검증은 생략했다.
- fetch 후 base 추가 변경 없음. merge-tree 충돌 없음, tree `1ac6db0c501dfd264e97fd55352dce05801e893e`.
- 실제 과거 실패 재생은 설치 오류와 GHAS High 경고를 함께 보고했다. 이는 로컬 read-only 증적이며
  새 workflow가 운영 환경에서 실행되었다는 증거는 아니다.

## 남은 gate와 운영 경계

1. 이 review·작업 기록을 같은 PR의 후속 문서 commit으로 반영하고 최신 head checks를 확인한다.
   review-only fast-pass 여부는 실제 실행 결과로 확인하며 미리 성공을 가정하지 않는다.
2. 최신 head/base의 CI·충돌 상태 재확인 후 메인테이너의 별도 병합 승인을 받아 merge commit으로 병합한다.
3. devel 병합만으로 default branch main의 Controller workflow는 바뀌지 않는다.
   별도 승인된 운영 적용과 live summary·원본 링크·판정 보존 확인 전에는 #6899를 닫지 않는다.

main push/dispatch, 보호 규칙 변경, GitHub approve/comment/merge, issue close는 이 검토에서 수행하지 않는다.
