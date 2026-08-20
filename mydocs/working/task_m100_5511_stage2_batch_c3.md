# #5511 Stage 2 기능군 배치 C3 — cell·table·equation command

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 최종 통합 기준: `upstream/devel` `1139f28d17d55b499f553354f8711ecc60b110dd`
- 최종 코드 HEAD: `86b0901590733e830dd6f2ed6c5e8c8ed688ac97`
- 수행일: 2026-08-20
- 상태: 완료 — C3 종료, C4 진입 승인 대기

## 1. 결과

cell content·properties, table 좌표·격자·구조·layout, equation lifecycle command와 전용 helper를
책임별 소유 모듈로 분리했다.

```text
src/cli/commands/edit/
├── cells.rs
├── equations.rs
└── tables/
    ├── mod.rs
    ├── coordinates.rs
    ├── structure.rs
    ├── grid.rs
    └── layout.rs
```

| 책임 | 최종 파일 | 줄 수 |
|---|---|---:|
| cell content·properties·공유 측정 helper | `src/cli/commands/edit/cells.rs` | 969 |
| equation lifecycle | `src/cli/commands/edit/equations.rs` | 348 |
| table coordinate | `src/cli/commands/edit/tables/coordinates.rs` | 108 |
| table lifecycle | `src/cli/commands/edit/tables/structure.rs` | 547 |
| row·column·cell grid mutation | `src/cli/commands/edit/tables/grid.rs` | 853 |
| table size·position·properties | `src/cli/commands/edit/tables/layout.rs` | 701 |
| table module projection | `src/cli/commands/edit/tables/mod.rs` | 20 |

모든 새 파일은 1,200줄 상한 이하다. `src/main.rs`는 C3 시작의 11,601줄에서 8,085줄로
3,516줄 줄었고, inventory에서 확정한 C3 원래 함수 31개는 root에 남지 않았다. 마스터 계획의
3,639줄·33함수는 공유 query/output 단위 변환 seam인 `hu_to_mm`·`hu_to_mm_i`를 잘못 포함했다.
현재 호출 계보를 기준으로 약 3,525줄·31함수로 보정했으며, module 선언·가시성·재노출을 포함한
실제 root 정산은 3,516줄 감소다.

## 2. 소유권 결정

`cells.rs`는 네 cell handler와 글자색 보정, 폭 측정, overflow 판정, control-character 거부를
소유한다. 이동 중 `recolor_cell_text_black`과 overflow helper가 CLI뿐 아니라 MCP·protocol에서도
사용됨을 확인했다. 복제하지 않고 `cells.rs`를 단일 정본으로 삼아 edit parent와 root가 Stage 2
호환 seam으로 다시 노출한다.

`tables/coordinates.rs`는 `CellResolveError`와 `resolve_table_cell`을 소유한다. C5 formatting
handler가 아직 이 resolver를 사용하므로 crate 내부의 좁은 임시 seam으로 유지했다.
`resolve_top_table`은 C3 전용이라 table 모듈 밖으로 노출하지 않았다. 계획상 단일 structure
파일은 1,200줄을 넘기므로 table lifecycle, grid mutation, layout/property 세 책임으로 나눴다.
`equations.rs`는 equation insert/delete/property lifecycle을 단독 소유한다.

공개 schema, core table mutation, parser·serializer·renderer 알고리즘은 바꾸지 않았다.
`DocumentService`, typed error와 전역 인증 제거는 Stage 3 입력으로 남겼고, C4 문서 구조와 C5
formatting 책임도 끌어오지 않았다.

## 3. 최신 기준선과 커밋 계보

C3 시작 직전 Studio 변경 `73939045e`를 `05cae2eb7`에서 정상 merge했다. 구현과 첫 전체 검증이
끝난 뒤 `devel`이 `1139f28d1`로 다시 전진했다. 이 커밋은 renderer와 새 회귀 source 세 개를
변경했으며 C3 edit 경로와 merge-tree 충돌은 없었다. `86b090159`에서 정상 merge한 뒤 파생 test
harness를 새 HEAD에서 다시 생성해 신규 source까지 포함한 전체 검증을 재수행했다.

| 커밋 | 역할 |
|---|---|
| `05cae2eb7` | C3 시작 전 최신 `upstream/devel` 정상 merge |
| `b9bf080ed` | C3 실범위·137개 보호 계약·좌표 seam inventory |
| `93693163a` | cell command와 coordinate·공유 helper 이동 |
| `6ed2bec48` | table lifecycle·grid·layout command 이동 |
| `a71416efe` | equation lifecycle command 이동 |
| `86b090159` | C3 완료 전 최신 `upstream/devel` 정상 merge |

## 4. 직접 계약

이동 전과 C3 구현 HEAD에서 직접 계약 137/137을 통과했고, 최신 `devel` 결합 HEAD에서는 이
계약을 포함한 전체 모집단으로 다시 검증했다.

| 계약 축 | 모듈 | 건수 |
|---|---|---:|
| cell content·properties | `edit_set_cell_contract`, `insert_text_in_cell_contract`, `delete_text_in_cell_contract`, `set_cell_props_contract` | 17 |
| table structure·layout | table insert/row/column/merge/split/fit/resize/property/move/delete/transpose 계약 18개 | 65 |
| equation lifecycle·golden | `insert_equation_contract`, `delete_equation_contract`, `set_equation_properties_contract`, `equation_command_goldens` | 14 |
| JSON·provenance 봉투 | `cli_json_contract`, `provenance_contract` | 41 |

구현 절편별로 cell 17/17, table 65/65, equation 14/14도 통과했다. 최종 계약은 dry-run 무쓰기,
잘못된 option·좌표 거부, HWP/HWPX 형식 보존, table 구조·치수·속성, equation 생성·삭제·속성,
저장 후 verify, MCP 등록과 JSON·provenance를 보호한다. 기존 계약이 C3 관찰면을 모두 덮어 신규
characterization은 추가하지 않았다.

## 5. 최종 검증

| 검증 | 결과 |
|---|---|
| C3 직접 focused 계약 | 이동 전·구현 후 137/137 통과 |
| 최신 결합 HEAD 전체 nextest | 8,197/8,197 통과, 3 slow, 39 skipped, 172.581초 |
| `cargo fmt --all -- --check`·`git diff --check` | 통과 |
| `cargo check --locked --all-targets` | 통과 |
| `cargo clippy --locked --all-targets -- -D warnings` | 통과 |
| C3 모듈 CC 25 상한 | 초과 경고 0건 |
| `cargo doc --locked --no-deps` | 성공, 기존 rustdoc 경고만 존재 |
| `cargo test --locked --doc` | 8/8 통과, 3 ignored |
| integration manifest 정책·현재 상태 | 20/20, 806 sources / 3,965 attrs / 41/48 targets 통과 |
| unit-tier 정책·현재 상태 | 12/12, 4,225 tests / 299 modules 통과 |
| CI impact Node·Python workflow 계약 | 62/62, 163/163 통과 |

전체 회귀는 매뉴얼의 고정 target 명령인 `--cargo-profile release-test --target-dir
target/pr-review --tests --test-threads 8 --no-fail-fast`로 실행했다. merge 직후의 이전 파생 harness
모집단 8,192건도 통과했지만, 그 결과를 최종 증거로 삼지 않았다. 새 HEAD에서 `--prepare`한
806-source harness로 원격 신규 테스트 5개를 포함한 8,197건을 다시 실행해 전건 통과했다. 로컬
nextest 0.9.137이 저장소 권고 0.9.140보다 낮다는 경고는 있었으나 모집과 실행은 정상 완료됐다.

`rust-test-suite-manifest --prepare`의 파생 harness와 Cargo target은 추적 변경에 포함하지 않았다.
C3는 CLI adapter move-only이고 renderer·layout·WASM·native-skia·시각 검증 발생 조건에 해당하지
않는다. 최신 원격 renderer 변경 자체는 원격 통합 커밋의 범위이므로 all-features 전체 회귀로
결합 안전성을 검증했다.

## 6. unit-tier 누적 브랜치 주의점

현 브랜치 기준 `rust-unit-test-tiers --check`는 4,225개·299 module로 통과하고 C3는 source-side
test를 추가하지 않았다. 다만 `--base-ref upstream/devel`로 장기 브랜치 전체를 비교하면 Q7에서
추가한 `src/cli/queries/ir_comparison.rs::tests#1`을 base에 없는 신규 `cfg(test)` module로 감지해
실패한다. C0~C2 완료 때도 현 브랜치 기준선 검사를 사용했으며, 이번 C3에서 정책을 완화하거나
Q7 테스트를 임의 이동해 이 누적 차이를 숨기지 않았다.

현재 승인된 통합 경로는 이 장기 작업 브랜치를 로컬 `devel`에 병합한 뒤 admin 권한으로 원격
`devel`에 반영하는 방식이라 push workflow에서는 PR base 인자가 주어지지 않는다. 향후 경로가
PR 제출로 바뀌면 Q7 테스트를 integration source로 옮기는 별도 정정이 선행되어야 한다.

## 7. 최신 devel과 열린 PR

최종 검증 기준 `upstream/devel`은 `1139f28d17d55b499f553354f8711ecc60b110dd`이며 최종 코드
HEAD가 이를 조상으로 포함한다. 완료 보고서 작성 전 기준으로 브랜치는 원격보다 53개 커밋 앞,
뒤처진 커밋은 없다.

열린 devel 대상 PR은 #5689, #5691, #5695, #5707이다. 각 최신 head의 변경 경로를 다시
조회했으며 C3의 `src/main.rs`, `src/cli/commands/edit/cells.rs`, `equations.rs`, `tables/**`, 직접
계약과 #5511 C3 문서 경로에 겹침이 없다.

이 판정은 시점 증거다. 향후 push 직전에 exact base SHA, 열린 PR head와 merge 가능성을 다시
확인한다. 이 보고서 작성 시점에는 remote push를 수행하지 않았다.

## 8. 다음 승인 단위

C3 완료로 cell·table·equation 상태 변경 adapter의 물리 경계를 확정했다. 다음 기능군 C4는
paragraph·page·section·note·bookmark·control 구조 command다. 문서 위치 선택, page/section
수명주기와 note/control 주소 seam을 먼저 inventory한다. C4는 메인테이너의 C3 완료 승인과 별도
진입 승인 전 시작하지 않는다.
