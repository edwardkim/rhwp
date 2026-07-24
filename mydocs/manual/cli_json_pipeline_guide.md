---
kind: guide
status: active
canonical: mydocs/manual/cli_commands.md
last_verified: 2026-07-24
---

# CLI JSON 파이프라인 가이드 — AI 에이전트·스크립트에서 rhwp 사용하기

`--json` 출력(#3237)과 `batch` 서브커맨드(#3238)로 rhwp 를 **기계가 소비하는 파이프라인
도구**로 쓰는 방법을 다룬다. 옵션의 canonical reference 는
[CLI 명령어 매뉴얼](cli_commands.md)이고, 본 문서는 검증된 사용 시나리오를 제공한다.
아래 모든 예시는 저장소 `samples` 코퍼스(HWP5·HWPX·HWP3·HML 271건)에서 실제 실행해
확인한 것이며, 표기한 수치는 32코어 Windows 릴리스 빌드 실측이다.

## 목적 — 왜 JSON 계약인가

에이전트·스크립트가 문서 도구를 소비하는 방식은 두 가지 계약으로 결정된다.

1. **실패의 계약** — 종료 코드(0 성공/1 런타임/2 사용법, #2707). `&&`·`set -e`·CI 게이트가 성립한다.
2. **성공의 계약** — 출력 스키마. `schemaVersion` 이 박힌 JSON 이라 파싱이 문구 변경에 깨지지 않는다.

이 두 계약 위에서 HWP 는 `jq`·인덱서·DB·LLM 파이프라인의 1급 입력이 된다.
규칙은 하나다: **stdout 은 데이터(JSON/NDJSON)만, 진단·진행·요약은 전부 stderr.**

## 계약 요약

| 명령 | stdout | 종료 코드 |
|---|---|---|
| `info --json <f>` | 문서 메타 JSON 1개 | 0 / 1 / 2 |
| `export-text --json <f> [-p N]` | 페이지 텍스트 JSON 1개 | 0 / 1 / 2 |
| `export-structure --json <f> [--mode m]` | 구조 봉투 JSON 1개 (한 줄) | 0 / 1 / 2 |
| `batch <export-text\|info\|export-structure> --json [--threads N]` | 파일당 NDJSON 1줄, **stdin 입력 순서 보존** | 0 / 전건 성공 아니면 1 / 2 |

- 실패한 단건 명령의 stdout 은 **0바이트**다 — 부분 JSON 을 파싱할 일이 없다.
- batch 의 건별 실패(읽기·파싱·추출·panic)는 `{"schemaVersion","source","error","exitClass":"runtime"}`
  레코드로 스트림에 남고, 하나라도 실패하면 최종 종료 코드가 1 이다.
- 스키마는 필드 **추가만** 허용된다(변경·삭제는 `tests/cli_json_contract.rs` 가 CI 에서 실패시킴).

## 시나리오 0 — 에이전트 온보딩 (도구 발견)

에이전트가 rhwp 를 처음 만나면 help 파싱도 플래그 추측도 필요 없다 — 한 번의 호출로
도구 전체를 발견한다.

```bash
rhwp capabilities | jq '{version, machineReadable: [.commands[]|select(.json==true)|.name], batch: .batch.subcommands}'
```

```console
{"version":"0.7.19","machineReadable":["info","export-text","export-structure","capabilities","batch"],"batch":["export-text","info","export-structure"]}
```

`exitCodes`·`jsonContract`(stdout 순수성·스키마 정책·실패 규약)·명령별 `recordFields` 가
전부 들어 있어, 에이전트 도구 정의(함수 호출 스키마·MCP 도구 목록)를 여기서 자동 생성할 수 있다.

## 시나리오 1 — 아카이브 선별 후 추출 (에이전트의 첫 작업)

에이전트가 아카이브를 만나면 첫 질문은 "무슨 파일이, 몇 페이지, 어떤 포맷인가"다.
본문 추출보다 메타 스윕이 압도적으로 싸다 (동일 271건 실측: info 3.0s vs export-text 67.4s).

```bash
# ① 메타데이터 스윕 — 271건 3.0s
find docs/ -name '*.hwp' | rhwp batch info --json > meta.ndjson

# ② 조건 선별 — 예: 10페이지 이상만
jq -r 'select(.pageCount >= 10) | .source' meta.ndjson > targets.txt

# ③ 선별본만 본문 추출
rhwp batch export-text --json < targets.txt > corpus.ndjson
```

실측: 270건 스윕 → 75건 선별 → 75건 추출 6.2s, 전건 성공, 총 5,205페이지.

## 시나리오 2 — 검색/RAG 인덱싱용 청킹

`export-text --json` 의 `pages[]` 는 페이지 단위 청킹을 공짜로 준다.

```bash
rhwp export-text --json 문서.hwp | jq -c '.pages[] | {page, text}'
```

```console
{"page":0,"text":"￼Creating Linux Virtual Servers\n..."}
{"page":1,"text":"..."}
```

대량 처리는 `batch export-text` 의 문서 단위 레코드(`text` = 전체 텍스트)를 쓰고,
페이지 단위가 필요한 문서만 단건 `--json` 으로 내려받는 조합이 실측상 가장 싸다.

## 시나리오 3 — 실패를 흘리지 않는 배치 (CI·무인 운영)

```bash
rhwp batch export-text --json < list.txt > out.ndjson
if [ $? -ne 0 ]; then
  jq -c 'select(.error) | {source, error}' out.ndjson   # 실패 건만 추출해 보고
fi
```

```console
{"source":"손상되거나_없는파일.hwp","error":"파일을 읽을 수 없습니다: ... (os error 2)"}
```

- 성공분은 이미 `out.ndjson` 에 있다 — 재시도는 실패 건만 하면 된다.
- 한 건의 파서 panic 도 해당 파일의 `error` 레코드로 격리된다(배치는 죽지 않는다).
- 소비자가 파이프를 끊으면(`| head` 등) rhwp 는 작업을 정리하고 1 로 끝난다.

## 시나리오 4 — 문서 구조 스윕 (조문 DB화·구조 기반 청킹)

평문 텍스트로는 조문("제3조 2항")·개요 계층이 사라진다. `export-structure` 축은
문서의 **구조 트리**를 주고, batch 로 아카이브 전체의 구조 지도를 만든다.

```bash
# 아카이브 전체 구조 스윕 → 모드(조문/개요)별 분포
find docs/ -name '*.hwp' | rhwp batch export-structure --json > structure.ndjson
jq -s 'group_by(.mode) | map({mode: .[0].mode, files: length, nodes: (map(.nodeCount)|add)})' structure.ndjson
```

```console
[{"mode":"clause","files":258,"nodes":7239},{"mode":"outline","files":13,"nodes":905}]
```

실측: 271건 구조 스윕 2.7s, 전건 성공. 조문 구조 문서만 골라 조문 단위로 청킹하면
"제N조" 검색·인용이 좌표째 성립한다 — 평문 RAG 가 못 하는 것.

## 성능 특성 (실측)

| 워크로드 | 결과 |
|---|---|
| 메타 스윕 (`batch info`) 271건 | 3.0s |
| 본문 추출 (`batch export-text`) 270건 | 7.0s — 파일당 프로세스 순차 45.1s 대비 6.4배 |
| 1,350건 연속 처리 | 27.8s (지속 48.6건/s) |

- 병렬 막차(makespan)는 **가장 느린 파일**이 결정한다. 극단 문서 1건(단독 57.5s)이
  섞이면 전체가 그에 수렴한다 — 시나리오 1 처럼 메타 스윕으로 먼저 선별하라.
- `--threads` 기본값은 CPU 코어 수다. 파일 간 병렬 효율은 8~16스레드에서 포화하는
  것이 관측되었다(엔진 수준 주제, 후속 이슈 예정) — 그 이상은 올려도 손해는 없다.
- 출력 순서는 병렬에서도 stdin 입력 순서와 같다. 완료-미방출 버퍼는 `threads×8` 로
  상한되므로 수십만 건을 흘려도 메모리는 평평하다.

## 주의사항

- **stdout 만 파싱하라.** 렌더 진단(`LAYOUT_OVERFLOW` 등)·진행 메시지·batch 요약은 stderr 다.
- `export-text --json` 의 `page` 는 `-p` 와 같은 **0 기준**이다.
- `--json` 모드는 파일을 쓰지 않는다 — `-o` 는 무시된다.
- `version` 은 HML 문서에서 `null` 이다 (HWPML 은 바이너리 버전 헤더가 없다).
