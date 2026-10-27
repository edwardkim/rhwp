---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase109-equation-input-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase109-equation-input-contracts/goal-checkpoint.md` · 원문 SHA-256: `26a1bcb832d25928c5075080bfb1942ba645cc4dfe7c447b8d77382009096144`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 109차 기록

직접 검토 10,895개 함수, Rust 테스트 6,737개/850개 파일이다. 미검토 10,443개, 증거 대기 568개, 보정 확정 102개 함수 항목으로 전수 조사는 미완료다.

LaTeX 입력 모듈 867줄·61개 함수를 읽고 기존 19개 판정을 보존하며 남은 42개(테스트 34·helper 8)를 새 판정했다. 수식 피연산자·스타일 enum·Unicode 기호·첨자 소유는 실물 px 좌표가 아니었다. 기존 DoS의 20000·64+8·16도 입력 길이·깊이 상한·합성 글자 크기로 구분했다.

현재 parse/tokenize→EqNode 경로와 cfg(test) 모듈 연결을 확인했다. 새 판정 41개는 모듈 분리 전과 주석/공백 제외 AST token이 동일했다. 나머지 Binomial 검사는 c5e1c1b2에서 이미 피연산자 소유 관계로 보정된 범위이며 이번 신규 수정으로 세지 않았다. 과거 stage294의 50 PASS는 현재 실행 결과로 재인용하지 않았다. 추가 빌드·Rust 실행·제품/테스트 변경·이름 변경은 모두 0이다.

앞선 108차의 기존 Rust 9 PASS와 첫 줄 SVG 관측의 반례 및 미달 Native 첫 쪽 세 장은 별도 증거로 유지한다. 전수 조사와 미검증 증거 정리를 끝낸 후 최종 이슈 갱신·issue_8232 명명 보정·실제 재검증을 진행한다.
