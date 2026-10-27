---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase223-dormant-footnote-helpers/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase223-dormant-footnote-helpers/goal-checkpoint.md` · 원문 SHA-256: `3e0fc658f7194a6bd163c76652d78dd4305deacd6608d099b97ded89df084323`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 223차 기록

분할 표·각주 모듈의 현재 private 함수 61개와 test root 14개를 Rust 스코프로 다시 연결했다. 34개 도달/27개 미도달이며 이전의 동명 walk·collect·images 네 노드 과대 연결을 정정했다. 보류 중 미도달 23개는 모두 제거 전 동일 함수 본문과 일치하고, 과거 호출 root는 기존 제거 목록 78개에 속한다.

현재 호출되지 않는 좌표 단언 helper 17개는 기존 삭제를 유지하며 잔재 정리할 대상으로 기록했다. 자체 단언 없는 선택 helper 6개는 별도 px 결함으로 세지 않았다. 현재 active root 세 건의 범위/검출력 보류는 유지했다. 원본·PDF 네 파일과 활성 14개 검사는 보존했고, 과거 점수나 독립 PDF 좌표를 현재 head의 시각 통과 증거로 사용하지 않았다.

보류는 476→453개다. 보정·정리 함수 지점은 280→297개이며 이번 증가 17개는 비활성 소스 잔재다. 직접 검토 15,750개는 중복 증가시키지 않는다. 새 빌드·테스트 실행·캡처·소스 수정·개명·외부 게시 없이 감사 대장만 갱신했고 전체 감사는 미완료다.
