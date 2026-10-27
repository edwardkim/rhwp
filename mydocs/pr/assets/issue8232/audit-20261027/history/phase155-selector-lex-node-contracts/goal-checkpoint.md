---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase155-selector-lex-node-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase155-selector-lex-node-contracts/goal-checkpoint.md` · 원문 SHA-256: `f1869e22c88c64cb78153946b98077a482d8a108593780e08b8f0a0a39e12895`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 155차 기록

선택자 lexer 10개·Node 7개·평가기 9개 및 helper 8개를 직접 판정했다. 기존 동일 소스 Native 26 PASS이며 추가 빌드는 없다. 문자 offset·UTF-16·트리 소속·노드 상한을 페이지 px 핀과 구분했다.

서로게이트 결합이라는 검사는 실제로 literal emoji만 입력하여 Unicode escape 결합 분기에 도달하지 않는다. 독립 JSON decoder에서는 literal과 두 escape가 동일 문자로 복원되지만 두 입력 경로는 다르다. 기존 검사의 입력 보완을 보정 대상으로 추가했으며 제품 결함이나 mutant 검출의 실제 실행으로 주장하지 않는다.

보정 필요 195개 함수 항목이다. 전수 조사·최종 이슈 갱신·소스 보정·개명은 미완료다.
