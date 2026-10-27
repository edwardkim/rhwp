---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase157-cfb-ingest-binary-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase157-cfb-ingest-binary-contracts/goal-checkpoint.md` · 원문 SHA-256: `a561ac86087d17a3844c1f1e24bd5bc9ccdbc61c232f9c12cf76cf36553521e9`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 157차 기록

CFB/압축/ingest 검사 25개와 helper 2개를 판정했다. 기존 동일 소스 Native 25 PASS이며 추가 빌드는 없다. byte·sector·주소·경로·문법 계약을 실물 페이지 px 핀과 구분했다.

CFB v4 CLSID 검사는 major=0·header padding 결측 fixture에 현재 source 식과 같은 잘못된 위치를 적어 PASS한다. Microsoft header 사양과 별도 OLE reader로 정상 v3/v4 파일을 직접 읽어 이 기대값 오류를 확인했다. 새 정상 fixture로 제품 helper를 직접 실행한 FAIL-before 및 보정은 아직 미완료다.

보정 필요는 196개 함수 항목이며 제품 결함 수가 아니다. 같은 파일의 실제 편집 roundtrip 2개와 diagnostic 4개는 명시적으로 남겼다. 전수 조사·최종 이슈 갱신·소스 보정·개명은 미완료다.
