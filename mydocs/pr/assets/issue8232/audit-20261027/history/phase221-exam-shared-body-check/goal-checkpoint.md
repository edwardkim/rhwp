---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase221-exam-shared-body-check/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase221-exam-shared-body-check/goal-checkpoint.md` · 원문 SHA-256: `b2a78eccac0eb3949d3df9cb9076dc89d21e63a2b84930f73560491a5c78ea30`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 221차 기록

기존 시험지 공통 검사 check의 일반 본문 대응 보존 공백 한 건에 호출 함수 52개를 연결했다. 이 중 50개는 보류에서 보정 필요로 판정했고, 두 eq-002 함수는 직접 px 검사가 없다는 이전 결론만으로 공통 보완을 제외할 수 없어 보정 경로로 정정했다. 공통 원인을 52개 독립 결함으로 중복 집계하지 않는다.

동일 head의 Native/fresh WASM 104조건·1,566쪽과 기존 Native 52 PASS의 입력·PDF·글꼴·소스·실행 파일·로그 해시를 재확인했다. 좌표 대상 50조건의 선택 페이지는 양쪽 최저 98.57826%다. 대표 두 쪽을 직접 판독했으며 50개 전부를 새로 확대 판독하거나 실제 본문 누락 제품 결함을 재현했다는 뜻은 아니다. DTO 반례는 수집 범위를 설명하는 증거로만 사용했다.

증거 보류는 531→481개다. 보정 필요 함수 지점 수는 224→276개이며 증가분 52개는 기존 공통 원인의 영향 호출이다. 직접 점검 15,750개는 그대로다. 새 빌드·실행·캡처·소스 수정·개명·외부 게시 없이 감사 대장만 갱신했으며, 전체 감사와 실제 보정은 미완료다.
