# PR #1309 완료 보고서 — HWPX 그림 직렬화 flip/rotation/orgSz 및 isEmbeded 보존

## 1. 개요

| 항목 | 내용 |
|---|---|
| PR | #1309 |
| 관련 이슈 | #1269 |
| 작성자 | wonbbnote |
| 통합 브랜치 | `local/pr1309-integration` |
| 통합 방식 | PR 커밋 cherry-pick 후 메인테이너 회귀 테스트 추가 |
| PR 커밋 | `2eb02445` |

## 2. 처리 내용

PR #1309의 코드 변경을 현재 `devel` 위에 적용했다.

핵심 변경:

- `src/serializer/hwpx/picture.rs`
  - `<hp:orgSz>`를 `ShapeComponentAttr.original_width/original_height`로 직렬화
  - `<hp:flip>`을 `ShapeComponentAttr.horz_flip/vert_flip`로 직렬화
  - `<hp:rotationInfo>`를 `rotation_angle`, `rotation_center`, `rotate_image`로 직렬화
- `src/serializer/hwpx/content.rs`
  - BinData manifest item에 `isEmbeded="1"` 추가

추가 보강:

- `src/serializer/hwpx/picture.rs`
  - `shape_component_attrs_are_serialized` 단위 테스트 추가
  - `orgSz`, `flip`, `rotationInfo`가 실제 `ShapeComponentAttr` 값을 쓰는지 고정
- `src/serializer/hwpx/mod.rs`
  - `picture_bindata_roundtrip` 테스트에서 `Contents/content.hpf`의 `isEmbeded="1"` 확인 추가
- `tests/issue_1279_picture_rotation_save.rs`
  - HWPX→HWP 저장 교차 테스트 추가
  - `samples/hwpx/ta-pic-001-r.hwpx`를 `export_hwp_with_adapter()`로 저장 후 재파싱해 회전 그림 계약 보존 확인

## 3. HWPX→HWP 교차 확인

PR #1309는 HWPX→HWPX serializer 변경이다. HWPX→HWP 저장 경로는 별도이며 다음 흐름을 탄다.

```text
HWPX parse
→ DocumentCore::export_hwp_with_adapter()
→ document_core/converters/hwpx_to_hwp.rs
→ serializer/control.rs
→ HWP5 SHAPE_COMPONENT
```

확인 결과 HWPX→HWP 경로는 이미 `ShapeComponentAttr`의 원본/현재 크기, flip, rotation, rendering matrix를 HWP5 `SHAPE_COMPONENT`로 기록한다. 이번 PR 변경과 충돌하지 않으며, 새 교차 테스트로 보존 계약을 고정했다.

## 4. 검증 결과

| 명령 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --lib serializer::hwpx -- --nocapture` | 통과, 82 passed |
| `cargo test --test issue_1279_picture_rotation_save -- --nocapture` | 통과, 4 passed |
| `cargo clippy --lib -- -D warnings` | 통과 |
| HWPX 저장본 한컴 편집기 열기 | 통과 |

GitHub PR checks 확인:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| Analyze (rust) | pass |
| Analyze (javascript-typescript) | pass |
| Analyze (python) | pass |
| CodeQL | pass |
| WASM Build | skipping |

한컴 편집기 판정용 산출물:

- `output/poc/pr1309-hwpx-save/pic2_roundtrip_pr1309.hwpx`

메인테이너가 한컴 편집기에서 파일 열기 성공을 확인했다.

## 5. 판정

**수용 가능**.

- 변경 범위가 HWPX picture serializer와 manifest item으로 좁다.
- parser가 이미 보존하는 `ShapeComponentAttr` 값을 serializer가 대칭적으로 쓰도록 보정한다.
- `isEmbeded="1"`은 현재 serializer가 ZIP 내부 embedded BinData만 manifest에 등록하는 구조와 맞다.
- HWPX→HWP 저장 경로도 별도 교차 테스트로 보존 계약을 확인했다.

## 6. 남은 절차

승인 후 진행:

1. 통합 브랜치 변경 커밋
2. `local/devel`로 merge
3. `devel`로 merge
4. `origin/devel` push
5. PR #1309 코멘트 및 close
6. 이슈 #1269 close
