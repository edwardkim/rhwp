---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase197-score-regression-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase197-score-regression-contracts/goal-checkpoint.md` · 원문 SHA-256: `3f7101a13b1118f31f6fec312774dce6698a12be9a974b44836041025c9063fe`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 197차 기록

채점기 회귀 20개와 helper 7개를 직접 판정했다. Python 20 PASS/0 FAIL/0 skip이며 실제 외부 바이너리·네트워크·output 밖 쓰기는 없다. 새 빈 pycache 경로와 bytecode 쓰기 금지로 현재 소스를 적재했고 실제 읽은 970개 파일의 SHA 불변을 확인했다. 첫 실행의 driver json import 누락은 INVALID로 보존하고, 수정한 새 실행만 사용했다.

독립 대조에서 계획서 입력 제약을 제거해도 출처 보호 검사가 PASS하고, SHA helper를 잘못된 64자리 상수로 바꿔도 기대값까지 같이 바뀌어 기존 검사가 PASS하는 두 공백을 재현했다. 현재 제품 구현 결함으로 세지 않으며 보정 필요 누계는 213개다. 인덱스·행/열·exit·hash 길이는 화면 px가 아니다.

원 이슈와 원 PR·전체 논의, 실제 통합 SHA 및 후속 명령 계약 정정을 대조했다. 이 감사 결과를 실제 replay나 최종 문서 출력 검증으로 확대하지 않는다. 추가 빌드·제품/검사 수정·개명·외부 게시가 없으며 전수 감사는 미완료다.
