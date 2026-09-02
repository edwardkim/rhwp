# 닥터 JSON 리포트 스키마 (schemaVersion 1.1)

`python tools/agent_onboarding/rhwp_doctor.py --json` 의 stdout 은 JSON 객체 하나다.
사람용 텍스트는 stderr. 에이전트는 stdout 만 파싱한다.

필드는 **추가만** 한다. 1.0 소비자가 읽던 `binary` / `checks` / `mcpJson` /
`recipes` / `ok` / `exitCode` 는 그대로다.

## 최상위

| 키 | 형 | 뜻 |
|---|---|---|
| `schemaVersion` | string | `"1.1"` |
| `tool` | string | `"rhwp_doctor"` |
| `ok` | bool | 임계 검사가 모두 PASS 이고 바이너리가 있음 |
| `exitCode` | int | 0/1/2/3 |
| `repoRoot` | string | 해석된 저장소 루트 |
| `binary` | object | 고른 바이너리 |
| `binaryInventory` | array | 탐색 자리마다 hit/miss |
| `sample` | string\|null | 고른 샘플 경로 |
| `sampleClassification` | object\|null | 매직 분류 |
| `checks` | array | 검사 결과 |
| `exceptions` | array | 예외 플레이북 |
| `network` | object | 프로브 결과 |
| `mcpJson` | object | A형 스니펫 |
| `mcpJsonWritten` | string\|null | `--write` 가 쓴 경로 |
| `mcpHost` | object\|null | `--host` 가 고른 모양 |
| `recipes` | array | 5대 과제 + 실존 플래그 |
| `first5Min` | array | 5분 단계 + 실존 플래그 |
| `references` | array | 온보딩 문서 실존 |
| `buildCommand` | string | 항상 `cargo build --release --bin rhwp` |
| `python` | object | `{version, executable}` |

## `binary`

| 키 | 형 | 뜻 |
|---|---|---|
| `found` | bool | 실행 파일을 골랐는가 |
| `path` | string\|null | 절대 경로 |
| `source` | string | `--rhwp` / `PATH` / `target/release` / `target/debug` / `RHWP_BIN` / `cargo-bin` / `(미발견)` / `--rhwp(미발견)` |
| `onPath` | bool | PATH 히트. 스니펫 command 가 `rhwp` |
| `version` | string\|null | `--version` 첫 줄 |

## `binaryInventory[]`

| 키 | 형 |
|---|---|
| `source` | string |
| `path` | string |
| `kind` | `override` / `env` / `which` / `file` |
| `resolved` | string\|null |
| `exists` | bool |

## `sampleClassification`

| 키 | 형 | 값 |
|---|---|---|
| `ok` | bool | 자가검증을 돌릴 가치 |
| `kind` | string | `missing` `empty` `too_small` `not_document` `avoid` `hwp5` `hwpx` `hwp3` |
| `reason` | string | 사람/에이전트용 한 줄 |
| `sizeBytes` | int | |
| `magicHex` | string | 선두 최대 8바이트 hex |
| `path` | string\|null | |

## `checks[]`

| 키 | 형 | 뜻 |
|---|---|---|
| `id` | string | 안정 식별자 |
| `title` | string | 사람용 |
| `status` | string | `PASS` `FAIL` `SKIP` |
| `command` | string | 재현 명령 |
| `detail` | string | 한 줄 이유 |
| `critical` | bool | `ok` 집계에 들어가는가 |
| `version` | string? | version 검사만 |
| `exception` | string? | 예외 kind |
| `nextSteps` | array? | 예외가 있을 때 |

알려진 `id`:

| id | 임계 | 언제 |
|---|---|---|
| `python` | 아니오 | 항상 |
| `version` | 예 | 바이너리 있음 |
| `selftest-info` | 예 | 자가검증 |
| `selftest-export-text` | 예 | 자가검증 |
| `selftest-explain` | 아니오 | `--skip-extra` 아니면 |
| `selftest-digest` | 아니오 | 동일 |
| `selftest-inspect-injection` | 아니오 | 동일 |
| `network` | 아니오 | 항상 (`--offline` 이면 SKIP) |

## `exceptions[]`

| 키 | 형 |
|---|---|
| `kind` | `missing_binary` `bad_sample` `no_network` `write_exists` `selftest_timeout` `selftest_parse` |
| `title` | 짧은 제목 |
| `detail` | 이번 실행의 이유 |
| `path` | 관련 경로 또는 null |
| `nextSteps` | 문자열 배열. 마지막은 보통 references/*.md |

`kind` 가 중복되면 한 줄만 남긴다.

## `network`

| 키 | 형 |
|---|---|
| `probed` | bool |
| `reachable` | bool\|null |
| `offline` | bool |
| `targets` | `{host,port,ok,error}[]` |
| `reason` | `--offline` 등, 생략 시 없을 수 있음 |

## `mcpJson`

항상 A형:

```json
{
  "mcpServers": {
    "rhwp": {
      "command": "rhwp",
      "args": ["mcp-serve"]
    }
  }
}
```

`onPath==false` 이면 `command` 가 절대 경로다. `args` 는 복사본이라 호출자가
배열을 고쳐도 스니펫이 바뀌지 않는다.

## `mcpHost` (`--host` 있을 때만)

| 키 | 형 |
|---|---|
| `host` | id |
| `title` | 사람용 이름 |
| `file` | 설정 파일 힌트 |
| `shape` | `A` `B` `zed` `goose` `continue` |
| `confidence` | `repo-proven` `high` `medium` |
| `snippet` | 그 호스트 모양의 객체 |

## `recipes[]`

| 키 | 형 |
|---|---|
| `task` | 과제 한 줄 |
| `command` | 1차 명령 |
| `skill` | 스킬 디렉터리 이름 |
| `skillPath` | `.claude/skills/<skill>` |
| `skillExists` | `SKILL.md` 실존 |
| `recipe` | 매뉴얼 레시피 상대 경로 또는 null |
| `recipeExists` | 파일 실존. `recipe` 가 null 이면 항상 false |

## `first5Min[]`

| 키 | 형 |
|---|---|
| `id` | `triage` `tables` `form-read` `security` `attach` |
| `title` | |
| `minutes` | 1 |
| `commands` | 기존 CLI 문자열 |
| `skill` | 위임 스킬 |
| `skillExists` | |
| `reference` | references/ 상대 경로 |
| `referenceExists` | |
| `readOnly` | 항상 true |
| `gate` | 판정 한 줄 |

## `references[]`

| 키 | 형 |
|---|---|
| `id` | 안정 식별자 |
| `path` | 저장소 상대 경로 |
| `role` | 한 줄 |
| `exists` | |

## 집계 규칙

```text
if not binary.found:
    ok = false
    exitCode = 3   # --write 거부(2)가 나중에 덮을 수 있음
else if any critical check status != PASS:
    ok = false
    exitCode = 1
else:
    ok = true
    exitCode = 0
```

`--write` 가 기존 파일을 `--force` 없이 만나면 리포트를 찍고 **프로세스 종료 코드 2**.
`report.exitCode` 는 집계값(0/1/3)일 수 있다. 소비자는 **프로세스 종료 코드**를
우선한다. 덮어쓰기 거부는 사용법 오류다.

`--list-hosts` / `--list-recipes` 는 이 스키마가 아니라 작은 JSON 을 내고 exit 0.

## 소비 예

```bash
python tools/agent_onboarding/rhwp_doctor.py --json --offline > doctor.json
```

에이전트 의사코드:

```text
r = parse(stdout)
if process.exit == 2: fix argv
if process.exit == 3 or r.binary.found == false:
    follow exceptions[kind=missing_binary].nextSteps
if process.exit == 1:
    read exceptions[] ; if bad_sample: change --sample
if process.exit == 0:
    paste r.mcpJson
    for step in r.first5Min where step.referenceExists:
        open step.reference
```

## 하지 말 것

- stderr 를 JSON 으로 파싱하지 않는다.
- `ok==true` 인데 `selftest-info` 가 없다고 `--skip-selftest` 를 성공 시연으로 과장하지 않는다.
- 없는 `recipes[].recipe` 를 있는 것처럼 인용하지 않는다 (`recipeExists`).
- gym 필드가 이 스키마에 생기기를 기다리지 않는다. 없다.

## 픽스처

`tools/agent_onboarding/fixtures/reports/*.shape.json` 은 런타임 골든이 아니라
형태 메모다. 테스트는 함수 단위로 같은 규칙을 가드한다.
