---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase106-host-table-source-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase106-host-table-source-contracts/goal-checkpoint.md` · 원문 SHA-256: `c41d5f51e708b20eaeba30cd18ff874a9bc862e01d9351bab69cd5e2b9d945a0`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 106차 기록

직접 검토10,806개 함수, Rust 테스트6,684개/847개 파일이다. 미검토10,497개, 증거 대기567개, 보정 필요 확정95개로 전수 조사는 미완료다.

표 관련 두 파일의 전체24개 함수와 원본/독립 PDF를 조사했다. 기존 동일 HEAD 바이너리로10PASS/0FAIL/0IGNORE, 추가 빌드0이다. #2097 본문 아래끝1028.04/1009.12는 실제 HWP5/HWPX 용지·여백의 source 관계다. 독립 Print의 큰/중간 행과 tail/뒤 본문 소유도 일치했다.

#7418의70833 p12 Native raw92.23473에서 행 y와 첫 줄 x를 고정하는2개 검사를 보정 대상으로 확정했다. p13은98.78454,78494 p20은98.78040이지만 fresh WASM/전체 완료 근거는 없으므로2개 caller를 보류했다. review/overlay3쌍을 직접 판독했다.

독립 재저장 midpage 검사1개는 BTreeSet 비교가 같은 행 중복을 지우는 검출 한계를 확인했다. source predicate의 논리 반례이며 제품에 실제 중복이 생겼다거나 mutant Rust를 실행했다고 주장하지 않는다. 의미 관계·내용 소유·원본 cardinality 보강 계획을 기록했다.

이번 단계의 소스/테스트 수정·이름 변경은0이며 기존 JS 변경을 보존했다. 미검토/보류 조사를 마친 뒤 최종 이슈 갱신과 issue_8232 이름의 필요한 보정·실제 재검증을 수행한다.
