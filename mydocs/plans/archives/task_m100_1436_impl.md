# Task 1436 구현 계획서

## 구현 범위

이번 작업은 이미 존재하는 `CommonObjAttr.size_protect`를 Studio와 명령 경로에 연결하는 작업이다. 파일 포맷 모델을 새로 만들지 않고, HWP5/HWPX 기존 매핑을 UI/명령 계층까지 전달한다.

## Stage 1: 속성 JSON 노출 및 저장

- `src/document_core/commands/object_ops.rs`의 공통 개체 JSON에 `sizeProtect`를 추가한다.
- 그림 전용 속성 JSON에도 동일하게 `sizeProtect`를 포함한다.
- `apply_common_obj_attr_from_json`과 picture 속성 적용 경로에서 `sizeProtect`를 받아 `CommonObjAttr.size_protect` 및 attr bit 20을 갱신한다.
- Shape/Picture/Equation 등 공통 속성 API 간 이름을 `sizeProtect`로 통일한다.
- `rhwp-studio/src/core/types.ts`의 `PictureProperties`, `ShapeProperties`, 필요 시 `EquationProperties` 타입에 `sizeProtect`를 추가한다.

## Stage 2: 개체 속성 UI 정합화

- `rhwp-studio/src/ui/picture-props-dialog.ts`에서 `sizeFixedCheck`를 `props.sizeProtect`와 연결한다.
- `sizeFixedCheck` 변경 시 관련 컨트롤 비활성화 상태를 즉시 갱신한다.
- `sizeProtect`가 켜진 상태에서는 다음 컨트롤을 비활성화한다.
  - 너비/높이 입력
  - 기본 탭 `비율 유지` 체크
  - 회전각 입력
  - 기울이기 입력
  - UI에 연결되어 있다면 대칭/회전 보조 조작
- 확인 시 크기 고정이 켜져 있으면 기존 크기/회전/기울기 값 변경을 전송하지 않고, `sizeProtect` 토글만 전송한다.

## Stage 3: 직접 조작 차단

- `rhwp-studio/src/engine/input-handler-picture.ts`의 키보드 리사이즈, 드래그 리사이즈, 회전 드래그 시작/완료 경로에서 대상 속성을 조회해 `sizeProtect`가 켜진 개체를 건너뛴다.
- 다중 선택 리사이즈는 잠긴 개체를 제외하거나 전체 조작을 중단하는 한컴 동작을 확인해 더 가까운 쪽으로 맞춘다.
- `ResizeObjectCommand`나 WASM setter만으로 우회되지 않도록 Rust `setPictureProperties`/`setShapeProperties` 계층에도 최소 방어를 둔다.

## Stage 4: 모달 외부 클릭 닫힘 재점검

- #1428에서 정리된 `ModalDialog`와 `PicturePropsDialog`의 외부 클릭 닫힘 상태를 재확인한다.
- 개체 속성의 보조 설명 모달, 테이블 생성처럼 모달로 동작해야 하는 팝업에 남은 overlay click close가 있으면 제거한다.
- 메뉴/드롭다운형 transient UI는 범위에서 제외하고 문서에 명시한다.

## Stage 5: 검증

- Rust 단위/통합 테스트:
  - `sizeProtect` JSON 노출 및 setter round-trip
  - `sizeProtect` 상태에서 width/height/rotation 변경 무시 또는 차단
  - HWPX `hp:sz@protect` 보존
- Studio 테스트:
  - 개체 속성 대화상자에서 `크기 고정` ON 시 입력 비활성화
  - overlay 클릭 후 모달 유지
  - 크기 고정 개체 리사이즈/회전 조작 불가
- 회귀 확인:
  - #1428 `비율 유지` 저장 설정
  - #1282 표 안 그림 크기/쪽 영역 제한 관련 시각 기준

## PR 전 검증

- `cargo build --release`
- `cargo test --release --lib`
- `cargo test --profile release-test --tests`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- 필요 시 `wasm-pack build --target web --out-dir pkg`
- 필요 시 rhwp-studio TypeScript/E2E 검증

## 주의사항

- `크기 고정(sizeProtect)`은 파일 포맷 속성이고, #1428의 `비율 유지(keepRatio)`는 사용자 UI 설정이다.
- UI만 막지 말고 명령/저장 경로에도 방어를 둔다.
- 외부 클릭 닫힘 차단은 모달 대화상자에 한정한다.
- 기여자 모드이므로 오늘할일 문서는 만들지 않는다.
