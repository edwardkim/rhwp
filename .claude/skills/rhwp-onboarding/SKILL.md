---
name: rhwp-onboarding
description: rhwp 를 처음 만나는 에이전트를 한 명령으로 온보딩합니다. tools/agent_onboarding/rhwp_doctor.py 하나로 바이너리 위치·버전 확인 → 번들 샘플 자가검증(info/export-text) → 붙여넣기용 .mcp.json 방출 → 첫 5분 레시피 지도(트리아지·표 추출·서식 채우기·보안 스윕·작업 영수증)까지 끝내고, 종료 코드로 정상/빌드필요를 신호합니다. 트리거 — 사용자가 "rhwp 처음/설치/시작/온보딩", "rhwp 어떻게 붙여/시작해", "rhwp 돌아가는지 확인", "rhwp 셋업/부트스트랩", "rhwp 뭐부터", ".mcp.json 만들어줘" 등을 요청할 때. 5분 경로 정본은 mydocs/manual/agent_onboarding.md.
---

# rhwp-onboarding — 제로프릭션 온보딩 Skill

## 목적

rhwp 를 **처음 보는** 에이전트(또는 그 사람)를 "설치 → 검증 → MCP 배선 → 첫 레시피"까지
한 번에 데려간다. 이 스킬은 얇다 — 실제 일은 닥터 스크립트와 5분 경로 문서가 한다.

- 닥터: [`tools/agent_onboarding/rhwp_doctor.py`](../../../tools/agent_onboarding/rhwp_doctor.py) (순수 Python 3, 의존성 0)
- 5분 경로 정본: [`mydocs/manual/agent_onboarding.md`](../../../mydocs/manual/agent_onboarding.md)

이미 MCP 로 붙어 있고 **세션/무상태 도구 선택**이 논점이면 이 스킬이 아니라
`rhwp-mcp-session` 을 쓴다. 이 스킬은 그 앞단(0→1 부트스트랩) 전용이다.

## 한 명령

저장소 루트에서:

```bash
python tools/agent_onboarding/rhwp_doctor.py            # 사람용 리포트
python tools/agent_onboarding/rhwp_doctor.py --json     # 기계 판독(stdout=JSON 하나)
```

닥터가 하는 일:

1. **바이너리 위치·버전** — `PATH` → `target/release/rhwp` 순으로 찾고 `--version` 확인.
   없으면 `cargo build --release --bin rhwp` 를 찍고 **종료 코드 3** 으로 신호(긴 빌드를
   대신 돌리지 않는다).
2. **자가검증** — `samples/` 의 작은 문서로 `info` / `export-text --json` 을 돌려 구조 출력을
   확인한다. 통과를 위조하지 않는다 — 못 돌린 검사는 `SKIP`/`FAIL` 로 정직하게 보고.
3. **`.mcp.json` 방출** — `{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }`.
   `PATH` 에 없으면 절대 경로를 채워준다. `--write <경로>` 로 파일로 쓰되 기존 파일은
   `--force` 없이 덮어쓰지 않는다.
4. **첫 5분 레시피 지도** — 실존하는 스킬·레시피만 인용해 5대 고가치 과제를 명령과 함께 제시.

## 닥터가 실제로 돌리는 명령 (손으로 확인할 때)

닥터가 `FAIL` 을 내면 같은 명령을 직접 쳐서 원인을 본다 — 닥터는 아래를 감싼 것뿐이다.

```bash
rhwp --version
rhwp info samples/basic/english.hwp --json
rhwp export-text samples/basic/english.hwp --json --max-chars 2000
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
| 1 | 임계 실패 | `FAIL` 상세 진단 |
| 2 | 사용법 오류 | 인자 교정 |
| 3 | 바이너리 미발견 | `cargo build --release --bin rhwp` |

## 다음

- MCP 통합 전체 절차: [`mydocs/manual/mcp_integration_guide.md`](../../../mydocs/manual/mcp_integration_guide.md)
- CLI 전체 명령: [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md)
- 과제별 스킬: `rhwp-doc-triage` · `rhwp-table-exchange` · `rhwp-form-fill` ·
  `rhwp-security-sweep` · `rhwp-work-receipt`
