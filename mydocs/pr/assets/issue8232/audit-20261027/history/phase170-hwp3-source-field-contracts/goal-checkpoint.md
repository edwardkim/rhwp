---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase170-hwp3-source-field-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase170-hwp3-source-field-contracts/goal-checkpoint.md` · 원문 SHA-256: `3aef2802c7ec4815aa559610a3f1c916ca5c351e065500bae7132ca08a306cb6`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사170차 기록

HWP3 원본 속성·손상 입력 검사9개를 직접 판정했고 현재 동일 소스 Native9 PASS다. bit·byte·bool·shade 비율·BorderFill 종류를 실물 page px와 구분했다. 한컴 변환본의 실제 문단 border 참조39개를 독립 XML로 읽어 모두 네 변NONE임을 확인했다. 이 속성 대응을 전체 문서나 장식선 paint의 시각 검증으로 확대하지 않는다.

손상 입력 crash smoke와 정상 parser smoke는 적절한 목적의 검사이며 assertion 개수만으로 부적합하게 분류하지 않는다. release-test 실행을 Debug overflow FAIL-before나 WASM32 입증으로 바꾸지 않는다. 새 보정 항목은 없고 누계202개다. 추가 빌드·제품/테스트 수정·개명 없음. 전체 감사는 미완료이며166차 전쪽 수집은 별도 실행 중이다. 현재 원장을 덮어쓰지 않고 그 결과만 나중에 병합한다.
