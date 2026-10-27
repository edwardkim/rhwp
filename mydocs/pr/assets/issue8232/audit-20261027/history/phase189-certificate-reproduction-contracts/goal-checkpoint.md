---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase189-certificate-reproduction-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase189-certificate-reproduction-contracts/goal-checkpoint.md` · 원문 SHA-256: `30578cfa06abecde192549833e41d8f87a9542a74d8bc0d2a17e9490b8bbb97b`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 189차 기록

인증서 검사 71개와 helper 13개를 직접 판정했다. Python 71 PASS/0 FAIL/0 skip이며 실제 report.py·rhwp 채점은 실행하지 않았다. SHA 문자열 길이·점수 백분율·종료 코드는 페이지 px와 구분했다. 독립 hashlib로 경로+NUL+내용 SHA-256의 누적 지문과 실제 임시 트리의 결과를 대조하고 재현 core 다섯 필드·변동 메타 제외·미가용 집합을 확인했다.

원 PR #5286·통합 #5429의 Git blob과 현재 파일을 대조했다. 테스트는 원본과 같고 제품은 모듈 문서의 archives 경로 한 줄만 다르다. 84개를 새로 판정했고 새 수정 대상은 0개, 누계 208개다. 추가 빌드·소스/테스트 수정·개명·GitHub 게시가 없으며 전수 감사는 미완료다.
