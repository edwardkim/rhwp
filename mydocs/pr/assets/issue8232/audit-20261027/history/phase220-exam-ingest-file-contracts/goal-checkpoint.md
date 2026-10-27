---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase220-exam-ingest-file-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase220-exam-ingest-file-contracts/goal-checkpoint.md` · 원문 SHA-256: `32781262e1e0d279a4f4a7b4d42c0e140e44e4eb2bd300f614f33327a4a46164`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 220차 기록

시험지 ingest 검사 549줄 전체의 함수 39개(검사 36개·준비 helper 3개)를 판정했다. 정상 36개는 모두 PASS다. 실제 DOCX helper는 없는 입력·인자 부족 두 오류 경로만 output cwd에서 실행해 exit 1과 JSON 코드를 확인했다.

정본 Question.number는 정수·최소 1이지만, 정상 fixture 번호를 메모리에서 1.5로 바꾸어도 단일 root와 전체 36개가 PASS했다. >=1만 확인하는 형식 검출 공백 한 건이다. 이미지 입력 bbox·DPI의 정수/범위 계약은 실물 문서의 화면 px 핀으로 분류하지 않았다.

이번 연속 작업에서 새 함수 79개와 정상 Python 검사 73개를 점검했다. 현재 직접 점검 15,750개·Rust 테스트 7,882개/888개 파일, 미검토 6,555개, 증거 보류 531개, 보정 필요 224개다. 추가 Cargo/Rust 빌드·소스/검사 수정·개명·외부 게시 모두 0건이며 전체 감사는 미완료다.
