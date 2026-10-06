# PR #1349 검토 - HWPX 그림 effects/shadow roundtrip 보존 (#1345)

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1349 |
| 작성자 | `Mireutale` |
| 상태 | open / draft 아님 |
| base | `devel` (`54e6393c`) |
| head | `fix/issue-1345-hwpx-picture-effects` (`7a32012b`) |
| 관련 이슈 | #1345 |
| 변경 규모 | 6 files, +401 / -7 |
| mergeable | `MERGEABLE` |

## 2. 변경 요약

문제:

- `samples/hwpx/aift.hwpx`를 parse → serialize_hwpx roundtrip하면
  `Contents/section2.xml`의 그림 효과 정보가 사라진다.
- 원본에는 `hp:effects`, `hp:shadow`, `hp:scale`, `hp:effectsColor`, `hp:rgb`
  태그가 있으나 roundtrip 결과에서는 효과 관련 태그 수가 0개가 된다.
- #1345 코멘트 기준으로 대상 shadow 그림은 `hp:container` 내부에 있어, 그룹 자식
  직렬화 누락도 함께 해결해야 테스트가 성립한다.

수정:

- `Picture` IR에 HWPX 그림 효과 보존용 모델 추가:
  - `PictureEffects`
  - `PictureShadow`
  - `EffectPoint`
  - `EffectColor`
  - `EffectRgb`
- HWPX parser가 `<hp:effects><hp:shadow>...` 하위 값을 읽어 `Picture.effects`에 보존.
- HWPX picture serializer가 보존된 shadow/effects 값을 다시 출력.
- `ShapeObject::Group` 직렬화 시 `hp:container` open/children/close 루프를 연결해
  그룹 자식 그림이 누락되지 않도록 함.
- `samples/hwpx/aift.hwpx` 기반 roundtrip 회귀 테스트와 단위 serializer 테스트 추가.

## 3. GitHub 상태

GitHub Actions:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| CodeQL Analyze (javascript-typescript) | pass |
| CodeQL Analyze (python) | pass |
| CodeQL Analyze (rust) | pass |
| CodeQL | pass |
| WASM Build | skipped |

리뷰 상태:

- 기존 review 없음.
- review request: `edwardkim`.

관련 이슈:

- #1345는 open 상태.
- PR 본문에는 `closes #1345`가 있으나, `gh pr view`의 `closingIssuesReferences`는 빈 배열로 확인됨.
- 통합 후 #1345 자동 close 여부를 별도로 확인해야 한다.

## 4. 로컬 검토 방식

검토 기준 브랜치:

```text
local/devel @ 54e6393c
```

PR head fetch:

```text
local/pr1349-upstream @ 7a32012b
```

변경 파일:

- `src/model/image.rs`
- `src/parser/hwpx/section.rs`
- `src/serializer/hwpx/mod.rs`
- `src/serializer/hwpx/picture.rs`
- `src/serializer/hwpx/section.rs`
- `src/wasm_api/tests.rs`

## 5. 로컬 검증

실행 완료:

```bash
cargo fmt --all -- --check
cargo test --lib issue_1345_picture_effects_shadow_roundtrip
cargo test --lib picture_effects_shadow_are_serialized
cargo clippy --lib -- -D warnings
cargo test --lib serializer::hwpx
cargo test --test hwpx_roundtrip_integration
cargo test --offline --target-dir /Users/edwardkim/vspace/rhwp/target --test hwpx_to_hwp_adapter stage5_export_hwp_with_adapter_idempotent_on_repeated_calls -- --nocapture
cargo test --offline --target-dir /Users/edwardkim/vspace/rhwp/target --test issue_1279_picture_rotation_save issue_1279_hwpx_to_hwp_export_preserves_picture_rotation_contract -- --nocapture
cargo run --offline --target-dir /Users/edwardkim/vspace/rhwp/target --bin rhwp -- convert samples/hwpx/aift.hwpx /private/tmp/rhwp-pr1349-aift.hwp
cargo test --offline --target-dir /Users/edwardkim/vspace/rhwp/target --test pr1349_hwpx2hwp_cross_check -- --nocapture
```

결과:

| 명령 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --lib issue_1345_picture_effects_shadow_roundtrip` | 통과 |
| `cargo test --lib picture_effects_shadow_are_serialized` | 통과 |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `cargo test --lib serializer::hwpx` | 87 passed |
| `cargo test --test hwpx_roundtrip_integration` | 18 passed |
| `cargo test --test hwpx_to_hwp_adapter stage5_export_hwp_with_adapter_idempotent_on_repeated_calls` | 통과 |
| `cargo test --test issue_1279_picture_rotation_save issue_1279_hwpx_to_hwp_export_preserves_picture_rotation_contract` | 통과 |
| `rhwp convert samples/hwpx/aift.hwpx /private/tmp/rhwp-pr1349-aift.hwp` | 통과, 4498KB HWP 생성 |
| 임시 `pr1349_hwpx2hwp_cross_check` 테스트 | 통과, `aift.hwpx`의 shadow 효과 존재 확인 후 HWP export/reload 성공 |

비고:

- macOS 환경에서 `xcrun` SDK 탐색 관련 warning이 출력되었으나 테스트/검증은 통과했다.
- `hwpx2hwp` cross-check는 `/private/tmp/rhwp-pr1349-cross` 별도 임시 worktree에서 수행한 뒤 제거했다.
- 임시 `pr1349_hwpx2hwp_cross_check`는 PR 코드에 포함하지 않고 검증용으로만 사용했다.

## 6. 검토 의견

차단 이슈는 발견하지 못했다.

수정 방향은 타당하다.

- `Picture` IR에 HWPX effects 보존 모델을 추가하는 방식은 parser → serializer
  roundtrip 데이터 손실을 막는 목적에 맞다.
- HWP5 저장/렌더 공통 의미로 과도하게 해석하지 않고 문자열 기반으로 보존하는 접근은
  이번 범위에 적절하다.
- `hp:container` 자식 직렬화 연결은 #1345 대상 샘플에서 그림 자체가 그룹 안에 있으므로
  effects 보존 테스트의 전제조건이다.
- 기존 serializer는 빈 `<hp:effects>`를 이미 출력하고 있었으므로, shadow가 없는 그림의
  기본 출력 정책도 크게 바뀌지 않는다.
- `hwpx2hwp` 저장 경로도 cross-check 했다. HWP serializer의 `serialize_picture_data()`는
  HWP5 `SHAPE_COMPONENT_PICTURE` 필드(`image_attr`, `raw_picture_extra`, instance 등)만
  직렬화하고 신규 `Picture.effects`를 HWP 바이너리 의미로 해석하지 않는다. 따라서 이번 PR은
  HWPX roundtrip 보존을 확장하는 변경이며, HWP export 경로에는 "효과 변환"이 추가되지 않는다.
  별도 검증에서 `aift.hwpx`의 shadow 효과가 IR에 존재하는 상태로 `export_hwp_with_adapter()`
  직렬화 및 HWP 재로드가 성공했고, 그림 컨트롤도 재파싱 가능했다.

주의점:

- 이번 PR은 `hp:effects` 전체가 아니라 shadow 계열 보존에 초점을 둔다. glow/reflection 등
  다른 effect 하위 요소는 별도 후속 범위로 남을 수 있다.
- HWP 바이너리 포맷으로 shadow/effects를 완전 변환하는 것은 이번 PR의 범위가 아니다. 한컴 HWP
  저장 결과와 효과까지 맞추려면 별도 HWP5 레코드 매핑 분석이 필요하다.
- `closingIssuesReferences`가 비어 있으므로 merge 후 #1345 close 상태를 반드시 확인한다.
- PR 본문에서 WASM/시각 확인은 미완료로 표시되어 있으나, 변경 본질은 HWPX XML roundtrip
  데이터 보존이다. 화면 렌더 변화는 직접 목표가 아니다.

## 7. 권장 처리

권장: **수용 가능**.

권장 절차:

1. 작업지시자 승인.
2. 현재 `devel` 기준으로 PR head 통합.
3. 최종 검증:
   - `cargo fmt --all -- --check`
   - `cargo test --lib serializer::hwpx`
   - `cargo test --test hwpx_roundtrip_integration`
   - 필요 시 `cargo clippy --lib -- -D warnings`
4. `devel` push.
5. PR #1349에 검토/통합 코멘트 작성.
6. PR #1349 merge/close 상태 확인.
7. #1345 close 여부 확인. 자동 close가 되지 않으면 completed로 수동 close.

## 8. PR 코멘트 초안

```markdown
검토 완료했습니다.

`samples/hwpx/aift.hwpx` roundtrip에서 `hp:effects`/`hp:shadow` 계열 정보가 사라지는 원인을 `Picture` IR의 effects 보존 누락과 `hp:container` 자식 직렬화 누락으로 나누어 처리한 방향이 타당하다고 확인했습니다.

로컬 검증:

- `cargo fmt --all -- --check`
- `cargo test --lib issue_1345_picture_effects_shadow_roundtrip`
- `cargo test --lib picture_effects_shadow_are_serialized`
- `cargo clippy --lib -- -D warnings`
- `cargo test --lib serializer::hwpx`
- `cargo test --test hwpx_roundtrip_integration`
- `cargo test --test hwpx_to_hwp_adapter stage5_export_hwp_with_adapter_idempotent_on_repeated_calls`
- `cargo test --test issue_1279_picture_rotation_save issue_1279_hwpx_to_hwp_export_preserves_picture_rotation_contract`
- `rhwp convert samples/hwpx/aift.hwpx /private/tmp/rhwp-pr1349-aift.hwp`
- 임시 `pr1349_hwpx2hwp_cross_check`: `aift.hwpx` shadow 효과 확인 후 HWP export/reload 성공

GitHub Actions의 Build & Test / CodeQL success도 확인했습니다.

차단 이슈는 발견하지 못했습니다. 수용 절차로 진행하겠습니다.
```
