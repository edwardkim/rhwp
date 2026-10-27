---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase138-wasm-large-diagnostic-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase138-wasm-large-diagnostic-contracts/goal-checkpoint.md` · 원문 SHA-256: `ab7e3d371f62a421540ec536e14be55c922cc21d36d3b017db7d69a4547d9cb2`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 138차 기록

대형 wasm_api 검사 27개(본문 합계 10,252줄)의 전체 AST와 단언/실패/파일/호출 경로를 추출하고 중요 분기를 직접 대조했다. 전체를 수동 판독했다고 표현하지 않는다. helper 32개는 호출 문맥과 AST를 보존했지만 개별 의미 판정은 미완료 대장에 추가했으며 직접 점검 완료 수에 넣지 않았다.

같은 컴파일 소스 Native 바이너리로 27 PASS를 관측했다. 그러나 19개는 필수 입력 부재, 2개는 비교본 일부 부재, 4개는 출력 전용 진단이었다. 나머지 2개는 assertion이 실행됐지만 필수 fixture의 조용한 반환 또는 마지막 hit-test 대상/단언 공백이 남는다. PASS를 27개 결함 검출 성공으로 보고하지 않는다.

중첩 API는 2단계 target/pageIndex 1을 선택한 뒤 page 0/(400,600)을 클릭해 다른 1단계 경로를 받아도 성공 표시를 냈다. 기존 7개 API 단언을 보존하고 실제 대상·쪽·전체 경로 관계를 검증하도록 보정할 계획이다. 27개 root의 보정 필요를 추가해 누적 154개 함수 항목이 됐다. 이는 154개 제품 결함이라는 뜻이 아니다.

원본에서 존재하는 입력 10개는 SHA를 확인해 소유된 cwd에 복사했다. 원본에서 없는 입력은 새로 채우지 않았다. 생성된 7,680바이트 empty 저장본 1개도 감사 cwd 안에만 보존했다. 추가 빌드·제품/test 수정·개명·최종 이슈 갱신은 없다.

누적 직접 점검 12,214개 함수·Rust 검사 7,327개, 미점검 9,441개·증적 대기 573개·함수 의미 추가 점검 문맥 195개다. 전수 조사는 미완료이며 승인된 목표를 계속 유지한다.
