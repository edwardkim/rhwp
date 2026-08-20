# #5511 Stage 2 기능군 배치 Q7 — IR·검증 adapter 경계

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 구현·최종 통합 기준: `upstream/devel` `b914bdf4bf1a8f922f03ea6b141f0d9c2b10a98f`
- 최종 코드 HEAD: `a9c2e98012445dfd9fdf3e9f6c1a5ec2f7d0f55e`
- 수행일: 2026-08-20
- 상태: 완료 — Wave Q 종료, M1 진입 승인 대기

## 1. 결과

`test-field`, `verify`, `dump-anchors`, `dump-carets`, `ir-sweep`, `ir-diff`와 전용 helper를
`src/main.rs`에서 네 책임 모듈로 분리했다. parser·serializer·renderer·WASM 구현과 공개 CLI
계약은 바꾸지 않았다. `collect_field_records`는 single·batch·MCP가 공유하므로 root seam에
남겼다.

| 모듈 | 책임 | 최종 줄 수 |
|---|---|---:|
| `cli/commands/internal_validation.rs` | 필드 HWP 저장·재로딩 내부 검증 command | 107 |
| `cli/queries/position_diagnostics.rs` | anchor·caret 위치 진단 | 165 |
| `cli/queries/verification.rs` | 기대조건 parsing·실측·판정 출력 | 393 |
| `cli/queries/ir_comparison.rs` | IR diff/sweep와 비교 helper | 1,073 |

네 모듈은 모두 1,200줄 상한 이하다. `src/main.rs`는 Q7 시작의 29,944줄에서 28,295줄로
1,649줄 줄었고 최상위 함수는 241개에서 225개로 줄었다. root의
`rhwp::wasm_api::HwpDocument` 직접 참조는 23개에서 22개가 됐다. 이는 adapter 물리 이동의
결과이며 service 경계 전환은 Stage 3 입력으로 남긴다.

## 2. A안 이행과 복잡도 정정

inventory에서 확인한 `ir_diff_paragraph_fields` CC 28, `cmd_verify` CC 29, `ir_diff` CC 38을
분해 없이 새 파일로 숨기지 않았다.

- 문단 비교를 scalar, LineSeg, control/table/textbox, char-shape 비교로 분리했다.
- `verify`를 `VerifyArgs` parsing과 문서 실측·판정 실행으로 분리했다.
- `ir-diff`를 `IrDiffArgs` parsing, 암호 적용 문서 load, section/paragraph, ParaShape,
  TabDef 비교와 출력으로 분리했다.

최종 `cargo clippy --bin rhwp -W clippy::cognitive_complexity`에서 Q7 모듈의 CC 25 초과 경고는
0건이다. 최초 물리 이동 뒤 세 수치가 그대로 남아 있음을 마감 대조에서 발견했고, 완료 보고를
중단한 뒤 승인된 A안 범위의 책임 분해 커밋 `a9c2e9801`로 정정했다.

## 3. 보호 계약

이동 전에 `test-field` 성공 저장·입력 무훼손, anchor 사람 출력, caret JSON 순수성·filter·실패,
IR sweep의 JSON·text exit 의미를 6개 characterization으로 고정했다. 기존 12개 직접 계약
104개와 합친 Q7 focused 110/110이 최종 책임 분해 뒤 통과했다.

보호 범위는 `verify` 기대조건 순서와 exit 0/1/2/3, `ir-diff` 동일·차이·구역·표 셀·암호·summary,
JSON provenance와 stdout/stderr 분리, 내부 저장 출력과 입력 무훼손, catalog·help·MCP 참여다.
관찰 가능한 출력, 카테고리, diff count, truncation 또는 파일 부작용 차이는 발견되지 않았다.

## 4. 커밋 계보

| 커밋 | 역할 |
|---|---|
| `440fe79da` | Q7 범위·계약 공백·복잡도 중단 inventory |
| `f69c7357d` | Q7 진단·검증 characterization 6개 |
| `21df42d8a` | internal field validation command 이동 |
| `b40c9db73` | anchor·caret position diagnostics 이동 |
| `02b4cbf8b` | verify query 이동 |
| `0c9169404` | IR diff/sweep와 비교 helper 이동 |
| `031d7063b` | 이동된 IR helper unit test의 소유 모듈 정합화 |
| `10e0c31b5` | verify adapter 문서 주석 소유권 정합화 |
| `a9c2e9801` | A안의 세 고복잡도 함수 책임 분해 |

첫 전체 회귀는 이동된 `tab_ext_semantic_differs`의 root unit test import 잔존을 발견했고, 첫
all-target clippy는 이동 뒤 root에 남은 verify 문서 주석 소유권을 발견했다. 각각 실제 소유
모듈로 함께 옮겨 파생 harness나 lint 예외 없이 해결했다.

## 5. 최종 검증

| 검증 | 결과 |
|---|---|
| Q7 focused 계약 | 110/110 통과 |
| 최종 release-test 전체 nextest | 8,005/8,005 통과, 3 slow, 38 skipped, 173.942초 |
| Q7 모듈 CC 25 상한 | 초과 경고 0건 |
| `cargo fmt --all -- --check`·`git diff --check` | 통과 |
| `cargo check --all-targets` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --doc` | 8/8 통과, 3 ignored |
| integration manifest | 803 sources / 3,956 static test attrs / 41/48 targets, 통과 |
| unit-tier | 4,225 tests / 299 modules, 통과 |
| CI impact Node·Python workflow 계약 | 62/62, 149/149 통과 |
| Markdown link check | 기존 capability 등록부 무결성 오류 16건, Q7 신규 오류 없음 |

`node scripts/rust-test-suite-manifest.mjs --prepare`가 만든 suite harness는 검증 파생물이며 추적
변경에 포함하지 않았다. 로컬 nextest 0.9.137이 저장소 권고 0.9.140보다 낮다는 경고가 있었지만
전체 모집단은 정상 실행되어 전건 통과했다.

Q7은 move-only CLI adapter와 책임 분해이므로 renderer·layout·WASM·native-skia·시각 검증 발생
조건에 해당하지 않는다.

## 6. 최신 devel과 열린 PR

최종 fetch에서 `origin/devel`과 `upstream/devel`은 모두 `b914bdf4b`이며 최종 코드 HEAD의
조상이다. 열린 devel 대상 PR은 #5647, #5689, #5691, #5693, #5695, #5707, #5709, #5710이다.
renderer, Studio, q-more, skill-router, 별도 계약·보고서 경로이며 Q7의 네 구현 모듈과 신규
characterization source를 변경하는 PR은 없다.

이 판정은 시점 증거다. 향후 통합·push 직전에 exact base SHA, 열린 PR head와 merge 가능성을
다시 확인한다. 이 보고서 작성 시점에는 remote push를 수행하지 않았다.

## 7. 다음 승인 단위

Q7 완료로 Wave Q의 조회·출력 adapter 배치를 마쳤다. 다음 기능군은 M1
`MCP definitions·capabilities payload·help projection`이다. 7,569줄을 한 모듈로 옮기지 않고
schema, capabilities projection, help projection을 각각 1,200줄 이하로 나누며 catalog 정본과의
동형성을 보호한다. M1은 메인테이너의 Q7 완료 승인과 별도 진입 승인 전 시작하지 않는다.
