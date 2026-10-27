---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase195-bindata-storage-contract/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase195-bindata-storage-contract/goal-checkpoint.md` · 원문 SHA-256: `36d170ca048844fc31aa74c06973b14b8187469046c2dadf39e84254ef8a3d7f`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 195차 기록

그림 저장 ID 충돌의 원 이슈 #2038·원 PR #2040·전체 논의·실제 merge 및 후속 공통 등록 경로를 대조했다. 두 Native 검사가 같은 현재 소스의 기존 바이너리에서 PASS했다. 저장 왕복 root 한 개와 helper 두 개를 새로 판정했고 이미 판정한 ID 유일성 대조 검사는 중복 집계하지 않았다.

9000·6000은 작성 HU 크기이며 1·1은 자연 이미지 크기 입력이다. 화면의 절대 px 핀이 아니므로 이 감사에서 불필요하게 변경하거나 개명하지 않는다. 기존 이미지 3바이트는 데이터 보존용 합성 marker이며 실제 PNG 출력 검증으로 승격하지 않았다. 원본·제품·검사 수정, 추가 빌드, 외부 게시가 없다. 전수 조사는 미완료다.
