# #5511 Stage 2 기능군 배치 Q4 — scan·batch CQRS 경계

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 구현 시작 기준: `b96a1f5e5`
- 수행일: 2026-08-19
- 상태: 완료 — Q5 진입 승인 대기

## 1. 결과

메인테이너가 승인한 A안에 따라 `cmd_scan`을 먼저 책임별로 분해하고, scan과 batch pipeline을
CQRS 소유권에 맞는 여섯 모듈로 이동했다.

| 모듈 | 책임 | 최종 줄 수 |
|---|---|---:|
| `cli/queries/scan.rs` | 코퍼스 발견·분류 query | 356 |
| `cli/batch/mod.rs` | batch 인자·stdin orchestration과 record routing | 429 |
| `cli/batch/ordered.rs` | bounded reorder buffer·NDJSON 순서·exit 집계 | 143 |
| `cli/batch/query.rs` | read-only per-document projection | 182 |
| `cli/commands/batch_fill.rs` | form fill·직렬화 command | 504 |
| `cli/commands/batch_convert.rs` | HWP5 변환·쓰기 command | 160 |

`src/main.rs`는 Q4 시작의 34,495줄에서 32,821줄로 1,674줄 줄었다. 모든 새 모듈은
1,200줄 상한 이하다. `fill`과 `convert`는 파일을 쓰므로 command 경계가 소유하고,
info·structure·tables·fields·search·text·extract-data record는 read-only batch query가 소유한다.
ordered runtime은 문서 의미를 모르고 입력 순서·역압·집계만 담당한다.

single CLI·batch·MCP가 함께 쓰는 `info_json_value`, `structure_json_value`, `tables_json_value`,
`fields_json_value`, `search_json_value`, `extract_data_json_value`는 root에 한 번만 남겼다. 이를 batch
안으로 옮기거나 복제해 schema 정본을 갈라놓지 않았다.

## 2. A안 책임 분해와 보호 계약

이동 전에 `cmd_scan`의 option parsing, 재귀 walk, file probe·record, summary, 사람·JSON 출력을
서로 다른 함수로 분리했다.

| handler | 분해 전 CC | 분해·이동 후 판정 |
|---|---:|---|
| `cmd_scan` | 28 | 대상 모듈 CC>25 경고 없음 |

기존 계약에 없던 두 축도 먼저 고정했다.

- 사람용 출력의 제목·파일행·확장자 불일치 note·합계 바이트
- Unix 디렉터리 walk의 파일·폴더 symlink 비추적과 중복 root dedup

새 scan 계약은 분해 전과 이동 후 모두 10/10 통과했다. 최종 Q4 인접 계약 94/94도 통과해
JSON/NDJSON, 입력 순서, threads 1/3/8 바이트 결정성, record 실패 격리, fill·convert 파일 쓰기,
exit aggregation과 single-command schema 동형성이 유지됐다.

## 3. 커밋 계보

| 커밋 | 역할 |
|---|---|
| `49d998d9c` | Q4 inventory·복잡도 중단·A안 선택지 기록 |
| `6d462fffa` | scan 사람 출력·symlink traversal characterization |
| `bbbda1d22` | `cmd_scan`을 제자리에서 CC 25 이하로 책임 분해 |
| `4b4ffa1a8` | scan query adapter 물리 이동 |
| `51c32ad93` | ordered batch·query·fill/convert command CQRS 이동 |

## 4. 최종 검증

| 검증 | 결과 |
|---|---|
| scan characterization 분해 전·이동 후 | 각각 10/10 통과 |
| 이동 후 Q4 focused | 94/94 통과 |
| release-test 전체 nextest | 7,826/7,826 통과, 4 slow, 38 skipped, 184.785초 |
| 대상 모듈 CC 25 상한 | 경고 없음 |
| `cargo fmt --all -- --check`·`git diff --check` | 통과 |
| `cargo check --all-targets` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --doc` | 8/8 통과, 3 ignored |
| integration manifest 정책 자체 계약 | 17/17 통과 |
| 최신 base manifest check | 756 sources / 3,777 static test attrs / 44/48 integration targets, 통과 |
| unit-tier 정책 자체 계약과 base check | 12/12, 4,225 tests / 298 modules, 통과 |
| CI impact Node·workflow 계약 | 62/62, 30/30 통과 |
| Markdown link check | 기존 capability 등록부 무결성 오류 16건, Q4 신규 오류 없음 |

검증 준비가 `Cargo.toml`에 만든 세 singleton integration target과 Cargo가 재정렬한 lockfile package
순서는 추적 변경에서 제거했다. 새 integration source나 generated test target은 커밋하지 않았다.

로컬 nextest 0.9.137은 저장소 권고 0.9.140보다 낮다는 경고를 냈지만, 전체 모집단을 정상 실행해
전건 통과했다. 이번 변경은 CLI adapter 제어 흐름과 물리 위치만 바꾸며 parser·serializer·renderer·
layout·WASM 경계를 수정하지 않는다. 따라서 시각 sweep과 WASM 빌드 발생 조건에는 해당하지 않는다.

## 5. 원격 병합 위험

최종 fetch 기준 `origin/devel`과 `upstream/devel`은 `b96a1f5e5`로 같고, 구현 HEAD는 5커밋 앞·
0커밋 뒤다. 열린 devel 대상 PR 중 #5617만 `src/main.rs`를 변경하지만 hunk는 진단용 help 문구로
Q4 구현 경계와 직접 겹치지 않는다.

이 판정은 시점 증거다. 로컬 devel 통합과 admin push 직전에 exact base SHA·PR head·merge-tree를
다시 확인한다. 이 보고서 작성 시점에는 remote push를 수행하지 않았다.

## 6. 다음 승인 단위

다음 기능군은 Q5 `info·dump-pages·dump-controls`다. CC 68인 `dump_controls`를 그대로 옮기지 않고
진단 query의 관찰 가능한 stdout/stderr·exit·JSON 계약과 공유 seam을 먼저 inventory한다. Q5는
메인테이너의 Q4 배치 종료 승인과 진입 승인 전 시작하지 않는다.
