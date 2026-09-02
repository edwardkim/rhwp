# #5511 Stage 2 기능군 배치 Q5 — info·page·control 진단 경계

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 구현 시작 기준: `52d8bf8eb3c3351cbabba00ce2b4e299d1930c01`
- 최종 통합 기준: `upstream/devel` `61baa6783`
- 수행일: 2026-08-19
- 상태: 완료 — Q6 진입 승인 대기

## 1. 결과

`info`, `dump-pages`, `dump`의 read-only 진단 adapter를 `src/main.rs`에서 분리했다. CC 68이던
`dump_controls`는 그대로 옮기지 않고 순회, shape, table, story 출력 책임으로 나눴다.

| 모듈 | 책임 | 최종 줄 수 |
|---|---|---:|
| `cli/queries/info.rs` | info 인자·문서 meta 사람 출력·공유 JSON seam 호출 | 400 |
| `cli/queries/page_dump.rs` | page filter·compat 옵션·JSON/사람 page dump | 118 |
| `cli/queries/control_dump/mod.rs` | dump 인자·문서/구역/문단 순회·완료 집계 | 576 |
| `cli/queries/control_dump/shape.rs` | 도형 공통 속성·재귀 shape 출력 | 343 |
| `cli/queries/control_dump/table.rs` | 표·셀·중첩 표 출력 | 191 |
| `cli/queries/control_dump/story.rs` | master page·header/footer story 출력 | 299 |

모든 새 모듈은 1,200줄 상한 이하다. `src/main.rs`는 Q5 시작의 32,821줄에서 31,095줄로
1,726줄 줄었다. Q5 이동 전 CC 34였던 `show_info`와 CC 68이었던 `dump_controls`는 책임 helper로
분해했고, 대상 모듈에서 CC 25 초과 경고가 남지 않았다.

single CLI·batch·digest·MCP가 함께 쓰는 `info_json_value`는 root의 schema 공유 seam으로
보존했다. vector output과 diagnostics가 함께 쓰는 `hu_to_mm`·`hu_to_mm_i`도 복제하거나 Q5
모듈 안으로 숨기지 않았다. parser·serializer·renderer·layout·WASM 구현은 Q5 커밋에서
변경하지 않았다.

## 2. 보호 계약과 최신 devel 정합

이동 전에 대표 HWP3 fixture의 경로만 정규화해 `info`, `dump-pages -p 0`, 필터형 `dump`의 성공
stdout 전체를 SHA-256으로 고정했다. 구현 완료 시 Q5 인접 계약 127/127이 통과해 사람 출력,
JSON schema, flag 거부, exit code, HML, batch·MCP 동형성이 유지됐다.

완료 직전 원격 `devel`이 `52d8bf8eb3`에서 `61baa6783`으로 전진했다. merge-tree는 충돌 없이
생성됐고 Q5 경로와 겹치는 원격 파일은 없었으므로 정상 merge commit으로 흡수했다. 결합 후
필터형 `dump` 계약 한 건만 다음과 같이 변했다.

| 기준 | 정규화 stdout SHA-256 |
|---|---|
| Q5 시작 | `bb27d62a90f3deec83bf8b1a8270680baaf9253cb6b511e6f052fdc0422957ca` |
| 최신 devel 결합 | `45f876bb12042d4a6539780c4eaf043975fca51e21e479c80fd79e975ea8641d` |

원인은 원격 #5542 변경이 HWP3 구역 첫 문단에 `SectionDef`를 합성하고 control-slot 좌표를
계상한 것이다. 이에 따라 `dump --section 0 --para 0`에 구역정의 행과 정합화된 문자 위치가
추가됐다. Q5 코드 이동 전에는 기존 해시가 통과했고 merge는 Q5 소스를 변경하지 않았으므로,
의도된 최신 parser 결과로 계약 기준만 정합화했다. 나머지 두 stdout 해시는 그대로였고,
정합 후 Q5 계약 3/3과 전체 회귀가 통과했다.

## 3. 커밋 계보

| 커밋 | 역할 |
|---|---|
| `4d315e3b4` | Q5 사람용 진단 출력 characterization |
| `df0b01867` | info·page diagnostic query 이동 |
| `1728b823f` | control dump 책임 분해·이동 |
| `2acfca11b` | 최신 원격 devel 정상 merge |
| `ee54c58a0` | #5542 HWP3 출력에 Q5 계약 기준 정합 |

## 4. 최종 검증

| 검증 | 결과 |
|---|---|
| 이동 완료 시 Q5 focused | 127/127 통과 |
| 최신 devel 정합 후 Q5 stdout 계약 | 3/3 통과 |
| 최신 결합 HEAD release-test 전체 nextest | 7,860/7,860 통과, 3 slow, 38 skipped, 181.492초 |
| 대상 모듈 CC 25 상한 | 경고 없음 |
| `cargo fmt --all -- --check`·`git diff --check` | 통과 |
| `cargo check --all-targets` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --doc` | 8/8 통과, 3 ignored |
| integration manifest·unit-tier 정책 자체 계약 | 29/29 통과 |
| 최신 base manifest check | 772 sources / 3,811 static test attrs / 44/48 integration targets, 통과 |
| unit-tier base check | 4,225 tests / 298 modules, 통과 |
| CI impact Node·Python workflow 계약 | 62/62, 67/67 통과 |
| Markdown link check | 기존 capability 등록부 무결성 오류 16건, Q5 신규 오류 없음 |

검증 준비가 `Cargo.toml`에 만든 세 singleton integration target과 Cargo가 재정렬한 lockfile package
순서는 추적 변경에서 제거했다. 새 integration source는 `tests/cases/`에만 두었고 generated test
target은 커밋하지 않았다.

로컬 nextest 0.9.137은 저장소 권고 0.9.140보다 낮다는 경고를 냈지만 전체 모집단을 정상 실행해
전건 통과했다. Q5 자체는 CLI adapter 제어 흐름과 물리 위치만 바꾸므로 시각 sweep과 WASM
빌드 발생 조건에는 해당하지 않는다. 원격에서 함께 흡수한 renderer 변경의 관문은 원격 통합
커밋 #5617에서 이미 소유하며, 이 결합 HEAD에서도 전체 Rust 회귀를 다시 통과했다.

## 5. 원격 병합 위험

최종 fetch 기준 `origin/devel`과 `upstream/devel`은 `61baa6783`으로 같고 이를 작업 브랜치에
정상 merge했다. merge 뒤 열린 devel 대상 PR 26개의 파일을 다시 확인했으며 `src/main.rs`,
`src/cli/queries/`, Q5 characterization 파일과 겹치는 PR은 없었다.

이 판정은 시점 증거다. 향후 로컬 devel 통합과 admin push 직전에 exact base SHA·PR head·
merge-tree를 다시 확인한다. 이 보고서 작성 시점에는 remote push를 수행하지 않았다.

## 6. 다음 승인 단위

다음 기능군은 Q6 `convert·extract-pages·HWPX/HML·ingest·scaffold`다. parser·serializer 동작
변경 없이 adapter만 분리하고, 변환 파일 바이트·exit code·검증 실패·출력 경로 계약과 공유 seam을
먼저 inventory한다. Q6는 메인테이너의 Q5 배치 종료 승인과 진입 승인 전 시작하지 않는다.
