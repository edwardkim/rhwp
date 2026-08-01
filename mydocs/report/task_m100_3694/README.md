---
kind: report
status: active
canonical: mydocs/report/task_m100_3694/README.md
last_verified: 2026-08-02
---

# #3694 처리 기록 — did-you-mean (내성 P1, #3630 1호)

## 구현

- CLI: 알 수 없는 명령 exit 2 stderr 에 `힌트: 가장 가까운 명령은 '…' 입니다` 1줄.
  후보는 **capabilities 명령 목록 단일 출처** — `show_capabilities` 의 commands vec 을
  `capabilities_command_entries()` 로 승격해 자기서술과 힌트가 같은 원천을 쓴다.
- 판정: 의존성 없는 소형 레벤슈타인 + 임계(길이/3, 1~3) 초과 시 무제안 — **오제안 0
  원칙**(엉뚱한 제안은 경량 에이전트를 더 깊은 루프로 밀어넣는다).
- MCP: 미지 도구 isError 텍스트를 `{"error":"<기존 원문>","didYouMean":[…]}` 로 승격 —
  error 필드가 원문을 담아 하위호환, 후보는 tool_defs 실존 도구만.

## 실측 (evidence.txt 원문 동봉)

- `rhwp exprot-svg` → exit 2 + `힌트: 가장 가까운 명령은 'export-svg' 입니다`
- MCP `hwp_serch` → `{"error":"알 수 없는 도구: hwp_serch","didYouMean":["hwp_search"]}`

## 검증

- 신규 `did_you_mean_contract` 3건 green (오타 힌트+exit 2 유지 / 헛소리 무제안 /
  MCP 구조화·하위호환), `cli_json_contract` 22건·`mcp_server_contract` 6건 무회귀,
  clippy 0, rustfmt clean. commands vec 승격은 동작 무변경(자기서술 22건이 가드).
