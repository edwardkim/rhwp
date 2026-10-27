---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase103-wasm-binding-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase103-wasm-binding-contracts/goal-checkpoint.md` · 원문 SHA-256: `adbaaebd8bc0325803d92781c5f1c2f9061f1e1c21e6209e23a7e49457c18189`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 103차 기록

직접 검토 10,528개 함수, Rust 테스트 6,622개/844개 파일이다. 미검토 그룹 10,718개, 증거 대기 515개, 보정 필요 확정 항목 92개로 전수 조사는 미완료다.

WASM API 검사 구간의 43개 함수 중 신규 32개를 판정하고 기존 11개를 재검토했다. 논리 주소/flag/작성 입력과 반환 동등성은 실물 페이지 px 핀과 구분했다. HML·JSON·PNG를 독립 관측했고 PNG helper 두 개의 동일 bytes에서 잘못된 IDAT CRC를 확인해 입력 보정 대상으로 추가했다. 기존 PNG의 1×1 의미와 API 인수 검출 목적은 유지한다.

현재 제품 Rust 검사나 fresh WASM을 실행하지 않았고, 보유 metadata에 lib 실행 파일이 없어 빌드를 추가하지 않았다. 실제 원본의 모든 출력/메타데이터 정확도를 이 정적 판정으로 수용하지 않는다. 제품/테스트 소스 변경과 이름 변경은 아직 0이다.

미검토·보류 항목의 조사를 마친 뒤 최종 이슈 갱신과 필요한 보정·issue_8232 이름 변경을 수행한다.
