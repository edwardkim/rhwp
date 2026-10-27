---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase224-active-footnote-coverage/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase224-active-footnote-coverage/goal-checkpoint.md` · 원문 SHA-256: `217841132bff12c6bc9cbd9af5617804f95ed42deab5619d7e7b9eaac45575e0`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 224차 기록

현재 활성 각주 검사 세 개는 동일 head에서 빌드한 기존 suite026 바이너리로3 PASS다. 원본 HWPX XML의 각주107/108/116은 각각 한 개이며 원 instId도 구분했다. 그러나 원 검사의 any/contains·>0인 쪽 목록은 같은 쪽의 marker/몸통/표 조각 중복 수를 버린다. 추상 관측 모델에서 같은 쪽 중복과 범위 밖 추가 소유가 그 관측을 바꾸지 않는 반례를 기록했다. 원 Rust root에 결함을 주입한 실행이나 실제 제품 중복 재현으로 표현하지 않는다.

앞 조각에서 아직 소비하지 않은 각주 표시 금지, 실제 표/후속 글줄과 footer 영역의 비침범 관계는 유지해야 한다. 이를 절대 px 핀 문제와 섞어0.5px 포함 여유를 제거하지 않는다. 보정 때 실제 소유별 전쪽 개수/순서·누락/중복과 렌더 오류 검출을 추가할 대상이다.

이 모듈의 보류26개는 미사용23개/활성3개로 판정을 마쳤다. 전체 증거 보류는453→450개, 보정·정리 함수 지점은297→300개다. 이번 turn 누적 보류26개를 판정했으며 직접 검토15,750개는 중복 증가시키지 않는다. 새 빌드·캡처·소스 수정·개명·외부 게시 없이 전체 감사는 진행 중이다.
