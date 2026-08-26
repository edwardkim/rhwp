---
kind: guide
status: active
canonical: mydocs/manual/agent_onboarding.md
last_verified: 2026-08-15
---

# 에이전트 제로프릭션 온보딩 — 한 명령으로 설치·검증·MCP배선·첫 레시피

**목표 한 줄**: rhwp 를 처음 보는 AI 에이전트(또는 그 사람)가 **명령 하나**로
"바이너리 있음 → 자가검증 통과 → MCP 배선 완료 → 첫 레시피를 안다" 상태까지 도달한다.

그 한 명령이 [`tools/agent_onboarding/rhwp_doctor.py`](../../tools/agent_onboarding/rhwp_doctor.py)
(순수 Python 3 표준 라이브러리, 외부 의존성 0)다. 이 문서는 그 5분 경로를 설명한다.
전체 통합 절차는 [MCP 통합 가이드](mcp_integration_guide.md), 전체 명령 표면은
[CLI 명령어 매뉴얼](cli_commands.md)이 정본이며 여기서 중복하지 않는다.

## 전제 (정직하게)

- **빌드된 rhwp 바이너리 1개.** rhwp 는 Rust 크레이트라 배포 바이너리를 쓰거나 직접 빌드한다.
  아직 없으면 저장소 루트에서 한 번:
  ```bash
  cargo build --release --bin rhwp     # 산출물: target/release/rhwp (Windows: rhwp.exe)
  ```
  닥터는 이 빌드를 **대신 돌려주지 않는다** — 없으면 위 명령을 찍고 종료 코드 3으로 신호한다
  (긴 빌드로 매달리지 않기 위해서다).
- **Python 3.** 닥터는 표준 라이브러리만 쓴다. 설치할 패키지가 없다.

## 5분 경로

### 1. 빌드 (최초 1회, 이미 있으면 건너뜀)

```bash
cargo build --release --bin rhwp
```

### 2. 닥터 실행 — 넷을 한 번에

```bash
python tools/agent_onboarding/rhwp_doctor.py
```

이 한 줄이 다음을 순서대로 한다:

1. **바이너리 위치·버전** — `PATH` → `target/release/rhwp` 순으로 찾고 `rhwp --version` 을 확인한다.
2. **번들 샘플 자가검증** — `samples/` 의 작은 문서로 `info` 와 `export-text --json` 을 돌려
   구조 출력이 실제로 나오는지 확인한다. **통과를 위조하지 않는다** — 못 돌린 검사는
   이유와 함께 `SKIP`/`FAIL` 로 정직하게 보고한다.
3. **붙여넣기용 `.mcp.json`** 을 방출한다(다음 단계).
4. **첫 5분 레시피 지도**를 출력한다(4단계 아래 표).

### 3. `.mcp.json` 붙여넣기

닥터가 찍어준 스니펫을 **호스트(Claude Code 등) 프로젝트 루트**의 `.mcp.json` 에 두거나,
이미 파일이 있으면 `mcpServers` 키만 병합한다:

```jsonc
{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }
```

- `rhwp` 가 `PATH` 에 없으면 닥터는 `command` 에 **바이너리 절대 경로**를 넣어준다
  (예: `target/release/rhwp.exe`). 이는 [MCP 통합 가이드](mcp_integration_guide.md)의
  계약과 같다 — 전송은 stdio 뿐이라 포트·인증 설정이 없다.
- 파일로 바로 쓰려면 `--write <경로>` 를 준다. **기존 파일은 덮어쓰지 않는다** —
  덮어쓰려면 `--force` 를 명시해야 한다.
  ```bash
  python tools/agent_onboarding/rhwp_doctor.py --write .mcp.json          # 없을 때만 기록
  python tools/agent_onboarding/rhwp_doctor.py --write .mcp.json --force  # 덮어쓰기 허용
  ```
- 저장소 루트 [`.mcp.json`](../../.mcp.json) 은 Claude Code 용으로 이미 rhwp 를 붙여 둔다.
  다른 호스트(Cursor·Cline·Continue·Zed 등) 설정은 [MCP 부착 키트](mcp_attach_kit.md) 참조.

### 4. 첫 레시피 — 가장 값어치 높은 5과제

닥터가 이 표를 출력하며, **실존하는 스킬·레시피만** 인용한다(런타임에 파일 존재를 확인해
`[OK]`/`[missing]` 로 표시). 스킬은 트리거 문구로 자동 발동하고, 레시피는 실측 절차서다.

| 과제 | 명령(1차) | 스킬 | 레시피 |
|---|---|---|---|
| 문서 트리아지 (처음 보는 문서 파악) | `rhwp digest "<파일>" --json` | [`rhwp-doc-triage`](../../.claude/skills/rhwp-doc-triage/SKILL.md) | — |
| 표 추출 (병합 보존 / CSV 왕복) | `rhwp export-tables "<파일>" --json` | [`rhwp-table-exchange`](../../.claude/skills/rhwp-table-exchange/SKILL.md) | [레시피 02](recipes/02_table_csv_roundtrip.md) |
| 서식 채우기 (누름틀 → 제출본) | `rhwp fields "<파일>" --json` → `rhwp edit fill-fields …` | [`rhwp-form-fill`](../../.claude/skills/rhwp-form-fill/SKILL.md) | [레시피 01](recipes/01_fill_form_and_submit.md) · [05](recipes/05_mail_merge_batch_fill.md) |
| 보안 스윕 (주입·은닉·유니코드) | `rhwp inspect injection "<파일>" --json` | [`rhwp-security-sweep`](../../.claude/skills/rhwp-security-sweep/SKILL.md) | [레시피 10](recipes/10_security_sweep_before_share.md) · [04](recipes/04_safety_check_untrusted_doc.md) |
| 작업 영수증 (3-해시 증명) | `rhwp replay --plan-json '{…}' --json` | [`rhwp-work-receipt`](../../.claude/skills/rhwp-work-receipt/SKILL.md) | — |

과제를 무엇으로 풀지 막힐 때의 판단 트리·봉투 실측은
[에이전트 실무 대체 예제집](agent_task_playbook.md)과
[CLI JSON 파이프라인 가이드](cli_json_pipeline_guide.md)가 잇는다.

### 5. 판정 읽기

닥터는 마지막에 판정과 종료 코드를 찍는다. 에이전트는 `--json` 으로 기계 판독한다:

```bash
python tools/agent_onboarding/rhwp_doctor.py --json | jq '{ok, exitCode, checks: [.checks[] | {id, status}]}'
```

| 종료 코드 | 뜻 | 다음 행동 |
|---:|---|---|
| 0 | 모든 임계 검사 통과 | `.mcp.json` 붙이고 첫 레시피로 진행 |
| 1 | 임계 검사 실패(바이너리는 있으나 버전/자가검증이 깨짐) | 출력의 `FAIL` 상세를 보고 진단 |
| 2 | 사용법 오류(`--write` 덮어쓰기 거부 등) | 인자를 고쳐 재실행 |
| 3 | 바이너리 미발견(아직 빌드 안 됨) | 위 `cargo build --release --bin rhwp` 실행 |

`--json` 모드에서는 **stdout 에 리포트 JSON 하나만** 나가고(에이전트가 그대로 파싱),
사람용 텍스트는 stderr 로 간다. 리포트 스키마: `{schemaVersion, tool, ok, exitCode,
binary{found,path,source,onPath,version}, sample, checks[], mcpJson, recipes[], buildCommand}`.

## 닥터가 하는 검사 (요약)

| 검사 id | 하는 일 | 통과 조건 |
|---|---|---|
| `version` | `rhwp --version` | 종료 0 + 비어 있지 않은 버전 문자열 |
| `selftest-info` | `rhwp info <샘플> --json` | JSON 파싱 + `format`·`pageCount` 필드 존재 |
| `selftest-export-text` | `rhwp export-text <샘플> --json --max-chars 2000` | JSON 파싱 + `pages` 배열 비어 있지 않음 |

- 자가검증 샘플은 `samples/basic/english.hwp` 같은 **평범한 문서**를 우선 고른다. 없으면
  `--sample <경로>` 로 지정한다.
- 모든 하위 프로세스는 타임아웃으로 감싸므로 어떤 검사도 매달리지 않는다.
- 바이너리를 직접 지목하려면 `--rhwp <경로>`, 저장소 루트를 옮기려면 `--repo-root <경로>`.

## 문제 해결

- **`exit=3`, "rhwp 미발견"** — 아직 빌드 안 됐다. `cargo build --release --bin rhwp`.
  네이티브 빌드는 항상 로컬 cargo 를 쓴다(Docker 는 WASM 전용) —
  [개발 환경 가이드](dev_environment_guide.md).
- **"샘플 문서를 찾지 못함"** — `samples/` 가 없는 축소 체크아웃이다. `--sample <파일>` 로
  아무 `.hwp`/`.hwpx` 를 준다.
- **Windows 콘솔 한글 깨짐** — 닥터는 stdout/stderr 를 UTF-8 로 맞춰 cp949 콘솔에서도
  한글·JSON 이 깨지지 않게 한다. 그래도 콘솔 폰트 문제로 보이면 `--json` 을 파일로 리다이렉트해
  읽는다.

## 다음 단계

- MCP 세션 도구(재파싱 없는 반복 조회 `hwp_open`→`hwp_doc_text`→`hwp_close`)와 무상태 도구
  선택 기준: [MCP 통합 가이드](mcp_integration_guide.md), 스킬 [`rhwp-mcp-session`](../../.claude/skills/rhwp-mcp-session/SKILL.md).
- rhwp 참조 문서 전체 지도: [에이전트 지식 지도](agent_knowledge_map.md).
- 사람(기여자) 관점의 저장소 진입점: [rhwp 온보딩 가이드](onboarding_guide.md).
