---
kind: report
status: active
canonical: mydocs/report/task_m100_3918_promotion3_scan.md
last_verified: 2026-08-08
---

# 처리 결과 — `scan` 코퍼스 발견·분류 (#3918 승격 3호)

## 판정 — 나머지 실험 표면 8종 중 왜 scan 인가

| 표면 | 판정 | 근거 한 줄 |
|---|---|---|
| `caps` | 승격 불필요 | 본 CLI `capabilities` 가 이미 같은 축(자기서술 단일 출처)이다 |
| `pii-scan` | 승격 불필요 | 1호 재판정(#4112→#4184) — `inspect` 계열 기존 표면으로 충분 |
| `verify` | 완료 | 2호(#4113→#4186·#4192) |
| **`scan`** | **승격 (이번)** | `batch` 는 "경로 목록을 이미 갖고 있다"는 전제 — 목록을 만드는 표면이 없고, 매직↔확장자 대조·암호 선별은 셸 `find` 로 합성 불가(파서가 필요) |
| `fingerprint` | 후속 후보 | "같은 파일의 어제와 오늘" 드리프트 게이트는 실존 공백이나, 기준선 파일 관리(상태)가 붙어 scan 보다 비용이 크고 `evidence` 와 한 몸으로 판단하는 게 낫다 |
| `diff-text` | 보류 | `export-text --json` 2회 + 범용 diff 로 합성 가능 — 1호 재판정 정신(기존 표면으로 충분)에 가깝다 |
| `chunk-plan` | 보류 | 실행 수단(`digest --pages`)이 이미 있고 계획 계산은 `export-text` 쪽별 문자수로 합성 가능 |
| `doctor` | 보류 | `--version`·`capabilities`·표본 1회 파싱으로 합성 가능, 에이전트 파이프라인 상시 축이 아니다 |
| `evidence` | 보류 | `fingerprint`+`diff-text` 합성 보고서 — 구성 요소 승격 판단이 먼저다 |

## 분석 — 공백의 정확한 모양

`batch` 는 stdin 경로 목록 전제(#3918 공백 1번). 아카이브를 받은 에이전트가 그
목록을 만들려면 셸 `find` 를 쓸 수밖에 없는데, 그 경로에는 rhwp 만 아는 판정이
빠진다: 확장자 주장과 매직 감지의 대조(`.hwp` 를 자칭하는 hwpx/쓰레기), 암호
문서 선별, 파싱 가능성. rhwp-agent 실험 표면의 `scan`(#3922)이 이 축을 검증해
두었고, 그 승격이다.

## red → green

- **red 실측**: 구현 전 devel(5a4f26d0d) 바이너리에서
  `rhwp scan samples --json`
  → `오류: 알 수 없는 명령입니다 - scan` + exit 2.
- **green**: 구현 후 계약 8본 전부 통과 —
  `cargo test --release --test scan_contract` → **8 passed; 0 failed** (0.08s).

## 설계 판단

- **판정은 데이터**: 발견은 판정이 아니다 — 확장자 불일치·파싱 실패도 exit 0 의
  데이터(`extMismatch`·`probe.parseOk`)로 싣는다. 게이트 코드 3 없음. 실행 실패
  stdout 0 B + 1, 조립 오류 2(미지 옵션 침묵 무시 금지).
- **재사용만 한다**: `parser::detect_format`·`load_document`(전역 --password
  경로 공유)·`page_count`. 새 조회 로직 0 — rhwp-agent 쪽 코드를 옮기지 않고
  본체 관례(중앙 provenance 지도·`LoadError` 분류)로 재구성했다.
- **결정성**: 파일 순서는 경로 문자열 오름차순 고정, `--limit` 은 정렬 **뒤**
  적용 — 같은 트리는 언제나 같은 목록. 재현 가능한 코퍼스가 batch 파이프라인의
  전제다.
- **출처 표지**: 문서 파생 가능 값은 `files[].probe.error` 하나(파서가 문서
  바이트를 읽다 만든 문자열) — 중앙 지도에 등재하고, 표지는 그 필드가 실제로
  실린 호출에만 붙는다(rhwp-agent 의 인라인 선언 → 본체 중앙 지도로 이동).
- **등재 일습**: dispatch·`--help`·capabilities(recordFields 4종)·MCP
  `hwp_scan`(선택 인자는 `cli.optionalArgs` — mcp-serve 의 실행 경로가 해석하는
  자리)·업무 프로필 `아카이브검색`(#4186 CI 실패 교훈: 신규 MCP 도구는 프로필
  등재 필요).
- rhwp-agent 의 `--jsonl` 축은 이번에 승격하지 않는다 — 소비 경로는
  `--json` + `jq -r '.files[].path'` 로 충분하고, 스트리밍 축은 대형 코퍼스
  실측이 생기면 후속 판단(#4186 이 잔여 축을 #4192 로 미룬 선례).

## 실측 (재현: release 바이너리)

```
$ rhwp scan <코퍼스> --probe --json
{"schemaVersion":"1.0","roots":[...],"files":[
 {"path":".../a-정상.hwp","bytes":514560,"extFormat":"hwp","magicFormat":"hwp5",
  "extMismatch":false,"probe":{"parseOk":true,"needsPassword":false,"pageCount":3,"ms":…}},
 {"path":".../b-거짓말.hwp","extFormat":"hwp","magicFormat":"hwpx","extMismatch":true,…},
 {"path":".../d-깨짐.hwp","magicFormat":"unknown","extMismatch":true,
  "probe":{"parseOk":false,"needsPassword":false,"error":"…"}},…],
 "summary":{"total":4,"byFormat":{"hwp5":1,"hwpx":2,"unknown":1},"extMismatch":2,
  "probed":true,"probeFailed":1,"needsPassword":0,"truncated":false},
 "untrustedContent":true,"untrustedFields":["files[].probe.error"]}   → exit 0

$ rhwp scan 없는-폴더 --json      → stdout 0 B, exit 1
$ rhwp scan <코퍼스> --bogus      → 미지 옵션 거부, exit 2
```

## 무회귀

- Rust 게이트 전부 green: `scan_contract` 8 · `provenance_contract` 9(스윕에
  scan 호출 등재) · `capabilities_schema_contract` 17 ·
  `capabilities_subcommands_contract` 4 · `cli_json_contract` 31(help↔
  capabilities·MCP 왕복) · `agent_profile_router_contract` 8. clippy
  `--all-targets -- -D warnings` green, rustfmt(변경 파일) green.
- Node: `envelopes.ts` 재생성(ScanEnvelope, 봉투 34→35, `gen:check` 최신),
  `scan()` 래퍼 + argv 회귀, vitest **449 passed**(17 파일, 파리티 7 포함).
- Python: `scan()` 래퍼 + argv 회귀 + 커버리지 파리티 집합 등재, pytest
  **261 passed / 1 failed** — 그 1건은 base(devel)에도 있는 기존 red
  (`explain`·`export-plan-schema`·`export-agent-manifest` 래퍼 부재, 열린 PR
  #4196 이 수리 중; scan 은 통과 목록에 있다). 파리티 집합·`__all__` 삽입 줄은
  #4196 과 다른 줄로 골라 무충돌.
- 같은 파일(`envelopes.ts` 헤더 봉투 수)을 승격 2호 #4186 도 바꾼다 — 어느 쪽이
  먼저 머지되든 나중 쪽에서 `npm run gen:types` 1회로 수렴한다.
