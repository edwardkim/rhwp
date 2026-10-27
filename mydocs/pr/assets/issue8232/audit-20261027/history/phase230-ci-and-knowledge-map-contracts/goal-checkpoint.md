---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase230-ci-and-knowledge-map-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase230-ci-and-knowledge-map-contracts/goal-checkpoint.md` · 원문 SHA-256: `16a44f514f6e47f4da3d40f47ece7abbc608591fdfb58c1f69b1450b317deba3`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 230차 기록

CI 배선·지식 지도 두 파일의 기존 검사 65개와 같은 파일의 helper 32개를 함수별로 판정했다. 원 검사 65개를 실제 한 번 실행했고 62 PASS / 3 FAIL / 0 skip이다. 세 실패는 지도 날짜·과거 발췌·이웃 스킬 부재 기대의 현재 비시각 계약 실패이며, px 위치 검사나 제품 조판 회귀로 세지 않는다. 현행 #8232 범위를 live 조회해 확인했고 코드나 기대값을 바꾸지 않았다.

검사 본문과 helper는 각각 1,215줄과 426줄 전체를 읽었다. 생성기는 import한 상수 4개만 사용하므로 emit/main 실행이나 전체 재생성 수용으로 확대하지 않는다. 실제 자식 프로세스는 로컬 Bash 상태 집계 15개이며 Cargo·Native·시각 캡처·원격 CI는 없다. 최초 분석기에서 helper 32개를 빠뜨린 선택 수 assertion이 실제 검사 전에 실패한 0건 기록도 보존했다. 이 오류를 세 개의 실제 기존 FAIL과 혼동하지 않는다.

직접 검토는 15,750→15,847개(새 검사 65개·helper 32개), 미검토는 6,555→6,490개다. 기존 증거 보류 417개와 보정 함수 지점 324개는 유지한다. helper 32개를 호출 경로 원장에 연결하여 전체 항목은 22,558→22,590개다. 새 빌드·소스 수정·개명·외부 게시는 0이며 전체 감사는 진행 중이다.
