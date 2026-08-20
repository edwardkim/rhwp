# #5511 Stage 2 기능군 배치 C6 — story command

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 최종 통합 기준: `upstream/devel` `d5a99a6f726afeb0aa71503c80bb4128a88bacae`
- 최종 코드 HEAD: `b36508b462c1119a2d0495e2336ffc274ddf18d1`
- 수행일: 2026-08-20
- 상태: 완료 — C6·계획된 Wave C 종료, Stage 2 종료 inventory 승인 대기

## 1. 결과

header/footer와 footnote/endnote 내부 story command를 세 책임 모듈로 분리했다.

```text
src/cli/commands/edit/
├── header_footer_content.rs
├── header_footer_properties.rs
└── note_content.rs
```

| 책임 | 최종 파일 | 줄 수 |
|---|---|---:|
| header/footer lifecycle·text·paragraph·field | `src/cli/commands/edit/header_footer_content.rs` | 1,158 |
| header/footer picture·template·visibility·format | `src/cli/commands/edit/header_footer_properties.rs` | 567 |
| footnote/endnote text·paragraph·shape·format | `src/cli/commands/edit/note_content.rs` | 701 |

세 파일은 모두 1,200줄 상한 이하다. 마스터 계획의 1,152줄·9함수 추정은 실제 root에 남은
story command를 절반만 포착했다. inventory에서 실범위를 2,430 source line·18 handler로
보정하고 그 전부를 이동했다. `src/main.rs`는 C6 시작의 4,499줄에서 2,089줄로 2,410줄
줄었으며 최상위 함수는 63개에서 45개로 줄었다. 함수 본문 2,430줄과 root의 순감소량 차이는
module dispatch 조립과 기존 주석·공백의 재배치에서 생긴다. 이제 root에 `edit_*` handler는
남지 않는다.

## 2. 소유권 결정

`header_footer_content.rs`는 header/footer control과 그 내부 text·paragraph·field lifecycle을,
`header_footer_properties.rs`는 이미 존재하는 story의 picture·template·visibility·paragraph
properties를 소유한다. `note_content.rs`는 footnote/endnote 내부 text·paragraph와 endnote
shape를 소유한다. `edit/mod.rs`는 모듈 선언과 dispatch만 조립한다.

C4의 `notes.rs`에는 본문 control을 만들고 지우는 `insert-footnote`, `insert-endnote`,
`delete-footnote`를 유지했다. C6는 그 control 안쪽 story만 편집하므로 두 책임을 합치지 않았다.
C0의 load·serialize·verify·write runtime을 그대로 재사용했고 공개 schema, core mutation,
parser·serializer·renderer 알고리즘은 바꾸지 않았다. `DocumentService`, typed error와 전역 인증
제거는 Stage 3 입력으로 유지했다.

## 3. 커밋 계보와 최신 기준선

| 커밋 | 역할 |
|---|---|
| `06f8cc3c8` | 최신 `upstream/devel` `d5a99a6f7` 정상 merge |
| `6a10cd5f2` | C6 실범위·보호 공백·story 소유권 inventory |
| `e91085d01` | `set-hf-picture` 성공 저장 characterization 추가 |
| `b36508b46` | story command 18개를 세 모듈로 이동 |

C6 완료와 전체 검증 뒤 `upstream/devel`과 `origin/devel`을 다시 fetch했다. 둘 다
`d5a99a6f726afeb0aa71503c80bb4128a88bacae`로 유지되어 추가 merge는 필요하지 않았다. 코드
HEAD는 기준선보다 71개 커밋 앞이고 뒤처진 커밋은 없다.

## 4. 직접 계약과 `set-hf-picture` 관찰

C6 전용 계약과 공통 JSON·provenance 계약을 합친 113/113이 최종 코드 HEAD에서 통과했다.

| 계약 축 | 건수 |
|---|---:|
| header/footer content | 32 |
| header/footer properties | 16 |
| note story | 24 |
| JSON·provenance 봉투 | 41 |

기존 `set-hf-picture` 3건은 dry-run, 미등록 option, MCP 등록만 확인했다. dry-run은 mutation을
실행하지 않아 지정 주소가 실제 header/footer picture인지조차 증명하지 못했다. 신규 계약은
header 안에 유효한 picture control이 있는 문서를 구성하고 CLI로 width를 바꾼 뒤 저장 파일을
재파싱해 값의 변화를 확인한다. 성공 JSON의 story 종류·instance·paragraph·control 주소도 함께
고정했다. 이 characterization은 이동 전 구현에서 4/4 통과했고 이동 후 focused 관문에도
포함됐다.

## 5. 최종 검증

| 검증 | 결과 |
|---|---|
| C6 직접 focused 계약 | 113/113 통과 |
| 전체 nextest | 8,212/8,212 통과, 3 slow, 39 skipped, 189.707초 |
| `cargo fmt --all -- --check`·`git diff --check` | 통과 |
| `cargo check --locked --bin rhwp`·`--all-targets` | 통과 |
| `cargo clippy --locked --all-targets -- -D warnings` | 통과 |
| C6 모듈 CC 25 상한 | 초과 경고 0건 |
| `cargo doc --locked --no-deps` | 성공, 기존 rustdoc 경고만 존재 |
| `cargo test --locked --doc` | 8/8 통과, 3 ignored |
| integration manifest 정책·현재 상태 | 810 sources / 3,980 attrs / 41/48 targets, 정책 18/18 통과 |
| unit-tier 정책·현재 상태 | 4,225 tests / 299 modules, 정책 12/12 통과 |
| CI impact Node·Python workflow 계약 | 31/31, 169/169 통과 |

전체 회귀는 매뉴얼의 고정 target 명령인 `--cargo-profile release-test --target-dir
target/pr-review --tests --test-threads 8 --no-fail-fast`로 실행했다. 로컬 nextest 0.9.137이
저장소 권고 0.9.140보다 낮다는 경고는 있었으나 모집과 실행은 정상 완료됐다. 문서 생성 시
발생한 기존 rustdoc 경고와 저장소 전체의 기존 CC 경고는 C6 신규 경고가 아니며, 대상 세
모듈의 CC 초과는 0건이다.

검증 순서 중 `cargo fmt`가 ignored 파생 integration harness의 항목 순서를 바꿔 manifest
drift를 한 번 감지했다. 이는 tracked source 결함이 아니며
`rust-test-suite-manifest --prepare`로 현 HEAD의 harness를 다시 생성한 뒤 manifest check와
정책 테스트를 재실행해 통과했다. 파생 harness와 Cargo target은 추적 변경에 포함하지 않았다.
C6는 CLI adapter move-only이며 renderer·layout·WASM·native-skia·시각 검증 발생 조건에
해당하지 않는다.

## 6. unit-tier 누적 브랜치 주의점

현 브랜치 기준 `rust-unit-test-tiers --check`는 4,225개·299 module로 통과한다. 다만
`--base-ref upstream/devel`로 장기 브랜치 전체를 비교하면 Q7에서 추가한
`src/cli/queries/ir_comparison.rs::tests#1`을 base에 없는 신규 `cfg(test)` module로 감지해
실패한다. C6에서 이 선행 누적 차이를 숨기기 위해 정책을 완화하거나 Q7 테스트를 이동하지
않았다. 향후 통합 경로가 PR 제출로 바뀌면 별도 정정이 선행되어야 한다.

## 7. 최신 devel과 열린 PR

최종 검증 뒤 다시 조회한 `upstream/devel`과 `origin/devel`은 모두
`d5a99a6f726afeb0aa71503c80bb4128a88bacae`이다. 보고서 작성 전 코드 HEAD는 원격보다 71개
커밋 앞이고 뒤처진 커밋은 없다.

열린 devel 대상 PR은 #5739, #5741~#5745, #5754, #5758, #5762, #5766이다. 기존 PR은
Studio·Docker·renderer·parser·model·serializer 경로이고, 새 #5766은 q-pack 생성 source를
공통 probe로 축소한다. 어느 PR도 C6의 `src/main.rs`, `src/cli/commands/edit/**`,
`tests/cases/set_hf_picture_contract.rs`, #5511 계획·보고 경로와 직접 겹치지 않는다.

이 판정은 시점 증거다. 향후 push 직전에 exact base SHA, 열린 PR head와 merge 가능성을 다시
확인한다. 이 보고서 작성 시점에는 remote push를 수행하지 않았다.

## 8. 다음 승인 단위

C6로 계획된 Wave C 기능군과 편집 handler 92개의 물리 분리가 끝났다. 그러나 Stage 2 자체의
종료 조건은 아직 충족하지 않았다. `src/main.rs`는 2,089줄로 1,200줄 상한까지 889줄이 남고,
45개 최상위 함수 중 metadata JSON helper, generation/internal command, 전역 인증·load seam,
entrypoint와 root 단위 테스트가 섞여 있다.

따라서 다음 승인 단위는 임의의 C7이나 Stage 3 구현이 아니다. 먼저 Stage 2 종료 inventory에서
잔여 45개 함수와 root 주석을 entrypoint 필수 조립, 기존 모듈로 귀속할 adapter, Stage 3 입력
seam으로 분류한다. 그 결과로 최소 정정 배치를 확정하고 Stage 2 종료 조건을 다시 판정한다.
이 inventory와 후속 구현은 별도 승인 전 시작하지 않는다.
