# PR #1279 리뷰 — 글상자/셀 중첩 그림 마우스 드래그 조작

## 1. 대상

- PR: [#1279](https://github.com/edwardkim/rhwp/pull/1279)
- 관련 이슈: [#1273](https://github.com/edwardkim/rhwp/issues/1273)
- 제목: `fix: 글상자/셀 중첩 그림 마우스 드래그 조작(리사이즈·회전·이동) 실패 + floating 리사이즈 이탈 (#1273)`
- 작성자: `@johndoekim`
- 상태: open, non-draft
- base: `devel`

## 2. 변경 범위

PR 자체는 rhwp-studio TypeScript 동작 경로를 수정한다.
메인테이너 통합 과정에서 회전 그림의 HWP 저장 호환성 보강을 Rust 쪽에 추가했다.

핵심 변경:

- 단일 개체 마우스 드래그 상태의 `ref`에 `cellPath`/`headerFooter` 보존
- `MovePictureCommand`/`MoveShapeCommand`의 cellPath by-path 지원
- 단일 선택 리사이즈 offset을 페이지 절대좌표가 아니라 기존 저장 offset + 페이지좌표 delta로 계산
- 실제 InputHandler 드래그 경로를 구동하는 E2E 추가
- HWP 저장 시 회전 그림의 외곽 박스/현재 크기 분리 및 rendering matrix 생성

변경 파일:

- `rhwp-studio/src/engine/input-handler.ts`
- `rhwp-studio/src/engine/input-handler-mouse.ts`
- `rhwp-studio/src/engine/input-handler-picture.ts`
- `rhwp-studio/src/engine/command.ts`
- `rhwp-studio/e2e/textbox-picture-ops-1273.test.mjs`
- `src/document_core/commands/object_ops.rs`
- `src/serializer/control.rs`
- `tests/issue_1279_picture_rotation_save.rs`
- `samples/ta-pic-001-r.hwp`
- `samples/hwpx/ta-pic-001-r.hwpx`
- `pdf-large/hwpx/ta-pic-001-r.pdf`
- `mydocs/*` 작업 문서

## 3. 코드 검토

### 3.1 드래그 상태 ref 보존

`input-handler-mouse.ts`의 리사이즈/회전/이동 시작 지점에서 기존 선택 ref를 `{ sec, ppi, ci, type }`로 재구성하던 것이 문제였다.
PR은 여기에 `cellPath`와 `headerFooter`를 그대로 복사한다.

이 변경은 기존 body-level 그림에는 영향을 주지 않고, 글상자/셀 내부 그림에서 by-path setter로 분기할 수 있게 한다.

### 3.2 이동 undo/redo by-path

`MovePictureCommand`/`MoveShapeCommand`는 기존 scalar API만 사용했다.
PR은 `cellPath`가 있을 때 `getCellPicturePropertiesByPath` / `setCellPicturePropertiesByPath` 또는 shape 대응 API를 사용하도록 분기한다.

`mergeWith`에서도 `cellPath`가 같은 이동만 병합하므로 서로 다른 중첩 위치 개체가 잘못 병합되는 위험은 낮다.

### 3.3 floating 리사이즈 offset

단일 선택 리사이즈에서 offset을 페이지 절대값으로 기록하면 글상자/셀 내부 그림이 컨테이너 밖으로 이탈한다.
PR은 기존 저장 offset에 페이지좌표 delta를 더하는 방식으로 변경했다.

본문 그림은 기존 저장 offset과 bbox 기반 offset이 사실상 같은 좌표계를 가지므로 기존 동작 영향은 제한적이다.

### 3.4 회전 그림 HWP 저장 호환성

웹 조작은 통과했지만, 회전한 셀 내부 그림을 저장한 HWP를 한컴편집기에서 열면
`rotationAngle` 속성만 바인딩되고 실제 이미지는 회전되지 않는 문제가 확인되었다.

`samples/ta-pic-001-r.hwp`와 `samples/hwpx/ta-pic-001-r.hwpx`를 비교한 결과,
한컴 호환 저장에는 다음 계약이 필요하다.

- `CommonObjAttr.width/height`: 회전 후 외곽 박스
- `ShapeComponentAttr.current_width/current_height`: 회전 전 실제 표시 크기
- SHAPE_COMPONENT rendering block: `transMatrix + scaleMatrix + rotMatrix`

메인테이너 보강은 그림 회전 변경 시 위 크기 계약을 materialize하고,
raw rendering matrix가 없는 회전 그림을 HWP로 저장할 때 한컴이 실제 회전에 사용하는
`rotMatrix`를 생성하도록 했다.

## 4. 검증 결과

최신 `devel` 기준 검증 브랜치:

```text
local/pr1279-verify
```

적용:

```text
git cherry-pick f4ae3bf2 d6f4d9ad 2a22849a 41c6465d 08614a6f
```

결과: 충돌 없음.

로컬 검증:

```text
cd rhwp-studio
./node_modules/.bin/tsc --noEmit
```

결과: 통과.

```text
VITE_URL=http://127.0.0.1:7701 node e2e/textbox-picture-ops-1273.test.mjs --mode=headless
```

결과: 통과.

확인 값:

```text
warnings: []
resize: width 15040 -> 18038 -> undo 15040
rotate: angle 0 -> 32
floating: vertOffset 0 -> 2250, bbox y 799 -> 829
```

기존 회귀 E2E:

```text
VITE_URL=http://127.0.0.1:7701 node e2e/textbox-picture-1171.test.mjs --mode=headless
VITE_URL=http://127.0.0.1:7701 node e2e/textbox-picture-insert-1171.test.mjs --mode=headless
```

결과: 통과.

참고:

- `textbox-picture-1171.test.mjs` 첫 실행에서 headless Chrome launch가 한 번 실패했으나 재실행 통과.
- 실패 지점은 브라우저 프로세스 시작 단계였고 PR 코드 실행 전이므로 코드 회귀로 보지 않는다.

빌드:

```text
npm run build
```

결과: 통과.

후속 Rust 검증:

```text
cargo test --test issue_1279_picture_rotation_save
cargo test --lib document_core::commands::object_ops
cargo test --lib serializer::control
cargo test --test issue_1067_shape_rotation
```

결과: 통과.

## 5. 통합 처리

PR에 포함된 `mydocs/plans`, `mydocs/report`, `mydocs/working` 문서는 현재 repo 정책에 맞춰 통합 브랜치에서 archive로 이동했다.

이동 완료:

```text
mydocs/plans/task_m100_1273.md -> mydocs/plans/archives/task_m100_1273.md
mydocs/plans/task_m100_1273_impl.md -> mydocs/plans/archives/task_m100_1273_impl.md
mydocs/report/task_m100_1273_report.md -> mydocs/report/archives/task_m100_1273_report.md
mydocs/working/task_m100_1273_stage1.md -> mydocs/working/archives/task_m100_1273_stage1.md
mydocs/working/task_m100_1273_stage2.md -> mydocs/working/archives/task_m100_1273_stage2.md
mydocs/working/task_m100_1273_stage3.md -> mydocs/working/archives/task_m100_1273_stage3.md
mydocs/working/task_m100_1273_stage4.md -> mydocs/working/archives/task_m100_1273_stage4.md
```

## 6. 권장안

권장: 수용.

근거:

- 사용자 보고 오류였던 `컨트롤 인덱스 범위 초과` / `지정된 Shape 컨트롤이 그림이 아닙니다` 경로를 직접 다룬다.
- 실제 InputHandler 마우스 드래그 경로 E2E가 추가되어 기존 테스트 공백을 메운다.
- TS 타입 체크, 신규 E2E, 기존 관련 E2E, production build가 통과했다.
- 후속 Rust 테스트로 회전 그림의 HWP 저장 rendering matrix 계약도 고정했다.

진행 절차:

1. `local/pr1279-integration` 브랜치 생성 완료
2. PR 커밋 반영 완료
3. 작업 문서 archive 이동 완료
4. `rhwp-studio` 검증 재실행 완료
5. `devel` 병합, push, CI 확인
6. PR #1279 및 이슈 #1273 종료 처리
