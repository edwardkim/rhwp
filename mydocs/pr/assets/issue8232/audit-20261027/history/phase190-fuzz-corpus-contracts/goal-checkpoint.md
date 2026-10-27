---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase190-fuzz-corpus-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase190-fuzz-corpus-contracts/goal-checkpoint.md` · 원문 SHA-256: `1a5196cdadb231a33fa80381bec6eed55e4a41eb60e0154aec932f90c2f870c5`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 190차 기록

퍼징 코퍼스 검사 78개와 helper 16개를 직접 판정했다. Python 78 PASS/0 FAIL/0 skip이다. 실제 부재 실행 파일 호출 한 번은 ENOENT로 끝났으며 성공한 외부 바이너리 실행·네트워크·output 밖 쓰기가 없다. 불러온 core pyc 네 개는 현재 소스 compile code와 동등함을 확인했다.

byte 오프셋·길이·시간·보고 건수를 페이지 px와 구분했다. 독립 byte 기대값 일곱 개와 오류 역할을 확인했다. 출력 부모 경로가 없는데 missing-bin으로 표시해도 기존 쓰기 검사가 통과하며, 이를 timeout으로 바꾼 진단에서도 같은 assertion으로 1 PASS다. 오류 역할 검출 누락 한 개를 보정 대상으로 추가해 누계 209개다.

원 PR #5270·실제 통합 #5425와 현재 byte 차이를 확인했다. 이후 문서 archives 경로와 #7861의 시험지 입력 제외 연결만 다르다. 추가 빌드·소스/테스트 수정·개명·외부 게시가 없으며 전수 감사는 미완료다.
