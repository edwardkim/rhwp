---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase201-endnote-nominal-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase201-endnote-nominal-contracts/goal-checkpoint.md` · 원문 SHA-256: `7c03f2f23473984ffe819da207f2cbb1a998fa357cb11aa80f06949678dcbf1d`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 201차 기록

미주 커서의 보류 11건과 정상 대조군 5건을 기존 동일 소스 바이너리로 실행해 Native 16 PASS를 확인했다. 추가 빌드가 없다. 원 설계의 폴백 10px·완충 40/18px·soft backtrack 10px는 합성 커서 단위 정책이며 실제 문서의 절대 위치 핀과 구분해 5건의 보류를 해소했다.

특히 #1261의 실제 20mm 문서에서 10px 겹침 방지값을 쓰던 첫 보정은 폐기된 중간 단계다. 후속 공통 간격 보정과 독립 PDF 관측이 원 기록에 있으며, 현재 10px unit은 간격 미제공 폴백만 보호한다. 5669HU 명시 간격 대조군도 별도로 통과했다.

이 결과는 현재 실물 페이지의 직접 시각 완료나 px 핀 자격 판정이 아니다. 6800HU 앵커 등 이 커서의 보류 6건은 남겼다. 전체 추가 근거 보류는 541개, 보정 필요 누계는 214개, 직접 점검 누계는 15179개다. 소스·검사 수정, 개명, 외부 게시가 없으며 전체 감사는 미완료다.
