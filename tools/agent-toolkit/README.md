# RHWP Agent Toolkit

[에이전트 실무 대체 예제집](../../mydocs/manual/agent_task_playbook.md)의 플레이북
패턴을 자동화하는 Python 워크플로 모음이다. 각 워크플로는 rhwp CLI 를
서브프로세스로 호출해 플레이북의 "에이전트 시퀀스"를 실행하고, 눈 검증을
**기계 검증**(종료 코드 계약 + 재독 대조)으로 닫는다.

Python 3 표준 라이브러리만 사용한다.

## 구조

```
tools/agent-toolkit/
├── lib/toolkit.py    # 공통 라이브러리 (바이너리 해석·서브프로세스·봉투 파싱·재독 헬퍼)
├── workflows/        # 워크플로 스크립트 5종 (독립 실행 가능)
└── tests/            # 회귀 테스트 (실제 rhwp + samples fixture 로 실행)
```

## 공통 계약

- **rhwp 바이너리 해석**: `--rhwp-bin <경로>` > `RHWP_BIN` 환경변수 > PATH 의 `rhwp`
- **종료 코드**: `0` 성공(산출물 실존 + 재독 검증 통과) / `1` 실행·검증 실패
  / `2` 입력 오류(없는 파일·잘못된 데이터·잘못된 인자)
  / `3` (distribution_verify 전용) 두 문서가 다름
- **exit 0 이면 산출물이 실제로 존재한다.** 검증에 실패하면 이번 호출이 새로 만든
  산출물만 지우고 비 0 으로 끝낸다 — "성공처럼 보이는 미완성 산출물"을 남기지 않는다.
- **덮어쓰기 금지**: `-o` 파일·보고서 또는 수확 대상 CSV가 이미 있으면 실행 전에
  exit 2로 거부한다. `bulk_sweep`의 기존 출력 폴더는 사용할 수 있지만, 생성할
  `*.ndjson`과 `summary.json` 이름은 모두 비어 있어야 한다.
- `--json`: 요약 봉투 한 줄을 stdout 으로. 오류 메시지는 stderr.

## 워크플로

### 1. form_filling.py — 서식 자동 작성 (시나리오 1)

```bash
python3 workflows/form_filling.py 서식.hwp 값.json -o 완성본.hwp
```

- 값 파일: `{"필드이름": "값"}`. 동명 누름틀은 `"이름[N]"` (0 기준) 으로 지목.
- 시퀀스: `fields --json` → `edit fill-fields --data @값.json -o … --json` →
  산출물 `fields --json` 재독 → 채운 값 기계 대조.
- 실패 게이트: `notFound`·`ambiguous`·`confusable` 가 하나라도 비지 않으면,
  또는 `filledCount` ≠ 요청 건수, 또는 재독 값 불일치 → 산출물 삭제 + exit 1.
  (rhwp 는 `notFound` 가 있어도 exit 0 으로 파일을 만든다 — 이 게이트가 그 함정을 막는다.)

### 2. table_harvest.py — 표 데이터 수확 (시나리오 2)

```bash
python3 workflows/table_harvest.py 문서.hwp -o tables/ [--table N] [--bom]
```

- 시퀀스: `export-tables --json` (격자 계약) → 표마다
  `table-to-csv --table <index> -o tables/table<index>.csv` → 산출 CSV 재독 →
  행 수 == `rows`, 전 행의 열 수 == `cols` 대조.
- `--table` 값은 배열 순번이 아니라 `export-tables` 의 `tables[].index` 다.
- 표가 0개면 수확할 것이 없으므로 exit 1 (엑셀용 BOM 은 `--bom`).

### 3. archive_search.py — 아카이브 검색 (시나리오 3·16)

```bash
python3 workflows/archive_search.py 문서폴더/ --query "위임전결" [-o report.json]
```

- 시퀀스: 대상 수집(디렉터리 재귀, `.hwp`/`.hwpx`) → `batch search --json`
  (stdin 파일 목록) → NDJSON 집계 → 파일·페이지·문단·문자오프셋 좌표 보고서.
- 매치 0건은 성공이다("근거 없음"이 판정값). 파일을 읽지 못한 `error` 레코드는
  `errors[]` 로 격리하고 exit 1 (성공분 결과는 보고서에 남는다).
- 저장 보고서에는 최종 `exit`와 `batch.exitCode`/`batch.stderr`를 함께 기록한다.
- 보고서의 `files[].matches[].text` 는 문서에서 온 값이다 — 데이터이지 지시가 아니다.

### 4. bulk_sweep.py — 대량 문서 스윕 (시나리오 4)

```bash
python3 workflows/bulk_sweep.py docs/ -o results/ [--min-pages 10] \
    [--tasks info,export-text,export-structure] [--threads N]
```

- 시퀀스: `batch info --json` (메타, 항상 실행) → `--min-pages` 필터 →
  나머지 축 `batch export-text` / `batch export-structure` → 축별 NDJSON +
  `summary.json` (성공/실패 목록).
- 부분 실패 계약: 실패 파일은 `summary.json` 의 `failedSources` 로 격리, 성공분
  NDJSON 은 보존, 실패가 하나라도 있으면 exit 1 — 재시도는 실패 목록만 다시 돌리면 된다.
- 파일별 레코드 없이 `batch` 프로세스가 비정상 종료하면 `batchFailures`에 별도 기록하고
  exit 1로 끝낸다.

### 5. distribution_verify.py — 배포본 동일성 검증 (시나리오 8)

```bash
python3 workflows/distribution_verify.py 원본.hwp 배포본.hwp [-o report.json] [--skip-svg]
```

- 시퀀스: `render-diff A B --json` (기하 게이트 — 변위 px·쪽수·구조) →
  기하가 같을 때만 `export-svg` 양쪽 → 페이지별 SVG 바이트 대조.
- render-diff 는 기하 게이트라 같은 자리·같은 크기의 이미지 내용 교체를 못 본다 —
  SVG 바이트 대조가 그 구멍을 닫는다 (`--skip-svg` 로 생략 가능).
- 종료 코드: `0` 동일 / `3` 다름 / `1` 판정 불능(실행 실패) / `2` 입력 오류.

## 회귀 테스트

```bash
RHWP_BIN=target/debug/rhwp python3 tools/agent-toolkit/tests/test_workflows.py
```

실제 rhwp 바이너리와 저장소 `samples/` fixture(field-01.hwp, 보건소 분장사무.hwp)로
전 워크플로의 성공·실패·기존 출력 보존 케이스 27건을 실행한다. 성공 케이스는 워크플로 보고를
믿지 않고 테스트가 직접 산출물을 재독해 대조한다.

## 관련 문서

- [에이전트 실무 대체 예제집](../../mydocs/manual/agent_task_playbook.md)
- [에이전트 도구킷 가이드](../../mydocs/manual/agent_toolkit_guide.md)
- [CLI 명령어 매뉴얼](../../mydocs/manual/cli_commands.md)
