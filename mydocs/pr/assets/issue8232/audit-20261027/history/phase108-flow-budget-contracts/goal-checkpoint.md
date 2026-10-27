---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase108-flow-budget-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase108-flow-budget-contracts/goal-checkpoint.md` · 원문 SHA-256: `3f66d26193d4ba6a7394d1a6c660126f108679cb7f559acb8cc127cf28ebf917`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 108차 기록

직접 검토 10,853개 함수, Rust 테스트 6,703개/850개 파일이다. 미검토 10,477개, 증거 대기 568개, 보정 확정 102개 함수 항목으로 전수 조사는 미완료다.

세 검사 모듈 602줄·32개 함수(12개 root·20개 helper)를 검토했다. 출처가 검증된 동일 HEAD suite023을 재사용해 9 PASS / 0 FAIL / 0 IGNORE다. 현재 suite025의 보유 실행 근거가 없는 6102의 세 root는 새 Cargo 빌드를 하지 않고 미실행으로 구분했다. 제품 빌드는 0이며 std-only 관측 helper 반례만 rustc로 1회 컴파일했다.

첫 줄 검사의 raw x/고정 y 대역 관측은 defs 비표시·조상 translate·글리프 오른쪽 경계 초과를 놓친다. 원래 helper를 바이트 그대로 추출한 실제 반례 실행으로 한 공통 원인과 영향 함수 5개를 확정했다. 실제 제품에서 이 mutant가 발생했다거나 원래 DocumentCore 검사 세 건이 통과했다고 확대하지 않는다.

현재 Native 첫 쪽은 초과근무 88.69514%, 차량 검사비 74.57361%, 교통법규 소명자료 72.03226%다. 정본 review와 standalone overlay 세 쌍을 직접 판독했고 문자 전진·굵기·본문/표의 paint 차이가 남는다. 모두 90% 미만 gate로 종료됐으며 캡처 실패나 전체 문서 완료가 아니다. 이전 RGB 일치율과 현재 raw 값을 직접 빼서 회귀율로 쓰지 않았다.

표의 1000HU 줄 높이·141+141HU 여백과 실제 본문 포함, 16px 합성 예산(15.2133<16<17.0933), 앵커 줄과 표의 실제 상대 순서는 유효한 source/경계 계약으로 구분했다. 기존 최소 높이 282HU 입력과 삭제된 비정상 패딩 판별 검사는 혼동하지 않았다.

소스/테스트 수정·이름 변경은 0이며 기존 JS 변경을 보존했다. 직전 상태 보고 턴은 no progress로 분류하고 이번 실제 증거 수집·장부 갱신으로 작업을 진전시켰다. 미검토/보류 조사를 완료한 후 최종 이슈 갱신과 issue_8232 명명의 필요한 보정·실제 재검증을 수행한다.
