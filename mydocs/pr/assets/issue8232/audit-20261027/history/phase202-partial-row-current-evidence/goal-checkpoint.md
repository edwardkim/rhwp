---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase202-partial-row-current-evidence/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase202-partial-row-current-evidence/goal-checkpoint.md` · 원문 SHA-256: `2378172c36924902698d69a8d1824fb679994084f8d45287e012d342b4bce74e`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 202차 기록

#1133 일반 행의 부분 프레임 검사3개와 helper5개를 직접 대조했다. 신규 직접 점검7개와 기존 보류1개를 확정했다. 독립 Print4종·현재 Native/fresh WASM8조건24쪽은 최저98.04532%이며 실제 좌표를 고정한 여백 대조군의3쪽은 양쪽100%다. 대표 review/overlay12쌍과 핵심4개 확대 영역을 읽고 독립 선 좌표·내용 소속을 확인했다.

공개 JSON의 소수 한 자리 반올림으로 엄격한 source 최소 관계를 판정하지 않았다. suite027 한 개의 실제 준비 빌드5.274초 후 원 Rust검사3개를 변경 없이 실행해3PASS를 확인했다. WASM의 미반올림 Rust assertion을 직접 실행한 결과라고 주장하지 않는다. 원본 최소·같은 물리 상자·후속 행 연속성 관계와 자격 있는3쪽의±2px 좌표는 유지 대상이다. 보정 필요214개는 변하지 않는다.

직접 점검15186개, Rust회귀7842개/885파일, 보류540개다. 소스·검사 수정과 개명·외부 게시가 없고 전체 감사는 미완료다. 진단 과정의 잘못된 전체 쪽 텍스트 선택·반올림 최소 오인·출력 디렉터리 누락·페이지 표기 오류는 각각 INVALID로 분리했으며 제품 실패로 세지 않았다.
