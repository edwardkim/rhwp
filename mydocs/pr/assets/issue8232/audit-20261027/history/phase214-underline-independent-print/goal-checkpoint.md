---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase214-underline-independent-print/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase214-underline-independent-print/goal-checkpoint.md` · 원문 SHA-256: `807467c7d068dd1fa2a49a263979bd91330ad226860c0b940afa80720adbe6ed`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사214차 기록

원 밑줄 프로브를 MCP2020/Print0으로 독립 인쇄하고6크기의 text matrix·밑줄 path를 직접 읽었다.9pt는 약0.160em이며 모든 크기가0.167~0.170인 것은 아니다.6개 모두 기존0.17em 근사 허용 범위 안에 있고 현재 Native/fresh WASM은 각각1쪽100%다. review/overlay2쌍과 확대2개를 직접 읽고 원 단위 검사·원 프로브 통합 검사2PASS를 확인했다. 기본 SVG1px 선과 실제 한컴 font별 밑줄 굵기는 구분했다.

보류535→533개, 직접 점검15514개·Rust7882개/888파일, 보정 필요218개다. 추가 빌드0·소스/검사 수정0·개명0·외부 게시0이다. 전체 감사는 미완료이며 다중 밑줄·원 코퍼스 문서 전체·Studio/CanvasKit 수용으로 확대하지 않는다.
