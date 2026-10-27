---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase110-gym-differential-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase110-gym-differential-contracts/goal-checkpoint.md` · 원문 SHA-256: `a85ecee7b6ea680241514b353acd0d340b8db45fa7783c2e7307f9e45a7c018d`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 110차 기록

직접 검토 11,082개 함수, Rust 테스트 6,737개/850개 파일이다. 미검토 10,275개, 증거 대기 568개, 보정 확정 102개 함수 항목으로 전수 조사는 미완료다.

Gym differential 검사 1870줄·187개 함수(168개 root·19개 helper)와 직접 도구 1075줄을 읽었다. 현재 Python 168 PASS / 0 FAIL / 0 ERROR / 0 SKIP, exit 0이며 0.855초가 걸렸다. 187개 대상 함수의 실제 호출을 profile로 확인했다. 모든 분기의 완전한 검증을 주장하지 않는다.

실제 subprocess 시도는 0이고 실물 HWP/HWPX/PDF 읽기도 0이다. 임시 경로의 확장자 탐색용 파일 29개를 쓰기만 했다. CLI 응답은 주입됐으며 숫자는 항목 수·JSON 숫자·종료 코드·초/문자열 길이·해시 길이 계약이었다. 실제 페이지 px golden은 없었다. 이 단위 PASS를 전체 Gym 판별력이나 독립 한컴 출력 정합으로 승격하지 않는다.

원 PR #5228은 직접 merge되지 않았고 #5425 통합에 반영됐다. 현재 함수 정의의 출처는 9d3f1a7e/8a9ca3dd 두 커밋이며 현재 ancestry를 확인했다. 현재 Gym benchmark contracts workflow의 명시 모듈 경로와도 연결했다. 옛 문서의 139쌍 실측이나 과거 PASS는 현재 실행으로 쓰지 않았다.

추가 빌드·제품/테스트 변경·이름 변경은 0이고 기존 JS 변경을 보존했다. 109차는 실제 장부 갱신을 완료한 progress이며 이번에도 판정과 실행 증거를 추가했다. 미검토/보류 조사가 끝난 후 최종 이슈 갱신·issue_8232 명명 보정·실제 재검증을 진행한다.
