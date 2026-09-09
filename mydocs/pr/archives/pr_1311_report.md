# PR #1311 처리 보고서 — HWPX slash/backSlash type 보존

## 1. 처리 개요

| 항목 | 내용 |
|---|---|
| PR | #1311 |
| 관련 이슈 | #1278 |
| 작성자 | Mireutale |
| 통합 브랜치 | `local/pr1311-integration` |
| 통합 방식 | 현재 `devel` 기준으로 `local/pr1311-upstream` 병합 |
| 통합 커밋 | `cac17f90` |

## 2. 반영 내용

`src/serializer/hwpx/header.rs`의 HWPX `borderFill` 직렬화에서 `hh:slash` / `hh:backSlash` type을 더 이상 `NONE`으로 고정하지 않는다.

| 대상 | 기존 | 변경 |
|---|---|---|
| `hh:slash@type` | 항상 `NONE` | `BorderFill.attr` bits 2..4를 HWPX type으로 복원 |
| `hh:backSlash@type` | 항상 `NONE` | `BorderFill.attr` bits 5..7을 HWPX type으로 복원 |

추가 매핑:

| attr code | HWPX type |
|---|---|
| `0` | `NONE` |
| `0b010` | `CENTER` |
| `0b011` | `CENTER_BELOW` |
| `0b110` | `CENTER_ABOVE` |
| 기타 | `ALL` |

## 3. 검증 결과

| 명령 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --lib serializer::hwpx::header -- --nocapture` | 통과, 6 passed |
| `cargo test --lib parser::hwpx::header -- --nocapture` | 통과, 22 passed |
| `cargo test --test issue_1267_hwpx_tab_and_diagonal -- --nocapture` | 통과, 2 passed |
| `cargo clippy --lib -- -D warnings` | 통과 |

GitHub PR checks도 사전 확인했다.

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| Analyze (rust) | pass |
| Analyze (javascript-typescript) | pass |
| Analyze (python) | pass |
| CodeQL | pass |
| WASM Build | skipping |

## 4. HWPX 저장 API 공개 검토

이번 PR은 HWPX 저장 안정성을 높이는 변경이다. 다만 변경 범위는 `borderFill`의 slash/backSlash type 보존으로 한정된다.

현재 코드에는 이미 다음 경로가 존재한다.

- Rust core: `serializer::serialize_hwpx(doc)`
- DocumentCore: `export_hwpx_native()`
- WASM: `HwpDocument.exportHwpx()`
- rhwp-studio bridge: `wasm.exportHwpx()`

반면 일반 저장 UI와 hwpctl `SaveAs`는 HWPX 출처 직접 저장을 아직 차단한다.

판단:

- 이번 #1311 처리와 함께 **일반 사용자 저장 UI를 즉시 개방하지는 않는다**.
- HWPX 저장 API는 “명시적/실험적 export API”로 문서화하거나 자동화 API에서 제한적으로 여는 방향은 검토 가능하다.
- 원본 HWPX 덮어쓰기 허용은 별도 이슈에서 warning, backup, roundtrip 검증 정책과 함께 다루는 것이 안전하다.

## 5. 남은 절차

1. 완료 보고서 승인
2. `local/pr1311-integration`을 `devel`에 반영
3. 원격 `devel` push
4. PR #1311에 메인테이너 코멘트 추가
5. PR #1311 종료 처리
6. 이슈 #1278 종료 처리
