# gym 에이전트 세션 트레이스

이 문서는 `gym/tools/agent_session.py` 의 한국어 정본이다. 종점 채점만 있으면
에이전트가 우회·역순·여분 명령으로도 만점을 받는다. 이 도구는 **선언된 세션**
(명령 열 + 기대 종료)과 **기록된 트레이스**(JSONL)를 기계 대조한다.

- 채점 축: 명령 계열(`argv[0]`) · 종료 코드 · 순서
- 쓰지 않는 것: LLM-judge, 골든 경로 문자열 비교, 새 rhwp CLI
- 재생(`score-replay`)은 rhwp 없이 픽스처 JSONL 만으로 돈다
- 기록(`record`)은 `--bin` 이 실재할 때만 실행한다. 없으면 거절한다

관련 이슈 #5206 · PR #5211.

## 왜 이 도구인가

gym 채점은 산출(종점)을 본다. `trajectory.py` 는 기준 풀이에서 마지막 외부
스텝을 잘라 그 스텝이 load-bearing 인지 감사한다. 둘 다 "에이전트가 실제로
어떤 명령 열을 어떤 순서로 밟았는가"는 파일로 남기지 않는다.

이 도구는 그 구멍을 메운다.

| 도구 | 보는 것 | 보지 않는 것 |
|------|---------|--------------|
| gym 채점 (`score.py`) | 종점 산출 | 명령 열 |
| `trajectory.py` | 마지막 스텝이 필요한가 | 에이전트가 밟은 경로 |
| `agent_session.py` | 선언 경로 vs 관측 경로 | 산출 내용·LLM 판정 |

종점이 맞아도 경로가 틀리면 실패다. 경로가 맞아도 종료 코드가 틀리면 실패다.
여분 스텝·누락 스텝·역순은 각각 다른 사유로 남는다.

## 하지 않는 것

- **새 rhwp CLI 를 만들지 않는다.** `info`, `export-text` 같은 기존 명령을
  세션 `run` 에 적을 뿐이다. `rhwp agent-session` 같은 하위명령은 없다.
- **재생에 바이너리를 요구하지 않는다.** `score-replay` 는 `--bin` 인자가
  없고 PATH 에서 rhwp 를 찾지도 않는다.
- **없는 바이너리를 가장해 트레이스를 위조하지 않는다.** `record --bin` 이
  없거나 경로가 파일이 아니면 `RecordRefused`(종료 2) 로 거절한다.
- **pack·README·profiles·checks·coverage 를 건드리지 않는다.** 이 도구는
  기본 채점 경로에 연결하지 않는다.
- **자리표를 몰래 해석해 디스크를 요구하지 않는다.** 재생 채점의 합격 축은
  계열·종료·순서다. `expectPath` 디스크 존재는 `check_paths=True`(기록 직후
  재검증)일 때만 합격에 넣는다.

## 세션 정의

```json
{
  "id": "inspect-then-export",
  "input": "samples/x.hwp",
  "subDir": "work",
  "steps": [
    {"run": ["info", "{input}", "--json"], "expectExit": 0},
    {
      "run": ["export-text", "{input}", "-o", "{sub:out.txt}"],
      "expectExit": 0,
      "expectPath": "{sub:out.txt}"
    }
  ]
}
```

### 필드

| 필드 | 필수 | 의미 |
|------|------|------|
| `id` | 예 | 비어 있지 않은 문자열. 리포트 `sessionId` |
| `input` | 아니오 | `{input}` 자리표 기본값. 문자열 |
| `subDir` | 아니오 | `{sub:이름}` 자리표 기본값. 작업 폴더 |
| `steps` | 예 | 비어 있지 않은 객체 배열 |
| `steps[].run` | 예 | 비어 있지 않은 문자열 배열. `run[0]` 이 명령 계열 |
| `steps[].expectExit` | 아니오 | 정수. 기본 0. 불리언 금지 |
| `steps[].expectPath` | 아니오 | 기대 산출 경로. 자리표 가능 |

`run` 은 rhwp 의 새 표면이 아니다. 이미 있는 하위명령 이름만 적는다.
이 도구는 명령 존재 여부를 바이너리에 묻지 않는다 — 선언과 관측의
`argv[0]` 이 같은지만 본다.

### 자리표

| 자리표 | 해석 | 컨텍스트가 없을 때 |
|--------|------|-------------------|
| `{input}` | 세션/CLI 입력 경로 | 그대로 남김 |
| `{sub:이름}` | `subDir/이름` | 그대로 남김 |
| `{그 외}` | 알 수 없음 | 검증에서 위반, 해석은 그대로 남김 |

중괄호가 짝이 아니면 검증 위반이다. `{sub:}` 처럼 이름이 비거나 공백·중괄호가
들어가면 위반이다. 토큰 안에 자리표를 끼울 수 있다
(`pre-{sub:a.txt}-post`).

엄격 모드(`resolve_token(..., strict=True)`)는 미해석 자리표를
`PlaceholderError` 로 올린다. 재생 채점의 기본은 비엄격이다. 작업 폴더가
없어도 픽스처 재생이 실패하지 않게 하기 위함이다.

## 트레이스 JSONL

한 줄 = 한 스텝.

```json
{"ts":"2026-08-18T00:00:00Z","argv":["info","samples/x.hwp","--json"],"exit":0,"stdoutSha256":"<hex64>","ok":true}
```

| 필드 | 필수 | 의미 |
|------|------|------|
| `ts` | 예 | 비어 있지 않은 문자열 (UTC `YYYY-MM-DDTHH:MM:SSZ` 권장) |
| `argv` | 예 | 비어 있지 않은 문자열 배열. `argv[0]` 이 관측 계열 |
| `exit` | 예 | 정수. 불리언 금지 |
| `ok` | 예 | 불리언. 기록 당시 `exit==expectExit` 그리고 경로 검사 |
| `stdoutSha256` | 아니오 | 64자리 hex. 있으면 형식 검사 |

빈 줄은 건너뛴다. 본문이 UTF-8 BOM 으로 시작하면 한 번 벗긴다. 한 줄이
배열·스칼라이면 `TraceParseError` 다. `ok` 가 없거나 `stdoutSha256` 이
hex64 가 아니면 `TraceSchemaError` 다.

기록기가 stdout 을 받았을 때만 해시를 넣는다. 재생 채점은 해시 값을
비교하지 않는다 — 해시는 추적용이다.

## 채점 리포트

```json
{
  "kind": "gymAgentSession",
  "schemaVersion": "1.0",
  "ok": false,
  "sessionId": "inspect-then-export",
  "declared": 2,
  "observed": 2,
  "matched": 1,
  "orderOk": false,
  "steps": [],
  "extraSteps": [],
  "missingSteps": [],
  "mismatches": [{"reason": "wrongOrder", "declared": ["info","export-text"], "observed": ["export-text","info"]}]
}
```

`kind` 와 `schemaVersion` 은 고정이다. 다른 도구의 리포트와 섞이지 않게 한다.

### mismatch 사유

| reason | 언제 |
|--------|------|
| `wrongCommand` | 같은 자리에서 계열이 다르다 (`info` vs `search`) |
| `wrongOrder` | 계열 다중집합은 같은데 순서가 다르다 |
| `wrongExit` | 계열·순서는 맞는데 종료 코드가 다르다 |
| `extraStep` | 관측이 선언보다 길다 (접두가 같거나 LCS 삽입) |
| `missingStep` | 관측이 선언보다 짧다 |
| `wrongPath` | `check_paths=True` 이고 `expectPath` 가 디스크에 없다 |
| `badSession` | 세션 파일이 없거나 JSON/스키마가 깨졌다 |
| `badTrace` | 트레이스가 없거나 JSONL/스키마가 깨졌다 |

인접 `del+ins` 는 `sub`(교체)로 접어 여분+누락으로 오인하지 않는다.
같은 다중집합의 순열은 `wrongOrder` 이지 `wrongCommand` 가 아니다.

재생 채점에서 `expectPath` 가 없어도 통과할 수 있다. 기록 직후 재검증
(`check_paths=True`)에서만 경로 부재가 `wrongPath` 가 된다.

## CLI

이 도구의 CLI 는 **Python 모듈 표면**이다. rhwp 바이너리의 하위명령이 아니다.

```
python gym/tools/agent_session.py validate --session S.json
python gym/tools/agent_session.py score-replay --session S.json --replay T.jsonl
python gym/tools/agent_session.py record --session S.json --bin target/debug/rhwp --out T.jsonl
```

공통: `--json` 이면 리포트를 JSON 으로 낸다. 없으면 한국어 한 줄+목록.

### validate

세션 정의만 본다. 바이너리 불요. 파일이 없거나 JSON 이 아니면 이슈 목록과
종료 1. 스키마 위반(빈 `id`, 빈 `steps`, 자리표 오류)도 종료 1.
성공이면 `kind=gymAgentSessionValidate`, `ok=true`.

### score-replay

기록된 JSONL 을 바이너리 없이 채점한다.

- `--session` 세션 JSON
- `--replay` 트레이스 JSONL
- `--input` 세션 `input` 덮어쓰기 (자리표 해석용, 디스크 필수 아님)
- `--sub-dir` 작업 폴더 (재생 합격에는 쓰지 않음)

`--bin` 인자가 없다. 구현이 PATH 에서 rhwp 를 찾지도 않는다. 단위 시험은
픽스처 두 파일만으로 통과·실패 사유를 고정한다.

로드 실패는 채점 리포트로 접힌다.

- 세션 쪽 예외 → `reason=badSession`
- 트레이스 쪽 예외 → `reason=badTrace`
- 세션을 읽은 뒤 트레이스가 깨지면 `sessionId` 는 유지한다

종료 0 = 경로 합격, 종료 1 = 불합격 또는 입력 오류.

### record

세션을 실행해 JSONL 을 쓴다. **`--bin` 이 필수**다.

- `--bin` 이 없거나 공백 → `RecordRefused`, 종료 2, 파일 없음
- `--bin` 경로가 파일이 아님 → `RecordRefused`, 종료 2, 파일 없음
- 세션/실행/쓰기 실패 → `SessionError` 계열, 종료 1
- 성공 후 자기 채점 리포트를 낸다 (`kind=gymAgentSessionRecord`)

주입 실행기(`record_session(..., execute=...)`)를 쓰는 단위 시험도
`--bin` 자리(존재하는 더미 파일)를 요구한다. 실행기를 바꿔도 위조 금지
계약은 남는다.

## 예외 계층

모든 계약 위반은 `SessionError`(ValueError 하위)다. CLI 와 시험은
`code` 로 유형을 가른다.

| 유형 | code | 종료 | 의미 |
|------|------|------|------|
| `SessionError` | `sessionError` | 1 | 분류되지 않은 계약 위반 |
| `RecordRefused` | `recordRefused` | 2 | `--bin` 없이 기록 위조 시도 |
| `SessionFileError` | `sessionFile` | 1 | 세션 JSON 을 열 수 없음 |
| `SessionParseError` | `sessionParse` | 1 | 세션이 UTF-8 JSON 이 아님 |
| `SessionSchemaError` | `sessionSchema` | 1 | id/steps/자리표 스키마 위반 |
| `TraceFileError` | `traceFile` | 1 | 트레이스 JSONL 을 열 수 없음 |
| `TraceParseError` | `traceParse` | 1 | 줄 파싱 실패 또는 이벤트 없음 |
| `TraceSchemaError` | `traceSchema` | 1 | ts/argv/exit/ok 필드 위반 |
| `ExecuteError` | `executeError` | 1 | 실행기 예외 또는 exit 미반환 |
| `WriteError` | `writeError` | 1 | JSONL 쓰기 실패 |
| `PlaceholderError` | `placeholderError` | 1 | 엄격 자리표 해석 실패 |

`to_dict()` 는 `type`, `code`, `message`, `exitCode` 와 선택 필드
`path` / `line` / `detail` 을 낸다. `classify_exception()` 은 세션 쪽을
`badSession`, 트레이스 쪽을 `badTrace` 로 접는다.

### 입출력 가드

- 빈 경로: 읽기면 `SessionFileError` / `TraceFileError`, 쓰기면 `WriteError`
- 없는 파일: 같은 유형, 메시지에 `찾을 수 없다`
- 디렉터리: Windows 는 `PermissionError` 가 나기도 하므로 `isdir` 을 우선한다
- UTF-8 이 아님: `SessionParseError` / `TraceParseError`
- JSON 깨짐: 세션은 `SessionParseError`, 줄 단위는 `TraceParseError`(줄 번호)
- 직렬화 불가 이벤트: `WriteError`
- 실행기 `RuntimeError`: `ExecuteError` 로 감싼다. 이미 `SessionError` 이면
  다시 감싸지 않는다
- 실행기 결과가 `exit` 없거나 정수가 아님: `ExecuteError`
- 작업 폴더/`{sub:}` 부모 생성 실패: `WriteError`

`validate_session` / `validate_trace` 는 이슈 목록을 돌려 예외를 던지지
않는다. 파일 로더(`load_session_file` / `load_trace_file`)가 이슈를
`SessionSchemaError` / `TraceSchemaError` 로 올린다. 채점 함수
`score_session` 은 예외 대신 `badSession` / `badTrace` 리포트를 낸다.

## 재생 없이 시험하는 방법

바이너리가 없는 환경(sparse checkout, CI 의 Python 전용 job)에서도
다음이 돌아가야 한다.

```
python -m unittest scripts.tests.test_gym_agent_session
python -m unittest scripts.tests.test_gym_agent_session_errors
python gym/tools/audit.py
```

통과 픽스처는 `info` → `export-text` 두 줄 JSONL 이다. 실패 픽스처는
잘못된 명령(`search`), 역순, 여분(`digest`), 누락, 종료 코드 2 다.
기록 경로는 존재하는 더미 파일 + 주입 실행기로만 시험한다. 없는
바이너리를 실행하지 않는다.

`audit.py` 는 pack 정합 감사다. 이 도구는 pack 을 추가하지 않으므로
기존 pack 전부가 계속 통과해야 한다.

## 작업 예

통과:

```
gym 에이전트 세션: inspect-then-export — 통과 (2/2 스텝, 순서·계열·종료 일치)
  [0] 일치 — 기대 info exit=0, 관측 info exit=0
  [1] 일치 — 기대 export-text exit=0, 관측 export-text exit=0
```

역순:

```
gym 에이전트 세션: inspect-then-export — 실패 (일치 0/2, 관측 2)
  [0] 불일치 — 기대 info exit=0, 관측 export-text exit=0
  [1] 불일치 — 기대 export-text exit=0, 관측 info exit=0
  순서 불일치: 선언 ['info', 'export-text'] / 관측 ['export-text', 'info']
```

기록 거절:

```
record 모드는 --bin 이 필요합니다. 바이너리 없이 트레이스를 위조하지 않습니다.
```

종료 코드는 2 이고 `--out` 경로는 생기지 않는다.

## 구현 위치

| 경로 | 역할 |
|------|------|
| `gym/tools/agent_session.py` | 도구 본체 (검증·채점·기록·CLI·예외) |
| `scripts/tests/test_gym_agent_session.py` | 경로 채점 계약 (통과·역순·여분·누락) |
| `scripts/tests/test_gym_agent_session_errors.py` | 예외·입출력·CLI 실패 |
| `gym/docs/agent_session.md` | 이 문서 |
| `mydocs/working/gym_agent_session.md` | 작업 노트 |

`cargo fmt --all` 은 이 변경에 필요 없다. Python 도구와 문서만 추가한다.
