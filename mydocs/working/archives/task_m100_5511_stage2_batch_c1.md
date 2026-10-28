# #5511 Stage 2 기능군 배치 C1 — field·text·privacy command

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 최종 통합 기준: `upstream/devel` `cfe2c351e834d7579a521c8ed7f6839674cc9ad1`
- 최종 코드 HEAD: `5c3cb0beb5f24cd51baccc78034b8d87ad90353c`
- 수행일: 2026-08-20
- 상태: 완료 — C1 종료, C2 진입 승인 대기

## 1. 결과

field occurrence·fill, 문서 전역 replace-text, 개인정보 redact와 metadata sanitize command를
책임별 소유 모듈로 분리했다.

```text
src/cli/commands/edit/
├── fields.rs    # field occurrence parser·fill command·공유 fill core
├── text.rs      # replace-text argument parser와 실행
└── privacy.rs   # redact argument parser·실행과 sanitize helper
```

| 책임 | 최종 파일 | 줄 수 |
|---|---|---:|
| field occurrence·fill | `src/cli/commands/edit/fields.rs` | 365 |
| 문서 전역 replace | `src/cli/commands/edit/text.rs` | 349 |
| redact·sanitize | `src/cli/commands/edit/privacy.rs` | 802 |

세 파일은 모두 1,200줄 상한 이하다. `src/main.rs`는 C1 시작의 15,140줄에서 13,736줄로
1,404줄 줄었다. 기존 계획의 연속 구현 블록은 1,380줄·11함수였지만 root에 떨어져 있던
`parse_field_key` 20줄·1함수가 field target 규약의 정본이었다. 이를 포함해 실범위를
1,400줄·12함수로 보정했다. 새 모듈의 import, argument 구조와 visibility wiring을 포함한 총
추가량은 1,516줄이며 root wrapper나 알고리즘 복제는 남기지 않았다.

## 2. 소유권과 복잡도 결정

`fields.rs`가 `parse_field_key`와 `fill_fields_core`를 소유하고 두 함수만 `pub(crate)`로 노출한다.
단건 edit, batch fill, protocol plan과 MCP session은 같은 occurrence·confusable·fill 규약을 직접
재사용한다. command handler는 edit parent에만 보인다. root가 field 좌표 해석을 계속 소유하거나
edit·batch·protocol이 서로를 역참조하는 구조는 만들지 않았다.

`text.rs`는 C0의 `edit::runtime`과 범용 `cli::integrity`를 소비한다. `privacy.rs`는 기존
atomic writer, PII query와 serializer 경계를 그대로 사용한다. 공개 schema, parser·serializer,
PII 탐지, 치환·저장 알고리즘은 바꾸지 않았다.

이동 전 `edit_replace_text`는 CC 29, `edit_redact`는 CC 33이었다. 두 함수의 option·kind·mask·
destination parsing과 실행 수명주기를 private argument 구조로 분리했다. 기존 진단 순서, exit code,
stdout/stderr, 파일 부작용은 유지했고 C1 세 모듈의 cognitive-complexity 25 초과 경고는 0건이다.

`edit_insert_text`와 `edit_delete_text`는 C1에 끌어오지 않았다. 두 명령은 문단 좌표·분할·병합과
같은 구조 편집 seam을 공유하므로 계획대로 C4 document-structure 기능군에서 다룬다.

## 3. 커밋 계보

| 커밋 | 역할 |
|---|---|
| `7a6c65f1f` | C1 실범위·113개 보호 계약·복잡도 분해 기준 inventory |
| `cbd57d2a1` | occurrence parser·fill command·공유 fill core 이동 |
| `864074745` | replace-text 이동과 argument parsing·실행 분리 |
| `5c3cb0beb` | redact·sanitize 이동과 redact parsing·실행 분리 |

## 4. 직접 계약

이동 전과 최종 코드 HEAD에서 9개 직접 계약 모듈 113/113을 통과했다.

| 계약 축 | 모듈 | 건수 |
|---|---|---:|
| field 단건·occurrence | `edit_fill_fields_contract`, `edit_field_occurrence_contract` | 11 |
| replace text | `edit_replace_text_contract` | 5 |
| redact·sanitize | `redact_sanitize_contract` | 15 |
| batch·MCP·plan 재사용 | `batch_fill_contract`, `mcp_session_edit_contract`, `run_plan_contract` | 41 |
| JSON·provenance 봉투 | `cli_json_contract`, `provenance_contract` | 41 |

field 절편의 직접 소비 계약 52/52, replace 절편 5/5, privacy 절편 15/15도 각각 구현 직후
통과했다. 최종 113개 계약은 occurrence와 모호성·confusable 보고, dry-run 무쓰기, 입력 형식
보존, 저장 후 verify, replace 0건 무산출, CAS, redact 목적지 명시·원본 보호·`--no-raw`,
HWP/HWPX metadata와 preview 정리, batch 행 격리, MCP session·plan 재사용, provenance 표지를
보호한다.

## 5. 최종 검증

| 검증 | 결과 |
|---|---|
| C1 직접 focused 계약 | 이동 전·최종 113/113 통과 |
| 최종 release-test 전체 nextest | 8,005/8,005 통과, 3 slow, 38 skipped, 160.071초 |
| `cargo fmt --all -- --check`·`git diff --check` | 통과 |
| `cargo check --locked --all-targets` | 통과 |
| `cargo clippy --locked --all-targets -- -D warnings` | 통과 |
| C1 모듈 CC 25 상한 | 초과 경고 0건 |
| `cargo test --locked --doc` | 8/8 통과, 3 ignored |
| integration manifest 정책·현재 상태 | 18/18, 803 sources / 3,956 attrs / 41/48 targets 통과 |
| unit-tier 정책·현재 상태 | 12/12, 4,225 tests / 299 modules 통과 |
| CI impact Node·Python workflow 계약 | 62/62, 163/163 통과 |

전체 회귀는 매뉴얼의 고정 target 명령인 `--cargo-profile release-test --target-dir
target/pr-review --tests --test-threads 8 --no-fail-fast`로 실행했다. 변경분 release-test 재컴파일은
43.05초였고 실제 8,005개 실행은 160.071초였다. 로컬 nextest 0.9.137이 저장소 권고 0.9.140보다
낮다는 경고가 있었지만 전체 모집단은 정상 실행되어 전건 통과했다.

`rust-test-suite-manifest --prepare`의 파생 harness와 Cargo target 및 Cargo.lock의 파생 정렬은
추적 변경에 포함하지 않았고 최종 worktree에 남지 않았다. C1은 move-only CLI command 변경이므로
renderer·layout·WASM·native-skia·시각 검증 발생 조건에 해당하지 않는다.

## 6. 최신 devel과 열린 PR

최종 검증 뒤 `upstream/devel`과 `origin/devel`을 다시 fetch했으며 둘 다
`cfe2c351e834d7579a521c8ed7f6839674cc9ad1`로 C1 시작 이후 전진하지 않았다. 따라서 별도 merge는
필요하지 않다. 최종 코드 HEAD는 최신 `upstream/devel`을 조상으로 포함하며 원격보다 40개 커밋
앞서고 뒤처진 커밋은 없다.

열린 devel 대상 PR은 #5647, #5689, #5691, #5695, #5707, #5709, #5710, #5718, #5719다.
각 최신 head의 변경 경로를 다시 조회했으며 C1의 root, MCP, edit command·protocol, 직접 계약과
#5511 C1 문서 경로에 겹침이 없다.

이 판정은 시점 증거다. 향후 push 직전에 exact base SHA, 열린 PR head와 merge 가능성을 다시
확인한다. 이 보고서 작성 시점에는 remote push를 수행하지 않았다.

## 7. 다음 승인 단위

C1 완료로 field target 정본, 문서 전역 치환과 privacy edit command의 물리 경계를 확정했다.
다음 기능군은 C2 chart·shape·form·image·picture이며 binary 자산, anchor와 target 선택 계약을 먼저
inventory한다. C2는 메인테이너의 C1 완료 승인과 별도 진입 승인 전 시작하지 않는다.
