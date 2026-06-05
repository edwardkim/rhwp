# PR #1279 처리 보고서

## 대상

- PR: [#1279](https://github.com/edwardkim/rhwp/pull/1279)
- 관련 이슈: [#1273](https://github.com/edwardkim/rhwp/issues/1273)
- 제목: `fix: 글상자/셀 중첩 그림 마우스 드래그 조작(리사이즈·회전·이동) 실패 + floating 리사이즈 이탈 (#1273)`
- 처리 브랜치: `local/pr1279-integration`

## 반영 내용

PR #1279의 5개 커밋을 최신 `devel` 기준으로 cherry-pick했다.

```text
f4ae3bf2 Task #1273: Stage 1 — 드래그 상태 ref에 cellPath/headerFooter 보존
d6f4d9ad Task #1273: Stage 2 — 이동 커맨드 by-path 지원 (undo/redo)
2a22849a Task #1273: Stage 3 — 드래그 조작 lifecycle E2E + 재발방지 문서
41c6465d Task #1273: Stage 4 — 리사이즈 offset 델타 기반 (라이브+확정)
08614a6f Task #1273: 최종 결과보고서
```

추가 통합 처리:

- PR 문서를 archive 정책에 맞춰 이동했다.
- PR 리뷰 문서 `mydocs/pr/pr_1279_review.md`를 작성했다.
- 본 처리 보고서 `mydocs/pr/pr_1279_report.md`를 작성했다.

후속 보강:

- rhwp-studio에서 회전한 셀 내부 그림을 HWP로 저장하면 한컴편집기에서
  `rotationAngle` 속성은 바인딩되지만 실제 이미지가 회전되지 않는 문제를 확인했다.
- 원인은 HWP 저장 경로가 `ShapeComponentAttr.rotation_angle`만 갱신하고,
  한컴이 실제 회전에 사용하는 SHAPE_COMPONENT rendering matrix는 identity로 남기는 것이었다.
- `samples/ta-pic-001-r.hwp`, `samples/hwpx/ta-pic-001-r.hwpx`,
  `pdf-large/hwpx/ta-pic-001-r.pdf`를 기준 파일로 추가했다.
- 저장 시 회전 그림의 `common.width/height`는 회전 후 외곽 박스,
  `shape_attr.current_width/current_height`는 회전 전 실제 표시 크기로 분리되도록 보강했다.
- raw rendering matrix가 없는 회전 그림은 HWP 직렬화 시
  `transMatrix + scaleMatrix + rotMatrix`를 생성하도록 보강했다.
- 리사이즈 경로의 `width/height`는 UI 표시 박스 크기이므로, 회전 그림에서는 기존
  `curSz` 비율로 실제 표시 크기를 스케일하고 회전 외곽 재계산은 `rotationAngle`
  변경 시에만 수행하도록 분리했다.
- 더 정밀한 UX 정합은 후속 이슈 [#1282](https://github.com/edwardkim/rhwp/issues/1282)로 분리했다.

## 문서 이동

```text
mydocs/plans/task_m100_1273.md -> mydocs/plans/archives/task_m100_1273.md
mydocs/plans/task_m100_1273_impl.md -> mydocs/plans/archives/task_m100_1273_impl.md
mydocs/report/task_m100_1273_report.md -> mydocs/report/archives/task_m100_1273_report.md
mydocs/working/task_m100_1273_stage1.md -> mydocs/working/archives/task_m100_1273_stage1.md
mydocs/working/task_m100_1273_stage2.md -> mydocs/working/archives/task_m100_1273_stage2.md
mydocs/working/task_m100_1273_stage3.md -> mydocs/working/archives/task_m100_1273_stage3.md
mydocs/working/task_m100_1273_stage4.md -> mydocs/working/archives/task_m100_1273_stage4.md
```

## 검증

통과:

```text
cd rhwp-studio
./node_modules/.bin/tsc --noEmit
npm run build
VITE_URL=http://127.0.0.1:7701 node e2e/textbox-picture-ops-1273.test.mjs --mode=headless
VITE_URL=http://127.0.0.1:7701 node e2e/textbox-picture-1171.test.mjs --mode=headless
VITE_URL=http://127.0.0.1:7701 node e2e/textbox-picture-insert-1171.test.mjs --mode=headless
```

후속 Rust 검증:

```text
cargo test --test issue_1279_picture_rotation_save
cargo test --lib document_core::commands::object_ops
cargo test --lib serializer::control
cargo test --test issue_1067_shape_rotation
```

주요 E2E 확인:

```text
warnings: []
resize: width 15040 -> 18038 -> undo 15040
rotate: angle 0 -> 32
floating: vertOffset 0 -> 2250, bbox y 799 -> 829
```

참고:

- `textbox-picture-insert-1171.test.mjs` 첫 실행에서 headless Chrome launch가 한 번 실패했다.
- 실패 지점은 테스트 코드 실행 전 브라우저 프로세스 시작 단계였고, 동일 명령 재실행은 통과했다.

## 판정

수용 가능.

근거:

- 보고된 조작 오류 경로를 직접 수정한다.
- 실제 InputHandler 마우스 드래그 경로를 구동하는 E2E가 추가되었다.
- 관련 기존 E2E와 production build가 통과했다.
- 후속 저장 호환성 회귀 테스트로 회전 그림의 HWP rendering matrix 계약을 고정했다.

## 다음 절차

```text
1. 통합 커밋 생성
2. push
3. GitHub CI 확인
4. PR #1279 / 이슈 #1273 종료 처리
```
