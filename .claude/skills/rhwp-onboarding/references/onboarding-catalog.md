# 온보딩 카탈로그 — 명령·예외·호스트 교차표

에이전트가 추측하지 않도록, 닥터와 레시피가 실제로 인용하는 표면만 표로 모은다.
새 명령을 추가하는 표가 아니다.

## A. 읽기 전용 명령 × 실패 신호

| 명령 | exit 0 오해 | 진짜 게이트 | 온보딩 단계 |
|---|---|---|---|
| `info --json` | 없음. 키 없으면 실패 | `format`,`pageCount` | 자가검증·트리아지 |
| `export-text --json` | 빈 pages | `pages` 길이 ≥ 1 | 자가검증 |
| `explain --json` | 구버전 없음 | 없으면 SKIP | 트리아지 |
| `digest --json` | excerpt 를 전체로 | `truncated` | 트리아지 |
| `export-tables --json` | tableCount 0 을 오류 | 0 이면 축 포기 | 표 |
| `table-to-csv --json` | 병합 표 왕복 | span 먼저 | 표 |
| `fields --json` | fieldCount 0 을 고장 | 0 이면 축 포기 | 서식 |
| `inspect hidden-text` | 신호 = 실패 | `clean` | 보안 |
| `inspect injection` | 신호 = 실패 | `clean`,`signalCount` | 보안 |
| `inspect unicode` | 신호 = 실패 | `clean` | 보안 |
| `search --json` | 0건 = 실패 | `matchCount` | 트리아지 확장 |
| `capabilities --mcp` | 도구 수 고정 | 손에 든 바이너리 | 부착 |
| `mcp-serve` | 포트 필요 | stdio | 부착 |
| `replay --plan-json` | 해시 위조 | 기존 계획 스키마 | 영수증 입구 |

## B. 예외 × 종료 코드 × 다음 문서

| kind | 기본 exit | 임계 | 문서 |
|---|---:|---|---|
| missing_binary | 3 | 예 (바이너리 없음) | exception-missing-binary.md |
| bad_sample | 1 | 예 (자가검증) | exception-bad-sample.md |
| no_network | 0 가능 | 아니오 | exception-no-network.md |
| write_exists | 2 | 사용법 | mcp-json-paste.md |
| selftest_timeout | 1 | 해당 검사 | sample-selftest.md |
| selftest_parse | 1 | 해당 검사 | sample-selftest.md |

## C. 호스트 × 스니펫 필드

| host | command 키 | args 키 | type 필드 |
|---|---|---|---|
| claude-code | mcpServers.rhwp.command | mcpServers.rhwp.args | 없음 |
| claude-desktop | mcpServers.rhwp.command | mcpServers.rhwp.args | 없음 |
| cursor | mcpServers.rhwp.command | mcpServers.rhwp.args | 없음 |
| cline | mcpServers.rhwp.command | mcpServers.rhwp.args | 없음 |
| windsurf | mcpServers.rhwp.command | mcpServers.rhwp.args | 없음 |
| vscode | servers.rhwp.command | servers.rhwp.args | stdio |
| gemini-cli | mcpServers.rhwp.command | mcpServers.rhwp.args | 없음 |
| qwen-code | mcpServers.rhwp.command | mcpServers.rhwp.args | 없음 |
| roo | mcpServers.rhwp.command | mcpServers.rhwp.args | 없음 |
| kilo | mcpServers.rhwp.command | mcpServers.rhwp.args | 없음 |
| kiro | mcpServers.rhwp.command | mcpServers.rhwp.args | 없음 |
| amazon-q | mcpServers.rhwp.command | mcpServers.rhwp.args | 없음 |
| zed | context_servers.rhwp.command.path | context_servers.rhwp.command.args | 없음 |
| goose | rhwp.cmd | rhwp.args | stdio |
| continue | mcpServers[0].command | mcpServers[0].args | 없음 |

## D. 샘플 후보 × 용도

| 경로 | 자가검증 | 트리아지 | 표 | 서식 | 보안 |
|---|---|---|---|---|---|
| samples/basic/english.hwp | 1순위 | 예 | 표 없으면 축 포기 | fieldCount 0 가능 | 음성 |
| samples/basic/KTX.hwp | 2순위 | 예 | 가능 | 조사만 | 음성 |
| samples/basic/BookReview.hwp | 3순위 | 예 | 가능 | 조사만 | 음성 |
| samples/hwp_table_test.hwp | 아님(후보 목록 밖) | 가능 | 1순위 | 아님 | 음성 |
| samples/form-01.hwp | 아님 | 가능 | 아님 | 1순위 | 음성 |
| samples/field-01.hwp | 아님 | 가능 | 아님 | 가능 | 입구 |
| fixtures/samples/* | 실패 경로만 | 금지 | 금지 | 금지 | 금지 |
| gym/** | 금지 | 금지 | 금지 | 금지 | 금지 |

## E. 닥터 CLI 플래그

| 플래그 | 기본 | 효과 |
|---|---|---|
| `--json` | off | stdout=리포트 JSON, 사람용은 stderr |
| `--write PATH` | 없음 | A형 스니펫 기록 |
| `--force` | off | 기존 파일 덮어쓰기 |
| `--rhwp PATH` | 탐색 | 이 파일만 |
| `--sample PATH` | 후보 | 이 파일(존재 시) |
| `--repo-root PATH` | 스크립트 유도 | 워크트리 고정 |
| `--offline` | off | 프로브 생략 |
| `--skip-selftest` | off | info/export-text 생략 |
| `--skip-extra` | off | explain/digest/inspect 생략 |
| `--host NAME` | 없음 | mcpHost 리포트 |
| `--list-hosts` | — | 호스트 id JSON, exit 0 |
| `--list-recipes` | — | 레시피 실존 JSON, exit 0 |

## F. 관련 정본 (복제하지 말고 따른다)

| 주제 | 정본 |
|---|---|
| 5분 경로 | mydocs/manual/agent_onboarding.md |
| CLI 계약 | mydocs/manual/cli_commands.md |
| MCP 통합 | mydocs/manual/mcp_integration_guide.md |
| 호스트 부착 | mydocs/manual/mcp_attach_kit.md |
| 표 왕복 | mydocs/manual/recipes/02_table_csv_roundtrip.md |
| 서식 채움 | mydocs/manual/recipes/01_fill_form_and_submit.md |
| 수신 안전 | mydocs/manual/recipes/04_safety_check_untrusted_doc.md |
| 송신 스윕 | mydocs/manual/recipes/10_security_sweep_before_share.md |
| 실패 사전 | mydocs/manual/agent_troubleshooting_guide.md |

## G. 결정 트리 (의사코드)

```text
run doctor
if exit == 2: fix argv or --force; stop
if exit == 3: cargo build --release --bin rhwp; rerun; stop
if exceptions has bad_sample: replace sample; rerun; stop
read first5Min[] where referenceExists
do triage on user file or samples/basic/english.hwp
if tables: do table coordinate read
if fieldCount>0: hand off to rhwp-form-fill (do not invent fill)
run inspect 3-axis
paste mcp snippet for the actual host
if work product must be proven: rhwp-work-receipt
never start gym here
```

## H. Windows / Unix 명령 쌍

온보딩 에이전트는 셸이 다르다. 같은 일을 두 줄로 적는다.

| 일 | Unix | Windows |
|---|---|---|
| 버전 | `rhwp --version` | `rhwp --version` |
| 닥터 | `python tools/agent_onboarding/rhwp_doctor.py --json` | `python tools/agent_onboarding/rhwp_doctor.py --json` |
| info | `rhwp info samples/basic/english.hwp --json` | `rhwp info samples/basic/english.hwp --json` |
| 빌드 | `cargo build --release --bin rhwp` | `cargo build --release --bin rhwp` |
| which | `command -v rhwp` | `Get-Command rhwp` |
| release 확인 | `ls target/release/rhwp` | `Get-Item target\release\rhwp.exe` |
| MCP 핸드셰이크 | `printf ... | rhwp mcp-serve` | `같은 JSON 을 파이프. PowerShell 은 UTF-8 유지` |

## I. 자주 묻는 분기 (단답)

### 닥터가 cargo build 를 해 주나?

아니다. 명령만 찍고 exit 3.

### 오프라인이면 온보딩 실패인가?

아니다. 비임계 SKIP.

### gym 을 돌려야 하나?

아니다. 이 스킬의 비범위.

### edit fill-fields 플래그를 여기 적나?

아니다. rhwp-form-fill 로 위임.

### 새 MCP 포트를 여나?

아니다. stdio 만.

### samples 가 없으면?

--sample 로 실제 문서를 준다. 없으면 SKIP/예외.

### PATH 와 release 가 둘 다 있으면?

PATH 가 이긴다. 저장소 산출을 쓰려면 --rhwp.

### --write 가 호스트 모양을 쓰나?

A형만. 다른 모양은 리포트에서 병합.

### JSON 스키마 1.1 이 깨지나?

필드 추가만. 기존 키 유지.

### 테스트에 진짜 rhwp 가 필요하나?

아니다. 순수 로직 가드.
