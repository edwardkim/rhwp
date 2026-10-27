---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase143-re-sample-generator-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase143-re-sample-generator-contracts/goal-checkpoint.md` · 원문 SHA-256: `29e763b3ca50c5f915032a15cf2a50353fa2ecce03f092f29754dd3aea76566d`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 143차 기록

등록된 생성/진단 13개와 helper 7개를 판정했다. 원본과 분리한 cwd에서 각각 실행해 출력 경합을 피했고 템플릿 결측 대조 1회도 별도로 확인했다. 14회 libtest PASS는 생성/진단 실행 상태이며 의미 회귀 또는 시각 검증 PASS가 아니다. 추가 빌드는 없다.

생성 HWP 102개를 독립 CFB/Deflate/record 판독했다. 글꼴별 자료 56개 중 일반/empty를 포함한 12개에서 요청한 family가 없다. 굴림·굴림체·돋움체 및 맑은 고딕/Courier New 요청의 fallback이 기본 템플릿만 사용한 결과다. 두 root와 공통 선택 helper 1개를 추가 보정 대상으로 확정했다. 명시적 수동 생성기라는 이유만으로 다른 모든 root를 잘못된 px 회귀로 분류하지 않았다.

누적 직접 점검 12,495개, Rust 등록 함수 7,384개, 보정 필요 162개 함수 항목이다. 미점검 9,234개가 남으며 전수 조사·최종 이슈 갱신·소스 보정·개명은 미완료다. 사용자 JS·원본·공유 target은 보존했다.
