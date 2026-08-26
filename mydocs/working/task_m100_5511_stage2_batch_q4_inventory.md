# #5511 Stage 2 기능군 배치 Q4 — scan·batch inventory와 복잡도 중단

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 시작 기준선: `b96a1f5e5`
- 수행일: 2026-08-19
- 상태: 중단 조건 발동 — 책임 분해안 승인 대기

## 1. 실제 범위와 CQRS 소유권

Q4 후보는 `src/main.rs`의 1,953줄(`cmd_scan` 시작부터 batch용 공용 query 봉투
끝까지)이다. 물리적으로 한 덩어리지만 책임은 다섯 가지다.

| 현재 범위 | 약식 줄 수 | 책임 | 이동 후보 |
|---|---:|---|---|
| `cmd_scan` | 286 | 파일 발견·분류 query | `cli/queries/scan.rs` |
| `run_batch`·ordered stream | 478 | NDJSON 입력·병렬 실행·순서 복원 | `cli/batch/mod.rs`, `cli/batch/ordered.rs` |
| `run_batch_fill`·row parser | 497 | 문서 변경·직렬화 command | `cli/commands/batch_fill.rs` |
| batch query·convert record | 384 | query projection과 state-changing convert 혼합 | `cli/batch/query.rs`, `cli/commands/batch_convert.rs` |
| info 등 공용 value builder | 308 | single CLI·batch·MCP 공유 seam | 해당 query 소유 모듈 또는 현재 위치 유지 |

`fill`과 `convert`는 파일을 쓰므로 query 모듈로 이동할 수 없다. 반대로 info·structure·tables·
fields·search·export-text·extract-data record는 read-only projection이다. ordered stream은 두 쪽이
공유하되 문서 의미를 알지 않는 실행 기반으로 한정해야 한다. single CLI와 MCP도 쓰는 value builder를
batch 안으로 끌어오거나 복제하면 새 순환 의존과 스키마 분기가 생기므로 Q4 이동 대상에서 제외한다.

이 경계대로 나누면 어느 새 모듈도 1,200줄을 넘지 않는다. 다만 실제 이동 전에 scan 내부 책임
분해가 필요하다는 중단 조건이 아래에서 확인됐다.

## 2. 보호 계약 inventory

현재 Q4 인접 integration target 여섯 개에는 총 92개 테스트가 있다.

| target | 테스트 수 | 직접 보호하는 축 |
|---|---:|---|
| `scan_contract` | 8 | format 분류, 경로 정렬, probe, depth, limit, provenance, exit/stdout |
| `batch_parallel_determinism_contract` | 3 | threads 1/3/8 바이트 동일성, 입력 순서, record 실패 격리, deadlock 경계 |
| `batch_axes_contract` | 17 | query schema 동형성, 순서·부분 실패, convert 쓰기·충돌·exit aggregation |
| `batch_extract_data_contract` | 8 | single command 동형성, 문서별 limit·kind, 순서·실패 격리 |
| `batch_fill_contract` | 25 | JSONL/CSV row, 이름·경로 안전, dry-run, verify, 병렬 순서, MCP/help |
| `cli_json_contract` | 31 | 공통 JSON/NDJSON 봉투와 기존 batch 축 |

Q4 핵심 불변식인 NDJSON 입력 순서, 병렬 결정성, per-record failure isolation은 이미 전용 계약이
있다. scan도 JSON 경로는 강하게 보호된다. 그러나 리팩터링 전에 다음 두 characterization 공백을
먼저 고정하는 편이 안전하다.

- scan의 사람이 읽는 기본 출력 문구·요약·확장자 불일치 note
- Unix에서 파일·디렉터리 symlink를 따라가지 않는 traversal 안전성과 중복 root dedup

이는 새 기능이 아니라 현재 구현의 관찰 가능한 동작을 고정하는 계약이다. 승인안 A에서는 두 계약을
독립 커밋하고 기존 92개 focused 범위의 이동 전 기준선을 먼저 만든다.

## 3. 중단 조건 증거와 원인

`cargo clippy --bin rhwp -- -W clippy::cognitive_complexity` 계측은 다음 Q4 handler를 보고했다.

| handler | CC | 상한 | 판정 |
|---|---:|---:|---|
| `cmd_scan` | 28 | 25 | 중단 |

`cmd_scan`은 option parsing, 재귀 walk, 확장자·magic 판정, probe, 통계 집계, JSON·사람 출력을
한 함수 안에서 수행한다. 함수 안의 nested helper가 줄 수는 감췄지만 의사결정 책임을 줄이지는
못했다. 이 함수를 그대로 `cli/queries/scan.rs`로 옮기면 root의 줄 수만 줄이고 복잡도를 새 위치에
숨기므로 마스터 계획의 중단 조건에 정확히 해당한다.

`run_batch`, ordered stream, fill·convert·query record 함수에는 이번 기본 상한의 Q4 경고가
없었다. 제품 코드는 아직 변경하지 않았다. 계측이 다시 정렬한 `Cargo.lock` 패키지 두 항목도
원문으로 복원해 작업 트리를 오염시키지 않았다.

## 4. 선택지

### A. scan 책임 분해 후 CQRS 경계로 이동 — 권장

같은 Q4 안에서 `cmd_scan`을 먼저 제자리에서 다음 책임으로 나누고 각 함수 CC를 25 이하로 만든다.

- `ScanOptions` parsing·usage validation
- 파일 root 수집과 symlink 비추적 재귀 walk
- 단일 파일 magic/probe record 생성과 summary 집계
- JSON 또는 사람 출력

characterization → 제자리 분해 → focused 검증을 각각 독립 커밋한 뒤, scan·ordered runtime·batch
query·fill command·convert command 경계로 이동한다. 공용 value builder는 복제하지 않는다.
최종 HEAD에서 같은 focused 범위와 전체 release-test·정적·정책 게이트를 수행한다. stdout/stderr,
JSON/NDJSON, 파일 바이트, 종료 코드가 달라지면 즉시 중단한다.

### B. batch만 먼저 이동

고복잡도 `cmd_scan`은 root에 남기고 나머지 batch 책임만 이동한다. 즉시 위험은 작지만 Q4가 둘로
갈라지고 scan query 소유권과 root 감축 목표를 달성하려면 별도 승인 배치를 반복해야 한다.

### C. Q4 보류

제품·계약 변경 없이 다음 기능군으로 넘어간다. 위험은 없지만 scan·batch 책임 약 1,953줄과
write/read 혼합이 root에 남아 Stage 2 종료 조건을 달성할 수 없다.

## 5. 원격 위험

최종 fetch 기준 `HEAD`, `origin/devel`, `upstream/devel`은 모두 `b96a1f5e5`이고 divergence는
0/0이다. 열린 devel 대상 PR은 30개다. 그중 #5617만 `src/main.rs`를 변경하며, 변경 hunk는
진단용 help 문구 9줄로 Q4 본문과 직접 겹치지 않는다. 그러나 파일 수준 교집합은 존재하고 PR이
병합되면 기준 SHA가 바뀌므로 구현 재개 직전과 각 push 직전에 exact base·PR head·merge-tree를
다시 확인한다.

remote push는 수행하지 않았다.

## 6. 승인 요청

권장안 A는 `cmd_scan`의 복잡도를 새 파일에 숨기지 않고 현재 동작을 계약으로 고정한 뒤 CQRS
소유권에 맞게 Q4 전체를 이동한다. 제품 기능이나 출력 규약은 바꾸지 않는다. 승인되면 두 scan
characterization, 제자리 책임 분해와 focused 검증을 먼저 수행하고, 그 결과가 안정적일 때만
batch 기능군 이동으로 이어간다.
