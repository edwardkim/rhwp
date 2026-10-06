# 구현계획서 — task_m100_4132

- **이슈**: [#4132](https://github.com/edwardkim/rhwp/issues/4132)
- **수행계획서**: [`task_m100_4132.md`](task_m100_4132.md)
- **브랜치**: `issue-4132-native-cli-exit`
- **기록 시각**: 2026-08-09 KST

## 1. 파일별 변경

| 파일 | 변경 |
| --- | --- |
| `tests/cli_exit_codes.rs` | native 전용 함수 이동, 공용 helper 사용 |
| `tests/cli_exit_codes_native.rs` | file-gated native 전용 종료 코드 계약 test |
| `tests/support/cli_exit_code_support.rs` | 실행 경로·임시 경로·종료 코드 assertion 공유 |
| `tests/issue_1144.rs` | native 전용 함수 이동, 공용 fixture helper 사용 |
| `tests/issue_1144_native.rs` | file-gated native 전용 PNG filename cache 계약 test |
| `tests/support/issue_1144_support.rs` | #1144 document fixture와 layer text 추출 공유 |
| `.github/workflows/ci.yml` | 새 target 2개를 Native Skia release-test·release 양쪽에 추가 |
| `scripts/ci-impact-classifier.cjs` | 새 파일 2개를 `NATIVE_SKIA_RUST_FILES`에 추가 |
| `scripts/tests/test_ci_impact_workflow.py` | 함수 게이트 재발 감시와 파일 게이트 known set 갱신 |
| `scripts/tests/ci-impact-classifier.test.cjs` | 새 파일 단독 classifier 결과 고정 |
| `mydocs/orders/20260809.md` | #4040 후처리와 #4132 진행 상태 |
| `mydocs/working/task_m100_4132_stage1.md` | 구현·검증 결과 |

## 2. 공용 CLI test helper

`tests/support/cli_exit_code_support.rs`에 다음 함수를 둔다.

- `unique_temp_path`: 병렬 실행에서도 충돌하지 않는 임시 경로
- `describe`: 실패 시 CLI args/stdout/stderr 표시
- `assert_code`: `rhwp` 실행과 종료 코드 단언
- 내부 `rhwp_bin`: 런타임 `CARGO_BIN_EXE_rhwp` 우선, 컴파일타임 경로 fallback

두 integration crate는 `#[path = "support/cli_exit_code_support.rs"] mod ...;`로 같은 helper를 컴파일한다.
공유 범위는 종료 코드 검증에 필요한 최소 함수로 제한한다.

`tests/support/issue_1144_support.rs`에는 `document_with_filename_footer`와 `layer_tree_texts`를 옮긴다.
기존 네 default test와 새 native test가 동일한 document fixture·텍스트 추출 경로를 사용한다.

## 3. native 전용 test 파일

`tests/cli_exit_codes_native.rs`와 `tests/issue_1144_native.rs`는 crate 수준에서 다음 조건을 사용한다.

```rust
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
```

기존 `export_png_follows_the_same_contract`의 행위는 바꾸지 않는다.

- 인자 없는 `export-png` → exit 2
- 존재하지 않는 입력을 지정한 `export-png` → exit 1

default feature의 `export_png_without_native_skia_reports_usage_error`는 원 파일에 남겨 feature가 없는 CLI의
오류 메시지와 exit 2 계약을 계속 보호한다.

`issue_1144_native`는 PNG export 뒤 파일명을 바꾸고 PageLayerTree에 새 이름만 남는 기존 행위를 그대로
옮긴다. native test가 공유 helper의 private 구현에 의존하지 않도록 필요한 함수만 `pub`로 노출한다.

## 4. 함수 게이트 재발 감시

기존 Rust 비코드 마스킹과 cfg 의미 판정기를 재사용해 `tests/*.rs`의 outer attribute와 함수·inline module
body 범위를 찾는다. 파일 전체가 native-skia로 게이트되지 않은 crate에서 함수 cfg, module cfg 또는
`cfg_attr(native-skia, test)`가 test를 native 전용으로 만들면 목록에 넣는다. 문자열·주석·함수 내부
속성은 대상이 아니며 합성 입력으로 넓은 쪽 오탐도 고정한다.
이미 workflow·classifier에 배선된 `issue_2225`의 함수 한 건은 명시적 allowlist로 고정하고, 그 밖의 새
항목이 생기면 실패한다.

현재 `cli_exit_codes`와 `issue_1144`의 누락 두 건을 먼저 RED로 포착한 뒤 파일 분리로 GREEN을 만든다.

## 5. workflow·classifier

- Native Skia job의 PR/devel `release-test` 경로와 그 밖의 `release` 경로에
  `--test cli_exit_codes_native`, `--test issue_1144_native`를 한 줄씩 추가한다.
- classifier의 native 소유 목록에 두 새 target과 두 `#[path]` support를 정렬 순서로 추가한다.
- classifier 단위 테스트는 네 새 경로를 각각 하나만 입력해 전체 결과와 reason을 확인한다.
- #4170의 `test_every_file_gated_native_skia_test_is_wired`와 양 프로파일 동등성 계약은 별도 보정 없이 새
  파일을 자동으로 검사해야 한다. file-gated target의 `#[path]` support도 classifier에 포함되는지 별도
  계약으로 자동 발견한다.

## 6. RED→GREEN과 focused 검증

1. 함수 게이트 재발 테스트 추가 → 현재 `cli_exit_codes`·`issue_1144` 두 건을 지목하는지 확인
2. 새 파일 2개 생성 뒤 #4170 파일 게이트 계약이 job·classifier 누락을 지목하는지 확인
3. workflow·classifier 배선 뒤 Python 계약 전건 GREEN
4. default/native 두 Rust integration target 실행
5. classifier Node test, fmt, actionlint, Node syntax, diff check

전체 회귀·clippy·원격 CI·PR 생성은 별도 승인 전에는 수행하지 않는다.

## 7. 롤백

새 test 파일과 helper를 제거하고 원 함수를 `cli_exit_codes.rs`에 되돌린 뒤 workflow·classifier 한 줄씩을
제거하면 원상 복구된다. 다만 이 경우 native `export-png` 종료 코드 검증 공백이 다시 생긴다.
