---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase135-hwpx-section-attribute-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase135-hwpx-section-attribute-contracts/goal-checkpoint.md` · 원문 SHA-256: `4326ab9233fbb9c238287598bc1c53aa966931256c3d38d82eb8cca54fbc2b80`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 135차 기록

새 검사 22개와 helper 6개를 직접 판정했다. 누적 직접 점검 12,021개 함수·Rust 검사 7,166개이며 미점검 9,602개, 증적 대기 552개다. 보정 필요 125개 항목은 유지된다.

구역 XML 속성·CDATA·참조·자원 깊이의 원본 계약을 화면 px와 구분했다. Native 22 PASS를 재실행하거나 재빌드하지 않고 완료 로그와 컴파일 입력 동일성을 확인했다. 독립 XML 18건과 공개 import/export 6건·저장 속성 3건을 대조했다. 구역 parser의 깊이 거절과 전체 reader의 빈 구역 복구를 따로 기록했으며 CLI exit 0을 내용 보존으로 표현하지 않는다.

처음 파서 전용 표본에 info의 렌더링을 요청한 관측기 오류와 그 뒤 whole-document 비정상 종료를 잘못 기대한 관측기 오류를 정정했다. 실패 입력·로그는 보존했고 제품·기존 테스트를 바꾸지 않았다. 본문 파싱 계약에 불필요한 PNG·Print·빌드를 반복하지 않았다.

전수 조사는 미완료이며 최종 이슈 갱신·실제 테스트 보정과 issue_8232 새 명명은 아직 수행하지 않았다. 원본·PDF·공유 target·사용자 Studio JS를 보존했다.
