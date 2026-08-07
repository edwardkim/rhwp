---
kind: guide
status: active
canonical: tools/agent-toolkit/README.md
last_verified: 2026-08-06
---

# 에이전트 자동화 도구킷 가이드

## 개요

**RHWP Agent Toolkit**은 [에이전트 실무 대체 예제집](agent_task_playbook.md)의
플레이북 패턴을 자동화하는 Python 워크플로 모음이다. 각 워크플로는 rhwp CLI 를
서브프로세스로 호출해 플레이북의 "에이전트 시퀀스"를 실행하고, 결과를 재독
대조로 검증한 뒤에만 성공(exit 0)을 보고한다. Python 3 표준 라이브러리만 쓴다.

옵션·봉투·종료 코드의 canonical reference 는
[tools/agent-toolkit/README.md](../../tools/agent-toolkit/README.md)다.

### 위치

```
tools/agent-toolkit/
├── lib/toolkit.py    # 공통 라이브러리
├── workflows/        # 워크플로 스크립트 5종
└── tests/            # 회귀 테스트
```

## 공통 계약

- **바이너리 해석**: `--rhwp-bin <경로>` > `RHWP_BIN` 환경변수 > PATH 의 `rhwp`
- **종료 코드**: 0 성공 / 1 실행·검증 실패 / 2 입력 오류
  / 3 (distribution_verify 전용) 두 문서가 다름
- **성공 시에만 산출물이 남는다**: 재독 검증에 실패하면 이번 호출이 새로 만든 파일만 지우고
  비 0 으로 끝낸다. rhwp `fill-fields` 는 `notFound` 가 있어도 exit 0 으로
  파일을 만들기 때문에, 종료 코드만 보는 자동화는 조용한 유실을 겪는다 —
  워크플로가 그 게이트를 대신 닫는다.
- **덮어쓰기 금지**: 출력 파일·보고서·수확 CSV가 이미 있으면 exit 2로 중단하고
  기존 파일을 보존한다. `bulk_sweep`은 기존 폴더를 쓸 수 있지만 생성할 NDJSON과
  `summary.json` 이름이 비어 있어야 한다.
- `--json` 은 요약 봉투 한 줄을 stdout 으로 낸다. 오류는 stderr.

## 워크플로

### 1. form_filling.py — 서식 자동 작성
- **플레이북**: 시나리오 1 (누름틀 채우기)
- **자동화**: `fields` → `edit fill-fields` → 산출물 `fields` 재독 → 값 기계 대조
- **실패 게이트**: `notFound`/`ambiguous`/`confusable` 비지 않음, `filledCount`
  부족, 재독 값 불일치 → 산출물 삭제 + exit 1

```bash
python3 tools/agent-toolkit/workflows/form_filling.py 서식.hwp 값.json -o 완성본.hwp
# 값.json = {"회사명": "한국수자원공사", "담당자[0]": "홍길동"}  (동명 필드는 이름[N])
```

### 2. table_harvest.py — 표 데이터 수확
- **플레이북**: 시나리오 2 (HWP 표 → CSV)
- **자동화**: `export-tables --json` → 표마다 `table-to-csv` → CSV 재독으로
  행/열 수를 격자 계약과 대조. 표 0개면 exit 1.

```bash
python3 tools/agent-toolkit/workflows/table_harvest.py 문서.hwp -o tables/ --bom
```

### 3. archive_search.py — 아카이브 검색
- **플레이북**: 시나리오 3·16 (여러 문서에서 조항 찾기)
- **자동화**: 디렉터리 수집 → `batch search --json` → 파일·페이지·문단·오프셋
  좌표 보고서. 매치 0건은 성공, 읽기 실패 파일은 `errors[]` 격리 + exit 1.
- **보고서 계약**: 저장 JSON에도 최종 `exit`와 `batch.exitCode`/`batch.stderr`를 남긴다.

```bash
python3 tools/agent-toolkit/workflows/archive_search.py 규정/ --query "위임전결" -o report.json
```

### 4. bulk_sweep.py — 대량 문서 스윕
- **플레이북**: 시나리오 4 (메타/본문/구조 일괄 수집)
- **자동화**: `batch info` → `--min-pages` 필터 → `batch export-text` /
  `batch export-structure` → 축별 NDJSON + `summary.json`
- **부분 실패 계약**: 성공분 NDJSON 보존, 실패 파일은 `failedSources` 목록,
  실패가 있으면 exit 1. 파일별 레코드 없이 batch 프로세스가 실패하면
  `batchFailures`로 남기고 exit 1.

```bash
python3 tools/agent-toolkit/workflows/bulk_sweep.py docs/ -o results/ --min-pages 10
```

### 5. distribution_verify.py — 배포본 동일성 검증
- **플레이북**: 시나리오 8 (변조·판본 확인)
- **자동화**: `render-diff A B --json` 기하 게이트 → 기하 동일 시 `export-svg`
  양쪽 → 페이지별 SVG 바이트 대조 (기하 게이트가 못 보는 이미지 내용 교체까지)
- **판정**: exit 0 동일 / 3 다름 / 1 판정 불능 / 2 입력 오류

```bash
python3 tools/agent-toolkit/workflows/distribution_verify.py 원본.hwp 배포본.hwp -o verdict.json
```

## 회귀 테스트

```bash
RHWP_BIN=target/debug/rhwp python3 tools/agent-toolkit/tests/test_workflows.py
```

실제 rhwp 바이너리 + `samples/` fixture 로 워크플로별 성공(산출물 생성 +
테스트의 독립 재독)·실패(없는 파일 exit 2, notFound/표 없음/부분 실패 exit 1,
동일성 다름 exit 3), 기존 출력 보존, batch 프로세스 실패 케이스 27건을 돌린다.

## 관련 문서

- [상세 문서 (canonical)](../../tools/agent-toolkit/README.md)
- [에이전트 실무 대체 예제집](agent_task_playbook.md)
- [CLI 명령어 매뉴얼](cli_commands.md)

---

**Last Updated**: 2026-08-06
