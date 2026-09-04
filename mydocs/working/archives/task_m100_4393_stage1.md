# Task M100 #4393 Stage 1 — 에이전트 노동 감사: 작업 캡슐 + audit (적층: #4391)

- 이슈: #4393 · 브랜치 task_m100_4393(fork/task_m100_4391 적층) · 2026-08-10 KST

## 왜 2년 선행인가
에이전트 실작업이 조직 규모가 되면 개별 영수증으로 부족하다 — 수천 건의
작업을 일괄 재검증해 "재현율"을 회계하는 감사 층이 필요해진다. 영수증(#4391)
→ 교환 캡슐 → 조직 감사의 3단 완결.

## 구현
- replay 실행 코어를 replay_execute_to_temp 로 추출(중복 금지 — replay·audit
  공용, 임시 산출·정리 일원화). replay 실패 봉투는 {error} 로 단순화(리팩터).
- `replay --capsule <path>`: 계획(원본 output 보존)+영수증의 자기완결
  workCapsule 발급.
- `audit <dir>`: *.capsule.json 전수(정렬·비재귀) 재실행·대조 →
  {root,total,reproduced,failed[],reproducedRate} 회계. 불일치=exit 3,
  빈 폴더=exit 2+stdout 0(실패 순수성), 캡슐별 실패 사유(변조=기대/실제,
  파손=error) 명세.
- 등재 4종: capabilities(audit + replay flags --capsule)·MCP hwp_audit
  (품질검증 프로필)·help·Node 봉투 38→39 재생성(gen:check 최신).

## 검증
- audit_contract 2/2 첫 실행: 캡슐 2건 발급→감사 전건 재현(rate 1.0)→1건
  변조→exit 3·failed[b] 기대/실제 64자리·빈 폴더 exit 2+무출력.
- replay_contract 4/4 무회귀(리팩터 후). 가드: cli_json 31·router 8·mcp 24·
  clippy 0·fmt.
