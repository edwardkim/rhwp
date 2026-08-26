# #5511 Stage 2 기능군 배치 C5 — formatting command

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 최종 통합 기준: `upstream/devel` `b32113be61aefab049d03d6ab618c217104c080c`
- 최종 코드 HEAD: `7e7ac649114ee470a8694af5df6eaf66bea6c200`
- 수행일: 2026-08-20
- 상태: 완료 — C5 종료, C6 진입 승인 대기

## 1. 결과

cell paragraph split·merge와 본문·cell의 char·paragraph·style formatting command를 두 책임
모듈로 분리했다.

```text
src/cli/commands/edit/
├── cell_paragraphs.rs
└── formatting.rs
```

| 책임 | 최종 파일 | 줄 수 |
|---|---|---:|
| cell paragraph split·merge | `src/cli/commands/edit/cell_paragraphs.rs` | 308 |
| body·cell char/paragraph/style formatting | `src/cli/commands/edit/formatting.rs` | 995 |

두 새 파일은 1,200줄 상한 이하다. C5 실범위는 1,289 source line, 8 handler와 전용 helper
`cell_para_lens` 1개다. `src/main.rs`는 C5 시작의 5,788줄에서 4,499줄로 정확히 1,289줄
줄었고, 최상위 함수는 72개에서 63개로 줄었다. 마스터 계획의 1,720줄·13함수에는 C6의
header/footer와 footnote/endnote story formatting까지 섞여 있었으므로 story 경계를 넘지 않고
실제 소유권에 맞춰 보정했다.

## 2. 소유권 결정

`cell_paragraphs.rs`는 C3에서 확정한 table coordinate resolver를 재사용해 cell 내부 문단의
split·merge만 소유한다. `formatting.rs`는 본문과 cell 주소에서 char shape, paragraph shape,
style을 적용하는 command와 cell paragraph 길이 계산 helper를 소유한다. `edit/mod.rs`는 모듈
선언과 dispatch만 조립한다.

C3의 `resolve_table_cell`·`CellResolveError`, C0의 serialize·verify·write runtime은 복제하지
않고 그대로 재사용했다. header/footer와 footnote/endnote formatting·lifecycle은 별도 story
주소 경계를 함께 다루므로 C6에 남겼다. 공개 schema, core mutation, parser·serializer·renderer
알고리즘은 바꾸지 않았다. `DocumentService`, typed error와 전역 인증 제거는 Stage 3 입력으로
유지했다.

## 3. 커밋 계보와 최신 기준선

| 커밋 | 역할 |
|---|---|
| `86e0ef3b8` | C5 실범위·보호 공백·책임 경계 inventory |
| `8ad8c56a9` | `apply-cell-style` characterization 4건 추가 |
| `9b08ea316` | cell paragraph split·merge command 이동 |
| `7e7ac6491` | body·cell formatting command 이동 |

C5 시작·완료 후 두 차례 원격을 확인했다. `upstream/devel`과 `origin/devel`은 모두
`b32113be61aefab049d03d6ab618c217104c080c`로 유지되어 추가 merge는 필요하지 않았다. 구현 완료
HEAD는 기준선보다 66개 커밋 앞이고 뒤처진 커밋은 없다.

## 4. 직접 계약과 `apply-cell-style` 관찰

C5 전용 계약과 공통 JSON·provenance 계약을 합친 73/73이 최종 코드 HEAD에서 통과했다.

| 계약 축 | 건수 |
|---|---:|
| cell paragraph split·merge | 8 |
| body char·paragraph·style formatting | 12 |
| cell char·paragraph·style formatting | 12 |
| JSON·provenance 봉투 | 41 |

기존 cell formatting 계약에는 `apply-cell-style`의 metadata만 있었고 저장 결과를 직접 확인하는
계약이 없었다. 신규 4건은 성공 JSON의 table·row·col·paragraph 주소와 저장 후 `style_id`,
dry-run 무쓰기, 잘못된 style과 미등록 option의 usage exit 2·빈 stdout, MCP 등록을 이동 전에
고정했다. characterization은 이동 전 구현에서 4/4 통과했고 이동 후 전체 focused 계약에도
포함됐다.

## 5. 최종 검증

| 검증 | 결과 |
|---|---|
| C5 직접 focused 계약 | 73/73 통과 |
| 전체 nextest | 8,209/8,209 통과, 3 slow, 39 skipped, 177.350초 |
| `cargo fmt --all -- --check`·`git diff --check` | 통과 |
| `cargo check --locked --bin rhwp`·`--all-targets` | 통과 |
| `cargo clippy --locked --all-targets -- -D warnings` | 통과 |
| C5 모듈 CC 25 상한 | 초과 경고 0건 |
| `cargo doc --locked --no-deps` | 성공, 기존 rustdoc 경고만 존재 |
| `cargo test --locked --doc` | 8/8 통과, 3 ignored |
| integration manifest 정책·현재 상태 | 809 sources / 3,977 attrs / 41/48 targets, 정책 18/18 통과 |
| unit-tier 정책·현재 상태 | 4,225 tests / 299 modules, 정책 12/12 통과 |
| CI impact Node·Python workflow 계약 | 62/62, 168/168 통과 |

전체 회귀는 매뉴얼의 고정 target 명령인 `--cargo-profile release-test --target-dir
target/pr-review --tests --test-threads 8 --no-fail-fast`로 실행했다. 로컬 nextest 0.9.137이
저장소 권고 0.9.140보다 낮다는 경고는 있었으나 모집과 실행은 정상 완료됐다. 문서 생성 시
발생한 기존 rustdoc 경고와 저장소 전체의 기존 CC 경고는 C5 신규 경고가 아니며, 대상 두
모듈의 CC 초과는 0건이다.

검증 순서 중 `cargo fmt`가 ignored 파생 integration harness를 정리해 manifest drift를 한 번
감지했다. 이는 tracked source 결함이 아니며 `rust-test-suite-manifest --prepare`로 현 HEAD의
harness를 다시 생성한 뒤 manifest check와 정책 테스트를 재실행해 통과했다. 파생 harness와
Cargo target은 추적 변경에 포함하지 않았다. C5는 CLI adapter move-only이며
renderer·layout·WASM·native-skia·시각 검증 발생 조건에 해당하지 않는다.

## 6. unit-tier 누적 브랜치 주의점

현 브랜치 기준 `rust-unit-test-tiers --check`는 4,225개·299 module로 통과한다. 다만
`--base-ref upstream/devel`로 장기 브랜치 전체를 비교하면 Q7에서 추가한
`src/cli/queries/ir_comparison.rs::tests#1`을 base에 없는 신규 `cfg(test)` module로 감지해
실패한다. C5에서 이 선행 누적 차이를 숨기기 위해 정책을 완화하거나 Q7 테스트를 이동하지
않았다. 향후 통합 경로가 PR 제출로 바뀌면 별도 정정이 선행되어야 한다.

## 7. 최신 devel과 열린 PR

최종 검증 뒤 다시 조회한 `upstream/devel`과 `origin/devel`은 모두
`b32113be61aefab049d03d6ab618c217104c080c`이다. 보고서 작성 전 브랜치는 원격보다 66개 커밋
앞이고 뒤처진 커밋은 없다.

열린 devel 대상 PR은 #5736, #5739, #5741~#5745다. #5739·#5741은 Studio, 나머지는
renderer·parser·model과 전용 sample·fixture·test를 변경한다. 어느 PR도 C5의 `src/main.rs`,
`src/cli/commands/edit/**`, `tests/cases/apply_cell_style_contract.rs`, #5511 계획·보고 경로와
겹치지 않는다.

이 판정은 시점 증거다. 향후 push 직전에 exact base SHA, 열린 PR head와 merge 가능성을 다시
확인한다. 이 보고서 작성 시점에는 remote push를 수행하지 않았다.

## 8. 다음 승인 단위

C5 완료로 일반 body와 cell formatting adapter의 물리 경계를 확정했다. 다음 기능군 C6는
header/footer와 footnote/endnote tail command다. story 종류·instance·paragraph 주소와 전용
formatting/lifecycle 계약, C4 note lifecycle과의 경계를 먼저 inventory한다. C6는 메인테이너의
C5 완료 승인과 별도 진입 승인 전 시작하지 않는다.
