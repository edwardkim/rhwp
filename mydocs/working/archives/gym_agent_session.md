---
kind: working
status: active
canonical: mydocs/working/gym_agent_session.md
last_verified: 2026-08-18
issue: 5206
pr: 5211
---

# gym 에이전트 세션 도구 작업 노트 (PR #5211)

## 한 줄

`feat/gym-agent-session` 초안(세션 정의 + JSONL 재생 채점, 약 1350줄)에
예외 계층·입출력 가드·실패 CLI 접기와 한국어 문서·경계 시험을 얹는다.
새 PR 을 열지 않고 같은 브랜치에 얹는다.

## 배경

이슈 #5206 · PR #5211 초안은 경로 채점의 핵심만 닫았다.

- 선언 세션 vs JSONL 트레이스를 계열·종료·순서로 대조
- `validate` / `score-replay` / `record` Python CLI
- 재생은 바이너리 없이 픽스처만, `record` 는 `--bin` 없으면 거절
- 단위 시험 30건, pack 미변경

초안에서 비어 있던 것:

- 파일 없음·디렉터리·UTF-8·JSON 깨짐을 한 `SessionError` 로만 접음
- 세션 실패와 트레이스 실패가 `score-replay` 에서 같은 `badTrace` 로 보임
- 실행기 예외·exit 미반환·쓰기 실패의 유형이 없음
- 한국어 사용 문서(`gym/docs`)와 작업 노트가 없음
- Windows 에서 디렉터리 open 이 `PermissionError` 로 나오는 경우 미분류

이 노트는 그 구멍을 메운 이유를 남긴다. 동작 계약의 정본은
`gym/docs/agent_session.md` 다.

## 범위

포함한 것:

- `gym/tools/agent_session.py` — 예외 계층, `wrap_io_error`,
  `classify_exception`, `fail_score_report`, 로더/기록기 가드,
  엄격 자리표, CLI 실패 분류
- `scripts/tests/test_gym_agent_session.py` — 렌더·재생 무바이너리
  계약·경로 검사·LCS 추가
- `scripts/tests/test_gym_agent_session_errors.py` — 예외 카탈로그와
  입출력·CLI 실패 전수
- `gym/docs/agent_session.md` — 한국어 사용 문서
- `mydocs/working/gym_agent_session.md` — 이 노트

넣지 않은 것:

- 새 rhwp CLI (`rhwp agent-session` 따위)
- pack·과제·기준 풀이·profiles·checks·coverage
- `gym/README.md` · `gym/PARK.md` 집계 문구
- `trajectory.py` 연동 (마지막 스텝 감사와 경로 채점은 별 도구)
- LLM-judge, 골든 경로 문자열 비교, stdout 본문 대조
- `cargo fmt --all` (Python·문서만)
- 기본 채점 경로(`score.py` / `certify.py`) 연결

## 설계 원칙

1. **재생은 바이너리 없이.** `score-replay` 서브파서에 `--bin` 이 없고
   구현이 PATH 를 보지 않는다. 단위 시험이 이 계약을 잠근다.
2. **기록은 위조하지 않는다.** `--bin` 없거나 파일이 아니면
   `RecordRefused`(종료 2). 주입 실행기도 `--bin` 자리를 요구한다.
3. **예외는 유형으로 가른다.** 없/권한/UTF-8/JSON/스키마/실행/쓰기를
   한 메시지로 뭉개지 않는다. CLI 는 유형을 `badSession` /
   `badTrace` / stderr 거절로 접는다.
4. **채점 함수는 예외 대신 리포트.** `score_session` 은 잘못된
   세션·트레이스를 `ok=false` 리포트로 낸다. 로더만 예외를 던진다.
5. **자리표 기본은 관대하다.** 재생이 작업 폴더 부재로 실패하지
   않는다. 엄격 모드는 별도 플래그.
6. **새 CLI 표면 금지.** `run[0]` 은 이미 있는 명령 이름일 뿐이고
   존재 여부를 바이너리에 묻지 않는다.

## 예외 계층을 나눈 이유

초안은 `SessionError` 와 `RecordRefused` 두 개만 있었다. 파일 없음과
JSON 깨짐과 스키마 위반이 같은 유형이면 `score-replay --json` 소비자가
재시도(파일 없음)와 수정(스키마)을 가릴 수 없다.

| 층 | 유형 | 소비자가 하는 일 |
|----|------|------------------|
| 파일 | `*FileError` | 경로를 고친다 |
| 파싱 | `*ParseError` | JSON/UTF-8 을 고친다 |
| 스키마 | `*SchemaError` | 필드·자리표를 고친다 |
| 실행 | `ExecuteError` | 바이너리·실행기를 본다 |
| 쓰기 | `WriteError` | 출력 경로·디스크를 본다 |
| 거절 | `RecordRefused` | `--bin` 을 준다. 위조하지 않는다 |

Windows 특이: 디렉터리를 `open` 하면 `IsADirectoryError` 대신
`PermissionError` 가 난다. `wrap_io_error` 는 `os.path.isdir` 을
우선해 "권한이 없다"로 오인하지 않는다.

UTF-8 BOM 은 JSONL 첫 줄을 깨뜨린다. `parse_trace_jsonl` 은 본문
선두 BOM 을 한 번 벗긴다. 줄 중간 BOM 은 그대로 두어 파싱 실패가 된다.

실행기가 이미 `SessionError` 를 던지면 다시 `ExecuteError` 로 감싸지
않는다. `RecordRefused` 가 실행기에서 나와도 거절로 남아야 한다.

## CLI 종료 코드

| 상황 | 종료 |
|------|------|
| 하위명령 없음 / argparse 사용법 | 2 |
| `RecordRefused` | 2 |
| 검증·채점 불합격, 파일/파싱/스키마/실행/쓰기 실패 | 1 |
| 합격 | 0 |

`validate` 의 파일 오류는 JSON 리포트(`issues`)로 나간다. `record` 의
거절·실패는 stderr 한 줄이다. `score-replay` 의 로드 실패는 채점
리포트(`mismatches[0].reason`)로 접힌다 — 세션 쪽은 `badSession`,
트레이스 쪽은 `badTrace`. 초안은 둘 다 `badTrace` 였다.

## 시험 지도

`scripts/tests/test_gym_agent_session.py` — 경로 채점 계약.

- 자리표 해석, 세션 검증, JSONL 파싱
- 통과 / 잘못된 명령 / 역순 / 여분 / 누락 / 종료 코드
- `record` 거절과 주입 실행기
- CLI validate / score-replay / record
- 렌더, 재생 무바이너리 계약, `check_paths`, LCS

`scripts/tests/test_gym_agent_session_errors.py` — 예외·입출력.

- 카탈로그 11코드, `to_dict`, 하위 유형
- `classify_exception` / `fail_score_report` / `wrap_io_error`
- 세션·트레이스 로더: 빈 경로, 없음, 디렉터리, UTF-8, JSON, 스키마
- JSONL: BOM, 배열 줄, 스칼라 줄, 빈 본문, 줄 번호
- 쓰기: 빈 경로, 디렉터리, 직렬화 불가, 빈 목록
- 엄격 자리표
- 실행기 예외 / exit 없음 / 비정수 exit / 시계 콜백
- CLI validate·score-replay·record 실패 종료 코드
- PATH 에서 rhwp 를 지워도 `score-replay` 가 통과

두 파일을 합쳐도 바이너리를 실행하지 않는다. 더미 파일은 `--bin`
자리만 채운다.

## 검증

```
python -m unittest scripts.tests.test_gym_agent_session
python -m unittest scripts.tests.test_gym_agent_session_errors
python gym/tools/audit.py
```

`cargo fmt --all` 은 돌리지 않는다. Rust 를 건드리지 않았고 sparse
checkout 에서 전체 fmt 는 범위 밖이다.

`audit.py` 는 pack 스키마·과제↔기준 짝·ID 고유를 본다. 이 작업은
pack 을 추가하지 않으므로 기존 18 pack 이 계속 통과해야 한다.

## 초안과의 호환

- `SessionError` / `RecordRefused` 이름은 유지. 새 유형은 하위 클래스
- `parse_trace_jsonl("")` 는 여전히 `SessionError` 로 잡을 수 있다
  (`TraceParseError` 하위)
- `require_record_bin` 메시지에 `--bin` / `찾을 수 없` / `위조` 유지
- 리포트 `kind=gymAgentSession`, `schemaVersion=1.0` 유지
- CLI 인자 이름은 그대로. `score-replay` 에 `--bin` 을 추가하지 않음
- 합격 축은 계열·종료·순서. stdout 본문 대조를 넣지 않음

깨뜨린 것:

- `score-replay` 가 세션 파일을 못 읽으면 이제 `badSession` 이다
  (초안은 `badTrace`). 초안 시험은 이 경로를 잠그지 않았다.
- `load_json_file` 의 JSON 실패는 `SessionParseError` 다. 메시지
  `JSON 파싱 실패:` 접두는 유지.

## 의도적으로 넣지 않은 확장

- 명령 화이트리스트 (`info`/`export-text` 만 허용). 이 도구는 계열
  문자열을 비교할 뿐 명령 사전을 갖지 않는다. 사전을 넣으면 새 CLI
  표면처럼 보인다.
- stdout 해시 대조. 해시는 추적용이고 재생 합격 축이 아니다.
- 세션 디렉터리 일괄 채점. 한 세션·한 트레이스가 단위다.
- pack 과제화. 기본 채점 경로에 연결하면 종점 채점과 섞인다.

## 커밋 단위

1. 예외 계층·입출력 가드 (도구 본체)
2. 경계 시험 (기존 + 예외 전용)
3. 한국어 문서 (`gym/docs` + 이 노트)

커밋 메시지는 한국어다. `git add -A` 는 쓰지 않는다. 같은 브랜치
`feat/gym-agent-session` 에만 push 하고 새 PR 을 열지 않는다.

## 크기 게이트

`git diff --shortstat upstream/devel...HEAD` 의 insertions 가
3000 이상이어야 한다. 초안 1350 + 예외/시험/문서. 숫자는 목적이
아니라 예외 분류와 실패 시험을 빠짐없이 잠근 결과다.
