---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase203-cli-unknown-flag-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase203-cli-unknown-flag-contracts/goal-checkpoint.md` · 원문 SHA-256: `5fae2d860af14663f3bdfcbc45866d5ac77bd67927b5f39ec95ee5790e07c3e9`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 203차 기록

기존 #3884 G2 진단 CLI 검사11개와 helper5개를 직접 점검했다. 사용법 오류2·stdout0바이트·구역번호0은 px 좌표가 아니다. 독립 CLI 명세와 현재 자기서술·실제 handler를 대조했고 현재 Native CLI에서 같은11개 명령이 모두 원 predicate를 충족했다. Rust harness를 실행한 것으로 세지 않는다. 추가 빌드·시각 캡처·소스/검사 수정·개명이 없다.

직접 점검 누계15202개, Rust회귀 함수7853개/886파일이다. 보정 필요214개와 보류540개는 그대로이며 전체 전수 조사는 미완료다. 잘못된 capabilities --json 진단은 별도 INVALID로 보존했고 최종 실행과 구분했다.
