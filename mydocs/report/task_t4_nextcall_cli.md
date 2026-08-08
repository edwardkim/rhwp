---
kind: report
status: done
canonical: mydocs/report/task_t4_nextcall_cli.md
last_verified: 2026-08-08
---

# T4 실패 봉투 수복 힌트 일반화 — 설계 검증과 안전 조각 구현 (refs #4220, #3907)

동향 조사(#4220/#4221) §5 T4 의 착수 게이트를 실측으로 통과시키고, 기존 계약과
충돌하지 않는 **안전 조각만** 구현한 처리 기록이다. 판정의 원칙은 T4 원문
그대로다 — 다음 호출이 **결정론적으로** 정해지는 실패 부류에만 힌트를 싣고,
아니면 침묵한다(오제안 0, R72).

## 1. 설계 검증 — 기존 계약 실측

구현 전에 세 계약을 저장소에서 실측했다.

| 계약 | 실측 | 근거 |
|---|---|---|
| 실패 3면 계약 (#2707) | 조립 오류 = exit 2 + stderr 산문, 실행 실패 = exit 1 + **stdout 0 B** | `src/main.rs` EXIT_* 상수 주석, `tests/cli_exit_codes.rs` |
| stdout 0 B 고정 테스트 | `stdout.is_empty()` 단언이 시험군 전반에 수십 건 | `tests/` 전수 grep — batch/digest/edit/export/hidden-text 등 |
| stderr 검사 방식 | 전부 `contains` 기반, **전문 일치·줄 수 고정 없음** | `tests/` 전수 grep (`assert_eq` 류는 `render_p37` 의 두 호출 stderr 동일성뿐 — 같은 경로라 무영향) |
| `nextCall` 방출부 | `src/mcp_serve.rs` 뿐 (도구 오타 교정 1 + 닫힌 핸들 8사이트, R72) | grep 실측 — CLI 에는 없음 |

**판정**: stderr 는 추가 전용으로 확장 가능하다(기존 단언이 전부 `contains`).
stdout 은 한 바이트도 건드릴 수 없다. 따라서 힌트의 자리는 **exit 2 의 stderr
마지막 한 줄**이 유일하게 안전하다.

## 2. 실패 부류 분류 — 안전 조각 / 후속 구분

### 안전 조각 (이번 구현, 3부류)

| 부류 | 다음 호출이 결정론적인 이유 | 방출 |
|---|---|---|
| 미지 명령 + 확신 교정 | #3694 임계(레벤슈타인, 길이/3·clamp 1~3) **안**일 때만 — 기존 did-you-mean 산문이 이미 같은 확신 판정을 함 | `nextCall.name` = 교정 명령 |
| 명령 누락 | 발견 경로는 언제나 자기서술 입구 하나 — `capabilities` | `nextCall.name` = `capabilities` |
| 미지 inspect 하위 명령 + 확신 교정 | 위와 같은 임계, 기존 "혹시 이것인가요?" 산문의 형식화 | `nextCall.name` = `inspect`, `subcommand` = 교정 하위 |

셋 다 **기존 확신 장치가 이미 있는 자리의 형식화**라 새 판단 로직이 없고,
방출부는 중앙 디스패치 1곳 + `inspect_command` 1곳뿐이다(전 명령 일괄 개조 없음).

### 침묵 유지 (오제안 0 — 테스트로 고정)

- 임계 밖 오타(gibberish): 힌트 산문도 수복 줄도 없음 (기존 #3694 계약 유지).
- inspect 하위 명령 누락: 세 축 중 무엇을 원했는지 결정론적으로 알 수 없음.
- 실행 실패(exit 1): 인자를 고칠 것이 없는데 교정을 지어내면 재시도 래퍼가
  "내 호출이 틀렸다"로 오독한다(#2707 취지 그대로).

### 후속 (이번에 구현하지 않은 후보와 사유)

| 후보 | 판정 | 사유 |
|---|---|---|
| (c) `--json` 판정 봉투(exit 3)의 `nextCall` 필드 확장 | **후속** | 봉투 새 필드는 `capabilities_schema` 동반 + 스키마 버전 인상이 계약(#4114 교훈). stderr 한 줄과 달리 별도 설계·별도 PR 감이다. |
| 미지 옵션 교정 | **후속** | 옵션 파서가 명령별 수기 루프 수십 곳이고 옵션 사전이 중앙에 없다 — 결정론 교정을 하려면 등록부 신설이 선행돼야 하며, 그 전 개조는 "전 명령 일괄 개조" 금지에 걸린다. |
| 필수 인자 누락 | **후속** | 다음 호출이 "같은 명령을 옳게" 뿐이라 nextCall 어휘가 맞지 않다. usage 산문이 이미 처방이고, 형식화하려면 스키마 지시(예: export-*-schema 연계) 등 다른 어휘가 필요하다. |
| 미지 edit 하위 명령 | **후속** | 현재 did-you-mean 산문 자체가 없다 — 형식화가 아니라 행동 추가라 이번 범위 밖. 어휘가 착지한 뒤 1줄짜리 후속이다. |
| capabilities 자기서술 등재(수복 줄 문법) | **후속** | 봉투 필드 추가 = 스키마 동반(#4114). 이번엔 매뉴얼(`cli_commands.md` §종료 코드)에만 등재했다. |

## 3. 구현 — 정형 수복 줄 문법

```
수복: {"nextCall":{"name":"export-svg","why":"요청한 이름이 없음 — 가장 가까운 실존 명령으로 교정"}}
```

- **자리**: exit 2 stderr 의 **마지막 줄**, 산문(오류·힌트·사용법) 뒤. 소비자는
  "마지막 `수복: ` 줄 하나"만 파싱하면 된다.
- **어휘**: `nextCall{name, subcommand?, why}` — MCP 오류 봉투(R72,
  `tool_error_with_next`)와 같은 모양. CLI·MCP·(장차 R22 계획 거부)가 한 어휘가
  되는 것이 T4 의 요지다.
- **`arguments` 는 싣지 않는다**: 호출자의 나머지 argv 가 옳다고 검증한 바 없고
  (오제안 0 은 인자에도 적용), `--password` 류 민감 인자를 stderr 로 되울리지
  않는 뜻도 겸한다. `why` 도 고정 문자열이라 argv·문서 유래 문자열이 줄에 섞일
  경로가 없다(S6 경계 계약과 정합).
- **stdout 무침해**: 실패 3면 계약에 stderr 한 줄만 더하는 추가 전용 확장.

변경 파일: `src/main.rs`(방출 헬퍼 `eprint_usage_recovery` + 사이트 2곳),
`tests/nextcall_cli_contract.rs`(신규 계약 8건), `mydocs/manual/cli_commands.md`
(§종료 코드에 문법 등재).

## 4. 게이트 실측

| 게이트 | 결과 |
|---|---|
| red 실증 | 구현 stash 후 신규 계약 8건 중 **방출 4건 red** / 침묵·무회귀 4건 green(침묵은 자명 성립) → 복원 후 전건 green |
| 신규 계약 | `nextcall_cli_contract` 8/8 통과 — ① 문법(마지막 줄·단일 줄·JSON·name 실존 capabilities 대조) ② 오제안 0(불확실 3경로 침묵) ③ stdout 0 B |
| 영향권 기존 | did_you_mean 20 · cli_exit_codes 17 · boundary_integrity 4 · capabilities_subcommands 10 · cli_json 31 · capabilities_schema 15 · diagnostics_flag 3 · mcp_server 13 · envelope_integrity 5 · agent_toolkit 22 — **전건 통과** |
| clippy | `--all-targets -- -D warnings` 통과 |
| rustfmt | 변경 파일 Diff 0건 (`Incorrect newline style` 4건은 이 PC CRLF 체크아웃 함정 — CI(LF)와 무관, 기지 사항) |

## 5. 남긴 것

§2 의 후속 5건이 그대로 다음 착수 목록이다. 특히 (c) exit 3 봉투 확장은 이
줄의 어휘가 메인테이너 승인으로 착지한 뒤에 여는 것이 순서다 — 같은 어휘를
두 표면에 동시에 제안하면 한쪽 보정이 다른 쪽 드리프트가 된다.
