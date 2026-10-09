---
kind: working
status: completed
last_verified: 2026-10-09
---

# #7688 4단계 — crop 좌표 단위의 공통 소비

사용자가 순차 구현을 승인했다. base는 fetch 후에도 `6f66932a73ee6fd05a74a83bdc438c9499a89a12`,
출발 head는 수식 시각 통과 기록 `27891204e`다. 기존 1단계의 동일 원문 한컴 Print PDF를 재사용한다.

## 독립 근거와 반례

`pic-crop-01.hwp`의 두 그림은 모두 639×70 JPEG다. crop은 각각 `(0,0,47940,5280)`과
`(0,0,47940,4366)`이며, 원본 SHAPE_PICTURE tail은 9바이트로 imgDim이 없다.
SHAPE_COMPONENT 원래 크기·삽입 사각형은 41400×4560이므로 crop 기준 전체 크기로 대신하지 않는다.
JPEG DPI는 72지만 #7015 JPEG의 DPI 300에도 저장 crop은 75 HU/px 좌표를 사용한다.
따라서 embedded DPI도 저장 crop 좌표 기준을 입증하지 않는다.

한컴 PDF 첫 그림은 639×70, 둘째는 639×61 raster와 아래쪽 clip을 사용한다. 둘째 그림의 보이는
높이는 원본 약 58행이다. 두 그림 모두 원본 전체 70행이라는 기존 Studio 검사는 잘못된 기대값이다.
HWPUNIT은 x/y에서 같은 길이 단위다. 명시적 imgDim 없이 zero-origin 끝점에서 서로 다른 배율을
선택하면 아래쪽 또는 오른쪽 자르기를 원본 전체로 늘려 지운다.

원래 전체 범위가 없는 fallback의 한계는 유지한다. 0에서 시작하는 축의 끝점을 원본에 수용하는
최소 공통 등방 배율을 사용한다(후보 중 최댓값). 시작점이 0이라는 사실을 ‘전체 범위 확인’으로
보고하지 않는다. imgDim 명시 경로는 변경하지 않고, 두 축의 시작점이 모두 0이 아니면 기존 표준
75 HU/px를 유지한다. 두 축 모두 잘린 모든 불완전 입력의 실제 기준 크기를 복원했다고 주장하지 않는다.
200dpi 스캔(#3239), 한 축 시작점이 잘린 로고(#7015), 명시적 imgDim과 무-crop을 대조한다.

## 실제 호출 경로

Picture crop/imgDim → ImageNode/PaintOp(originalSizeHu) → Rust `compute_image_crop_src` →
SVG crop viewBox, WebCanvas/Skia source rect. Studio는 같은 계약의 `imageCropScale` →
`imageCropSourceRect` → DOM PageRenderer / CanvasKit image replay를 사용한다.
유효한 imgDim의 축척 경로는 유지한다. 원점·줄 소속·목표 bbox를 바꾸거나 좌표를 추가 clamp하지 않는다.

ignored 증적은 `output/pr-review/renderer-backend-audit-20261009/crop-fix/`에 둔다.
시각 90% 선행 조건 충족 전 새 회귀 추가·기존 기준값 변경은 하지 않는다.

## 실행 결과와 검출 증거

제품 출발 head `27891204e`, base `6f66932a73ee6fd05a74a83bdc438c9499a89a12`.
Docker `rhwp-wasm:latest`에서 루트 wrapper를 실행했고 pkg/public JS·WASM SHA-256이 각각 일치한다.
JS `70cde06a…`, WASM `222f25b7…`를 새 Chrome의 실제 초기화 요청과 대조했다.
Studio 7788의 브라우저 화면에서 첫 그림의 원래 행과 둘째 그림의 아래쪽 선택을 직접 확인했다.
Vite 응답 JS는 source map과 URL 변환을 대조해 원본 hash를 확인했다.

| 같은 한컴 Print PDF의 1쪽 | Native SVG | fresh WASM SVG | 실제 Native PNG screen/print |
| --- | --- | --- | --- |
| pic-crop-01 | 99.83033% | 99.83033% | 99.83013% / 동일 |
| 200dpi scan | 93.49689% | 93.49689% | 92.22154% / 동일 |

crop 수정 전 banner의 Native SVG 86.54494%, Native PNG 86.52157%와 같은 영역의 review PNG를
직접 대조했다. scan과 #7015 로고 Native PNG는 두 profile 모두 수정 전후 byte 동일하다.
Native/fresh WASM review·overlay, 실제 Native PNG, Studio 캡처와 초기화 hash를 ignored
`crop-fix/`에 보존했다. PDF가 일부 재표본화했으므로 전체 70행과 약 58행의 의미를 비교한다.

시각 선행 조건 통과 뒤 `tests/cases/issue_7688_picture_crop.rs`를 추가했다.
정식 CLI의 screen/print SVG에서 동일 원문의 두 그림·순서와 선택 행 수를 검사한다.
독립 기대값은 PDF의 첫 그림 70행/둘째 약 58행이며 전체 SVG hash나 화면 좌표를 고정하지 않는다.
같은 검사 실행 파일에 기존/수정 Native CLI를 각각 전달해 수정 전
`bottom crop lost: selected 70 rows instead of PDF's ~58` FAIL / 수정 후 PASS를 확인했다.
초기 검사의 source-width 전제·임시 디렉터리 crate·CLI 옵션 오류는 검사 준비 오류로 분리했으며,
결함 재현으로 세지 않는다. 최종 before/after 로그와 바이너리 hash는 `crop-test-results.json`에 있다.

기존 Rust/Studio 검사의 zero-origin=무-crop 가정을 제거했다. 명시적인 imgDim의 full-image
대조군은 유지하며, 원래 전체 크기가 없는 합성 입력은 등방 단위 계약만 검사한다.
실제 200dpi 대조군은 독립 단위 7200/200 HU/px와 실제 출력 보존을 함께 확인한다.
Studio 전체 Node 검사 1,825 PASS/0 FAIL/2 skipped, TypeScript noEmit PASS다.
Rust 전체 회귀·세 Clippy·source unit tier·suite policy는 승인된 후속 구현을 마친 최종 head에서
실행한다. 이 로컬 단계 기록을 PR 제출에 필요한 최종 검증으로 대신하지 않는다.

남은 적용 한계: 원래 전체 크기를 잃고 두 축이 모두 잘린 입력을 완전히 복원했다고 주장하지 않는다.
임의 DPI/삽입 사각형을 기준으로 대신하지 않았고, 원점·bbox·쪽 소속은 바꾸지 않았다.
