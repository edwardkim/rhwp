# PR #6903 — CI Controller 실패 증적 보고

- 관련 이슈: #6899.
- 작성자 self-review 접수 기록. 메인테이너의 push·PR 생성 승인에 따른 기록이며 최종 self-review 확정·병합 승인은 별도다.
- 일자: 2026-09-08. GitHub 상태는 작성 시점 참고값이며 merge 직전에 재조회한다.

## 접수와 경로

| 항목 | 확인 결과 |
| --- | --- |
| PR | https://github.com/edwardkim/rhwp/pull/6903 |
| author / assignee | edwardkim / edwardkim |
| base / head branch | devel / task_m100_6899 |
| 검증 code candidate | `bdd1e8a6e0a7903e71dfe9d0f655edcd3b47f0c4` |
| 통합 base | `e7e9785893e7dffdaa600859cb1f28cca18621cb` |
| 생성 직후 규모 | 14 files, +1155 / -1. 이 리뷰·오늘할일 등 후속 기록은 별도 추가 |
| 상태 | OPEN, Draft 아님, MERGEABLE / BLOCKED |
| 트리야지 | milestone v1.0.0, labels ci / enhancement |
| reviewer | self PR이므로 지정하지 않음 |

- base route: `collaborator_self_merge`.
- modifiers: `intake_and_review`, `local_validation`, `rework_and_exceptions`, `review_only_fast_pass`.
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md` 및 위 자식 문서.
- 1,000줄 초과 변경이므로 대형 PR 규칙을 적용한다. admin merge로 검증을 건너뛰지 않는다.

## 범위와 근거

기존 status 판정 뒤 실행되는 best-effort 진단 helper와 workflow 배선, 오프라인 경계 테스트,
운영 안내를 추가했다. Stage 3에서 새 테스트 CI 미배선을 발견해 연결하고 배선 검사를 통과했다.
[결과보고서](../../report/task_m100_6899_report.md), [Stage 3 검증](../../working/task_m100_6899_stage3.md)에 상세 근거가 있다.

- policy/classifier 구현과 기존 status context·description·권한·concurrency·fast-pass 계약은 유지했다.
- run/job/step 식별 필드를 더한 입력과 기존 입력의 policy verdict 전체 동일성을 검사했다.
- helper는 trusted-base에서만 읽고 PR code/artifact를 실행하지 않는다.
- 원문 로그 대신 허용된 오류 형태를 추출한다. 인증 헤더 분리·redirect 제한·데이터/요청/시간 상한을 검사했다.
- 실제 과거 실패 로그의 핵심 오류 추출과 live PR stale 중단을 확인했다. 새 workflow의 운영 실행 증적은 아니다.
- renderer·샘플·Rust source/test 변경이 없으므로 visual sweep 대상이 아니다.

## 완료한 로컬 검증

검증 code candidate의 실행 파일은 Stage 3 최종 실행과 동일하다. push 직전 fetch에서 base 변경이
없음을 확인했고 `git merge-tree --write-tree HEAD upstream/devel` 및 diff 공백 검사를 통과했다.

- Node classifier/policy/reporter 112/112 PASS.
- trusted post-merge reuse 계약 83/83 PASS.
- Python `test_*workflow*.py` 227/227 PASS.
- actionlint v1.7.12: 변경 workflow 2개 PASS. node 구문·문서 링크·diff 검사 PASS.
- Cargo/Skia/WASM/Studio/시각 검증은 제품 변경이 없어 생략했다. 외부 ShellCheck는 미설치로 미실행이다.
- 원격 PR 본문을 API로 다시 읽어 로컬 UTF-8 본문과 완전 일치함을 확인했다.

## 잔여 위험과 최종 판정

- 최종 판정: **머지 보류**.
- 보류 사유: 최신 PR head의 GitHub CI가 아직 완료되지 않았으며 최종 self-review 확정·병합 승인 전이다.
- 해제 조건: 이 기록이 포함된 최신 head CI 성공, 동일 head/base의 상태 재확인, 승인된 self-review 및 병합 판단.
- review-only 후속 기록이라도 녹색 code candidate가 아직 없으면 Full CI가 수행될 수 있다. fast-pass 성공을 미리 주장하지 않는다.
- devel 병합은 구현 반영일 뿐 main Controller 운영 적용이 아니다. 운영 적용과 live 요약 확인 전 #6899를 닫지 않는다.
- 이 문서는 GitHub approve/comment/merge, main 배포 또는 issue close를 수행하지 않는다.
