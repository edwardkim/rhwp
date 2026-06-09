# PR #1349 완료 보고서 — HWPX 그림 effects/shadow roundtrip 보존

## 1. 개요

| 항목 | 내용 |
|---|---|
| PR | #1349 |
| 작성자 | `Mireutale` |
| 관련 이슈 | #1345 |
| 검토 브랜치 | `local/pr1349-upstream` |
| 통합 방식 | 현재 `devel` 기준 PR 단일 커밋 cherry-pick |
| 원 PR 커밋 | `7a32012b` |
| 반영 커밋 | `8a2f56d5` |
| devel merge | `35b5379a` |

## 2. 처리 내용

PR #1349의 변경을 현재 `local/devel` 위에 cherry-pick하여 반영했다.

핵심 변경:

- `Picture` IR에 HWPX 그림 효과 보존 모델 추가
  - `PictureEffects`
  - `PictureShadow`
  - `EffectPoint`
  - `EffectColor`
  - `EffectRgb`
- HWPX parser가 `<hp:effects><hp:shadow>...` 하위 값을 읽어 `Picture.effects`에 보존
- HWPX picture serializer가 보존된 shadow/effects 값을 다시 출력
- `hp:container` 그룹 자식 직렬화에서 내부 그림이 누락되지 않도록 children 직렬화 연결
- `samples/hwpx/aift.hwpx` 기반 roundtrip 회귀 테스트와 단위 serializer 테스트 추가

## 3. HWPX2HWP 교차 확인

이번 PR의 본질은 HWPX XML roundtrip 보존이다.

HWP serializer의 `serialize_picture_data()`는 HWP5 `SHAPE_COMPONENT_PICTURE` 필드
(`image_attr`, `raw_picture_extra`, instance 등)만 직렬화하고, 신규 `Picture.effects`를
HWP 바이너리 효과로 해석하지 않는다.

따라서 이번 변경은 HWP 바이너리 shadow/effects 변환을 추가하지 않는다. 다만 `aift.hwpx`에
shadow 효과가 존재하는 상태에서 `export_hwp_with_adapter()` 및 HWP 재로드 경로가 깨지지 않는지
별도 확인했고, 저장/재로드가 정상 동작했다.

## 4. 검증 결과

| 명령 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --offline --lib issue_1345_picture_effects_shadow_roundtrip -- --nocapture` | 통과 |
| `cargo test --offline --lib picture_effects_shadow_are_serialized -- --nocapture` | 통과 |
| `cargo test --offline --lib serializer::hwpx` | 통과, 87 passed |
| `cargo test --offline --test hwpx_roundtrip_integration` | 통과, 22 passed |
| `cargo test --offline --test hwpx_to_hwp_adapter stage5_export_hwp_with_adapter_idempotent_on_repeated_calls -- --nocapture` | 통과 |
| `cargo test --offline --test issue_1279_picture_rotation_save issue_1279_hwpx_to_hwp_export_preserves_picture_rotation_contract -- --nocapture` | 통과 |
| `cargo clippy --offline --lib -- -D warnings` | 통과 |
| `cargo run --offline --bin rhwp -- convert samples/hwpx/aift.hwpx /private/tmp/rhwp-pr1349-aift-final.hwp` | 통과, 4498KB HWP 생성 |

GitHub checks:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| CodeQL Analyze javascript-typescript/python/rust | pass |
| CodeQL | pass |
| WASM Build | skipped |

## 5. 판정

**수용 완료**.

- `samples/hwpx/aift.hwpx` roundtrip에서 손실되던 `hp:effects`/`hp:shadow` 계열 정보가
  parser → IR → serializer 경로에서 보존된다.
- `hp:container` 내부 그림이 직렬화되지 않으면 대상 샘플 검증이 성립하지 않으므로,
  그룹 자식 직렬화 보강은 이번 PR 범위에 필요한 정정이다.
- HWPX 전용 효과 정보를 HWP5 저장 의미로 과도하게 변환하지 않고 보존 모델로 다룬 점은
  이번 문제 범위에 적절하다.
- HWPX2HWP 저장 경로는 별도 cross-check 결과 깨지지 않았다.

## 6. 후속 처리

- [x] `local/devel`에 리뷰 문서 커밋 — `24654c52`
- [x] `local/devel`에 PR 커밋 cherry-pick — `8a2f56d5`
- [x] `devel` no-ff merge — `35b5379a`
- [x] `origin/devel` push — `35b5379a`
- [x] PR #1349 코멘트 작성 및 close — closedAt `2026-06-09T08:11:30Z`
- [x] Issue #1345 completed close — closedAt `2026-06-09T08:11:44Z`
