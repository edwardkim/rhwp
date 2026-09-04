# #5511 Stage 2 기능군 배치 C4 — document structure command

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 최종 통합 기준: `upstream/devel` `b32113be61aefab049d03d6ab618c217104c080c`
- 최종 코드 HEAD: `3c12a4a3e3f6122c48fc9ac180e645aa3df87b84`
- 수행일: 2026-08-20
- 상태: 완료 — C4 종료, C5 진입 승인 대기

## 1. 결과

본문 text·paragraph·break, page·section·column, note, bookmark, generic control 구조 command를
다섯 책임 모듈로 분리했다.

```text
src/cli/commands/edit/
├── document_text.rs
├── page.rs
├── notes.rs
├── bookmarks.rs
└── controls.rs
```

| 책임 | 최종 파일 | 줄 수 |
|---|---|---:|
| 본문 text·paragraph·break | `src/cli/commands/edit/document_text.rs` | 1,082 |
| page·section·column structure | `src/cli/commands/edit/page.rs` | 466 |
| note lifecycle | `src/cli/commands/edit/notes.rs` | 293 |
| bookmark lifecycle | `src/cli/commands/edit/bookmarks.rs` | 372 |
| generic structural control | `src/cli/commands/edit/controls.rs` | 103 |

모든 새 파일은 1,200줄 상한 이하다. C4 실범위 2,306줄·20 handler는 root에 남지 않았고,
`src/main.rs`는 C4 시작의 8,085줄에서 5,788줄로 줄었다. module 선언·dispatch 조립 차이를
포함한 실제 root 감소는 2,297줄이다. 마스터 계획의 3,181줄·26함수에는 C5의 cell
paragraph·formatting과 C6의 header/footer·footnote body tail이 섞여 있었으므로 이를 이동하지
않고 실제 소유권에 맞춰 보정했다.

## 2. 소유권 결정

`document_text.rs`는 본문 text·paragraph mutation, page/column break와 numbering restart를
소유한다. `page.rs`는 page/section/column definition과 page hide를 소유한다. note와 bookmark는
각 lifecycle 모듈로 분리했고, 임의 structural control 삭제는 `controls.rs`에 고립했다.
`edit/mod.rs`는 모듈 선언과 dispatch만 조립한다.

footnote body text·paragraph, header/footer lifecycle·body command는 story 주소 경계를 함께
다뤄야 하므로 C6에 남겼다. cell paragraph split/merge와 char·para·cell formatting은 C5에
남겼다. 공개 schema, core mutation, parser·serializer·renderer 알고리즘은 바꾸지 않았다.
`DocumentService`, typed error와 전역 인증 제거는 Stage 3 입력으로 유지했다.

## 3. 커밋 계보와 최신 기준선

| 커밋 | 역할 |
|---|---|
| `de887e33a` | C4 실범위·보호 공백·책임 경계 inventory |
| `d0282dae4` | `set-column-def` CLI characterization 4건 추가 |
| `287640970` | 본문 text·paragraph·break command 이동 |
| `49f193b49` | page·section·column structure command 이동 |
| `1c10d596d` | note·bookmark·generic control lifecycle 이동 |
| `3c12a4a3e` | 최신 `upstream/devel` 정상 merge |

C4 구현 뒤 `devel`이 `b32113be6`으로 전진했다. 원격 변경은 q-more, skill-router, Studio와
외부 기여 PR 누적 통합으로 175개 파일을 추가·변경했지만 C4 경로와 겹치지 않았다. merge-tree
충돌이 없음을 확인하고 정상 merge한 뒤 파생 integration harness를 새 HEAD에서 다시 생성해
전체 검증을 재수행했다.

## 4. 직접 계약과 `set-column-def` 관찰

C4 전용 계약, 새 column definition 계약과 공통 JSON·provenance 계약을 합친 117/117이 최신
결합 HEAD에서 통과했다.

| 계약 축 | 건수 |
|---|---:|
| 본문 text·paragraph·break | 34 |
| page·section·column definition | 16 |
| note lifecycle | 9 |
| bookmark·control lifecycle | 17 |
| JSON·provenance 봉투 | 41 |

새 `set-column-def` 계약 4건은 성공 JSON 값, 출력 파일 재파싱, dry-run 무쓰기, 잘못된 type과
미등록 option의 usage exit·빈 stdout, MCP 등록을 고정한다. inventory에서 계획한 엄격한 저장값
동등성은 현재 구현 결함 때문에 정상 규약으로 고정하지 않았다.

구체적으로 `set_column_def_native`는 기존 `ColumnDef`의 `column_count`, `column_type`,
`same_width`, `spacing`을 바꾸지만 `raw_attr`를 무효화하지 않는다. 반면 HWP5 serializer는
`raw_attr != 0`이면 구조화 필드보다 기존 bitfield를 우선한다. 따라서 CLI 성공 봉투가 2단을
보고해도 저장 후 다시 열면 원래 1단이 남을 수 있다. 또한 mixed-width 요청은 width·gap 배열을
새 단 수에 맞게 구성하지 않는다. 이는 C4 adapter 이동과 독립된 core 저장 결함이므로 이번
move-only 배치에서 숨겨 고치지 않았으며, 별도 후속 이슈로 core mutation·serializer 불변식을
정의한 뒤 처리해야 한다.

## 5. 최종 검증

| 검증 | 결과 |
|---|---|
| C4 직접 focused 계약 | 117/117 통과 |
| 최신 결합 HEAD 전체 nextest | 8,205/8,205 통과, 3 slow, 39 skipped, 164.654초 |
| `cargo fmt --all -- --check`·`git diff --check` | 통과 |
| `cargo check --locked --all-targets` | 통과 |
| `cargo clippy --locked --all-targets -- -D warnings` | 통과 |
| C4 모듈 CC 25 상한 | 초과 경고 0건 |
| `cargo doc --locked --no-deps` | 성공, 기존 rustdoc 경고만 존재 |
| `cargo test --locked --doc` | 8/8 통과, 3 ignored |
| integration manifest 정책·현재 상태 | 808 sources / 3,973 attrs / 41/48 targets 통과 |
| unit-tier 정책·현재 상태 | 4,225 tests / 299 modules 통과 |
| CI impact Node·Python workflow 계약 | 62/62, 168/168 통과 |

전체 회귀는 매뉴얼의 고정 target 명령인 `--cargo-profile release-test --target-dir
target/pr-review --tests --test-threads 8 --no-fail-fast`로 실행했다. 로컬 nextest 0.9.137이
저장소 권고 0.9.140보다 낮다는 경고는 있었으나 모집과 실행은 정상 완료됐다. 문서 생성 시
발생한 기존 rustdoc 경고와 저장소 전체의 기존 CC 경고는 C4 신규 경고가 아니며, 대상 다섯
모듈의 CC 초과는 0건이다.

`rust-test-suite-manifest --prepare`의 파생 harness와 Cargo target은 추적 변경에 포함하지
않았다. C4는 CLI adapter move-only이며 renderer·layout·WASM·native-skia·시각 검증 발생 조건에
해당하지 않는다.

## 6. unit-tier 누적 브랜치 주의점

현 브랜치 기준 `rust-unit-test-tiers --check`는 4,225개·299 module로 통과한다. 다만
`--base-ref upstream/devel`로 장기 브랜치 전체를 비교하면 Q7에서 추가한
`src/cli/queries/ir_comparison.rs::tests#1`을 base에 없는 신규 `cfg(test)` module로 감지해
실패한다. C4에서 이 선행 누적 차이를 숨기기 위해 정책을 완화하거나 Q7 테스트를 이동하지
않았다. 향후 통합 경로가 PR 제출로 바뀌면 별도 정정이 선행되어야 한다.

## 7. 최신 devel과 열린 PR

최종 검증 뒤 다시 조회한 `upstream/devel`과 `origin/devel`은 모두
`b32113be61aefab049d03d6ab618c217104c080c`이며, 최종 코드 HEAD는 이를 조상으로 포함한다.
보고서 작성 전 브랜치는 원격보다 61개 커밋 앞, 뒤처진 커밋은 없다.

열린 devel 대상 PR은 #5736 하나다. 최신 head `adef36628c9c0738fc2242667cb3d16da7212e66`은
renderer table layout, 전용 회귀 source·fixture와 sample을 변경하며 C4의 `src/main.rs`,
`src/cli/commands/edit/**`, `set_column_def_contract.rs`, #5511 문서 경로와 겹치지 않는다.
조회 시점에는 mergeable이지만 checks 때문에 `UNSTABLE` 상태였다.

이 판정은 시점 증거다. 향후 push 직전에 exact base SHA, 열린 PR head와 merge 가능성을 다시
확인한다. 이 보고서 작성 시점에는 remote push를 수행하지 않았다.

## 8. 다음 승인 단위

C4 완료로 본문과 문서 구조 lifecycle adapter의 물리 경계를 확정했다. 다음 기능군 C5는
char·para·cell style과 formatting command다. 범위·상속·스타일 적용 계약, C3에서 유지한 table
coordinate seam과 C6 story 경계의 교차 사용을 먼저 inventory한다. C5는 메인테이너의 C4 완료
승인과 별도 진입 승인 전 시작하지 않는다.
