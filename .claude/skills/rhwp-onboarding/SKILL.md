---
name: rhwp-onboarding
description: rhwp 를 처음 만나는 에이전트를 한 명령으로 온보딩합니다. tools/agent_onboarding/rhwp_doctor.py 하나로 바이너리 위치·버전 확인 → 번들 샘플 자가검증(info/export-text) → 붙여넣기용 .mcp.json 방출 → 첫 5분 레시피 지도(트리아지·표 추출·서식 조사·보안 스윕·작업 영수증)까지 끝내고, 종료 코드로 정상/빌드필요를 신호합니다. 트리거 — 사용자가 "rhwp 처음/설치/시작/온보딩", "rhwp 어떻게 붙여/시작해", "rhwp 돌아가는지 확인", "rhwp 셋업/부트스트랩", "rhwp 뭐부터", ".mcp.json 만들어줘" 등을 요청할 때. 5분 경로 정본은 mydocs/manual/agent_onboarding.md. gym 이 아니라 실사용 에이전트 경로다.
---

# rhwp-onboarding — 제로프릭션 온보딩 Skill

## 목적

rhwp 를 **처음 보는** 에이전트(또는 그 사람)를 "설치 → 검증 → MCP 배선 → 첫 레시피"까지
한 번에 데려간다. 이 스킬은 얇은 진입점이다. 실제 일은 닥터 스크립트와 `references/`
레시피가 한다.

- 닥터: [`tools/agent_onboarding/rhwp_doctor.py`](../../../tools/agent_onboarding/rhwp_doctor.py) (순수 Python 3, 의존성 0)
- 5분 경로 정본: [`mydocs/manual/agent_onboarding.md`](../../../mydocs/manual/agent_onboarding.md)
- 작업 기록: [`mydocs/working/agent_onboarding.md`](../../../mydocs/working/archives/agent_onboarding.md)

이미 MCP 로 붙어 있고 **세션/무상태 도구 선택**이 논점이면 이 스킬이 아니라
`rhwp-mcp-session` 을 쓴다. 이 스킬은 그 앞단(0→1 부트스트랩) 전용이다.

이 스킬은 **gym 을 돌리지 않는다.** 실사용 에이전트가 문서를 읽고 표를 뽑고 서식을
조사하고 보안 신호를 보고 MCP 에 붙는 경로만 다룬다.

## 한 명령

저장소 루트에서:

```bash
python tools/agent_onboarding/rhwp_doctor.py            # 사람용 리포트
python tools/agent_onboarding/rhwp_doctor.py --json     # 기계 판독(stdout=JSON 하나)
python tools/agent_onboarding/rhwp_doctor.py --offline  # 네트워크 프로브 생략
```

닥터가 하는 일:

1. **바이너리 위치·버전** — `PATH` → `RHWP_BIN` → `target/release/rhwp` →
   `target/debug/rhwp` → cargo bin 순으로 찾고 `--version` 확인.
   없으면 `cargo build --release --bin rhwp` 를 찍고 **종료 코드 3** 으로 신호
   (긴 빌드를 대신 돌리지 않는다). 상세는
   [binary-discovery.md](references/binary-discovery.md).
2. **자가검증** — `samples/` 의 작은 문서로 `info` / `export-text --json` 을 돌리기
   *전에* 매직 바이트로 불량 샘플을 거른다. 통과를 위조하지 않는다.
   상세는 [sample-selftest.md](references/sample-selftest.md).
3. **`.mcp.json` 방출** — `{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }`.
   `PATH` 에 없으면 절대 경로를 채워준다. `--write <경로>` 로 파일로 쓰되 기존
   파일은 `--force` 없이 덮어쓰지 않는다. 호스트별 모양은
   [mcp-json-paste.md](references/mcp-json-paste.md).
4. **첫 5분 레시피 지도** — 실존하는 스킬·레시피만 인용한다. 지도는
   [first-5-min.md](references/first-5-min.md).
5. **예외 경로** — `missing_binary` / `bad_sample` / `no_network` 를 데이터로 보고한다.

## 닥터가 실제로 돌리는 명령 (손으로 확인할 때)

닥터가 `FAIL` 을 내면 같은 명령을 직접 쳐서 원인을 본다 — 닥터는 아래를 감싼 것뿐이다.

```bash
rhwp --version
rhwp info samples/basic/english.hwp --json
rhwp export-text samples/basic/english.hwp --json --max-chars 2000
```

선택(비임계):

```bash
rhwp explain samples/basic/english.hwp --json
rhwp digest samples/basic/english.hwp --json --max-chars 500
rhwp inspect injection samples/basic/english.hwp --json
```

`.mcp.json` 이 띄우는 상주 서버도 같은 바이너리다 — 배선 전에 한 번 손으로 띄워 본다.

```bash
rhwp mcp-serve
```

붙였으면 첫 과제로 넘어간다. 어느 스킬로 갈지는 아래 지도를 따르되, 한 문서를 빠르게
파악하는 최단 경로는 이 두 명령이다.

```bash
rhwp explain samples/basic/english.hwp --json
rhwp digest samples/basic/english.hwp --json
```

## 종료 코드로 판정

| 코드 | 뜻 | 다음 |
|---:|---|---|
| 0 | 정상 | `.mcp.json` 붙이고 첫 레시피로 |
| 1 | 임계 실패 | `FAIL` 상세·`exceptions[]` 진단 |
| 2 | 사용법 오류 | 인자 교정 (`--host` 오타, `--write` 덮어쓰기 거부) |
| 3 | 바이너리 미발견 | `cargo build --release --bin rhwp` |

`--json` 리포트의 `exceptions[].kind` 는 `missing_binary` · `bad_sample` ·
`no_network` · `write_exists` · `selftest_timeout` · `selftest_parse` 다.
오프라인(`no_network`)은 임계 실패가 아니다.

## 예외 경로 — 여기서 갈라라

| 증상 | kind | 문서 |
|---|---|---|
| `exit=3`, rhwp 미발견 | `missing_binary` | [exception-missing-binary.md](references/exception-missing-binary.md) |
| 샘플이 빈 파일·txt·시그니처 없음 | `bad_sample` | [exception-bad-sample.md](references/exception-bad-sample.md) |
| 외부 TCP 불가 / `--offline` | `no_network` | [exception-no-network.md](references/exception-no-network.md) |

## 첫 5분 지도 (실사용, gym 아님)

| 분 | 과제 | 읽기 전용 명령 | 상세 |
|---:|---|---|---|
| 1 | 트리아지 | `info` → `explain` → `digest` | [first-5-min-triage.md](references/first-5-min-triage.md) |
| 2 | 표 좌표 | `export-tables` → `table-to-csv` | [first-5-min-tables.md](references/first-5-min-tables.md) |
| 3 | 서식 조사 | `fields` 만. 채움은 기존 스킬 | [first-5-min-form-read.md](references/first-5-min-form-read.md) |
| 4 | 보안 스윕 | `inspect` 3축 | [first-5-min-security.md](references/first-5-min-security.md) |
| 5 | MCP·영수증 입구 | `mcp-serve` / `capabilities --mcp` / `replay` | [mcp-json-paste.md](references/mcp-json-paste.md) |

편집 로직을 이 스킬에서 발명하지 않는다. `edit fill-fields` / `csv-to-table` /
`edit redact` 는 이미 있는 스킬·레시피로 위임한다.

## 하지 않는 것

- 새 rhwp CLI 하위명령을 만들지 않는다.
- gym 팩을 실행하거나 점수를 내지 않는다.
- 바이너리가 없을 때 `cargo build` 를 대신 돌리지 않는다.
- 네트워크가 없다고 온보딩을 실패로 만들지 않는다.
- 불량 샘플을 PASS 로 위조하지 않는다.
- 다른 스킬의 편집 절차를 여기 복제해 드리프트 시키지 않는다.

## 검증 (이 스킬을 고친 뒤)

```bash
python -m unittest tools/agent_onboarding/test_rhwp_doctor.py
python tools/agent_onboarding/rhwp_doctor.py --list-hosts
python tools/agent_onboarding/rhwp_doctor.py --list-recipes
```

## 다음

- MCP 통합 전체 절차: [`mydocs/manual/mcp_integration_guide.md`](../../../mydocs/manual/mcp_integration_guide.md)
- CLI 전체 명령: [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md)
- 과제별 스킬: `rhwp-doc-triage` · `rhwp-table-exchange` · `rhwp-form-fill` ·
  `rhwp-security-sweep` · `rhwp-work-receipt`
